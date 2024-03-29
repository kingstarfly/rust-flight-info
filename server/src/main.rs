use std::{
    collections::HashMap,
    fmt,
    net::{SocketAddr, UdpSocket},
    time::{SystemTime, UNIX_EPOCH},
};

use marshaling::{
    self, marshal_f32, marshal_string, marshal_u32, marshal_u32_array, unmarshal_string,
    unmarshal_u32, unmarshal_u8,
};
use networking;

struct Flight {
    id: u32,
    source: String,
    destination: String,
    departure_time: u32, // Unix time
    seats: u32,
    airfare: f32,
    baggage_capacity_kg: u32,
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

    fn reserve_baggage(&mut self, baggage_kg: u32) -> bool {
        if self.baggage_capacity_kg >= baggage_kg {
            self.baggage_capacity_kg -= baggage_kg;
            true
        } else {
            false
        }
    }
}

struct ResponseCacheValue {
    response_payload: Vec<u8>,
}

#[derive(Eq, Hash, PartialEq)]
struct ResponseCacheKey {
    request_id: u32,
    client_addr: SocketAddr,
}

#[derive(Eq, Hash, PartialEq, Clone)]
struct WatchlistEntry(u32, SocketAddr);

#[derive(PartialEq)]
enum InvocationSemantics {
    AtLeastOnce,
    AtMostOnce,
}

impl fmt::Debug for InvocationSemantics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InvocationSemantics::AtLeastOnce => write!(f, "at-least-once"),
            InvocationSemantics::AtMostOnce => write!(f, "at-most-once"),
        }
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let usage_message = "Usage: cargo run --bin server alo|amo true|false";

    // If the number of arguments is not 2, then print usage and exit.
    if args.len() < 3 {
        println!("{}", usage_message);
        return Ok(());
    }

    // If the second argument is not 'alo' or 'amo', then print usage and exit.
    let invocation_semantics = &args[1];
    if invocation_semantics != "alo" && invocation_semantics != "amo" {
        println!("{}", usage_message);
        return Ok(());
    }

    let simulate_failure = &args[2];
    if simulate_failure != "true" && simulate_failure != "false" {
        println!("{}", usage_message);
        return Ok(());
    }

    // Parse the invocation semantics.
    let invocation_semantics = match invocation_semantics.as_str() {
        "alo" => InvocationSemantics::AtLeastOnce,
        "amo" => InvocationSemantics::AtMostOnce,
        _ => {
            println!("Error: The invocation semantics is not 'alo' or 'amo'.");
            return Ok(());
        }
    };

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
            baggage_capacity_kg: 1000,
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
            baggage_capacity_kg: 1000,
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
            baggage_capacity_kg: 1000,
        },
    );

    // Build a hashmap of flight ID to a vector of WatchlistEntry
    let mut watchlist_db: HashMap<u32, Vec<WatchlistEntry>> = HashMap::new();

    // Build a hashmap of request ID to a ResponseCache
    let mut response_cache: HashMap<ResponseCacheKey, ResponseCacheValue> = HashMap::new();

    let socket = UdpSocket::bind("127.0.0.1:7878")?;
    println!(
        "-- Server is listening on port {}",
        socket.local_addr().unwrap().port()
    );
    println!("\n\nInvocation semantics = {:?}", invocation_semantics);

    let mut buf = [0; 2048];

    // Convert arg[2] to a boolean
    let should_simulate_failure = args[2].parse::<bool>().unwrap();

    let mut should_fail_next = true;

    loop {
        // Receives a single datagram message on the socket.
        // If `buf` is too small to hold
        // the message, it will be cut off.
        let (_, client_addr) = socket.recv_from(&mut buf)?;

        // Read the request ID in the first 4 bytes.
        let i: usize = 0;
        let (request_id, i) = unmarshal_u32(&buf, i);
        println!(
            "[server] Received Request ID: {} from Client: {}",
            request_id, client_addr
        );

        // Check if the request ID is in the response cache.
        // If it is, then use the cached payload.
        // Or else, read service ID and call handler

        let response_cache_key = ResponseCacheKey {
            request_id,
            client_addr,
        };

        // If the invocation semantics is at most once, then check the response cache.
        if invocation_semantics == InvocationSemantics::AtMostOnce {
            if response_cache.contains_key(&response_cache_key) {
                // If the request ID is in the response cache, then send the cached payload.
                println!(
                    "[server] Cache HIT for request ID {} from Client: {}",
                    request_id, client_addr
                );
                networking::send_response(
                    request_id,
                    response_cache
                        .get(&response_cache_key)
                        .unwrap()
                        .response_payload
                        .clone(),
                    &socket,
                    &client_addr,
                    should_simulate_failure && should_fail_next,
                );
                // Toggle the should_simulate_failure flag.
                should_fail_next = !should_fail_next;
                continue;
            } else {
                println!(
                    "[server] Cache MISS for request ID {} for Client: {}",
                    request_id, client_addr
                );
            }
        }

        // Read the service ID in the next byte.
        let (service_id, i) = unmarshal_u8(&buf, i);
        print!("[server] Handling Service {}...", service_id);

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
            5 => get_earliest_flight_ids(&buf[i..], &flight_db),
            6 => reserve_baggage_handler(&buf[i..], &mut flight_db),
            _ => {
                println!("Error: Handler byte is not 1-6.");
                vec![]
            }
        };

        println!("Done!");
        // Add to the response cache.
        response_cache.insert(
            ResponseCacheKey {
                request_id,
                client_addr,
            },
            ResponseCacheValue {
                response_payload: payload.clone(),
            },
        );

        networking::send_response(
            request_id,
            payload,
            &socket,
            &client_addr,
            should_simulate_failure && should_fail_next,
        );

        // Toggle the should_simulate_failure flag.
        should_fail_next = !should_fail_next;
    }
}

