use std::{io, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

use anyhow::Result;
use clap::Parser;
use reedline::{DefaultPrompt, Reedline, Signal};

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
async fn main() -> Result<()> {
    let args = Args::parse();
    let stream = connect_to_server(args.server).await?;
    let (read, mut write) = stream.into_split();
    let mut line_editor = Reedline::create();
    let prompt = DefaultPrompt::default();

    let arced_read = Arc::new(Mutex::new(read));
    tokio::spawn(async move {
        loop {
            let read = arced_read.clone();
            let mut buf = [0; 8191];
            let mut lock = read.lock().await;
            match lock.read(&mut buf).await.and_then(|n| {
                match String::from_utf8(Vec::from(&buf[0..n])) {
                    Ok(n) => Ok(n),
                    Err(_) => {
                        println!("ERROR: Malformed response sent from server!");
                        Err(std::io::Error::new(
                            io::ErrorKind::InvalidData,
                            "invalid data",
                        ))
                    }
                }
            }) {
                Ok(content) if content.is_empty() => {
                    println!("Server closed connection, exiting");
                    std::process::exit(0);
                }
                Ok(content) => {
                    dbg!("{}", content);
                    ()
                }
                Err(_) => {
                    println!("Internal error!");
                }
            }
        }
    });

    loop {
        let sig = line_editor.read_line(&prompt);
        match sig {
            Ok(Signal::Success(buffer)) => {
                write.write(buffer.as_bytes()).await?;
                write.flush().await?;
            }
            Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                break;
            }
            _ => (),
        }
    }

    Ok(())
}
