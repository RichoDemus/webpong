use crate::network::message::Message;

#[derive(Debug, Clone)]
pub enum WsEvent {
    Opened,
    Message(Message),
    Error(String),
    Closed,
}
