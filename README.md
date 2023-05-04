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
tokio_tungstenite (send):  22.08092ms
tokio_tungstenite (echo):  50.137045ms
tokio_tungstenite (recv):  29.825227ms
tokio_tungstenite:         102.059792ms


web-socket (send):  5.817706ms
web-socket (echo):  17.335015ms
web-socket (recv):  8.911808ms
web-socket:         32.064929ms


fastwebsockets (send):  8.323607ms
fastwebsockets (echo):  15.534314ms
fastwebsockets (recv):  9.254209ms
fastwebsockets:         33.11283ms
```
