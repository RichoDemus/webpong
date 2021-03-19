use log::*;
use tokio::time::Duration;

use crate::network::ws_server;
use crate::network::ws_event::WsEvent;
use crate::network::message::{Message, ClientMessage, ServerMessage, GameState};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use tungstenite::protocol::Role::Client;

pub async fn start() {
    let _ = env_logger::builder()
        .filter_module("webpong", log::LevelFilter::Trace)
        .try_init();
    let mut ws_server = ws_server::WebsocketServer::start()
        .await
        .expect("start ws server");


    let mut pre_lobby_players = vec![];
    let mut players_in_game = vec![];

    let mut new_player_in_game = false; //set to true when a new player joins to signal gamestate sync

    //lobby
    loop {
        while let Some(mut client) = ws_server.event_stream.next_event().await {
            info!("Client {} connected", client.id);
            let rand_string: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(5)
                .map(char::from)
                .collect();
            client.name = Some(rand_string.clone());
            client.send(&Message::ServerMessage(ServerMessage::SetName(rand_string))).await;
            pre_lobby_players.push(client);
        }

        pre_lobby_players = pre_lobby_players.into_iter()
            .filter_map(|mut player|{
                let event = player.event_stream.buffer().lock().expect("asd").pop_front();

                if let Some(WsEvent::Error(e)) = event {
                    warn!("Err: {:?}", e);
                    Some(player)
                } else if let Some(WsEvent::Message(Message::ClientMessage(ClientMessage::EnterGame))) = event {
                    info!("Player {} entering game", player);
                    players_in_game.push(player);
                    new_player_in_game = true;
                    None
                } else if let Some(WsEvent::Closed) = event {
                    None
                } else {
                    Some(player)
                }
            })
            .collect();

        for player in &mut players_in_game {
            if new_player_in_game {
                player.send(&Message::ServerMessage(ServerMessage::GameState(GameState::default()))).await;
            }
        }
        new_player_in_game = false;



        // info!("players, pre-game: {} game: {}", pre_lobby_players.len(), players_in_game.len());

        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
