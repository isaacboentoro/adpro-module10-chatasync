use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use http::Uri;
use std::error::Error; // Added for Box<dyn Error>
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_websockets::{ClientBuilder, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> { // Changed return type
    let (mut ws_stream, _) = ClientBuilder::from_uri(Uri::from_static("ws://127.0.0.1:2000"))
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
                            println!("From server: {}", text);
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
    // The loop is infinite, so Ok(()) might not be reached if the loop never breaks.
    // However, if the loop could terminate normally (e.g. specific input), Ok(()) would be needed here.
    // For this specific infinite loop, the returns inside handle termination.
    // Adding Ok(()) here for completeness in case the loop structure changes.
    // Ok(()) // This line is technically unreachable due to the infinite loop with internal returns.
             // However, if the loop could break, it would be necessary.
             // Given the current structure, the explicit returns within the loop cover all exit paths.
}