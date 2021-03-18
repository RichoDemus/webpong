#[cfg(not(target_arch = "wasm32"))]
use std::env;

mod draw;
pub mod event_stream;
mod simple_pong;
mod websocket_test;
#[cfg(not(target_arch = "wasm32"))]
mod ws_client;
#[cfg(target_arch = "wasm32")]
mod ws_client_wasm;
pub mod ws_event;
#[cfg(not(target_arch = "wasm32"))]
pub mod ws_server;
mod client_game_loop;

#[cfg(not(target_arch = "wasm32"))]
mod server_game_loop;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
#[cfg(not(target_arch = "wasm32"))]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if let Some(arg) = args.get(1) {
        if arg.eq("--server") {
            server_game_loop::run().await;
            return;
        }
    }
    client_game_loop::run();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    client_game_loop::run();
}
