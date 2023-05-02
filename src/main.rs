use std::{fmt::Debug, future::Future, str, time::Instant};
use tokio::io::BufReader;

const ITER: usize = 100000;
const MSG: &str = "Hello, World!\n";

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
    fastwebsockets_banchmark::run().await;
}

type Stream = BufReader<tokio::io::DuplexStream>;

async fn bench<S, C, SR, CR>(s: fn(Stream) -> S, c: fn(Stream) -> C)
where
    S: Future<Output = Result<(), SR>> + Send + 'static,
    C: Future<Output = Result<(), CR>> + Send + 'static,
    SR: Send + Debug + 'static,
    CR: Send + Debug + 'static,
{
    let (server_stream, client_stream) = tokio::io::duplex(ITER * (MSG.len() + 14));
    let server = tokio::spawn(s(BufReader::new(server_stream)));
    let client = tokio::spawn(c(BufReader::new(client_stream)));
    server.await.unwrap().unwrap();
    client.await.unwrap().unwrap();
}

mod websocket_banchmark {
    use super::*;
    use tokio::io::*;
    use web_socket::*;

    pub async fn run() {
        bench(server, client).await
    }

    async fn client(stream: Stream) -> Result<()> {
        let mut ws = WebSocket::client(stream);
        let time = Instant::now();
        for _ in 0..ITER {
            ws.send(MSG).await?;
        }
        for _ in 0..ITER {
            let Ok(Event::Data { ty, data }) = ws.recv().await else { panic!("invalid data") };
            assert_eq!(ty, DataType::Complete(MessageType::Text));
            assert_eq!(str::from_utf8(&data), Ok(MSG));
        }

        // ws.close(()).await?;
        ws.stream
            .write_all(CloseFrame::encode::<CLIENT>(()).as_ref()) // manually closing
            .await?;

        assert!(matches!(ws.recv().await, Ok(Event::Close { .. })));
        Ok(println!("web-socket:  {:?}", time.elapsed()))
    }

    async fn server(stream: Stream) -> Result<()> {
        let mut ws = WebSocket::server(stream);
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
                Event::Ping(data) => ws.send(Pong(data)).await?,
                Event::Error(..) | Event::Close { .. } => return ws.close(()).await,
            }
        }
    }
}

mod fastwebsockets_banchmark {
    use super::*;
    use fastwebsockets::*;

    type DynErr = Box<dyn std::error::Error + Sync + Send>;
    type Result<T, E = DynErr> = std::result::Result<T, E>;

    pub async fn run() {
        bench(server, client).await
    }

    async fn client(stream: Stream) -> Result<()> {
        let mut ws = WebSocket::after_handshake(stream, Role::Client);
        ws.set_auto_pong(true);
        ws.set_writev(true);
        ws.set_auto_close(true);

        let time = Instant::now();
        for _ in 0..ITER {
            ws.write_frame(Frame::new(
                true,
                OpCode::Text,
                Some(rand::random()),
                MSG.into(),
            ))
            .await?;
        }
        for _ in 0..ITER {
            let frame = ws.read_frame().await?;
            assert!(frame.fin);
            assert_eq!(frame.opcode, OpCode::Text);
            assert_eq!(std::str::from_utf8(&frame.payload), Ok(MSG));
        }

        ws.write_frame(Frame::new(
            true,
            OpCode::Close,
            Some(rand::random()),
            Vec::new(),
        ))
        .await?;

        assert_eq!(ws.read_frame().await.unwrap().opcode, OpCode::Close);
        Ok(println!("fastwebsockets:  {:?}", time.elapsed()))
    }

    async fn server(stream: Stream) -> Result<()> {
        let mut ws = WebSocket::after_handshake(stream, Role::Server);
        ws.set_auto_apply_mask(true);
        ws.set_auto_pong(true);
        ws.set_writev(true);
        ws.set_auto_close(true);

        loop {
            let frame = ws.read_frame().await?;
            match frame.opcode {
                OpCode::Text | OpCode::Binary => {
                    if let OpCode::Text = frame.opcode {
                        assert!(std::str::from_utf8(&frame.payload).is_ok())
                    }
                    ws.write_frame(Frame::new(true, frame.opcode, None, frame.payload))
                        .await?;
                }
                OpCode::Close => break Ok(()),
                _ => {}
            }
        }
    }
}

mod tokio_tungstenite_banchmark {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{
        accept_async, client_async,
        tungstenite::{Message, Result},
    };

    pub async fn run() {
        bench(server, client).await
    }

    async fn client(stream: Stream) -> Result<()> {
        let (mut ws, _) = client_async("ws://localhost:9001", stream).await.unwrap();

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
        ws.close(None).await?;
        assert!(matches!(ws.next().await, Some(Ok(Message::Close(..)))));
        Ok(println!("tokio-tungstenite: {:?}", time.elapsed()))
    }

    async fn server(stream: Stream) -> Result<()> {
        let mut ws = accept_async(stream).await.expect("Failed to accept");
        while let Some(msg) = ws.next().await {
            let msg = msg?;
            if msg.is_text() || msg.is_binary() {
                ws.send(msg).await?;
            }
        }
        Ok(())
    }
}
