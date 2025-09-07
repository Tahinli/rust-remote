use std::net::IpAddr;

use serde::{Deserialize, Serialize};

pub mod client;
pub mod server;
pub mod utils;

#[derive(Debug)]
pub enum Runner {
    Server,
    Client,
}
#[derive(Debug)]
pub enum RunnerMode {
    State(Runner, bool),
}

#[derive(Debug, Clone)]
pub struct Config {
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Payload {
    pub sudo: bool,
    pub user: String,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub payload: Payload,
    pub status: String,
    pub stdout: String,
    pub stderr: String,
}
