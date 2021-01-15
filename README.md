

```
cargo build --release --target wasm32-unknown-unknown --features quicksilver/web-sys
wasm-bindgen --no-typescript --target web --out-name wasm --out-dir target .\target\wasm32-unknown-unknown\release\quicksilver-web.wasm
simple-http-server
```
