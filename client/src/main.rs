use std::error::Error;
use std::io::{self, BufRead, Lines, StdinLock};
use std::net::{UdpSocket, SocketAddr};
// use std::env;
use std::{str, env};
use std::time::Duration;

use chrono::{DateTime, Utc, NaiveDateTime, Local, TimeZone};
use marshaling::{
    self, marshal_string, marshal_u32, marshal_u8, unmarshal_string, unmarshal_u32, unmarshal_u8, unmarshal_u32_array, unmarshal_f32,
};
use networking;

fn main() -> std::io::Result<()> {
    
    let addrs = [
        SocketAddr::from(([127, 0, 0, 1], 7879)),
        SocketAddr::from(([127, 0, 0, 1], 7880)),
        SocketAddr::from(([127, 0, 0, 1], 7881)),
    ];

    let server_addr = SocketAddr::from(([127, 0, 0, 1], 7878));
    let socket = UdpSocket::bind(&addrs[..])?; // for UDP4/6
    socket
        .set_read_timeout(Some(Duration::new(5, 0)))
        .expect("Failed to set read timeout");
    socket
        .connect(&server_addr)
        .expect("couldn't connect to address");

    let stdin = io::stdin();

    // Acquire a lock on stdin
    let mut lines = stdin.lock().lines();

    // Create a buffer to store the data to send
    let mut buf = vec![0; 2048];

    // Create a buffer to store the data received from the server
    let mut receive_buf = [0; 2048];

    // Counter to keep track of the number of requests sent
    let mut request_id: u32 = 0;

    loop {
        // reset buf
        buf.clear();

        // Prompt the user to choose between 4 services.
        println!("\n");
        println!("=============================");
        println!("Choose a service:");
        println!("1. Get Flight Identifiers");
        println!("2. Get Flight Summary");
        println!("3. Reserve Seats");
        println!("4. Monitor Seat Availability");
        println!("5. Exit");

        // Read input from stdin and interpret it as a u32
        let service_choice = lines
            .next()
            .expect("Error on iteration")
            .expect("Error on read");

        // Debug message to show which service was read from stdin
        dbg!(&service_choice);

        // Convert the service choice to a u32
        let service_choice: u8 = service_choice
            .parse::<u8>()
            .expect("Error on parsing user's service choice");

        // Match the service choice to the appropriate service
        // Each service will return a byte array that will be sent to the server. Size is at most 2048 bytes.
        let buffer_to_send: Vec<u8> = match service_choice {
            1 => {
                match prepare_get_flight_identifiers(&mut lines) {
                    Ok(buffer) => buffer,
                    Err(e) => {
                        println!("Error: {}", e);
                        continue;
                    }
                }
            }
            2 => {
                match prepare_get_flight_summary(&mut lines) {
                    Ok(buffer) => buffer,
                    Err(e) => {
                        println!("Error: {}", e);
                        continue;
                    }
                }
            }
            3 => {
                match prepare_reserve_seats(&mut lines) {
                    Ok(buffer) => buffer,
                    Err(e) => {
                        println!("Error: {}", e);
                        continue;
                    }
                }
            }
            4 => {
                match prepare_monitor_seat_availability(&mut lines) {
                    Ok(buffer) => buffer,
                    Err(e) => {
                        println!("Error: {}", e);
                        continue;
                    }
                }
            }
            5 => {
                // Exit the program
                break;
            }
            _ => {
                // The user entered an invalid choice
                println!("Invalid choice, please try again.");

                // Skip the rest of the loop
                continue;
            }
        };

        println!("Sending {} bytes", buffer_to_send.len());
        // Send the buffer to the communication service which will handle communication with the server. Specify the request ID, the buffer to send and the socket.
        // Increment the request ID
        request_id += 1;
        send_request(request_id, buffer_to_send, &socket, &server_addr);

        // Receive from the server and print out the response. If error, print out error.
        let (amt, src) = match socket.recv_from(&mut receive_buf) {
            Ok((amt, src)) => {
                println!("Received {} bytes from {}", amt, src);
                (amt, src)
            }
            Err(_) => {
                // TODO: Retry depending on invocation semantics
                println!("Timed out waiting for response from the server");
                break;
            }
        };

        // Handle the response
        let i: usize = 0;
        // Check if request ID is the same as the one we sent, if not, keep waiting.
        let (received_request_id, i) = unmarshal_u32(&receive_buf, i);
        if received_request_id != request_id {
            println!("Request ID not the same as the one we sent. Continuing to wait.");
            continue;
        }

        // Check next byte and call specific handler
        let (handler_byte, i) = unmarshal_u8(&receive_buf, i);

        dbg!(handler_byte);

        match handler_byte {
            0 => {
                // Next bytes will be a string which is the error message.
                let (error_message, _) = unmarshal_string(&receive_buf, i);
                println!("Error: {}", error_message);
            }
            1 => {
                parse_get_flight_identifiers_response(&receive_buf[i..amt]);
            }
            2 => {
                parse_get_flight_summary_response(&receive_buf[i..amt]);
            }
            3 => {
                parse_reserve_seats_response(&receive_buf[i..amt])
            }
            4 => {
                parse_monitor_seat_availability_response(&receive_buf[i..amt]);
            }
            _ => {
                println!("Invalid handler byte");
            }
        }

    }

    Ok(())
}

