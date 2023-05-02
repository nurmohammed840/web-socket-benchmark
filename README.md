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
web-socket:  33.499539ms
fastwebsockets:  43.990645ms
tokio-tungstenite: 133.883056ms
```
