//use std::io::{self, Read, Write, BufRead};
use std::net::UdpSocket;
//use std::env;
//use std::str;

use marshaling;
use networking;
fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:7878")?;
    let mut buf = [0; 2048];

    loop {
        // Receives a single datagram message on the socket.
	    // If `buf` is too small to hold
        // the message, it will be cut off.
        let (amt, src) = socket.recv_from(&mut buf)?;

        // Prints out a received bytes
        println!("Received {} bytes from {}", amt, src);

        // Expect a string, a i32 and a f32 to be obtained after parsing buf
        // Declare my_string only.
        let my_string: String;
        let my_u32: u32;
        let my_f32: f32;

        let mut i = 0;

        (my_string, i) = unmarshal_string(&buf, i);
        (my_u32, i) = unmarshal_u32(&buf, i);
        (my_f32, i) = unmarshal_f32(&buf, i);

        let received_payload = format!("{} {} {}", my_string, my_u32, my_f32);

        // Print out the received data in one line
        println!("Received payload: {}", received_payload);

        // Convert the received data to bytes
        let received_payload = received_payload.as_bytes();

        // Print the number of bytes to be sent
        println!("Sending {} bytes", received_payload.len());

        // Redeclare `buf` as slice of the received data
        socket.send_to(&received_payload, &src)?;
    }
}