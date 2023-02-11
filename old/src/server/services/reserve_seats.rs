/*
A service that allows a user to make seat reservation on a flight by specifying the flight identifier and the number of seats to reserve. On successful reservation, an acknowledgement is returned to the client and the seat availability of the flight should be updated at the server. In case of incorrect user input (e.g., not-existing flight identifier or insufficient number of available seats), a proper error message should be returned.
*/

pub fn handle_reserve_seats(buf: &[u8]) -> Result<[u8; 1024], Box<dyn std::error::Error>> {
    // First byte is the request type and is ignored
    let mut buf = &buf[1..];

    // read the next 4 bytes as the flight identifier and convert to u32
    let flight_id = u32::from_be_bytes(buf[..4].try_into().unwrap());

    // read the next 4 bytes as the number of seats to reserve and convert to u32
    let num_seats = u32::from_be_bytes(buf[4..8].try_into().unwrap());

    // Deduct the number of seats from the flight's seat availability
    // If the number of seats is greater than the seat availability, return an error
    // Otherwise, update the seat availability and return an acknowledgement

    
    let mut response = [0; 1024];
    
    let response_string = "Seats reserved successfully";
    // For success, first byte is 0
    response[0] = 0;
    // First byte is length of response string
    response[1] = response_string.len() as u8;
    // Next bytes are the response string
    response[2..response_string.len() + 1].copy_from_slice(response_string.as_bytes());

    Ok(response)  
    
}
