use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub enum Message {
    Ping,
    ClientMessage(ClientMessage),
    ServerMessage(ServerMessage),
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub enum ClientMessage {
    EnterGame,
    PaddleUp,
    PaddleDown,
    PaddleStop,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Default)]
pub struct GameState {

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let msg = Message::ClientMessage(ClientMessage::SetName(String::from("Richo")));

        let str = serde_json::to_string(&msg).expect("serialize to json");

        let expected = r#"{"ClientMessage":{"SetName":"Richo"}}"#;
        assert_eq!(expected, str);

        let enum_again = serde_json::from_str(str.as_str()).expect("deserialize json");

        assert_eq!(msg, enum_again);
    }
}
