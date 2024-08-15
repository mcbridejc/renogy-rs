use std::sync::Arc;

use tokio::sync::Mutex;


#[tokio::main]
async fn main() {

    let addr = 240;
    let port_path = "/dev/ttyUSB1";

    let port = match renogy::Port::new(port_path) {
        Ok(p) => p,
        Err(e) => {
            println!("Could not open port {}: {:?}", port_path, e);
            std::process::exit(-1);
        }
    };
    let port = Arc::new(Mutex::new(port));

    let battery = renogy::Battery::new(port.clone(), addr);

    println!("Reading...");
    let state = battery.read_all().await.unwrap();
    println!("Battery at addr {}: {:?}", addr, state);
}