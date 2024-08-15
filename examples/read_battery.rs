use std::sync::Arc;

use renogy::Battery;
use tokio::sync::Mutex;


use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    port: String
}


fn open_battery(port: &str, addr: u8) -> Battery {
    let port = match renogy::Port::new(port) {
        Ok(p) => p,
        Err(e) => {
            println!("Could not open port {}: {:?}", port, e);
            std::process::exit(-1);
        }
    };
    let port = Arc::new(Mutex::new(port));

    Battery::new(port.clone(), addr)
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Read status from two batteries with IDs 246 and 247
    let battery1 = open_battery(&args.port, 246);
    let battery2 = open_battery(&args.port, 247);

    println!("Reading 246");
    match battery1.read_all().await {
        Ok(state) => println!("State: {:?}", state),
        Err(e) => println!("Error: {:?}", e),
    }

    println!("Reading 247");
    match battery2.read_all().await {
        Ok(state) => println!("State: {:?}", state),
        Err(e) => println!("Error: {:?}", e),
    }
}
