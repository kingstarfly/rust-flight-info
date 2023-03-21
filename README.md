# Rust Flight Information System

## Table of Contents

- [Rust Flight Information System](#rust-flight-information-system)
  - [Table of Contents](#table-of-contents)
  - [About ](#about-)
  - [Getting Started ](#getting-started-)
  - [Usage ](#usage-)

## About <a name = "about"></a>

This is a Rust project built for the purposes of the CZ4013 - Distributed Systems course from NTU. I wrote a [blog](https://xingxiang.hashnode.dev/flying-high-with-rust-and-udp-building-a-client-server-system-for-flight-reservations) about this, check it out!

## Getting Started <a name = "getting_started"></a>
You need Rust installed on your machine. Check out the [Rust website](https://www.rust-lang.org/tools/install) for more information.

## Usage <a name = "usage"></a>

To run the server: `cargo run --package server alo true`
- Argument 1: alo (at-least-once) / amo (at-most-once) 
- Argument 2: true (enable simulation of network failure) / false (disable simulation of network failure)

To run the client: `cargo run --package client`