use std::{env, fs::File, io::Read};

use crate::{Config, Runner, RunnerMode};

pub fn take_args() -> Option<RunnerMode> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let runner = match &args[1][..] {
            "--server" => Runner::Server,
            "--client" => Runner::Client,
            _ => return None,
        };
        let debug = if args.len() > 2 {
            match &args[2][..] {
                "--debug" => true,
                _ => return None,
            }
        } else {
            false
        };
        Some(RunnerMode::State(runner, debug))
    } else {
        None
    }
}

pub fn read_config() -> Option<Config> {
    let mut config_file = match File::open("configs/config.txt") {
        Ok(config_file) => config_file,
        Err(_) => return None,
    };
    let mut configs = String::new();
    match config_file.read_to_string(&mut configs) {
        Ok(_) => {
            let configs: Vec<String> = configs.split('\n').map(|x| x.to_string()).collect();
            let server_address = match configs[0].split(':').last() {
                Some(server_address_unchecked) => match server_address_unchecked.parse() {
                    Ok(server_address) => server_address,
                    Err(_) => return None,
                },
                None => return None,
            };
            let port = match configs[1].split(':').last() {
                Some(port_unchecked) => match port_unchecked.parse() {
                    Ok(port) => port,
                    Err(_) => return None,
                },
                None => return None,
            };
            Some(Config {
                server_address,
                port,
            })
        }
        Err(_) => None,
    }
}
