use connection::Connection;

use anyhow::Result;
use clap::Parser;
use reedline::{DefaultPrompt, Reedline, Signal};

mod connection;

/// A small DIY IRC client
#[derive(Parser, Debug)]
struct Args {
    /// URI or IP of the server to connect to
    server: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut connection = Connection::new(args.server).await?;
    let mut line_editor = Reedline::create();
    let prompt = DefaultPrompt::default();

    // FIXME: Do we need this clone?
    let read_stream = connection.read_stream.clone();
    tokio::spawn(async move {
        loop {
            match Connection::extern_read_message(&read_stream).await {
                Ok(message) => println!("{}", message),
                Err(err) => eprintln!("{:?}", err),
            };
        }
    });

    loop {
        let sig = line_editor.read_line(&prompt);
        match sig {
            Ok(Signal::Success(buffer)) => {
                connection.write_message(buffer).await?;
            }
            Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                break;
            }
            _ => (),
        }
    }

    Ok(())
}
