use std::cell::RefCell;
use std::num::ParseIntError;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use futures::channel::mpsc::{Receiver, Sender, TryRecvError};
use futures::StreamExt;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};
use crate::event_stream::{EventStream};
use crate::ws_event::WsEvent;

// wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub struct Websocket {
    pub event_stream: EventStream,
    ws: WebSocket,
}

impl Websocket {
    pub async fn open(url: &str) -> Self {
        // console_log!("Time to open socket");
        let ws: WebSocket = WebSocket::new(url).unwrap();
        // console_log!("opened, kinda");

        let event_stream = EventStream::new();

        {
            // On Error
            let buffer = event_stream.buffer();
            let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
                // console_log!("onerror");
                buffer.borrow_mut().push(WsEvent::Error(e.message().into()));
            }) as Box<dyn FnMut(ErrorEvent)>);
            ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
            onerror_callback.forget();
        }

        {
            // On Message
            let buffer = event_stream.buffer();
            let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
                // console_log!("onmessage: {:?}", e);
                if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                    // console_log!("onmessage: {:?}", txt);
                    buffer.borrow_mut().push(WsEvent::Message(txt.into()));
                    // console_log!("after push");
                }
            }) as Box<dyn FnMut(MessageEvent)>);
            ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
            onmessage_callback.forget();
        }

        {
            // on close
            let buffer = event_stream.buffer();
            let onclose_callback = Closure::wrap(Box::new(move |_| {
                // console_log!("onclose");
                buffer.borrow_mut().push(WsEvent::Closed);
            }) as Box<dyn FnMut(JsValue)>);
            ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
            onclose_callback.forget();
        }

        {
            // on open
            let buffer = event_stream.buffer();
            let onopen_callback = Closure::wrap(Box::new(move |_| {
                // console_log!("onopen");
                buffer.borrow_mut().push(WsEvent::Opened);
            }) as Box<dyn FnMut(JsValue)>);
            ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
            onopen_callback.forget();
        }

        // console_log!("return from ws client");
        Websocket {
            event_stream,
            ws,
        }
    }

    pub fn send(&mut self, str: &str) {
        self.ws.send_with_str(str);
    }

    fn close(&mut self) {
        let _ = self.ws.close();
    }
}
