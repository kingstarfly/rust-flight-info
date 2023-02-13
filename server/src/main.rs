//use std::io::{self, Read, Write, BufRead};
use std::{net::{UdpSocket, SocketAddr}, collections::HashMap};
//use std::env;
//use std::str;

use marshaling::{self, unmarshal_f32, unmarshal_string, unmarshal_u32, unmarshal_u8, marshal_string, marshal_u32_array, marshal_u8, marshal_u32, marshal_f32};
use networking;
struct Flight {
    id: u32,
    source: String,
    destination: String,
    departure_time: u32, // Unix time
    seats: u32,
    airfare: f32,
}
fn main() -> std::io::Result<()> {
    
    
    let flight_db = [
        Flight {
            id: 1,
            source: "SFO".to_string(),
            destination: "LAX".to_string(),
            departure_time: 1700000000,
            seats: 10,
            airfare: 10.1,
        },
        Flight {
            id: 2,
            source: "SFO".to_string(),
            destination: "LAX".to_string(),
            departure_time: 1700000000,
            seats: 20,
            airfare: 20.2,
        },
        Flight {
            id: 3,
            source: "AUS".to_string(),
            destination: "CHN".to_string(),
            departure_time: 1700000000,
            seats: 30,
            airfare: 30.3,
        },
    ];

    let src_dest_id_map: HashMap<(String, String), Vec<u32>> = flight_db
        .iter()
        .map(|flight| {
            (
                (flight.source.clone(), flight.destination.clone()),
                flight.id,
            )
        })
        .fold(HashMap::new(), |mut acc, (src_dest, id)| {
            acc.entry(src_dest)
                .and_modify(|ids| ids.push(id))
                .or_insert(vec![id]);
            acc
        });

    let flight_id_map: HashMap<u32, &Flight> = flight_db
        .iter()
        .map(|flight| (flight.id, flight.clone()))
        .collect();



    struct WatchlistEntry(u32, SocketAddr);
    // Build a hashmap of flight ID to a vector of WatchlistEntry
    let mut watchlist: HashMap<u32, Vec<WatchlistEntry>> = HashMap::new();

    let socket = UdpSocket::bind("127.0.0.1:7878")?;
    // TODO: Set timeout for read?
    let mut buf = [0; 2048];

    loop {
        // Receives a single datagram message on the socket.
        // If `buf` is too small to hold
        // the message, it will be cut off.
        let (amt, client_addr) = socket.recv_from(&mut buf)?;

        // Prints out a received bytes
        println!("Received {} bytes from {}", amt, client_addr);

        // Read the request ID in the first 4 bytes.
        let i: usize = 0;
        let (request_id, i) = unmarshal_u32(&buf, i);
        println!("Request ID: {}", request_id);

        // Read the service ID in the next byte.
        let (service_id, i) = unmarshal_u8(&buf, i);
        println!("Service ID: {}", service_id);

        // Call the handler for the service. It should return a u8 vector payload.
        let payload: Vec<u8> = match service_id {
            0 => {
                // TODO 0: Error response handler
                // Next bytes will be a string which is the error message.
                let (error_message, _) = unmarshal_string(&buf, i);
                println!("Error: {}", error_message);
                vec![]
            }
            1 => {
                get_flight_ids_handler(&buf[i..], &src_dest_id_map)
            }
            2 => {
                get_flight_summary_handler(&buf[i..], &flight_id_map)
            }
            3 => {
                // TODO 3: Call the Reserve Seats service
                println!("TODO: Service 3 handler");
                vec![]
            }
            4 => {
                // TODO 4: Call the Monitor Seat Availability service
                println!("TODO: Service 4 handler");
                vec![]
            }
            _ => {
                println!("Error: The handler byte is not 0, 1, 2, 3, or 4.");
                vec![]
            }
        };

        // Send the response back to the client after prepending some information.
        // Create a buffer to store the data to send with capasity 2048 bytes
        let mut buffer_to_send: Vec<u8> = Vec::with_capacity(2048);

        // Add request ID as the first byte for the client to check if the correct response was received.
        buffer_to_send.extend_from_slice(&request_id.to_be_bytes());
        print!("Appending request ID: {} ", request_id);

        // Add payload to buffer
        buffer_to_send.extend_from_slice(&payload);
        println!("Appending payload: {:?}", payload);

        println!("Sending {} bytes to {}", buffer_to_send.len(), client_addr);
        socket
            .send_to(&buffer_to_send, &client_addr)
            .expect("Error on send");

        // Done.
    }
}

fn error_handler(error_message: &str) -> Vec<u8> {
    // Create a buffer to store the data to send with capasity 2048 bytes
    let mut buffer_to_send: Vec<u8> = Vec::with_capacity(2048);

    // Add the status/service ID as the first byte. 0 means error.
    buffer_to_send.push(0);

    // Add the error message to the buffer.
    marshal_string(error_message, &mut buffer_to_send);

    buffer_to_send
}
fn get_flight_ids_handler(
    buf: &[u8],
    src_dest_id_map: &HashMap<(String, String), Vec<u32>>,
) -> Vec<u8> {
    // Read the source and destination from the buffer.
    let (source, i) = unmarshal_string(buf, 0);
    let (destination, _) = unmarshal_string(buf, i);

    // Get the flight IDs from the hashmap.
    let result = src_dest_id_map.get(&(source, destination));    

    // If None, then call error handler.
    if result.is_none() {
        return error_handler("No flight identifiers (IDs) found for the given source and destination.");
    }

    // Get the flight IDs from the result.
    let flight_ids = result.unwrap();

    // Create a buffer to store the data to send with capasity 2048 bytes
    let mut buffer_to_send: Vec<u8> = Vec::with_capacity(2048);

    // Add the handler byte.
    buffer_to_send.push(1);

    // Add the flight IDs to the buffer.
    marshal_u32_array(&flight_ids, &mut buffer_to_send);

    buffer_to_send
}

fn get_flight_summary_handler(buf: &[u8], flight_id_map: &HashMap<u32, &Flight>) -> Vec<u8> {
    // Read the flight ID from the buffer.
    let (flight_id, _) = unmarshal_u32(buf, 0);

    // Get the flight from the hashmap.
    let result = flight_id_map.get(&flight_id);

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