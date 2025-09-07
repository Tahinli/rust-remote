use std::{env, net::IpAddr};

use crate::{Config, Runner, RunnerMode};

pub fn take_args() -> Option<(RunnerMode, Config)> {
    let args: Vec<String> = env::args().collect();
    let mut runner = Runner::Server;
    let mut debug = false;
    let mut ip = "127.0.0.1".to_string();
    let mut port = "3444".to_string();
    for (i, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "--server" | "-sv" => runner = Runner::Server,
            "--client" | "-cl" => runner = Runner::Client,
            "--debug" | "-d" => debug = true,
            "--ip" | "i" => ip = args[i + 1].clone(),
            "--port" | "p" => port = args[i + 1].clone(),
            "--help" | "-h" => {
                show_help();
                std::process::exit(0);
            }
            _ => continue,
        }
    }
    let ip = match ip.parse::<IpAddr>() {
        Ok(ip) => ip,
        Err(err_val) => {
            eprintln!("Error: IP Parse | {}", err_val);
            return None;
        }
    };

    let port = match port.parse::<u16>() {
        Ok(port) => port,
        Err(err_val) => {
            eprintln!("Error: Port Parse | {}", err_val);
            return None;
        }
    };

    let config = Config { ip, port };

    let runner_mode = RunnerMode::State(runner, debug);
    Some((runner_mode, config))
}

fn show_help() {
    println!("\n\n\n");
    println!("   Arguments          |  Details                    |  Defaults");
    println!("----------------------------------------------------------------------");
    println!("   -i  -> --ip        |  Specifies IP Address       |  127.0.0.1");
    println!("   -p  -> --port      |  Specifies Port Address     |  3444");
    println!("   -sv -> --server    |  Starts as a Server         |  True");
    println!("   -cl -> --client    |  Starts as a Client         |  False");
    println!("   -d  -> --debug     |  Starts in Debug Mode       |  False");
    println!("   -h  -> --help      |  Shows Help                 |  False");
    println!("\n\n\n");
}