fn parse_get_flight_identifiers_response(buf: &[u8]) {
    let (flight_ids, i) = unmarshal_u32_array(buf, 0);
    println!("Flight IDs: {:#?}", flight_ids);
}

fn parse_get_flight_summary_response(buf: &[u8]) {
    let (departure_time, i) = unmarshal_u32(buf, 0);
    let (airfare, i) = unmarshal_f32(buf, i);
    let (seats, i) = unmarshal_u32(buf, i);
    println!("Departure time: {}", convert_unix_time_to_datetime(departure_time).to_string());
    println!("Airfare: {}", airfare);
    println!("Seats: {}", seats);
}

fn parse_reserve_seats_response(buf: &[u8]) {
    let (has_succeeded, _) = unmarshal_u8(buf, 0);
    if has_succeeded == 1 {
        println!("Reservation succeeded");
    } else {
        // This should not be reachable because any error will be already caught by the handler byte being 0.
        println!("Reservation failed");
    }
}

fn parse_monitor_seat_availability_response(buf: &[u8]) {
    let (has_succeeded, _) = unmarshal_u8(buf, 0);
    if has_succeeded == 1 {
        println!("Subscription succeeded");
    } else {
        // This should not be reachable because any error will be already caught by the handler byte being 0.
        println!("Subscription failed");
    }
}

// Might return errors from IO, or from bad user input.
fn prepare_get_flight_identifiers(std_in_reader: &mut Lines<StdinLock>) -> Result<Vec<u8>, Box<dyn Error>> {
    const GET_FLIGHT_IDENTIFIERS_SERVICE_ID: u8 = 1;
    // Gets input from user for source and destination.
    println!("Enter source:");
    let source = std_in_reader
        .next().unwrap()?;

    println!("Enter destination:");
    let destination = std_in_reader
        .next().unwrap()?;

    // Create a buffer to store the data to send with capasity 2048 bytes
    let mut buffer_to_send: Vec<u8> = Vec::with_capacity(2048);

    // Add service ID
    marshal_u8(GET_FLIGHT_IDENTIFIERS_SERVICE_ID, &mut buffer_to_send);

    // Add source
    marshal_string(&source, &mut buffer_to_send);

    // Add destination
    marshal_string(&destination, &mut buffer_to_send);

    // Return the buffer
    Ok(buffer_to_send)
}

fn prepare_get_flight_summary(std_in_reader: &mut Lines<StdinLock>) -> Result<Vec<u8>, Box<dyn Error>> {
    const GET_FLIGHT_SUMMARY_SERVICE_ID: u8 = 2;

    // Gets input from user for flight ID.
    println!("Enter flight identifier:");
    let flight_id = std_in_reader
        .next()
        .unwrap()?;

    // Convert the flight ID to a u32
    let flight_id = flight_id
        .parse::<u32>()?;

    // Create a buffer to store the data to send with capacity 2048 bytes
    let mut buffer_to_send: Vec<u8> = Vec::with_capacity(2048);

    // Add service ID as first byte
    marshal_u8(GET_FLIGHT_SUMMARY_SERVICE_ID, &mut buffer_to_send);

    // Add flight ID
    marshal_u32(flight_id, &mut buffer_to_send);

    // Return the buffer
    Ok(buffer_to_send)
}

