### Candidate

- [web-socket](https://github.com/nurmohammed840/websocket.rs)
- [fastwebsockets](https://github.com/denoland/fastwebsockets)
- [tokio-tungstenite](https://github.com/snapview/tokio-tungstenite)

### Run benchmark

```bash
cargo run -r
```

### Result

```
tokio_tungstenite (send):  19.475364ms
tokio_tungstenite (echo):  44.17762ms
tokio_tungstenite (recv):  26.707251ms
tokio_tungstenite:         90.375235ms


web-socket (send):  5.73869ms
web-socket (echo):  16.861769ms
web-socket (recv):  8.589784ms
web-socket:         31.190643ms


fastwebsockets (send):  8.340585ms
fastwebsockets (echo):  16.953069ms
fastwebsockets (recv):  9.192684ms
fastwebsockets:         34.487038ms
```
