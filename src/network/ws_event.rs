use crate::network::message::Message;

#[derive(Debug, Clone)]
pub enum WsEvent {
    Opened,
    Message(Message),
    #[allow(dead_code)]
    Error(String),
    Closed,
}
