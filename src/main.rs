use std::env;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

#[cfg(not(target_arch = "wasm32"))]
use log::*;
use quicksilver::blinds::Key;
use quicksilver::graphics::VectorFont;
use quicksilver::input::Event;
use quicksilver::{
    geom::Vector, graphics::Color, run, Graphics, Input, Result, Settings, Timer, Window,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::simple_pong::SimplePong;
#[cfg(not(target_arch = "wasm32"))]
use crate::ws_client::Websocket;
#[cfg(target_arch = "wasm32")]
use crate::ws_client_wasm_two::Websocket;
use crate::ws_event::WsEvent;

mod draw;
pub mod event_stream_mutex;
#[cfg(not(target_arch = "wasm32"))]
pub mod event_stream_mutex_client;
mod simple_pong;
mod websocket_test;
#[cfg(not(target_arch = "wasm32"))]
mod ws_client;
#[cfg(target_arch = "wasm32")]
mod ws_client_wasm_two;
pub mod ws_event;
#[cfg(not(target_arch = "wasm32"))]
pub mod ws_server2;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
#[cfg(not(target_arch = "wasm32"))]
async fn main() {
    let args: Vec<String> = env::args().collect();

    #[cfg(not(target_arch = "wasm32"))]
    if let Some(arg) = args.get(1) {
        if arg.eq("--server") {
            // #[cfg(not(target_arch = "wasm32"))]
            //     let ws_server = ws_server::WsServer::new();

            server_logic().await;

            return;
        }
    }

    run(
        Settings {
            title: "Square Example",
            size: Vector::new(800., 600.),
            log_level: Level::Info,
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

#[cfg(not(target_arch = "wasm32"))]
async fn server_logic() {
    let _ = env_logger::builder()
        .filter_module("webpong", log::LevelFilter::Info)
        .try_init();
    let mut ws_server = ws_server2::WebsocketServer::start()
        .await
        .expect("start ws server");
    let time_between_ticks = Duration::from_secs_f32(1.0 / 10.);
    let start = Instant::now();

    let mut players = vec![];

    let mut simple_pong = SimplePong::new();
    // simple_pong.toggle_pause();

    let mut next_tick = start + time_between_ticks;
    loop {
        while let Some(client) = ws_server.event_stream.next_event().await {
            players.push(client);
        }

        let mut messages_to_send = vec![];
        let mut indexes_to_remove = vec![];
        for (i, player) in players.iter_mut().enumerate() {
            while let Some(msg) = player.event_stream.next_event().await {
                match msg {
                    WsEvent::Message(msg) => {
                        log::info!("Got message from client {}: {:?}", i, msg);
                        messages_to_send.push(format!("{} {}", i, msg));
                    }
                    WsEvent::Closed => {
                        info!("received error for {}, closing", i);
                        indexes_to_remove.push(i);
                    }
                    _ => {}
                }
            }
        }

        {
            // remove disconnected players
            indexes_to_remove.sort();
            indexes_to_remove.reverse();
            for index in indexes_to_remove {
                if let Some(_) = players.get(index) {
                    players.remove(index);
                    info!("Removed player {}", index);
                }
            }
        }

        for msg in messages_to_send {
            for player in &mut players {
                player.send(msg.as_str()).await;
            }
        }

        simple_pong.tick();

        // log::info!("{:?}", simple_pong.get_drawables());

        next_tick = next_tick + time_between_ticks;
        while Instant::now() < next_tick {
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }
    }
}

async fn app(window: Window, mut gfx: Graphics, mut input: Input) -> Result<()> {
    #[cfg(debug_assertions)]
    let ws_url = "ws://localhost:8080";
    #[cfg(not(debug_assertions))]
    let ws_url = "wss://webpong.richodemus.com";
    #[cfg(not(target_arch = "wasm32"))]
    // let mut ws: Websocket = ws_client::Websocket::open("ws://localhost:8080").await;
    let mut ws: Websocket = ws_client::Websocket::open(ws_url).await;
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
    #[cfg(target_arch = "wasm32")]
    let mut ws: Websocket = ws_client_wasm_two::Websocket::open(ws_url).await;
    // let mut ws: Websocket = ws_client_wasm_stream::start_ws_client().await;

    let mut simple_pong = simple_pong::SimplePong::new();

    let mut update_timer = Timer::time_per_second(60.0);
    let mut draw_timer = Timer::time_per_second(60.0);

    let ttf = VectorFont::from_slice(include_bytes!("BebasNeue-Regular.ttf"));
    let mut font = ttf.to_renderer(&gfx, 20.0)?;

    let last_ws_message = String::new();

    let mut is_w_pressed = false;
    let mut is_s_pressed = false;

    loop {
        // warn!("loop...");
        while let Some(evt) = input.next_event().await {
            match evt {
                Event::KeyboardInput(key) => match key.key() {
                    Key::P => {
                        if key.is_down() {
                            simple_pong.toggle_pause();
                        }
                    }
                    Key::W => {
                        if !key.is_down() {
                            is_w_pressed = false;
                            #[cfg(not(target_arch = "wasm32"))]
                            ws.send("not up").await;
                            #[cfg(target_arch = "wasm32")]
                            ws.send("not up");
                        } else if key.is_down() && !is_w_pressed {
                            is_w_pressed = true;
                            #[cfg(not(target_arch = "wasm32"))]
                            ws.send("up").await;
                            #[cfg(target_arch = "wasm32")]
                            ws.send("up");
                        }
                    }
                    Key::S => {
                        if !key.is_down() {
                            is_s_pressed = false;
                            #[cfg(not(target_arch = "wasm32"))]
                            ws.send("not down").await;
                            #[cfg(target_arch = "wasm32")]
                            ws.send("not down");
                        } else if key.is_down() && !is_s_pressed {
                            is_s_pressed = true;
                            #[cfg(not(target_arch = "wasm32"))]
                            ws.send("down").await;
                            #[cfg(target_arch = "wasm32")]
                            ws.send("down");
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        }

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
                    // i up
                    // i not up
                    // i down
                    // i not down
                    if !msg.contains("0") && !msg.contains("1") {
                        continue;
                    }
                    let left_paddle = msg.contains("0");
                    let stop_moving = msg.contains("not");
                    let up = msg.contains("up");

                    simple_pong.set_paddle_state(left_paddle, stop_moving, up);

                    // info!("inc: {:?}", msg);
                    // warn!("main: msg: {:?}", msg);
                    // last_ws_message = msg.clone();
                    // #[cfg(not(target_arch = "wasm32"))]
                    // ws.send(msg.as_str()).await;
                    // #[cfg(target_arch = "wasm32")]
                    // ws.send(msg.as_str());
                }
                WsEvent::Error(_) => {}
                WsEvent::Closed => (),
            }
        }

        while update_timer.tick() {
            if input.key_down(Key::W) {
                // simple_pong.move_up();
                // ws.send("move up").await;
                // ws.send("move up").await;
            }
            if input.key_down(Key::S) {
                // simple_pong.move_down();
                // ws.send("move down");
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
