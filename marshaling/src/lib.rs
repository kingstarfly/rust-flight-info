pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub fn marshal_string(string: &str, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&(string.len() as u8).to_be_bytes());
    buf.extend_from_slice(string.as_bytes());
}

pub fn marshal_u8(number: u8, buf: &mut Vec<u8>) {
    buf.push(number);
}

pub fn marshal_u32(number: u32, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&number.to_be_bytes());
}

pub fn marshal_f32(number: f32, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&number.to_be_bytes());
}

pub fn marshal_u32_array(numbers: &[u32], buf: &mut Vec<u8>) {
    buf.extend_from_slice(&(numbers.len() as u8).to_be_bytes());
    for number in numbers {
        buf.extend_from_slice(&number.to_be_bytes());
    }
}

// Function to read a string from a buffer. 
// The first byte is the length of the string.
// The next bytes are the string itself.
// The function takes in a buffer and the index of the first byte of the string.
// The function returns the string and the number of bytes read.

pub fn unmarshal_string(buf: &[u8], i: usize) -> (String, usize) {
    // Create a new index
    let mut i = i;

    // First read the first byte to determine length of string
    let string_length: usize = buf[i].into();
    i += 1;

    // Then read the string from utf
    let my_string = String::from_utf8_lossy(&buf[i..i+string_length]).to_string();
    i += string_length;

    return (my_string, i);
}

// Function to read an u32 from a buffer.
// The function takes in a buffer and the index of the first byte of the u32.
// The function returns the u32 and the number of bytes read.

pub fn unmarshal_u32(buf: &[u8], i: usize) -> (u32, usize) {
    // Create a new index
    let mut i = i;
    
    // Then read the u32
    let my_u32 = u32::from_be_bytes([buf[i], buf[i+1], buf[i+2], buf[i+3]]);
    i += 4;

    return (my_u32, i);
}

// Function to read an f32 from a buffer.
// The function takes in a buffer and the index of the first byte of the f32.
// The function returns the f32 and the number of bytes read.

pub fn unmarshal_f32(buf: &[u8], i: usize) -> (f32, usize) {
    // Create a new index
    let mut i = i;

    // Then read the f32
    let my_f32 = f32::from_be_bytes([buf[i], buf[i+1], buf[i+2], buf[i+3]]);
    i += 4;

    return (my_f32, i);
}

// Function to read an array of u32 from a buffer.
// The function takes in a buffer and the index to start reading from.
// The function returns the array of u32 and the number of bytes read.
// The first byte is the length of the array.
// The next bytes are the array itself.

pub fn unmarshal_u32_array(buf: &[u8], i: usize) -> (Vec<u32>, usize) {
    // Create a new index
    let mut i = i;

    // First read the first byte to determine length of array
    let array_length: usize = buf[i].into();
    i += 1;

    // Then read the array
    let mut my_array: Vec<u32> = Vec::with_capacity(array_length);
    for _ in 0..array_length {
        let my_u32 = u32::from_be_bytes([buf[i], buf[i+1], buf[i+2], buf[i+3]]);
        i += 4;
        my_array.push(my_u32);
    }

    return (my_array, i);
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
