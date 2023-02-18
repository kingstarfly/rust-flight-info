use std::net::{SocketAddr, UdpSocket};

pub fn send_request(
    request_id: u32,
    payload: Vec<u8>,
    socket: &UdpSocket,
    server_addr: &SocketAddr,
) {
    // Create a buffer to store the data to send with capacity 2048 bytes
    let mut buffer_to_send: Vec<u8> = Vec::with_capacity(2048);

    // Add request ID as the first byte for the server to differentiate requests from multiple clients.
    // Different from Service ID which is already handled by the respective `prepare` functions
    buffer_to_send.extend_from_slice(&request_id.to_be_bytes());

    // Add payload to buffer
    buffer_to_send.extend_from_slice(&payload);

    socket
        .send_to(&buffer_to_send, &server_addr)
        .expect("Error on send");
    println!("[networking] Sent request: {:?}", buffer_to_send);
}

// Should be able to simulate failure by not sending the response back to the client. Accepts a parameter to simulate failure.
pub fn send_response(
    request_id: u32,
    payload: Vec<u8>,
    socket: &UdpSocket,
    client_addr: &SocketAddr,
    simulate_failure: bool,
) {
    if simulate_failure {
        println!("[networking] Simulating failure");
        return;
    }
    // Send the response back to the client after prepending some information.
    // Create a buffer to store the data to send with capacity 2048 bytes
    let mut buffer_to_send: Vec<u8> = Vec::with_capacity(2048);

    // Add request ID as the first byte for the client to check if the correct response was received.
    buffer_to_send.extend_from_slice(&request_id.to_be_bytes());
    // println!("Appending request ID: {} ", request_id);

    // Add payload to buffer
    buffer_to_send.extend_from_slice(&payload);
    // println!("Appending payload: {:?}", payload);

    socket
        .send_to(&buffer_to_send, &client_addr)
        .expect("Error on send");
    println!("[networking] Sent response: {:?}", buffer_to_send);
}
