use crate::model::flight_info::FlightInfo;
use crate::model::request::{RequestType, Request,};
use crate::model::response::Response;
use std::str;
use serde::{Serialize, Deserialize};
use std::net::UdpSocket;
use std::io;

struct Client {
    // client state, such as server address and port
}

impl Client {
    // methods for initializing and starting the client, sending requests to the server, and handling server responses
    pub fn run_client(request_type: RequestType, flight_id: u32, flight_info: Option<FlightInfo>) {
    
        let request_message = Request {
            request_type,
            flight_id,
            flight_info        
        };
    
        let request_message_bytes = Request::marshall(&request_message);
    
        let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind client socket");
        socket
            .send_to(&request_message_bytes, "localhost:8080")
            .expect("Failed to send request to server");
    
        let mut buffer = [0; 1024];
        let (number_of_bytes, _src_address) = socket
            .recv_from(&mut buffer)
            .expect("Failed to receive response from server");
        let response_message_bytes = &buffer[..number_of_bytes];
    
        let response_message: ResponseMessage = bincode::deserialize(response_message_bytes).unwrap();
    
        match response_message.response_type {
            ResponseType::FlightInformation => println!(
                "Flight Information: {:?}",
                response_message.flight_info.unwrap()
            ),
            ResponseType::Acknowledgement => println!("Acknowledgement received"),
            ResponseType::Error => println!("Error: {}", response_message.flight_id),
            ResponseType::Update => println!("Update successful"),
        }
    }

    fn query_flight_identifiers(&self, source: &str, destination: &str) -> Result<Vec<i32>, &'static str> {
        // code to send request to the server and handle the response
    }

    fn query_flight_details(&self, flight_id: i32) -> Result<(Time, f32, i32), &'static str> {
        // code to send request to the server and handle the response
    }

    fn reserve_seats(&self, flight_id: i32, seats: i32) -> Result<(), &'static str> {
        // code to send request to the server and handle the response
    }

    fn register_for_updates(&self, flight_id: i32, interval: i32) {
        // code to send request to the server and handle the response
    }
}

