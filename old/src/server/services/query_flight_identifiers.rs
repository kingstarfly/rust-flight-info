// Returns a byte array 

pub fn handle_query_flight_identifiers(
    &data: &[u8; 1024],
) -> Result<[u8; 1024], Box<dyn std::error::Error>> {
    /*
    A service that allows a user to query the flight identifier(s) by specifying the source and destination places. If multiple flights match the source and destination Page 1 CE4013/CZ4013/SC4051 Distributed Systems   places, all of them should be returned to the user. If no flight matches the source and destination places, an error message should be returned.
    */
    // First byte is the request type and is ignored
    // Next 4 bytes are the length of the source string
    let source_len = u32::from_le_bytes([data[1], data[2], data[3], data[4]]);
    // Read the source string
    let source = String::from_utf8(data[5..5 + source_len as usize].to_vec()).unwrap();

    // Next 4 bytes are the length of the destination string
    let destination_len = u32::from_le_bytes([data[5 + source_len as usize], data[6 + source_len as usize], data[7 + source_len as usize], data[8 + source_len as usize]]);
    // Read the destination string
    let destination = String::from_utf8(data[9 + source_len as usize..9 + source_len as usize + destination_len as usize].to_vec()).unwrap();

    // print the source and destination
    println!("Source: {}", source);
    println!("Destination: {}", destination);

    // return the flight identifiers
    let flight_identifiers = vec![1, 2, 3];
    // TODO: Need access to a global store / database to get the flight identifiers

    // Marshall the flight identifiers into a byte array
    let mut response = [0; 1024];

    // For success, the first byte is 0
    response[0] = 0;
    response[1] = flight_identifiers.len() as u8;
    for (i, flight_identifier) in flight_identifiers.iter().enumerate() {
        response[3 + i] = *flight_identifier;
    }

    Ok(response)

}