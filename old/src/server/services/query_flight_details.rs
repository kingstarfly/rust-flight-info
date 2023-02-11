use chrono::NaiveDate;

pub fn handle_query_flight_details(
    &data: &[u8; 1024],
) -> Result<[u8; 1024], Box<dyn std::error::Error>> {
    /*
    A service that allows a user to query the departure time, airfare and seat availability by specifying the flight identifier. If the flight with the requested identifier does not exist, an error message should be returned
    */
    let mut buf = &data[1..];

    // Read the first 4 bytes as a u32 as the flight_id
    let flight_id = u32::from_be_bytes(buf[..4].try_into().unwrap());

    let date = NaiveDate::from_ymd_opt(2020, 1, 1);
    if date.is_none() {
        // TODO: How to handle errors here? Perhaps the first byte is a u8 that indicates the type of error or if the request was successful?
    }
    
    // Result is a tuple of (departure_time, airfare, seats_available)

    // Fake data
    let result: (NaiveDate, f32, u32) = (date.unwrap(), 100.0, 10);
    
    // Marshall the result into a byte array in the order of departure_time, airfare, seats_available
    let mut response = [0; 1024];
    
    // For success, first byte is 0
    response[0] = 0;
    // Length of departure_time string
    let departure_time_len = result.0.to_string().len();
    // Write the length of the departure_time string as a u32 and then write the string itself into the response byte array
    response[1..5].copy_from_slice(&departure_time_len.to_be_bytes());
    
    response[5..5 + departure_time_len].copy_from_slice(result.0.to_string().as_bytes());

    // Write the airfare as a f32
    response[5 + departure_time_len..5 + departure_time_len + 4].copy_from_slice(&result.1.to_be_bytes());

    // Write the seats_available as a u32
    response[5 + departure_time_len + 4..5 + departure_time_len + 4 + 4].copy_from_slice(&result.2.to_be_bytes());


    // Convert 55.55 to byte array
    let myFloat: f32 = 55.55;
    let myFloatBytes: [u8; 4] = myFloat.to_be_bytes();
    // print it in hexadecimals
    println!("{:?}", myFloatBytes);
    Ok(response)
}
