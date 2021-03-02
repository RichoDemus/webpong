#[cfg(not(target_arch = "wasm32"))]
mod ws_server;

#[cfg(not(target_arch = "wasm32"))]
mod ws_client;
#[cfg(target_arch = "wasm32")]
mod ws_client_wasm_two;
pub mod event_stream;
// mod ws_client_wasm_stream;

use std::env;

use quicksilver::{geom::{Rectangle, Vector}, Graphics, graphics::Color, Input, Result, run, Settings, Window, Timer};
use quicksilver::log::warn;

#[cfg(not(target_arch = "wasm32"))]
use crate::ws_client::Websocket;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use crate::ws_client_wasm_two::Websocket;
// use crate::ws_client_wasm_stream::Websocket;
use std::sync::mpsc::TryRecvError;
use futures::StreamExt;
use crate::event_stream::WsEvent;
use web_sys::console::warn;
use quicksilver::graphics::VectorFont;

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

// #[cfg(not(target_arch = "wasm32"))]
// #[tokio::main]
fn main() {
    let args: Vec<String> = env::args().collect();

    if let Some(arg) = args.get(1) {
        if arg.eq("--server") {
            #[cfg(not(target_arch = "wasm32"))]
            ws_server::start_ws_server();
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
    let mut ws: Websocket = ws_client::start_ws_client();
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
    #[cfg(target_arch = "wasm32")]
    let mut ws: Websocket = ws_client_wasm_two::Websocket::open("ws://localhost:8080/echo").await;
    // let mut ws: Websocket = ws_client_wasm_stream::start_ws_client().await;

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
        #[cfg(target_arch = "wasm32")]
        while let Some(evt) = ws.event_stream.next_event().await {
            let evt: WsEvent = evt;
            match evt {
                WsEvent::Opened => {
                    console_log!("main: open");
                    // ws.send("1");
                }
                WsEvent::Message(msg) => {
                    console_log!("main: msg: {:?}", msg);
                    last_ws_message = msg.clone();
                    ws.send(msg.as_str());
                }
                WsEvent::Error(_) => {}
                WsEvent::Closed => console_log!("main: closed"),
            }
        }

        while update_timer.tick() {
            rect.pos.x += 5.0;
        }

        if draw_timer.exhaust().is_some() {
            gfx.clear(Color::WHITE);
            gfx.fill_rect(&rect, Color::BLUE);
            gfx.stroke_rect(&rect, Color::RED);

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
