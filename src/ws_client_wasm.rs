use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};
use futures::SinkExt;
use futures::channel::mpsc::Receiver;
use send_wrapper::SendWrapper;

use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::RwLock;

pub struct Websocket {
    pub receiver: Receiver<String>,
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub fn start_ws_client() -> Websocket {
    let (mut tx, mut rx) = futures::channel::mpsc::channel::<String>(100);
    // let (mut tx, mut rx) = mpsc::channel::<String>();
    // let wrapped_send = SendWrapper::new(Rc::new(tx));
    let send_lock = RwLock::new(tx);
    // let mut sender = Some(tx);

    console_log!("starting ws client...");
    let ws = WebSocket::new("ws://localhost:8080/echo").unwrap();
    let cloned_ws = ws.clone();
    let onopen_callback = Closure::wrap(Box::new(move |_| {
        console_log!("socket opened");
        match cloned_ws.send_with_str("1") {
            Ok(_) => console_log!("message successfully sent"),
            Err(err) => console_log!("error sending message: {:?}", err),
        }
        // send off binary message
        // match cloned_ws.send_with_u8_array(&vec![0, 1, 2, 3]) {
        //     Ok(_) => console_log!("binary message successfully sent"),
        //     Err(err) => console_log!("error sending message: {:?}", err),
        // }
    }) as Box<dyn FnMut(JsValue)>);
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    let cloned_ws = ws.clone();
    let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
        if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            console_log!("message event, received Text: {:?}", txt);
            let mut send = send_lock.write().unwrap();
            send.try_send(txt.into()).expect("try send failed")
            // match sender.take() {
            //     None => console_log!("No more sender"),
            //     Some(mut send) => send.try_send(txt.into()).expect("try send failed"),
            // };
            // tx.send(txt.into());
        }
    }) as Box<dyn FnMut(MessageEvent)>);
    // set message event handler on WebSocket
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    // forget the callback to keep it alive
    onmessage_callback.forget();

    Websocket {receiver: rx}
}
