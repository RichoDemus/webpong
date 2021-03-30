#[cfg(not(target_arch = "wasm32"))]
use std::env;

mod network;

mod client_game_loop;
mod draw;
pub mod event_stream;
mod simple_pong;

#[cfg(not(target_arch = "wasm32"))]
mod server;
#[cfg(test)]
mod play_around_w_server;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
#[cfg(not(target_arch = "wasm32"))]
async fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let args: Vec<String> = env::args().collect();

    if let Some(arg) = args.get(1) {
        if arg.eq("--server") {
            server::server::start().await;
            return;
        }
    }
    client_game_loop::run();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    client_game_loop::run();
}
