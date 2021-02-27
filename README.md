# quicksilver wasm example
Shows how to compile a [quicksilver](https://github.com/ryanisaacg/quicksilver) program to wasm (and deploy it to gh pages)  
currently being deployed to [gh pages](https://richodemus.github.io/quicksilver-web-example/) via a github action

## deploy to gh pages
see rust.yaml

## locally
```
cargo build --release --target wasm32-unknown-unknown --features quicksilver/web-sys
wasm-bindgen --no-typescript --target web --out-name wasm --out-dir target .\target\wasm32-unknown-unknown\release\webpong.wasm
simple-http-server
```
