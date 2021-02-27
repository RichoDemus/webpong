pub mod event_stream;
pub mod ws_event;


use std::cell::RefCell;
use std::num::ParseIntError;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use futures::channel::mpsc::{Receiver, Sender, TryRecvError};
use futures::{StreamExt, Stream};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};
use std::task::Poll;
use crate::event_stream::EventStream;
use crate::ws_event::WsEvent;
use std::future::Future;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

struct Websocket {
    pub event_stream: EventStream,
    ws: WebSocket,
}

impl Websocket {
    async fn open(url: &str) -> Self {
        let ws: WebSocket = WebSocket::new(url).unwrap();

        let event_stream = EventStream::new();

        {
            // On Error
            let buffer = event_stream.buffer();
            let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
                web_sys::console::log_2(&"Closure: err:".into(), &e.to_string().into());
                let x:String = e.message().into();
                buffer.borrow_mut().push(format!("Error: {:?}", x));
            }) as Box<dyn FnMut(ErrorEvent)>);
            ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
            onerror_callback.forget();
        }

        {
            // On Message
            let buffer = event_stream.buffer();
            let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
                if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                    let str:String = txt.into();
                    buffer.borrow_mut().push(str);
                }
            }) as Box<dyn FnMut(MessageEvent)>);
            ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
            onmessage_callback.forget();
        }

        {
            // on close
            let buffer = event_stream.buffer();
            let onclose_callback = Closure::wrap(Box::new(move |_| {
                buffer.borrow_mut().push(String::from("Closed"));
            }) as Box<dyn FnMut(JsValue)>);
            ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
            onclose_callback.forget();
        }

        {
            // on open
            let buffer = event_stream.buffer();
            let onopen_callback = Closure::wrap(Box::new(move |_| {
                buffer.borrow_mut().push(String::from("Open"));
            }) as Box<dyn FnMut(JsValue)>);
            ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
            onopen_callback.forget();
        }

        Websocket {
            event_stream,
            ws,
        }
    }

    fn send(&mut self, str: &str) {
        self.ws.send_with_str(str);
    }

    fn close(&mut self) {
        let _ = self.ws.close();
    }
}

#[wasm_bindgen_test]
async fn pass() {
    let mut websocket = Websocket::open("ws://localhost:8080/echo").await;

    loop {
        match websocket.event_stream.next_event().await {
            None => console_log!("None"),
            Some(evt) => {
                console_log!("Evt: {:?}", evt);
                if evt.contains("Closed") {
                    break;
                } else if evt.contains("Open") {

                } else {
                    let int: i32 = evt.parse().expect("parse int from server");
                    if int > 3 {
                        websocket.close();
                    } else {
                        websocket.send(int.to_string().as_str());
                    }
                }
            },
        }
    }


    // loop {
    //     let maybe_result = websocket.receiver.next().await;
    //     console_log!("\tloop res: {:?}", maybe_result);
    //     if let Some(evt) = maybe_result {
    //         match evt {
    //             WsEvent::Opened => {
    //                 console_log!("websocket opened");
    //                 websocket.send("1");
    //             }
    //             WsEvent::Message(msg) => {
    //                 let int: i32 = msg.parse().expect("parse int from server");
    //                 if int > 3 {
    //                     websocket.close();
    //                 } else {
    //                     websocket.send(int.to_string().as_str());
    //                 }
    //             }
    //             WsEvent::Error(_) => {}
    //             WsEvent::Closed => {
    //                 console_log!("ws closed, breaking loop");
    //                 break;
    //             }
    //         }
    //     }
    // }
}

