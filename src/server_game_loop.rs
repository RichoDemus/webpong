use log::*;
use tokio::time::{Duration, Instant};

use crate::network::message::ServerMessage::PaddleDown;
use crate::network::message::ServerMessage::{PaddleStop, PaddleUp};
use crate::network::message::{ClientMessage, Message, PaddleId};
use crate::network::ws_event::WsEvent;
use crate::network::ws_server;
use crate::simple_pong::SimplePong;

pub async fn run() {
    server_logic().await;
}

async fn server_logic() {
    let _ = env_logger::builder()
        .filter_module("webpong", log::LevelFilter::Info)
        .try_init();
    let mut ws_server = ws_server::WebsocketServer::start()
        .await
        .expect("start ws server");
    let time_between_ticks = Duration::from_secs_f32(1.0 / 10.);
    let start = Instant::now();

    let mut players = vec![];

    let mut simple_pong = SimplePong::new();

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
                        match msg {
                            Message::Ping => {}
                            Message::ClientMessage(msg) => match msg {
                                ClientMessage::SetName(_) => {}
                                ClientMessage::PaddleUp => {
                                    if i > 1 {
                                        // this is a spectator
                                        continue;
                                    }
                                    let paddle_id = if i == 0 {
                                        PaddleId::Left
                                    } else {
                                        PaddleId::Right
                                    };

                                    messages_to_send
                                        .push(Message::ServerMessage(PaddleUp(paddle_id)));
                                }
                                ClientMessage::PaddleDown => {
                                    if i > 1 {
                                        // this is a spectator
                                        continue;
                                    }
                                    let paddle_id = if i == 0 {
                                        PaddleId::Left
                                    } else {
                                        PaddleId::Right
                                    };

                                    messages_to_send
                                        .push(Message::ServerMessage(PaddleDown(paddle_id)));
                                }
                                ClientMessage::PaddleStop => {
                                    if i > 1 {
                                        // this is a spectator
                                        continue;
                                    }
                                    let paddle_id = if i == 0 {
                                        PaddleId::Left
                                    } else {
                                        PaddleId::Right
                                    };

                                    messages_to_send
                                        .push(Message::ServerMessage(PaddleStop(paddle_id)));
                                }
                            },
                            Message::ServerMessage(_) => {}
                        }
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

        for msg in &messages_to_send {
            for player in &mut players {
                player.send(msg).await;
            }
        }

        simple_pong.tick();

        next_tick = next_tick + time_between_ticks;
        while Instant::now() < next_tick {
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }
    }
}
