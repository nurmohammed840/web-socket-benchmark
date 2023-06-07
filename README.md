### Candidates

- [fastwebsockets](https://github.com/denoland/fastwebsockets)
- [soketto](https://github.com/paritytech/soketto)
- [tokio-tungstenite](https://github.com/snapview/tokio-tungstenite)
- [web-socket](https://github.com/nurmohammed840/websocket.rs)

### Run benchmark

```bash
cargo r -r
```

### Results

#### [Intel® Core™ i9-13900K Processor](https://www.intel.com/content/www/us/en/products/sku/230496/intel-core-i913900k-processor-36m-cache-up-to-5-80-ghz/specifications.html) - [@AurevoirXavier](https://github.com/AurevoirXavier)
```
fastwebsockets (send):  3.1261ms
fastwebsockets (echo):  8.080872ms
fastwebsockets (recv):  5.525353ms
fastwebsockets:         16.732845ms


soketto (send):  7.998258ms
soketto (echo):  18.857447ms
soketto (recv):  10.228193ms
soketto:         37.08868ms


tokio_tungstenite (send):  7.722288ms
tokio_tungstenite (echo):  17.284609ms
tokio_tungstenite (recv):  10.407554ms
tokio_tungstenite:         35.427836ms


web-socket (send):  2.369528ms
web-socket (echo):  6.243006ms
web-socket (recv):  3.372092ms
web-socket:         11.985043ms
```
