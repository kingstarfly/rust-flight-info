//use std::io::{self, Read, Write, BufRead};
use std::net::UdpSocket;
//use std::env;
//use std::str;

use marshaling::{self, unmarshal_string, unmarshal_u32, unmarshal_f32, unmarshal_u8};
use networking;
fn main() -> std::io::Result<()> {
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
                // TODO 1: Call the Get Flight Identifiers service
                println!("TODO: Service 1 handler");
                vec![]

            }
            2 => {
                // TODO 2: Call the Get Flight Summary service
                println!("TODO: Service 2 handler");
                vec![]

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

        // Add payload to buffer
        buffer_to_send.extend_from_slice(&payload);

        socket
            .send_to(&buffer_to_send, &client_addr)
            .expect("Error on send");


        // Done.
    }
}