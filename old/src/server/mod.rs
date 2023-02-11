mod services;
use std::collections::HashMap;

use crate::{
    model::{query::RequestType, flight_info::FlightInfo},
    server::services::{
        query_flight_details::handle_query_flight_details,
        query_flight_identifiers::handle_query_flight_identifiers,
        register_for_updates::handle_register_for_updates,
        reserve_seats::handle_reserve_seats,
    },
};

use num::FromPrimitive;
use tokio::net::UdpSocket;

struct Server {
    // A hashmap of flight identifiers to flight info
    flight_identifier_to_info: HashMap<String, FlightInfo>,

    // A hashmap of source addresses to hashmaps of destination addresses to flight identifiers
    source_to_destination_to_flight_identifier: HashMap<String, HashMap<String, u32>>,
}

impl Server {
    // methods for initializing and starting the server, handling client requests, and responding to clients
    pub async fn new(addr: &str, invocation_semantics: &str) {
        let socket = UdpSocket::bind(addr).await.expect("Could not bind socket");

        // TODO: Implement the invocation semantics
        // Placeholder print invocation semantics
        println!("TODO: Invocation semantics: {invocation_semantics}");

        loop {
            let mut buf: [u8; 1024] = [0; 1024];
            let (size, src) = socket
                .recv_from(&mut buf)
                .await
                .expect("Could not read data into buffer");
                
            // Unmarshall the buffer into a Request using Request::unmarshall
            // First determine the request type to determine which unmarshall method to use
            let request_type_u8 = FromPrimitive::from_u8(buf[0]);
            let request_type = match request_type_u8 {
                Some(RequestType::QueryFlightIdentifiers) => RequestType::QueryFlightIdentifiers,
                Some(RequestType::QueryFlightDetails) => RequestType::QueryFlightDetails,
                Some(RequestType::ReserveSeats) => RequestType::ReserveSeats,
                Some(RequestType::RegisterForUpdates) => RequestType::RegisterForUpdates,
                Some(RequestType::Error) => RequestType::Error,
                _ => RequestType::Error,
            };

            // Next, invoke the handler for that request type
            // If the handler errors out, then break and run the error handler

            let response_data: Result<[u8; 1024], Box<dyn std::error::Error>> = match request_type {
                RequestType::QueryFlightIdentifiers => handle_query_flight_identifiers(&buf),

                RequestType::QueryFlightDetails => handle_query_flight_details(&buf),

                RequestType::ReserveSeats => handle_reserve_seats(&buf),

                RequestType::RegisterForUpdates => handle_register_for_updates(&buf),

                RequestType::Error => handle_error(),

                _ => handle_error(),
            };

            // Marshall the response and send it back to the client
            socket
                .send_to(&response_data, &src)
                .await
                .expect("Could not write data to socket");
        }
    }
}
