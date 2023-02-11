
pub fn handle_error() -> Result<[u8; 1024], Box<dyn std::error::Error>> {
    let mut response_data = [0; 1024];
    
    // If error, first byte is 1.
    response_data[0] = 1;

    let error_message = "Error: Invalid request type.";
    // Next byte is length of error string
    response_data[1] = error_message.len() as u8;

    // Next bytes are the error string
    for (i, byte) in error_message.bytes().enumerate() {
        response_data[i + 2] = byte;
    }

    Ok(response_data)
}