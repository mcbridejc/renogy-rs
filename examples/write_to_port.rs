use std::{sync::Arc, time::Duration};

use std::sync::Mutex;

#[tokio::main]
async fn main() {
    let port_path = "/dev/ttyUSB1";

    let mut port = tokio_serial::new(port_path, 9600)
        .timeout(Duration::from_millis(200))
        .open()
        .unwrap();
    loop {
        println!("writing...");
        port.write_all(&[0xff, 0x00, 0xaa, 0x55]).unwrap();
        std::thread::sleep(Duration::from_millis(1000));
        let mut buf = [0; 128];
        let bytes_read = port.read(&mut buf).unwrap();

    }
}
