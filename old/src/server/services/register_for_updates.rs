use std::net::SocketAddr;

pub fn handle_register_for_updates(
    &data: &[u8; 1024],
) -> Result<[u8; 1024], Box<dyn std::error::Error>>{
    /*
    A service that allows a user to monitor updates made to the seat availability information of a flight at the server through callback for a designated time period called monitor interval. To register, the client provides the flight identifier and the length of monitor interval to the server. After registration, the Internet address and the port number of the client are recorded by the server. During the monitor interval, every time a seat reservation is made by any client on the flight, the updated seat availability of the flight is sent by the server to the registered client(s) through callback. After the expiration of the monitor interval, the client record is removed from the server which will no longer deliver the updates of the flight to the client. For simplicity, you may assume that the user that has issued a register request for monitoring is blocked from inputting any new request until the monitor interval expires, i.e., the client simply waits for the updates from the server during the monitor interval. As a result, you do not have to use multiple threads at a client. However, your implementation should allow multiple clients to monitor updates to the flights concurrently.
     */
    // code to handle registration for updates

    // First byte is request type and is ignored
    // Second byte is length of flight id
    let flight_id_length = data[1] as usize;
    // Next byte is the interval as u32
    let interval = u32::from_be_bytes([data[2], data[3], data[4], data[5]]);
    

    // Placeholder print
    println!(
        "Registering socket address {} for updates for flight {} for {} seconds",
        client_addr.to_string(),
        flight_id,
        interval
    );

    let response_string = "Registered for updates successfully";
    let mut response_data = [0; 1024];
    // For success, first byte is 0
    response_data[0] = 0;
    // First byte is length of response string
    response_data[1] = response_string.len() as u8;
    // Next bytes are the response string
    response_data[2..response_string.len() + 1].copy_from_slice(response_string.as_bytes());

    Ok(response_data)
}