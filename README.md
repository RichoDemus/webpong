# webpong
currently being deployed to [gh pages](https://richodemus.github.io/webpong/) via a github action

## deploy to gh pages
see rust.yaml

### redeploy on server
```
ssh richodemus@richodemus.com "docker-compose -f richo-google-cloud/webpong/docker-compose.yaml pull && docker-compose -f richo-google-cloud/webpong/docker-compose.yaml down && docker-compose -f richo-google-cloud/webpong/docker-compose.yaml up -d"
```

## locally
```
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --target web --out-name wasm --out-dir target .\target\wasm32-unknown-unknown\release\webpong.wasm
simple-http-server
```

## locally one-liner
```
cargo build --target wasm32-unknown-unknown; wasm-bindgen --no-typescript --target web --out-name wasm --out-dir target .\target\wasm32-unknown-unknown\debug\webpong.wasm; cp index.html target\index.html; simple-http-server.exe target
```

## run a bunch of verifications
```
cargo check; cargo check --target wasm32-unknown-unknown; cargo check --tests
```

## Super strict clippy
```
cargo clippy -- -W clippy::nursery -W clippy::pedantic -W clippy::cargo
```
