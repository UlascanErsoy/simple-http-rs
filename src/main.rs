use clap::Parser;
use std::fs::metadata;

pub mod config;
pub mod server;

use config::ServerConfig;

#[derive(Parser, Debug)]
#[command(author = "Ulascan Ersoy. <ersoy.ulascan@gmail.com>")]
#[command(version)]
#[command(about="simple-http-rs!\nServes your files right!")]
struct Args {
    #[arg(short, long)]
    config: Option<String>,
}

fn main() {
    let args = Args::parse();

    let config_path = match &args.config {
        Some(path) => path,
        None => "config.yaml"
    };
    
    let config = ServerConfig::from_file(config_path);
    
    match metadata(&config.root) {
        Ok(md) => {
            if !md.is_dir() {
                panic!("config.root: {} is not a directory!", &config.root);
            }
        },
        Err(e) => panic!("Error: {e}")
    }

    let mut server = server::Server::new(config);

    server.bind().listen();
}
