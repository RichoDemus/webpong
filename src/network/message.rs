use nalgebra::{Point2, Vector2};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Message {
    Ping,
    ClientMessage(ClientMessage),
    ServerMessage(ServerMessage),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ClientMessage {
    EnterGame,
    PaddleUp,
    PaddleDown,
    PaddleStop,
    TogglePause,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ServerMessage {
    SetName(String),
    PaddleUp(PaddleId),
    PaddleDown(PaddleId),
    PaddleStop(PaddleId),
    GameState(GameState),
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub enum PaddleId {
    Left,
    Right,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum PaddleState {
    Up,
    Down,
    Still,
}
impl Default for PaddleState {
    fn default() -> Self {
        PaddleState::Still
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct GameState {
    pub left_paddle_y: f64,
    pub left_paddle_state: PaddleState,
    pub right_paddle_y: f64,
    pub right_paddle_state: PaddleState,
    pub left_player_name: String,
    pub right_player_name: String,
    pub ball_position: Point2<f64>,
    pub ball_velocity: Vector2<f64>,
    pub paused: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let msg = Message::ServerMessage(ServerMessage::SetName(String::from("Richo")));

        let str = serde_json::to_string(&msg).expect("serialize to json");

        let expected = r#"{"ServerMessage":{"SetName":"Richo"}}"#;
        assert_eq!(expected, str);

        let enum_again = serde_json::from_str(str.as_str()).expect("deserialize json");

        assert_eq!(msg, enum_again);
    }
}
