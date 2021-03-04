#[cfg(not(target_arch = "wasm32"))]
mod ws_server;

#[cfg(not(target_arch = "wasm32"))]
mod ws_client;
#[cfg(target_arch = "wasm32")]
mod ws_client_wasm_two;
pub mod event_stream;
pub mod event_stream_mutex;
pub mod ws_event;
mod simple_pong;
mod draw;
// mod ws_client_wasm_stream;

use std::env;

use quicksilver::{geom::{Rectangle, Vector}, Graphics, graphics::Color, Input, Result, run, Settings, Window, Timer};
use quicksilver::log::warn;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use crate::ws_client_wasm_two::Websocket;
// use crate::ws_client_wasm_stream::Websocket;
use std::sync::mpsc::TryRecvError;
use futures::StreamExt;
use web_sys::console::warn;
use quicksilver::graphics::VectorFont;
#[cfg(not(target_arch = "wasm32"))]
use crate::ws_client::Websocket;
use crate::ws_event::WsEvent;
use quicksilver::blinds::Key;

#[cfg(target_arch = "wasm32")]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
#[cfg(not(target_arch = "wasm32"))]
async fn main() {

    let args: Vec<String> = env::args().collect();

    if let Some(arg) = args.get(1) {
        if arg.eq("--server") {
            #[cfg(not(target_arch = "wasm32"))]
                ws_server::start_ws_server().await;
            return;
        }
    }

    run(
        Settings {
            title: "Square Example",
            size: Vector::new(1600., 800.),
            ..Settings::default()
        },
        app,
    );
}

#[cfg(target_arch = "wasm32")]
fn main() {

    let args: Vec<String> = env::args().collect();

    if let Some(arg) = args.get(1) {
        if arg.eq("--server") {
            #[cfg(not(target_arch = "wasm32"))]
            ws_server::start_ws_server().await;
            return;
        }
    }

        run(
        Settings {
            title: "Square Example",
            ..Settings::default()
        },
        app,
    );
}


// #[cfg(target_arch = "wasm32")]
// fn main() {
//     run(
//         Settings {
//             title: "Square Example",
//             ..Settings::default()
//         },
//         app,
//     );
// }

async fn app(window: Window, mut gfx: Graphics, mut input: Input) -> Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    let mut ws: Websocket = ws_client::Websocket::open("ws://localhost:8080/echo").await;
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
    #[cfg(target_arch = "wasm32")]
    let mut ws: Websocket = ws_client_wasm_two::Websocket::open("ws://localhost:8080/echo").await;
    // let mut ws: Websocket = ws_client_wasm_stream::start_ws_client().await;

    let mut simple_pong = simple_pong::SimplePong::new();

    let mut update_timer = Timer::time_per_second(30.0);
    let mut draw_timer = Timer::time_per_second(60.0);


    let ttf = VectorFont::from_slice(include_bytes!("BebasNeue-Regular.ttf"));
    let mut font = ttf.to_renderer(&gfx, 20.0)?;

    let mut rect = Rectangle::new(Vector::new(0.0, 100.0), Vector::new(100.0, 100.0));

    let mut last_ws_message = String::new();

    loop {
        // warn!("loop...");
        while let Some(_) = input.next_event().await {}

        // #[cfg(target_arch = "wasm32")]
        // while let Some(evt) = ws.event_stream.next_event().await {
        //     warn!("got event: {:?}", evt);
        // }

        // warn!("loop");
        // #[cfg(target_arch = "wasm32")]
        while let Some(evt) = ws.event_stream.next_event().await {
            let evt: WsEvent = evt;
            match evt {
                WsEvent::Opened => {
                    // warn!("main: open");
                    // ws.send("1");
                }
                WsEvent::Message(msg) => {
                    // warn!("main: msg: {:?}", msg);
                    last_ws_message = msg.clone();
                    #[cfg(not(target_arch = "wasm32"))]
                    ws.send(msg.as_str()).await;
                    #[cfg(target_arch = "wasm32")]
                    ws.send(msg.as_str());
                }
                WsEvent::Error(_) => {}
                WsEvent::Closed => (),
            }
        }

        while update_timer.tick() {
            if input.key_down(Key::W) {
                simple_pong.move_up();
            }
            if input.key_down(Key::S) {
                simple_pong.move_down();
            }
            simple_pong.tick();
        }

        if draw_timer.exhaust().is_some() {
            gfx.clear(Color::BLACK);

            let (left_paddle, right_paddle, ball) = simple_pong.get_drawables();

            draw::draw(&mut gfx, left_paddle, right_paddle, ball);

            font.draw(
                &mut gfx,
                format!("ws: {}", last_ws_message).as_str(),
                Color::GREEN,
                Vector::new(10.0, 30.0),
            )?;

            gfx.present(&window)?;
        }
    }
}
