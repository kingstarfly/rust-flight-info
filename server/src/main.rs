use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    time::{SystemTime, UNIX_EPOCH},
};

use networking;
use marshaling::{
    self, marshal_f32, marshal_string, marshal_u32, marshal_u32_array, unmarshal_string,
    unmarshal_u32, unmarshal_u8,
};

struct Flight {
    id: u32,
    source: String,
    destination: String,
    departure_time: u32, // Unix time
    seats: u32,
    airfare: f32,
}
impl Flight {
    fn reserve_seats(&mut self, num_seats: u32) -> bool {
        if self.seats >= num_seats {
            self.seats -= num_seats;
            true
        } else {
            false
        }
    }
}

struct ResponseCache {
    request_id: u32,
    response_payload: Vec<u8>,
}

#[derive(Eq, Hash, PartialEq, Clone)]
struct WatchlistEntry(u32, SocketAddr);
fn main() -> std::io::Result<()> {
    let mut flight_db = HashMap::new();
    flight_db.insert(
        1,
        Flight {
            id: 1,
            source: "A".to_string(),
            destination: "B".to_string(),
            departure_time: 1700000000,
            seats: 10,
            airfare: 10.1,
        },
    );

    flight_db.insert(
        2,
        Flight {
            id: 2,
            source: "A".to_string(),
            destination: "B".to_string(),
            departure_time: 1700000000,
            seats: 20,
            airfare: 20.2,
        },
    );

    flight_db.insert(
        3,
        Flight {
            id: 3,
            source: "C".to_string(),
            destination: "D".to_string(),
            departure_time: 1700000000,
            seats: 30,
            airfare: 30.3,
        },
    );

    // Build a hashmap of flight ID to a vector of WatchlistEntry
    let mut watchlist_db: HashMap<u32, Vec<WatchlistEntry>> = HashMap::new();

    // Build a hashmap of request ID to a ResponseCache
    let mut response_cache: HashMap<u32, ResponseCache> = HashMap::new();

    let socket = UdpSocket::bind("127.0.0.1:7878")?;
    // TODO: Set timeout for read?
    let mut buf = [0; 2048];

    loop {
        // Receives a single datagram message on the socket.
        // If `buf` is too small to hold
        // the message, it will be cut off.
        let (amt, client_addr) = socket.recv_from(&mut buf)?;

        // Prints out a received bytes
        println!("\nReceived {} bytes from {}", amt, client_addr);

        // Read the request ID in the first 4 bytes.
        let i: usize = 0;
        let (request_id, i) = unmarshal_u32(&buf, i);
        println!("Request ID: {}", request_id);

        // Check if the request ID is in the response cache.
        // If it is, then use the cached payload.
        // Or else, read service ID and call handler

        if let Some(response_cache_entry) = response_cache.get(&request_id) {
            // If the request ID is in the response cache, then send the cached payload.
            println!("Sending cached response for request ID {}", request_id);
            networking::send_response(
                request_id,
                response_cache_entry.response_payload.clone(),
                &socket,
                &client_addr,
            );
            continue;
        } 

        // Read the service ID in the next byte.
        let (service_id, i) = unmarshal_u8(&buf, i);
        println!("Service ID: {}", service_id);

        // Call the handler for the service. It should return a u8 vector payload.
        let payload: Vec<u8> = match service_id {
            1 => get_flight_ids_handler(&buf[i..], &flight_db),
            2 => get_flight_summary_handler(&buf[i..], &flight_db),
            3 => reserve_seats_handler(&buf[i..], &mut flight_db, &mut watchlist_db, &socket),
            4 => monitor_seat_availability_handler(
                &buf[i..],
                &mut flight_db,
                &mut watchlist_db,
                &client_addr,
            ),
            _ => {
                println!("Error: The handler byte is not 1, 2, 3, or 4.");
                vec![]
            }
        };

        // Add to the response cache.
        response_cache.insert(
            request_id,
            ResponseCache {
                request_id,
                response_payload: payload.clone(),
            },
        );

        networking::send_response(request_id, payload, &socket, &client_addr)
    }
}

fn error_handler(error_message: &str) -> Vec<u8> {
    println!("Preparing error response: {error_message}");

    // Create a buffer to store the data to send with capasity 2048 bytes
    let mut buffer_to_send: Vec<u8> = Vec::with_capacity(2048);

    // Add the status/service ID as the first byte. 0 means error.
    buffer_to_send.push(0);

    // Add the error message to the buffer.
    marshal_string(error_message, &mut buffer_to_send);

    buffer_to_send
}
fn get_flight_ids_handler(buf: &[u8], flight_db: &HashMap<u32, Flight>) -> Vec<u8> {
    // Read the source and destination from the buffer.
    let (source, i) = unmarshal_string(buf, 0);
    let (destination, _) = unmarshal_string(buf, i);

    // Get the flight IDs from the hashmap by searching every entry in the hashmap.
    let flight_ids = flight_db
        .values()
        .filter(|flight| flight.source == source && flight.destination == destination)
        .map(|flight| flight.id)
        .collect::<Vec<u32>>();

    // If no flight IDs, then call error handler.
    if flight_ids.is_empty() {
        return error_handler(
            "No flight identifiers (IDs) found for the given source and destination.",
        );
    }

    // Create a buffer to store the data to send with capasity 2048 bytes
    let mut buffer_to_send: Vec<u8> = Vec::with_capacity(2048);

    // Add the handler byte.
    buffer_to_send.push(1);

    // Add the flight IDs to the buffer.
    marshal_u32_array(&flight_ids, &mut buffer_to_send);

    buffer_to_send
}

