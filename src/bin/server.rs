use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{channel, Sender};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: Sender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    ws_stream
        .send(Message::text("Welcome to chat! Type a message".to_string()))
        .await?;
    let mut bcast_rx = bcast_tx.subscribe();

    loop {
        tokio::select! {
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        if let Some(text) = msg.as_text() {
                            println!("From client {addr:?} {text:?}");
                            bcast_tx.send(text.into())?;
                        }
                    }
                    Some(Err(err)) => return Err(err.into()),
                    None => return Ok(()),
                }
            }
            msg = bcast_rx.recv() => {
                ws_stream.send(Message::text(msg?)).await?;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = channel(16);

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("listening on port 2000");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {addr:?}");
        let bcast_tx_clone = bcast_tx.clone();
        tokio::spawn(async move {
            let task_logic = async {
                // Wrap the raw TCP stream into a websocket.
                // ServerBuilder::accept returns Result<(Request<()>, WebSocketStream<S>), Error>
                // Destructure the tuple and get the actual WebSocketStream.
                let (_request_details, ws_stream_actual) = ServerBuilder::new()
                    .accept(socket)
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?; // Map error type

                // Pass the actual WebSocketStream to handle_connection.
                handle_connection(addr, ws_stream_actual, bcast_tx_clone).await?;
                Ok::<(), Box<dyn Error + Send + Sync>>(()) // Explicit type annotation for Ok
            };

            // Execute the task logic and log any errors.
            if let Err(e) = task_logic.await {
                eprintln!("Error processing connection for {addr:?}: {e}");
            }
        });
    }
}