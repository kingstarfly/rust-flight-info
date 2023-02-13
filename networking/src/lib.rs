use std::net::{SocketAddr, UdpSocket};

pub fn send_request(
    request_id: u32,
    payload: Vec<u8>,
    socket: &UdpSocket,
    server_addr: &SocketAddr,
) {
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

pub fn send_response(
    request_id: u32,
    payload: Vec<u8>,
    socket: &UdpSocket,
    client_addr: &SocketAddr,
) {
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
}
