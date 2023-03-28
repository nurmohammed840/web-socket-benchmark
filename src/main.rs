use std::{str, time::Instant};
use tokio::{io::*, spawn};
use web_socket::*;

const ITER: usize = 100000;
const MSG: &str = "Hello, World!";

const HELP: &str = "Please run with `--release` flag for accurate results.
Example:
    cargo run --release
    cargo run -r -- -C codegen-units=1 -C opt-level=3
";

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if cfg!(debug_assertions) {
        println!("{HELP}");
    }
    tokio_tungstenite_banchmark::run().await;
    websocket_banchmark::run().await;
}

mod websocket_banchmark {
    use super::*;

    pub async fn run() {
        let (server_stream, client_stream) = duplex(ITER * (MSG.len() + 14));
        let server = spawn(server(server_stream));
        let client = spawn(client(client_stream));
        let _ = server.await.unwrap();
        client.await.unwrap().unwrap();
    }

    async fn client(stream: DuplexStream) -> Result<()> {
        let mut ws = WebSocket::client(BufReader::new(stream));
        let time = Instant::now();
        for _ in 0..ITER {
            ws.send(MSG).await?;
        }
        for _ in 0..ITER {
            let Event::Data { ty, data }= ws.recv().await? else { panic!("invalid data") };
            assert_eq!(ty, DataType::Complete(MessageType::Text));
            assert_eq!(MSG, str::from_utf8(&data).unwrap());
        }
        Ok(println!("web-socket:  {:?}", time.elapsed()))
    }

    async fn server(stream: DuplexStream) -> Result<()> {
        let mut ws = WebSocket::server(BufReader::new(stream));
        loop {
            match ws.recv().await? {
                Event::Data { data, ty } => match ty {
                    DataType::Fragment(_) => unimplemented!(),
                    DataType::Complete(ty) => match ty {
                        MessageType::Text => ws.send(str::from_utf8(&data).unwrap()).await?,
                        MessageType::Binary => ws.send(&*data).await?,
                    },
                },
                Event::Pong(..) => {}
                Event::Ping(data) => ws.send_pong(data).await?,
                Event::Error(..) | Event::Close { .. } => return ws.close(()).await,
            }
        }
    }
}

mod tokio_tungstenite_banchmark {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{
        accept_async, client_async,
        tungstenite::{Error, Message, Result},
    };

    pub async fn run() {
        let (server_stream, client_stream) = tokio::io::duplex(ITER * (MSG.len() + 14));
        let server = tokio::spawn(server(server_stream));
        let client = tokio::spawn(client(client_stream));
        server.await.unwrap();
        client.await.unwrap().unwrap();
    }

    async fn client(stream: DuplexStream) -> Result<()> {
        let (mut ws, _) = client_async("ws://localhost:9001", BufReader::new(stream))
            .await
            .unwrap();

        let time = Instant::now();

        for _ in 0..ITER {
            ws.send(Message::Text(MSG.to_owned())).await?;
        }
        for _ in 0..ITER {
            match ws.next().await.unwrap()? {
                Message::Text(data) => assert_eq!(MSG, data),
                _ => unimplemented!(),
            }
        }
        Ok(println!("tokio-tungstenite: {:?}", time.elapsed()))
    }

    async fn server(stream: DuplexStream) {
        if let Err(err) = handle_connection(stream).await {
            match err {
                Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
                err => panic!("Error processing connection: {err}"),
            }
        }
    }

    async fn handle_connection(stream: DuplexStream) -> Result<()> {
        let mut ws = accept_async(BufReader::new(stream))
            .await
            .expect("Failed to accept");
        while let Some(msg) = ws.next().await {
            let msg = msg?;
            if msg.is_text() || msg.is_binary() {
                ws.send(msg).await?;
            }
        }
        Ok(())
    }
}
