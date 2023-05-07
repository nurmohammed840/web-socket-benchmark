mod bench;
use std::time::Instant;

const ITER: usize = 100000;
const MSG: &str = "Hello, World!\n";
const CAPACITY: usize = (MSG.len() + 14) * ITER;

const HELP: &str = "Please run with `--release` flag for accurate results.
Example:
    cargo run --release
    cargo run -r -- -C codegen-units=1 -C opt-level=3
";

fn main() {
    if cfg!(debug_assertions) {
        println!("{HELP}");
    }
    bench::block_on(async {
        tokio_tungstenite_banchmark::run().await.unwrap();
        web_socket_banchmark::run().await.unwrap();
        fastwebsockets_banchmark::run().await.unwrap();
    });
}

mod web_socket_banchmark {
    use super::*;
    use std::io::Result;
    use tokio::io::AsyncWrite;
    use web_socket::*;

    async fn send_msg<IO>(ws: &mut WebSocket<IO>, ty: MessageType, buf: &[u8]) -> Result<()>
    where
        IO: Unpin + AsyncWrite,
    {
        match ty {
            MessageType::Text => ws.send(std::str::from_utf8(buf).unwrap()).await,
            MessageType::Binary => ws.send(buf).await,
        }
    }

    pub async fn run() -> Result<()> {
        let mut stream = bench::Stream::new(CAPACITY);
        let total = Instant::now();

        // ------------------------------------------------
        stream.role_client();

        let mut ws = WebSocket::client(&mut stream);
        let send = Instant::now();

        for _ in 0..ITER {
            ws.send(MSG).await?;
        }
        ws.close(()).await?;

        let send = send.elapsed();

        // ------------------------------------------------
        stream.role_server();

        let mut ws = WebSocket::server(&mut stream);
        let echo = Instant::now();

        let mut buf = Vec::new();
        loop {
            match ws.recv_event().await? {
                Event::Data { data, ty } => match ty {
                    DataType::Stream(stream) => {
                        buf.extend_from_slice(&data);
                        if let Stream::End(ty) = stream {
                            send_msg(&mut ws, ty, &buf).await?;
                            buf.clear();
                        }
                    }
                    DataType::Complete(ty) => send_msg(&mut ws, ty, &data).await?,
                },
                Event::Pong(..) => {}
                Event::Ping(data) => ws.send_ping(data).await?,
                Event::Error(..) | Event::Close { .. } => break ws.close(()).await?,
            }
        }
        let echo = echo.elapsed();

        // ------------------------------------------------
        stream.role_client();

        let mut ws = WebSocket::client(&mut stream);
        let recv = Instant::now();

        for _ in 0..ITER {
            let Ok(Event::Data { ty, data }) = ws.recv_event().await else { panic!("invalid data") };
            assert!(matches!(ty, DataType::Complete(MessageType::Text)));
            assert_eq!(std::str::from_utf8(&data), Ok(MSG));
        }
        assert!(matches!(ws.recv_event().await, Ok(Event::Close { .. })));
        let recv = recv.elapsed();

        // ------------------------------------------------
        let total = total.elapsed();
        println!("\n");
        println!("web-socket (send):  {send:?}");
        println!("web-socket (echo):  {echo:?}");
        println!("web-socket (recv):  {recv:?}");
        println!("web-socket:         {total:?}",);
        Ok(())
    }
}

mod fastwebsockets_banchmark {
    use super::*;
    use fastwebsockets::*;

    type DynErr = Box<dyn std::error::Error + Sync + Send>;
    type Result<T, E = DynErr> = std::result::Result<T, E>;

    pub async fn run() -> Result<()> {
        let mut stream = bench::Stream::new(CAPACITY);
        let total = Instant::now();

        // ------------------------------------------------
        stream.role_client();

        let mut ws = WebSocket::after_handshake(&mut stream, Role::Client);
        ws.set_auto_pong(true);
        ws.set_writev(true);
        ws.set_auto_close(true);

        let send = Instant::now();

        for _ in 0..ITER {
            ws.write_frame(Frame::new(true, OpCode::Text, None, MSG.as_bytes().into()))
                .await?;
        }
        ws.write_frame(Frame::new(true, OpCode::Close, None, (&[] as &[u8]).into()))
            .await?;

        let send = send.elapsed();

        // ------------------------------------------------
        stream.role_server();

        let mut ws = WebSocket::after_handshake(&mut stream, Role::Server);
        ws.set_auto_apply_mask(true);
        ws.set_auto_pong(true);
        ws.set_writev(true);
        ws.set_auto_close(true);

        let echo = Instant::now();
        let mut ws = FragmentCollector::new(ws);
        loop {
            let frame = ws.read_frame().await?;
            match frame.opcode {
                OpCode::Close => break,
                OpCode::Text | OpCode::Binary => {
                    ws.write_frame(frame).await?;
                }
                _ => {}
            }
        }
        let echo = echo.elapsed();

        // -----------------------------------------------
        stream.role_client();

        let mut ws = WebSocket::after_handshake(&mut stream, Role::Client);
        let recv = Instant::now();
        for _ in 0..ITER {
            let frame = ws.read_frame().await?;
            assert!(frame.fin);
            assert_eq!(frame.opcode, OpCode::Text);
            assert_eq!(frame.payload, MSG.as_bytes());
        }
        assert_eq!(ws.read_frame().await.unwrap().opcode, OpCode::Close);
        let recv = recv.elapsed();

        // ------------------------------------------------
        let total = total.elapsed();

        println!("\n");
        println!("fastwebsockets (send):  {send:?}");
        println!("fastwebsockets (echo):  {echo:?}");
        println!("fastwebsockets (recv):  {recv:?}");
        println!("fastwebsockets:         {total:?}",);
        Ok(())
    }
}

mod tokio_tungstenite_banchmark {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{
        tungstenite::{protocol::Role, Message, Result},
        WebSocketStream,
    };

    pub async fn run() -> Result<()> {
        let mut stream = bench::Stream::new(CAPACITY);
        let total = Instant::now();

        // ------------------------------------------------
        stream.role_client();

        let mut ws = WebSocketStream::from_raw_socket(&mut stream, Role::Client, None).await;
        let send = Instant::now();
        for _ in 0..ITER {
            ws.feed(Message::Text(MSG.to_owned())).await?;
        }
        ws.close(None).await?;
        let send = send.elapsed();
        // ------------------------------------------------
        stream.role_server();

        let mut ws = WebSocketStream::from_raw_socket(&mut stream, Role::Server, None).await;
        let echo = Instant::now();
        while let Some(msg) = ws.next().await {
            let msg = msg?;
            if msg.is_text() || msg.is_binary() {
                ws.feed(msg).await?;
            }
        }
        let echo = echo.elapsed();
        // ------------------------------------------------
        stream.role_client();

        let mut ws = WebSocketStream::from_raw_socket(&mut stream, Role::Client, None).await;
        let recv = Instant::now();
        for _ in 0..ITER {
            match ws.next().await.unwrap()? {
                Message::Text(data) => assert_eq!(MSG, data),
                _ => unimplemented!(),
            }
        }
        assert!(matches!(ws.next().await, Some(Ok(Message::Close(..)))));
        let recv = recv.elapsed();

        // ------------------------------------------------
        let total = total.elapsed();
        println!("\n");
        println!("tokio_tungstenite (send):  {send:?}");
        println!("tokio_tungstenite (echo):  {echo:?}");
        println!("tokio_tungstenite (recv):  {recv:?}");
        println!("tokio_tungstenite:         {total:?}",);
        Ok(())
    }
}