fn prepare_reserve_seats(std_in_reader: &mut Lines<StdinLock>) -> Result<Vec<u8>, Box<dyn Error>> {
    const RESERVE_SEATS_SERVICE_ID: u8 = 3;

    // Gets input from user for flight ID.
    println!("Enter flight identifier:");
    let flight_id = std_in_reader
        .next()
        .unwrap()?;

    // Gets input from user for number of seats.
    println!("Enter number of seats to reserve:");
    let seats = std_in_reader
        .next()
        .unwrap()?;

    // Convert the flight ID to a u32
    let flight_id = match flight_id
        .parse::<u32>() {
            Ok(flight_id) => flight_id,
            Err(_) => {
                println!("Invalid flight ID");
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid flight ID")));
            }
        };

    // Convert the number of seats to a u32
    let seats = match seats
        .parse::<u32>() {
            Ok(seats) => seats,
            Err(_) => {
                println!("Invalid number of seats");
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid number of seats")));
            }
        };

    // Create a buffer to store the data to send with capacity 2048 bytes
    let mut buffer_to_send: Vec<u8> = Vec::with_capacity(2048);

    // Add service ID as first byte
    marshal_u8(RESERVE_SEATS_SERVICE_ID, &mut buffer_to_send);

    // Add flight ID
    marshal_u32(flight_id, &mut buffer_to_send);

    // Add number of seats
    marshal_u32(seats, &mut buffer_to_send);

    // Return the buffer
    Ok(buffer_to_send)
}

fn prepare_monitor_seat_availability(std_in_reader: &mut Lines<StdinLock>) -> Result<Vec<u8>, Box<dyn Error>> {
    const MONITOR_SEAT_AVAILABILITY_SERVICE_ID: u8 = 4;

    // Gets input from user for flight ID.
    println!("Enter flight identifier:");
    let flight_id = std_in_reader
        .next()
        .unwrap()?;

    // Gets input from user for monitor_interval.
    const SECONDS_IN_YEAR: u32 = 31536000;
    println!("Enter monitor interval, up to {} seconds (1 year):", SECONDS_IN_YEAR);
    let monitor_interval = std_in_reader
        .next()
        .unwrap()?;

    // Convert the flight ID to a u32
    let flight_id = match flight_id
        .parse::<u32>() {
            Ok(flight_id) => flight_id,
            Err(_) => {
                println!("Invalid flight ID");
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid flight ID")));
            }
        };

    // Check if monitor interval fits in a u32
    if monitor_interval.len() > 10 {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Monitor interval is too big.")));
    }
    // Convert the monitor interval to a u32
    let monitor_interval = match monitor_interval
        .parse::<u32>() {
            Ok(monitor_interval) => monitor_interval,
            Err(_) => {
                println!("Invalid monitor interval");
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid monitor interval")));
            }
        };

    if monitor_interval > SECONDS_IN_YEAR {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Monitor interval is too big.")));
    }

    // Create a buffer to store the data to send with capacity 2048 bytes
    let mut buffer_to_send: Vec<u8> = Vec::with_capacity(2048);

    // Add service ID as first byte
    marshal_u8(MONITOR_SEAT_AVAILABILITY_SERVICE_ID, &mut buffer_to_send);

    // Add flight ID
    marshal_u32(flight_id, &mut buffer_to_send);

    // Add monitor interval
    marshal_u32(monitor_interval, &mut buffer_to_send);

    // Return the buffer
    Ok(buffer_to_send)
}
fn send_request(request_id: u32, payload: Vec<u8>, socket: &UdpSocket, server_addr: &SocketAddr) {
    // Create a buffer to store the data to send with capasity 2048 bytes
    let mut buffer_to_send: Vec<u8> = Vec::with_capacity(2048);

    // Add request ID as the first byte for the server to differentiate requests from multiple clients.
    // Different from Service ID which is already handled by the respective `prepare` functions
    buffer_to_send.extend_from_slice(&request_id.to_be_bytes());

    // Add payload to buffer
    buffer_to_send.extend_from_slice(&payload);

    socket
        .send_to(&buffer_to_send, &server_addr)
        .expect("Error on send");
}

pub fn convert_unix_time_to_datetime(timestamp: u32)  -> DateTime<Local> {
    let naive = NaiveDateTime::from_timestamp_opt(timestamp as i64, 0).unwrap();
    Local.from_local_datetime(&naive).unwrap()
}