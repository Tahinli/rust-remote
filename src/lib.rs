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

impl Runner {
    fn print(&self) {
        println!("-------");
        match self {
            Runner::Server => println!("Runner = Server"),
            Runner::Client => println!("Runner = Client"),
        }
    }
}
#[derive(Debug)]
pub enum RunnerMode {
    State(Runner, bool),
}

impl RunnerMode {
    pub fn print(&self) {
        match self {
            RunnerMode::State(runner, debug) => {
                runner.print();
                println!("Debug = {}", debug);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub ip: IpAddr,
    pub port: u16,
}

impl Config {
    pub fn print(&self) {
        println!("-------");
        println!("IP = {}", self.ip);
        println!("Port = {}", self.port);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Payload {
    pub args: String,
}

impl Payload {
    fn print(&self) {
        println!("-------");
        println!("args = {}", self.args);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub payload: Payload,
    pub status: String,
    pub stdout: String,
    pub stderr: String,
}

impl Report {
    fn print(&self) {
        println!("-------");
        println!("Payload ↓");
        self.payload.print();
        println!("-------");
        if !self.status.is_empty() {
            println!("Status ↓ \n{}", self.status);
            println!("-------");
        }
        if !self.stdout.is_empty() {
            println!("Stdout ↓ \n{}", self.stdout);
            println!("-------");
        }
        if !self.stderr.is_empty() {
            println!("Stderr ↓ \n{}", self.stderr);
        }
    }
}
