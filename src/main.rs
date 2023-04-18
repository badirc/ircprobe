use std::{
    io::{self, Write},
    thread,
};
use tokio::net::TcpStream;

use anyhow::Result;
use clap::Parser;

/// A small DIY IRC client
#[derive(Parser, Debug)]
struct Args {
    /// URI or IP of the server to connect to
    server: String,
}

async fn connect_to_server(endpoint: String) -> Result<TcpStream> {
    let stream = TcpStream::connect(endpoint).await?;

    Ok(stream)
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let stream = connect_to_server(args.server).await.unwrap();
    let (read, write) = stream.into_split();

    thread::spawn(move || loop {
        let mut input = String::new();

        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                println!("input: {:?}", input);
                write.try_write(input.as_bytes()).unwrap();
                ()
            }
            Err(error) => println!("Failed reading input: {:?}", error),
        }
    });

    loop {
        let mut buf = [0; 512];
        read.readable().await.unwrap();
        match read
            .try_read(&mut buf)
            .and_then(|n| match String::from_utf8(Vec::from(&buf[0..n])) {
                Ok(n) => Ok(n),
                Err(_) => {
                    println!("ERROR: Malformed response sent from server!");
                    Err(std::io::Error::new(
                        io::ErrorKind::InvalidData,
                        "invalid data",
                    ))
                }
            }) {
            Ok(response) => {
                println!("{}", response);
                print!("> ");
                io::stdout().flush().unwrap();
            }
            Err(error) => {
                println!("Internal error: {:?}", error);
            }
        }
    }
}