fn get_flight_summary_handler(buf: &[u8], flight_db: &HashMap<u32, Flight>) -> Vec<u8> {
    // Read the flight ID from the buffer.
    let (flight_id, _) = unmarshal_u32(buf, 0);

    // Get the flight from the hashmap.
    let result = flight_db.get(&flight_id);

    // If None, then call error handler.
    if result.is_none() {
        return error_handler("No flight found for the given flight ID.");
    }

    // Get the flight from the result.
    let flight = result.unwrap();

    // Create a buffer to store the data to send with capasity 2048 bytes
    let mut buffer_to_send: Vec<u8> = Vec::with_capacity(2048);

    // Add the handler byte.
    buffer_to_send.push(2);

    // Add the departure time, airfare and seats to the buffer.
    marshal_u32(flight.departure_time, &mut buffer_to_send);
    marshal_f32(flight.airfare, &mut buffer_to_send);
    marshal_u32(flight.seats, &mut buffer_to_send);

    buffer_to_send
}

fn reserve_seats_handler(
    buf: &[u8],
    flight_db: &mut HashMap<u32, Flight>,
    watchlist_db: &mut HashMap<u32, Vec<WatchlistEntry>>,
    socket: &UdpSocket,
) -> Vec<u8> {
    // Read id and num_seats from buf.
    let (flight_id, i) = unmarshal_u32(buf, 0);
    let (num_seats, _) = unmarshal_u32(buf, i);

    // Try to reserve the seats.
    if !flight_db.contains_key(&flight_id) {
        return error_handler("No flight found for the given flight ID.");
    }

    let flight = flight_db.get_mut(&flight_id).unwrap();

    let reservation_success = flight.reserve_seats(num_seats);

    if !reservation_success {
        println!("Reservation failed.");
        let current_seats = flight.seats;
        return error_handler(&format!("Not enough seats available. You tried to reserve {num_seats} seats, but there are only {current_seats} seats available."));
    }

    if watchlist_db.contains_key(&flight_id) {
        // First, go through each entry and obtain a cleaned vector of entries.
        // A cleaned vector of entries is a vector of entries that have not expired.

        let mut cleaned_watchlist = Vec::<WatchlistEntry>::new();

        for entry in watchlist_db.get(&flight_id).unwrap() {
            if u64::from(entry.0)
                > SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            {
                cleaned_watchlist.push(entry.clone());
            }
        }

        // Then, go through each entry of the cleaned vector and call inform_client.
        for entry in cleaned_watchlist.iter() {
            inform_client(socket, entry.1, flight_id, flight.seats);
        }

        // Update the watchlist
        watchlist_db.insert(flight_id, cleaned_watchlist);
    };

    // Create a buffer to store the data to send with capacity 2048 bytes
    let mut buffer_to_send: Vec<u8> = Vec::with_capacity(2048);

    // Add the handler byte.
    buffer_to_send.push(3);

    // Add 1 if successful.
    buffer_to_send.push(1);

    buffer_to_send
}

fn monitor_seat_availability_handler(
    buf: &[u8],
    flight_db: &mut HashMap<u32, Flight>,
    watchlist_db: &mut HashMap<u32, Vec<WatchlistEntry>>,
    client_addr: &SocketAddr,
) -> Vec<u8> {
    // Read id and monitor interval from buf.
    let (flight_id, i) = unmarshal_u32(buf, 0);
    let (monitor_interval, _) = unmarshal_u32(buf, i);

    // Check if the flight exists.
    if !flight_db.contains_key(&flight_id) {
        return error_handler("No flight found for the given flight ID.");
    }

    // Add the entry to the watchlist. Note: Converting u64 to u32 here is possibly unsafe, but we are relying on client to limit duration to 1 year (31536000).
    let entry = WatchlistEntry(
        (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + u64::from(monitor_interval))
        .try_into()
        .unwrap(),
        client_addr.clone(),
    );

    // Just append the entry to the watchlist.
    if watchlist_db.contains_key(&flight_id) {
        let watchlist = watchlist_db.get_mut(&flight_id).unwrap();
        watchlist.push(entry);
    } else {
        let mut watchlist = Vec::<WatchlistEntry>::new();
        watchlist.push(entry);
        watchlist_db.insert(flight_id, watchlist);
    }

    println!("Added entry to watchlist.");

    // Create a buffer to store the data to send with capacity 2048 bytes
    let mut buffer_to_send: Vec<u8> = Vec::with_capacity(2048);

    // Add the handler byte.
    buffer_to_send.push(4);

    // Add 1 if successful.
    buffer_to_send.push(1);

    buffer_to_send
}

// Sends a message to the socket to update them of the number of seats available.
fn inform_client(socket: &UdpSocket, client_addr: SocketAddr, flight_id: u32, seats: u32) {
    // Create a buffer to store the data to send with capacity 2048 bytes
    let mut buffer_to_send: Vec<u8> = Vec::with_capacity(2048);

    // Add the handler byte.
    buffer_to_send.push(4);

    // Add the flight ID and seats to the buffer.
    marshal_u32(flight_id, &mut buffer_to_send);
    marshal_u32(seats, &mut buffer_to_send);

    println!(
        "Informing client: {}, flight_id: {}, seats: {}",
        client_addr, flight_id, seats
    );
    // Send the message to the client.
    socket
        .send_to(&buffer_to_send, &client_addr)
        .expect("Error on send");
}
