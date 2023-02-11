use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlightInfo {
    pub flight_id: u32,
    pub source: String,
    pub destination: String,
    pub departure_time: String,
    pub airfare: f32,
    pub seat_availability: u32,
}

// Implement the marshal and unmarshal functions for FlightInfo. Store as bytes. Do it manually.
impl FlightInfo {
    pub fn marshall(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        result.extend_from_slice(&self.flight_id.to_be_bytes());
        result.extend_from_slice(&self.source.as_bytes());
        result.extend_from_slice(&self.destination.as_bytes());
        result.extend_from_slice(&self.departure_time.as_bytes());
        result.extend_from_slice(&self.airfare.to_be_bytes());
        result.extend_from_slice(&self.seat_availability.to_be_bytes());
        result
    }

    pub fn unmarshall(data: &[u8]) -> FlightInfo {
        let mut flight_id: u32 = 0;
        let mut source: String = String::new();
        let mut destination: String = String::new();
        let mut departure_time: String = String::new();
        let mut airfare: f32 = 0.0;
        let mut seat_availability: u32 = 0;

        let mut index: usize = 0;
        flight_id = u32::from_be_bytes([data[index], data[index + 1], data[index + 2], data[index + 3]]);
        index += 4;

        let mut i: usize = index;
        while data[i] != 0 {
            i += 1;
        }
        source = String::from_utf8(data[index..i].to_vec()).unwrap();
        index = i + 1;

        i = index;
        while data[i] != 0 {
            i += 1;
        }
        destination = String::from_utf8(data[index..i].to_vec()).unwrap();
        index = i + 1;

        i = index;
        while data[i] != 0 {
            i += 1;
        }
        departure_time = String::from_utf8(data[index..i].to_vec()).unwrap();
        index = i + 1;

        airfare = f32::from_be_bytes([data[index], data[index + 1], data[index + 2], data[index + 3]]);
        index += 4;

        seat_availability = u32::from_be_bytes([data[index], data[index + 1], data[index + 2], data[index + 3]]);

        FlightInfo {
            flight_id,
            source,
            destination,
            departure_time,
            airfare,
            seat_availability,
        }
    }
}