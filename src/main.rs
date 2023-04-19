use anyhow::Result;
use clap::Parser;
use futures::{SinkExt, StreamExt};
use rustyline_async::{Readline, ReadlineError};
use std::io::Write;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec};

mod parser;

/// A small DIY IRC client
#[derive(Parser, Debug)]
struct Args {
    /// URI or IP of the server to connect to
    server: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let (mut rl, mut stdout) = Readline::new("> ".to_owned())?;

    let connection = TcpStream::connect(args.server).await?;
    tokio::pin!(connection);
    let mut frame = Framed::new(connection, LinesCodec::new());
    loop {
        // We always want to pick either a Future where something from stdin is read, or a
        // Future where we receive a message from the server. Since both Futures are cancel-safe,
        // we don't actually lose data by disregarding the other one.
        tokio::select! {
            r = frame.next() => match r {
                None => {
                    writeln!(stdout, "ERROR: Connection closed by server. Exiting")?;
                    break;
                }
                Some(string) => {
                    writeln!(stdout, "{}", string?)?;
                }
            },
            input = rl.readline() => match input {
                Ok(line) => {
                    let parsed = parser::parse(line)?;
                    for command in parsed {
                        frame.send(command.clone()).await?;
                    }
                }
                Err(ReadlineError::Eof) | Err(ReadlineError::Closed) => break,
                Err(ReadlineError::Interrupted) => {
                    writeln!(stdout, "^C")?;
                    continue;
                }
                Err(_) => {}
            }
        }
    }

    rl.flush()?;
    Ok(())
}