fn error_handler(error_message: &str) -> Vec<u8> {
    println!("Preparing error response: {error_message}");

    // Create a buffer to store the data to send with capacity 2048 bytes
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

    // Create a buffer to store the data to send with capacity 2048 bytes
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

    // Create a buffer to store the data to send with capacity 2048 bytes
    let mut buffer_to_send: Vec<u8> = Vec::with_capacity(2048);

    // Add the handler byte.
    buffer_to_send.push(2);

    // Add the departure time, airfare, seats and remaining baggage capacity to the buffer.
    marshal_u32(flight.departure_time, &mut buffer_to_send);
    marshal_f32(flight.airfare, &mut buffer_to_send);
    marshal_u32(flight.seats, &mut buffer_to_send);
    marshal_u32(flight.baggage_capacity_kg, &mut buffer_to_send);

    buffer_to_send
}

fn get_earliest_flight_ids(buf: &[u8], flight_db: &HashMap<u32, Flight>) -> Vec<u8> {
    // Read the source from the buffer
    let (source, _) = unmarshal_string(buf, 0);

    // Get flights from flights_db with source and seats > 0
    // Using the minimum departure time from these flights, only select flights with that departure time
    // Return the flight IDs of these flights
    let valid_flights = flight_db
        .values()
        .filter(|flight| flight.source == source && flight.seats > 0)
        .collect::<Vec<&Flight>>();
    
    let earliest_time = valid_flights.iter().map(|flight| flight.departure_time).min().unwrap();

    let earliest_flight_ids = valid_flights
        .iter()
        .filter(|flight| flight.departure_time == earliest_time)
        .map(|flight| flight.id)
        .collect::<Vec<u32>>();

    // Sort the flight IDs in asc order.
    let mut earliest_flight_ids = earliest_flight_ids;
    earliest_flight_ids.sort();

    // Create a buffer to store the data to send with capacity 2048 bytes
    let mut buffer_to_send: Vec<u8> = Vec::with_capacity(2048);

    // Add the handler byte.
    buffer_to_send.push(5);

    // Add only the flight IDs to the buffer.
    marshal_u32_array(&earliest_flight_ids, &mut buffer_to_send);

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
    if !watchlist_db.contains_key(&flight_id) {
        let watchlist = Vec::<WatchlistEntry>::new();
        watchlist_db.insert(flight_id, watchlist);
    } 
    let watchlist = watchlist_db.get_mut(&flight_id).unwrap();

    // Go through each entry and remove any entries with the same client address.
    watchlist.retain(|entry| entry.1 != *client_addr);
    watchlist.push(entry);

    println!("Added entry to watchlist.");

    // Create a buffer to store the data to send with capacity 2048 bytes
    let mut buffer_to_send: Vec<u8> = Vec::with_capacity(2048);

    // Add the handler byte.
    buffer_to_send.push(4);

    // Add 1 if successful.
    buffer_to_send.push(1);

    buffer_to_send
}

fn reserve_baggage_handler(
    buf: &[u8],
    flight_db: &mut HashMap<u32, Flight>,
) -> Vec<u8> {
    // Read id and baggage weight to reserve in kg from buf.
    let (flight_id, i) = unmarshal_u32(buf, 0);
    let (baggage_weight, _) = unmarshal_u32(buf, i);

    // Try to reserve the baggage.
    if !flight_db.contains_key(&flight_id) {
        return error_handler("No flight found for the given flight ID.");
    }

    let flight = flight_db.get_mut(&flight_id).unwrap();

    let reservation_success = flight.reserve_baggage(baggage_weight);

    if !reservation_success {
        println!("Reservation of baggage failed.");
        let current_baggage_capacity = flight.baggage_capacity_kg;
        return error_handler(&format!("There is not enough baggage capacity. You tried to reserve {baggage_weight} kg of baggage, but there are only {current_baggage_capacity} kg of baggage remaining."));
    }

    // Create a buffer to store the data to send with capacity 2048 bytes
    let mut buffer_to_send: Vec<u8> = Vec::with_capacity(2048);

    // Add the handler byte.
    buffer_to_send.push(6);

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
