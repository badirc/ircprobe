use anyhow::Result;
use std::{io, sync::Arc};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::{
    io::AsyncReadExt,
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

pub struct Connection {
    pub endpoint: String,
    pub read_stream: Arc<Mutex<OwnedReadHalf>>,
    pub write_stream: Arc<Mutex<OwnedWriteHalf>>,
}

impl Connection {
    pub async fn new(endpoint: String) -> Result<Self> {
        let stream = TcpStream::connect(&endpoint).await?;
        let (read, write) = stream.into_split();

        Ok(Self {
            endpoint,
            read_stream: Arc::new(Mutex::new(read)),
            write_stream: Arc::new(Mutex::new(write)),
        })
    }

    /// Struct-level read message method, to prevent moving our connection instance into a
    /// closure.
    ///
    /// Background: IRC is annoying in the way that it can basically send messages from server to
    /// client at any time, meaning you always have to be prepared to receive messages. The way I
    /// do this here is to use a Tokio task, in which I run a loop that calls this method. Because
    /// it's not only a loop, but also an async closure _and_ a separate thread, I have to be very
    /// careful about what I move into it. If I call `connection.read_message()`, it'll capture the
    /// connection, which means I can't write into it later on. The solution I've come up with is to
    /// manually clone the read stream and pass it into this struct-level method, which is not the nicest,
    /// but it works for now.
    ///
    /// FIXME: Surely there's a more elegant way to do this that I'm not just aware of?
    pub async fn extern_read_message(read_stream: &Arc<Mutex<OwnedReadHalf>>) -> Result<String> {
        let read_stream = read_stream.clone();
        read_message_internal(read_stream).await
    }

    pub async fn write_message(&mut self, content: String) -> Result<()> {
        let write_stream = self.write_stream.clone();
        let mut lock = write_stream.lock().await;
        lock.write(content.as_bytes()).await?;
        lock.flush().await?;
        Ok(())
    }
}

async fn read_message_internal(read_stream: Arc<Mutex<OwnedReadHalf>>) -> Result<String> {
    // Maximum length of a server response when supporting message tags, which we just preempt.
    let mut buf = [0; 8191];
    let mut lock = read_stream.lock().await;
    match lock
        .read(&mut buf)
        .await
        .and_then(|n| match String::from_utf8(Vec::from(&buf[0..n])) {
            Ok(_) if n == 0 => {
                eprintln!("ERROR: Server closed connection. Exiting.");
                std::process::exit(1);
            }
            Ok(content) => Ok(content),
            Err(_) => {
                eprintln!("ERROR: Malformed response sent from server.");
                Err(io::Error::new(io::ErrorKind::InvalidData, "invalid data"))
            }
        }) {
        Ok(content) => Ok(content),
        Err(e) => {
            eprintln!("ERROR: Internal error when reading from server.");
            Err(e.into())
        }
    }
}
