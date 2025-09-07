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

use crate::{Config, Payload, Report};
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
                                if debug {
                                    eprintln!("Error: Send");
                                }
                                break;
                            }
                            tokio::spawn(async move {
                                if let Some(report) = receive(ws_receiver, debug).await {
                                    match serde_json::from_str::<Report>(&report) {
                                        Ok(report) => report.print(),
                                        Err(err_val) => {
                                            if debug {
                                                eprintln!("Error: Deserialize | {}", err_val);
                                            }
                                        }
                                    }
                                }
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
    println!("-------");
    println!("Command");
    get_input(debug).map(|args| Payload { args })
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
