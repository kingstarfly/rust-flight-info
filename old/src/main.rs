mod client;
mod server;

// include the model/flight_info.rs file
mod model;

use std::env;

pub const AT_LEAST_ONCE_INVOCATION: &str = "at-least-once";
pub const AT_MOST_ONCE_INVOCATION: &str = "at-most-once";

// Run with `cargo run -- <server/client> <ip-address> <invocation-semantics>`
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Check if the correct number of arguments were provided
    if args.len() != 4 {
        println!("Usage: cargo run -- <server/client> <ip-address> <invocation-semantics>");
        return Ok(());
    }

    let role = &args[1];
    let addr = &args[2];
    let invocation_semantics = &args[3];

    // Check if the invocation semantics are valid
    if invocation_semantics != AT_LEAST_ONCE_INVOCATION && invocation_semantics != AT_MOST_ONCE_INVOCATION {
        println!("Usage: cargo run -- <server/client> <ip-address> <invocation-semantics>");
        return Ok(());
    }

    if role == "server" {
        server::run_server(addr, invocation_semantics)?;
    } else if role == "client" {
        client::run_client(addr, invocation_semantics)?;
    } else {
        println!("Usage: cargo run -- <server/client> <ip-address> <invocation-semantics>");
    }

    Ok(())
}