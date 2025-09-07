use std::process::Output;

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::process::Command;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};

use crate::{Config, Payload, Report};

type WebSocketSender =
    SplitSink<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>;
type WebSocketReceiver =
    SplitStream<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>;

pub async fn start(config: Config) {
    let (ws_sender, ws_receiver) = match connect(config).await {
        Some((ws_sender, ws_receiver)) => (ws_sender, ws_receiver),
        None => return,
    };

    serve((ws_sender, ws_receiver)).await;
}

pub async fn connect(config: Config) -> Option<(WebSocketSender, WebSocketReceiver)> {
    let ws_connection =
        match connect_async(format!("ws://{}:{}", config.server_address, config.port)).await {
            Ok(ws_connection) => ws_connection,
            Err(err_val) => {
                eprintln!("Error: WebSocket Connection | {}", err_val);
                return None;
            }
        };

    let (ws_sender, ws_receiver) = ws_connection.0.split();
    Some((ws_sender, ws_receiver))
}

pub async fn serve((mut ws_sender, mut ws_receiver): (WebSocketSender, WebSocketReceiver)) -> ! {
    loop {
        match receive(&mut ws_receiver).await {
            Some(message) => {
                match serde_json::from_str(&message[..]) {
                    Ok(payload) => match execute(payload).await {
                        Some(output) => send(output, &mut ws_sender).await,
                        None => todo!(),
                    },
                    Err(err_val) => {
                        eprintln!("Error: Message to Payload | {}", err_val);
                        continue;
                    }
                };
            }
            None => continue,
        }
    }
}

pub async fn execute(payload: Payload) -> Option<Output> {
    println!("{:#?}", payload);
    match Command::new(payload.command)
        .args(payload.args)
        .output()
        .await
    {
        Ok(output) => Some(output),
        Err(err_val) => {
            eprintln!("Error: Command Execution | {}", err_val);
            return None;
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

pub async fn send(output: Output, ws_sender: &mut WebSocketSender) {
    let report = Report {
        status: output.status.to_string(),
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
    };
    let report = serde_json::json!(report);
    let _ = ws_sender.send(report.to_string().into()).await;
}
