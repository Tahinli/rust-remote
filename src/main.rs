use rust_remote::{
    utils::{read_config, take_args},
    Runner, RunnerMode,
};

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let config = match read_config() {
        Some(config) => config,
        None => {
            eprintln!("Error: Read Config");
            return;
        }
    };

    match take_args() {
        Some(runner_mode) => match runner_mode {
            RunnerMode::State(Runner::Server, false) => {
                rust_remote::server::start(config, false).await
            }
            RunnerMode::State(Runner::Server, true) => {
                rust_remote::server::start(config, true).await
            }
            RunnerMode::State(Runner::Client, false) => {
                rust_remote::client::start(config, false).await
            }
            RunnerMode::State(Runner::Client, true) => {
                rust_remote::client::start(config, true).await
            }
        },
        None => {
            eprintln!("Error: Take Args");
            return;
        }
    }
}
