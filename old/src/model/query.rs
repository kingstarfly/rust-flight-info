use num_derive::FromPrimitive;


#[derive(Debug, Clone, PartialEq, FromPrimitive)]
pub enum RequestType {
    QueryFlightIdentifiers,
    QueryFlightDetails,
    ReserveSeats,
    RegisterForUpdates,
    Error
}