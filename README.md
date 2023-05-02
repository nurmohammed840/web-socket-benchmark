### Candidate

- [tokio-tungstenite](https://github.com/snapview/tokio-tungstenite)
- [web-socket](https://github.com/nurmohammed840/websocket.rs)
- [fastwebsockets](https://github.com/denoland/fastwebsockets)

### Run benchmark

```bash
cargo run --release -- -C codegen-units=1 -C opt-level=3
```

### Result

Tested on the Github Action.

```
web-socket:  30.373163ms
tungstenite: 119.015605ms
```
