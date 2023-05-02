### Candidate

- [tokio-tungstenite](https://github.com/snapview/tokio-tungstenite)
- [web-socket](https://github.com/nurmohammed840/websocket.rs)
- [fastwebsockets](https://github.com/denoland/fastwebsockets)

### Run benchmark

```bash
cargo run --release -- -C codegen-units=1 -C opt-level=3
```

### Result

```
web-socket:  33.369744ms
fastwebsockets:  42.73404ms
tokio-tungstenite: 134.120882ms
```
