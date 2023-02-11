use std::io::{self, BufRead, Lines, StdinLock};
use std::net::UdpSocket;
// use std::env;
use std::str;

use marshaling;
use networking;

fn main() -> std::io::Result<()> {
    // let args: Vec<String> = env::args().collect();
    // if args.len() < 2 {
    //     println!("Usage {} hostname", args[0]);
    //     std::process::exit(1);
    // }
    // let hostname = &args[1];
    let hostname = "127.0.0.1";
    let server_addr = hostname.to_string() + &":7878";

    let socket = UdpSocket::bind("127.0.0.1:7879")?; // for UDP4/6
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
        let service_choice = service_choice
            .parse::<u32>()
            .expect("Error on parsing user's service choice");

        // Match the service choice to the appropriate service
        // Each service will return a byte array that will be sent to the server. Size is at most 2048 bytes.
        let buffer_to_send: Vec<u8> = match service_choice {
            1 => {
                // TODO 1: Call the Get Flight Identifiers service
                prepare_get_flight_identifiers(&mut lines)
            }
            2 => {
                // TODO 2: Call the Get Flight Summary service
                prepare_get_flight_summary(&mut lines)
            }
            3 => {
                // TODO 3: Call the Reserve Seats service
                // Placeholder return
                vec![0; 2048]
            }
            4 => {
                // TODO 4: Call the Monitor Seat Availability service
                // Placeholder return
                vec![0; 2048]
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

        // Send the buffer to the communication service which will handle communication with the server. Specify the request ID, the buffer to send and the socket.
        send_request(request_id, buffer_to_send, &socket, &server_addr);

        // Increment the request ID
        request_id += 1;

        /*
        // Read a string from stdin
        println!("Enter a string (BYE to exit):");
        let line = lines.next();
        let line = line.expect("Error on iteration").expect("Error on read");

        println!("Paramter 1 read from stdin '{}'", line);
        if &line == "BYE" {
            break;
        }
        marshal_string(&line, &mut buf);

        // Read input from stdin and interpret it as a u32
        println!("Enter a u32:");
        let line = lines.next();
        let line = line.expect("Error on iteration").expect("Error on read");
        println!("Parameter 2 read from stdin '{}'", line);

        let param2 = line.parse::<u32>().expect("Error on parse");
        marshal_u32(param2, &mut buf);

        // Read input from stdin and interpret it as a f32
        println!("Enter a f32:");
        let line = lines.next();
        let line = line.expect("Error on iteration").expect("Error on read");
        println!("Parameter 3 read from stdin '{}'", line);

        let param3 = line.parse::<f32>().expect("Error on parse");
        marshal_f32(param3, &mut buf);

        // Send the buffer to the server
        socket.send_to(&buf, &server_addr).expect("Error on send");

        // Receive the echo from the server
        let (amt, _src) = socket.recv_from(&mut receive_buf).expect("Error on recv");

        // Print the size of the received buffer
        println!("Received {} bytes", amt);

        let echo = str::from_utf8(&receive_buf[..amt]).unwrap();
        println!("Echo {}", echo);
        */
    }

    Ok(())
}

fn prepare_get_flight_identifiers(
    mut std_in_reader: &mut Lines<StdinLock>,
) -> Vec<u8> {
    const GET_FLIGHT_IDENTIFIERS_SERVICE_ID: u8 = 1;
    // Gets input from user for source and destination.
    println!("Enter source:");
    let source = std_in_reader
        .next()
        .expect("Error on iteration")
        .expect("Error on read");
    println!("Enter destination:");
    let destination = std_in_reader
        .next()
        .expect("Error on iteration")
        .expect("Error on read");

    // Create a buffer to store the data to send
    let mut buffer_to_send: Vec<u8> = vec![0; 2048];

    // Add service ID 
    marshal_u8(GET_FLIGHT_IDENTIFIERS_SERVICE_ID, &mut buffer_to_send);

    // Add source
    marshal_string(&source, &mut buffer_to_send);

    // Add destination
    marshal_string(&destination, &mut buffer_to_send);

    // Return the buffer
    buffer_to_send
}

fn prepare_get_flight_summary(mut std_in_reader: &mut Lines<StdinLock>) -> Vec<u8> {
    const GET_FLIGHT_SUMMARY_SERVICE_ID: u8 = 2;

    // Gets input from user for flight ID.
    println!("Enter flight identifier:");
    let flight_id = std_in_reader
        .next()
        .expect("Error on iteration")
        .expect("Error on read");

    // Convert the flight ID to a u32
    let flight_id = flight_id
        .parse::<u32>()
        .expect("Error on parsing user's flight ID");

    // Create a buffer to store the data to send
    let mut buffer_to_send: Vec<u8> = vec![0; 2048];

    // Add service ID as first byte
    marshal_u8(GET_FLIGHT_SUMMARY_SERVICE_ID, &mut buffer_to_send);

    // Add flight ID
    marshal_u32(flight_id, &mut buffer_to_send);

    // Return the buffer
    buffer_to_send
}

fn send_request(request_id: u32, payload: Vec<u8>, socket: &UdpSocket, server_addr: &String) {
    // Create a buffer to store the data to send
    let mut buffer_to_send = vec![0; 2048];

    // Add request ID as first byte
    buffer_to_send.extend_from_slice(&request_id.to_be_bytes());

    // Add status byte as second byte. 0 for success, 1 for failure
    buffer_to_send.push(0);

    // Add payload to buffer
    buffer_to_send.extend_from_slice(&payload);

    socket
        .send_to(&buffer_to_send, &server_addr)
        .expect("Error on send");
}


// TODO: If any part errors in the logic of the service, then set the 2nd byte of the buffer to 1. Otherwise set it to 0. (The first byte is the request ID). Then marshal the error message. Stop reading the rest of the input.

// TODO: Move all code into workspace and add a lib for marshaler to house the marshal and unmarshaling functions.
