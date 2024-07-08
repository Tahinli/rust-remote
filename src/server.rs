use std::{io::stdin, sync::Arc};

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};

use crate::{Config, Payload};
type WebSocketSender = SplitSink<WebSocketStream<TcpStream>, Message>;
type WebSocketReceiver = SplitStream<WebSocketStream<TcpStream>>;

pub async fn start(config: Config, debug: bool) {
    let listener = match setup(config, debug).await {
        Some(listener) => listener,
        None => return,
    };

    loop {
        match establish_connection(&listener, debug).await {
            Some((ws_sender, ws_receiver)) => {
                let ws_sender = Arc::new(Mutex::new(ws_sender));
                let ws_receiver = Arc::new(Mutex::new(ws_receiver));
                loop {
                    let ws_sender = ws_sender.clone();
                    let ws_receiver = ws_receiver.clone();
                    match payload_from_input(debug).await {
                        Some(payload) => {
                            if !send(payload, ws_sender, debug).await {
                                break;
                            }
                            tokio::spawn(async move {
                                let report = receive(ws_receiver, debug).await;
                                println!("{:#?}", report);
                            });
                        }
                        None => continue,
                    }
                }
            }
            None => return,
        }
    }
}
async fn setup(config: Config, debug: bool) -> Option<TcpListener> {
    match TcpListener::bind(format!("{}:{}", config.ip, config.port)).await {
        Ok(listener) => Some(listener),
        Err(err_val) => {
            if debug {
                eprintln!("Error: Listener | {}", err_val);
            }
            None
        }
    }
}

async fn establish_connection(
    listener: &TcpListener,
    debug: bool,
) -> Option<(WebSocketSender, WebSocketReceiver)> {
    match listener.accept().await {
        Ok(connection) => match accept_async(connection.0).await {
            Ok(ws_connection) => Some(ws_connection.split()),
            Err(err_val) => {
                if debug {
                    eprintln!("Error: WebSocket Upgrade | {}", err_val);
                }
                None
            }
        },
        Err(err_val) => {
            if debug {
                eprintln!("Error: Listener Accept | {}", err_val);
            }
            None
        }
    }
}

async fn payload_from_input(debug: bool) -> Option<Payload> {
    println!("User");
    // let user = match get_input() {
    //     Some(input) => input,
    //     None => return None,
    // };
    let user = "tahinli".to_string();
    println!("Command");
    match get_input(debug) {
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
                    user,
                    command,
                    args,
                })
            }
        }
        None => None,
    }
}

fn get_input(debug: bool) -> Option<String> {
    let mut payload_input: String = String::new();
    match stdin().read_line(&mut payload_input) {
        Ok(_) => Some(payload_input.trim().to_string()),
        Err(err_val) => {
            if debug {
                eprintln!("Error: Read Input | {}", err_val);
            }
            None
        }
    }
}

async fn receive(ws_receiver: Arc<Mutex<WebSocketReceiver>>, debug: bool) -> Option<String> {
    match ws_receiver.lock().await.next().await {
        Some(message) => match message {
            Ok(message) => {
                if let Message::Text(message) = message {
                    Some(message)
                } else {
                    if debug {
                        eprintln!("Error: Message Type | {:#?}", message);
                    }
                    None
                }
            }
            Err(err_val) => {
                if debug {
                    eprintln!("Error: Message | {}", err_val);
                }
                None
            }
        },
        None => {
            if debug {
                eprintln!("Error: WebSocket Receive");
            }
            None
        }
    }
}

async fn send(payload: Payload, ws_sender: Arc<Mutex<WebSocketSender>>, debug: bool) -> bool {
    let payload = serde_json::json!(payload);
    let result = ws_sender
        .lock()
        .await
        .send(payload.to_string().into())
        .await;
    match result {
        Ok(_) => {
            let result = ws_sender.lock().await.flush().await;
            match result {
                Ok(_) => true,
                Err(err_val) => {
                    if debug {
                        eprintln!("Error: WebSocket Flush | {}", err_val);
                    }
                    false
                }
            }
        }
        Err(err_val) => {
            if debug {
                eprintln!("Error: WebSocket Send | {}", err_val);
            }
            false
        }
    }
}
