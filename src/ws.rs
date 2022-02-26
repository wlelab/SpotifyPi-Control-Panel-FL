use futures_util::{future, pin_mut, StreamExt};
use futures::channel::mpsc::UnboundedReceiver;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use fltk::app::Sender;
use regex::Regex;

use crate::my_enums::{MyAppMessage, WSEventValue};


pub async fn connect_to_ws(url: url::Url, input_rx: UnboundedReceiver<Message>, output_tx: Sender<MyAppMessage>) {
    let (ws_stream, _) = match connect_async(url).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            output_tx.send(MyAppMessage::WSEventValue(WSEventValue::Connect(false)));
            return
        }
    };

    eprintln!("WebSocket handshake has been successfully completed");
    output_tx.send(MyAppMessage::WSEventValue(WSEventValue::Connect(true)));

    let (write, read) = ws_stream.split();

    let input_to_ws = input_rx.map(Ok).forward(write);
    let ws_to_output = {
        read.for_each(|message| async {
            match message {
                Ok(msg) => {
                    let data = msg.into_data();
                    if let Ok(text) = String::from_utf8(data) {
                        let event_value= convert_output_msg(text);
                        output_tx.send(MyAppMessage::WSEventValue(event_value));
                    }
                }
                Err(err) => eprintln!("Message unwrap failed: {}", err)
            }
        })
    };

    pin_mut!(input_to_ws, ws_to_output);
    future::select(input_to_ws, ws_to_output).await;

    eprintln!("WebSocket disconnected !!!");
    output_tx.send(MyAppMessage::WSEventValue(WSEventValue::Disconnect));
}

fn convert_output_msg(text: String) -> WSEventValue {
    let (event, value) = get_event_and_value_string(text);
    if event == "error" {
        if value == "missing" {
            return WSEventValue::Missing;
        } else if value == "not_found" {
            return WSEventValue::NotFound;
        }
    } else if event == "volume" {
        if let Ok(volume) = value.parse::<i32>() {
            return WSEventValue::Volume(volume);
        }
    } else if event == "prev_track" {
        if value == "ok" {
            return WSEventValue::PrevTrack(true);
        } else {
            return WSEventValue::PrevTrack(false);
        }
    } else if event == "toggle_play_pause" {
        if value == "ok" {
            return WSEventValue::TogglePlayPause(true);
        } else {
            return WSEventValue::TogglePlayPause(false);
        }
    } else if event == "next_track" {
        if value == "ok" {
            return WSEventValue::NextTrack(true);
        } else {
            return WSEventValue::NextTrack(false);
        }
    } else if event == "toggle_shuffle" {
        if value == "ok" {
            return WSEventValue::ToggleShuffle(true);
        } else {
            return WSEventValue::ToggleShuffle(false);
        }
    } else if event == "toggle_repeat_state" {
        if value == "ok" {
            return WSEventValue::ToggleRepeatState(true);
        } else {
            return WSEventValue::ToggleRepeatState(false);
        }
    }
    return WSEventValue::Unknown;
}

fn get_event_and_value_string(text: String) -> (String, String) {
    let re = Regex::new(r"\[(?P<event>.+?)\]\((?P<value>.*?)\)").unwrap();
    match re.captures(&text) {
        Some(caps) => {
            let event = &caps["event"];
            let value = &caps["value"];
            (event.to_string(), value.to_string())
        }
        None => ("".to_string(), "".to_string())
    }
}
