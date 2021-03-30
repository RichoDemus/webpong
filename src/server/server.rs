use log::*;
use tokio::time::Duration;

use crate::network::message::{ClientMessage, Message, ServerMessage};
use crate::network::ws_event::WsEvent;
use crate::network::ws_server;
use crate::server::pong_server::PongServer;
use quicksilver::Timer;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::sync::{Arc, Mutex};

pub struct Server {
    running: Arc<Mutex<bool>>,
}

pub async fn start() {
    let _ = env_logger::builder()
        .filter_module("webpong", log::LevelFilter::Trace)
        .try_init();
    let mut ws_server = ws_server::WebsocketServer::start()
        .await
        .expect("start ws server");

    let mut pre_lobby_players = vec![];
    // let mut players_in_game = vec![];

    let mut new_player_in_game = false; //set to true when a new player joins to signal gamestate sync

    let mut pong_server = PongServer::default();

    let mut update_timer = Timer::time_per_second(100.0);
    let mut running = Arc::new(Mutex::new(true));
    let mut running_clone = running.clone();
    //lobby
    while *running_clone.lock().unwrap() == true {
        while update_timer.tick() {
            // next_tick = Instant::now() + Duration::from_millis(10);
            while let Some(mut client) = ws_server.event_stream.next_event().await {
                info!("Client {} connected", client.id);
                let rand_string: String = thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(5)
                    .map(char::from)
                    .collect();
                client.name = Some(rand_string.clone());
                client
                    .send(&Message::ServerMessage(ServerMessage::SetName(rand_string)))
                    .await;
                pre_lobby_players.push(client);
            }

            pre_lobby_players = pre_lobby_players
                .into_iter()
                .filter_map(|player| {
                    let event = player
                        .event_stream
                        .buffer()
                        .lock()
                        .expect("asd")
                        .pop_front();

                    if let Some(WsEvent::Error(e)) = event {
                        warn!("Err: {:?}", e);
                        Some(player)
                    } else if let Some(WsEvent::Message(Message::ClientMessage(
                        ClientMessage::EnterGame,
                    ))) = event
                    {
                        info!("Player {} entering game", player);
                        pong_server.add_player(player);
                        new_player_in_game = true;
                        None
                    } else if let Some(WsEvent::Closed) = event {
                        None
                    } else {
                        Some(player)
                    }
                })
                .collect();

            // for player in &mut players_in_game {
            //     if new_player_in_game {
            //         player
            //             .send(&Message::ServerMessage(ServerMessage::GameState(
            //                 GameState::default(),
            //             )))
            //             .await;
            //     }
            // }
            new_player_in_game = false;

            pong_server.tick().await;

            // info!("players, pre-game: {} game: {}", pre_lobby_players.len(), players_in_game.len());
        }
        // tokio::time::sleep_until(next_tick).await;
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
}
