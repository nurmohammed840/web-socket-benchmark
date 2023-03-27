Run this benchmark with:

```bash
cargo run --release -- -C codegen-units=1 -C opt-level=3
```

### Result

Tested on the Github codespace.

```
web-socket:  30.373163ms
tungstenite: 119.015605ms
```