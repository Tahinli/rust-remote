use rust_remote::{
    client, server,
    utils::{read_config, take_args},
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
        Some(runner) => match runner {
            rust_remote::Runner::Server => server::start(config).await,
            rust_remote::Runner::Client => client::start(config).await,
        },
        None => {
            eprintln!("Error: Take Args");
            return;
        }
    }
}
