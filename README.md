### Candidate

- [web-socket](https://github.com/nurmohammed840/websocket.rs)
- [fastwebsockets](https://github.com/denoland/fastwebsockets)
- [tokio-tungstenite](https://github.com/snapview/tokio-tungstenite)

### Run benchmark

```bash
cargo run --release -- -C codegen-units=1 -C opt-level=3
```

### Result

```
web-socket:  38.334456ms
fastwebsockets:  39.671372ms
tokio-tungstenite: 134.829302ms
```
