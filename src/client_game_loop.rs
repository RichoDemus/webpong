#[cfg(not(target_arch = "wasm32"))]
use log::*;
use quicksilver::blinds::Key;
use quicksilver::graphics::VectorFont;
use quicksilver::input::Event;
use quicksilver::{
    geom::Vector, graphics::Color, Graphics, Input, Result, Settings, Timer, Window,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::ws_client::Websocket;
#[cfg(target_arch = "wasm32")]
use crate::ws_client_wasm::Websocket;
use crate::ws_event::WsEvent;
use crate::{draw, simple_pong};

pub fn run() {
    quicksilver::run(
        Settings {
            title: "Square Example",
            size: Vector::new(800., 600.),
            #[cfg(not(target_arch = "wasm32"))]
            log_level: Level::Info,
            ..Settings::default()
        },
        app,
    );
}

async fn app(window: Window, mut gfx: Graphics, mut input: Input) -> Result<()> {
    let mut ws: Websocket = Websocket::open().await;

    let mut simple_pong = simple_pong::SimplePong::new();

    let mut update_timer = Timer::time_per_second(60.0);
    let mut draw_timer = Timer::time_per_second(60.0);

    let ttf = VectorFont::from_slice(include_bytes!("BebasNeue-Regular.ttf"));
    let mut font = ttf.to_renderer(&gfx, 20.0)?;

    let last_ws_message = String::new();

    let mut is_w_pressed = false;
    let mut is_s_pressed = false;

    loop {
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
                            ws.send("not up").await;
                        } else if key.is_down() && !is_w_pressed {
                            is_w_pressed = true;
                            ws.send("up").await;
                        }
                    }
                    Key::S => {
                        if !key.is_down() {
                            is_s_pressed = false;
                            ws.send("not down").await;
                        } else if key.is_down() && !is_s_pressed {
                            is_s_pressed = true;
                            ws.send("down").await;
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        }

        while let Some(evt) = ws.event_stream.next_event().await {
            let evt: WsEvent = evt;
            match evt {
                WsEvent::Opened => {}
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
                }
                WsEvent::Error(_) => {}
                WsEvent::Closed => (),
            }
        }

        while update_timer.tick() {
            if input.key_down(Key::W) {}
            if input.key_down(Key::S) {}
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
