// this is a server
// create a unix_listener that accepts connections from client
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    thread::sleep,
    time::Duration,
};

use anyhow::Context;

use unix_socket_based_client_server::{
    message::{CommandStatus, Request, Response},
    socket::SocketBuilder,
};

fn main() -> anyhow::Result<()> {
    let socket_path = "/tmp/rst.sock";

    // Create the socket
    let socket = SocketBuilder::new()
        .with_path(socket_path)
        .with_permissions(0o700)
        .nonblocking(false)
        .build()
        .context("Could not create the socket")?;

    println!("Starting the unix socket server, Press Ctrl^C to stop...");

    // The loop allows to handle several connections, one after the other
    loop {
        // accept_connection() is a wrapper around UnixListener::accept()
        let (unix_stream, socket_address) = socket.accept_connection()?;

        println!(
            "Accepted connection. Stream: {:?}, address: {:?}",
            unix_stream, socket_address
        );

        handle_connection(unix_stream)?;
    }
}

fn handle_connection(mut stream: UnixStream) -> anyhow::Result<()> {
    // Receive a message
    let mut message = String::new();
    stream
        .read_to_string(&mut message)
        .context("Failed at reading the unix stream")?;

    println!("{}", message);

    // Parse the message
    let request = serde_json::from_str::<Request>(&message)
        .context("could no deserialize request message")?;

    println!("The parsed request is: {:?}", request);

    // Emulate processing time
    // Send 3 processings responses every second before sending the final one
    for _ in 0..2 {
        let mut processing = Response::new(
            "processing",
            CommandStatus::Processing,
            "still processing...",
        )
        .serialize_to_bytes()
        .context("Could not serialize response")?;

        // Add a zero byte, to separate instructions
        processing.push(0);

        println!("Sending processing response");
        stream
            .write(&processing)
            .context("Could not write processing response onto the unix stream")?;

        sleep(Duration::from_secs(1));
    }

    // Create a response that matches the request
    let response: Response = match request.id.as_str() {
        "request" => Response::new("response", CommandStatus::Ok, "Roger that"),
        _ => Response::new("what", CommandStatus::Error, "Sorry what?"),
    };

    let mut response_as_bytes = response
        .serialize_to_bytes()
        .context("Could not serialize response")?;

    // The zero byte is a separator, so that the client can distinguish between responses
    response_as_bytes.push(0);

    stream
        .write(&response_as_bytes)
        .context("Could not write response onto the unix stream")?;

    Ok(())
}
