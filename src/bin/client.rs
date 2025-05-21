use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use http::Uri;
use std::error::Error; // Added for Box<dyn Error>
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_websockets::{ClientBuilder, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> { // Changed return type
    let (mut ws_stream, _) = ClientBuilder::from_uri(Uri::from_static("ws://127.0.0.1:8080"))
        .connect()
        .await?;

    let stdin = tokio::io::stdin();
    let mut stdin = BufReader::new(stdin).lines();

    loop {
        tokio::select! {
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        if let Some(text) = msg.as_text() {
                            println!("{}", text);
                        }
                    },
                    Some(Err(err)) => return Err(err.into()), // .into() will now work with Box<dyn Error>
                    None => return Ok(()), // Explicit Ok(()) for early return
                }
            }
            res = stdin.next_line() => {
                match res {
                    Ok(None) => return Ok(()), // Explicit Ok(()) for early return
                    Ok(Some(line)) => ws_stream.send(Message::text(line.to_string())).await?,
                    Err(err) => return Err(err.into()), // .into() will now work with Box<dyn Error>
                }
            }
        }
    }

}