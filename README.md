# webpong
currently being deployed to [gh pages](https://richodemus.github.io/webpong/) via a github action

## deploy to gh pages
see rust.yaml

## locally
```
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --target web --out-name wasm --out-dir target .\target\wasm32-unknown-unknown\release\webpong.wasm
simple-http-server
```
