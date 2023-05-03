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
web-socket:  32.26691ms
fastwebsockets:  38.675652ms
tokio-tungstenite: 142.147423ms
```
