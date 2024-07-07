use std::io::stdin;

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};

use crate::{Config, Payload};
type WebSocketSender = SplitSink<WebSocketStream<TcpStream>, Message>;
type WebSocketReceiver = SplitStream<WebSocketStream<TcpStream>>;

pub async fn start(config: Config) {
    let listener =
        match TcpListener::bind(format!("{}:{}", config.server_address, config.port)).await {
            Ok(listener) => listener,
            Err(err_val) => {
                eprintln!("Error: Listener | {}", err_val);
                return;
            }
        };

    loop {
        if let Ok(connection) = listener.accept().await {
            let ws_connection = match accept_async(connection.0).await {
                Ok(ws_connection) => ws_connection,
                Err(err_val) => {
                    eprintln!("Error: WebSocket Upgrade | {}", err_val);
                    continue;
                }
            };

            let (mut ws_sender, mut ws_receiver) = ws_connection.split();
            loop {
                match payload_from_input().await {
                    Some(payload) => {
                        send(payload, &mut ws_sender).await;
                        let report = receive(&mut ws_receiver).await;
                        println!("{:#?}", report);
                    }
                    None => continue,
                }
            }
        }
    }
}

pub async fn payload_from_input() -> Option<Payload> {
    match get_input() {
        Some(input) => {
            let mut args: Vec<String> = input.split_ascii_whitespace().map(String::from).collect();
            if args.is_empty() {
                None
            } else {
                let mut sudo = false;
                let mut command = args.remove(0);
                if command == "sudo" {
                    if args.is_empty() {
                        return None;
                    }
                    sudo = true;
                    command = args.remove(0);
                }
                Some(Payload {
                    sudo,
                    command,
                    args,
                })
            }
        }
        None => None,
    }
}

pub fn get_input() -> Option<String> {
    let mut payload_input: String = String::new();
    match stdin().read_line(&mut payload_input) {
        Ok(_) => Some(payload_input.trim().to_string()),
        Err(err_val) => {
            eprintln!("Error: Read Input | {}", err_val);
            None
        }
    }
}

pub async fn receive(ws_receiver: &mut WebSocketReceiver) -> Option<String> {
    match ws_receiver.next().await {
        Some(message) => match message {
            Ok(message) => {
                if let Message::Text(message) = message {
                    Some(message)
                } else {
                    eprintln!("Error: Message Type | {:#?}", message);
                    None
                }
            }
            Err(err_val) => {
                eprintln!("Error: Message | {}", err_val);
                None
            }
        },
        None => {
            eprintln!("Error: WebSocket Receive");
            None
        }
    }
}

pub async fn send(payload: Payload, ws_sender: &mut WebSocketSender) {
    let payload = serde_json::json!(payload);
    let _ = ws_sender.send(payload.to_string().into()).await;
}
