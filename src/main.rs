use anyhow::Result;
use clap::Parser;
use futures::{SinkExt, StreamExt};
use rustyline_async::{Readline, ReadlineError};
use std::io::Write;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec};

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
                    frame.send(line).await?;
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
