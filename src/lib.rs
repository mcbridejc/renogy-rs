use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio_modbus::client::{Context, rtu};
use tokio_modbus::prelude::*;
use tokio_serial::SerialStream;

/// The baudrate of battery RS485 comms
const RENOGY_BAUDRATE: u32 = 9600;

pub struct Port {
    ctx: Context,
}

impl Port {
    pub fn new(dev: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let serial = SerialStream::open(
            &tokio_serial::new(dev, RENOGY_BAUDRATE).timeout(Duration::from_millis(400))
        )?;
        let ctx = rtu::attach(serial);
        Ok(Self { ctx })
    }
}

macro_rules! reader_i16 {
    ($name: ident, $addr: literal, $scale: literal, $doc: expr) => {
        #[doc=$doc]
        pub async fn $name(&self) -> Result<f64, Box<dyn std::error::Error>> {
            let mut port = self.port.lock().await;
            port.ctx.set_slave(Slave(self.addr));
            let raw_value = port.ctx.read_holding_registers($addr, 1).await?;
            assert!(raw_value.len() == 1);
            Ok((raw_value[0] as i16) as f64 * $scale as f64)
        }
    };
}

macro_rules! reader_i16_swap {
    ($name: ident, $addr: literal, $scale: literal, $doc: expr) => {
        #[doc=$doc]
        pub async fn $name(&self) -> Result<f64, Box<dyn std::error::Error>> {
            println!("Reading {}", stringify!($name));
            let mut port = self.port.lock().await;
            println!("Slave: {}", self.addr);
            port.ctx.set_slave(Slave(self.addr));
            let raw_value = port.ctx.read_holding_registers($addr, 1).await?;
            assert!(raw_value.len() == 1);
            Ok((raw_value[0].swap_bytes() as i16) as f64 * $scale as f64)
        }
    };
}

macro_rules! reader_u16 {
    ($name: ident, $addr: literal, $scale: literal, $doc: expr) => {
        #[doc=$doc]
        pub async fn $name(&self) -> Result<f64, Box<dyn std::error::Error>> {
            let mut port = self.port.lock().await;
            port.ctx.set_slave(Slave(self.addr));
            let raw_value = port.ctx.read_holding_registers($addr, 1).await?;
            assert!(raw_value.len() == 1);
            Ok((raw_value[0]) as f64 * $scale as f64)
        }
    };
}

macro_rules! reader_u32 {
    ($name: ident, $addr: literal, $scale: literal, $doc: expr) => {
        #[doc=$doc]
        pub async fn $name(&self) -> Result<f64, Box<dyn std::error::Error>> {
            let mut port = self.port.lock().await;
            port.ctx.set_slave(Slave(self.addr));
            let raw_value = port.ctx.read_holding_registers($addr, 2).await?;
            assert!(raw_value.len() == 2);
            let binary_value = raw_value[0] as u32 + ((raw_value[1] as u32) << 16);
            let scaled_value = binary_value as f64 * $scale as f64;
            Ok(scaled_value)
        }
    };
}

/// Represents all available battery stats
#[derive(Clone, Copy, Debug)]
pub struct BatteryState {
    current: f64,
    voltage: f64,
    remaining_charge: f64,
    capacity: f64,
    cell_voltage_1: f64,
    cell_voltage_2: f64,
    cell_voltage_3: f64,
    cell_voltage_4: f64,
    cell_temp_1: f64,
    cell_temp_2: f64,
    cell_temp_3: f64,
    cell_temp_4: f64,
    heater_level: f64,
}


pub struct Battery {
    port: Arc<Mutex<Port>>,
    addr: u8,
}

impl Battery {
    pub fn new(port: Arc<Mutex<Port>>, addr: u8) -> Self {
        Self { port, addr }
    }

    reader_i16_swap!(current, 0x13b2, 0.01, "Read current in A");
    reader_u16!(voltage, 0x13b3, 0.1, "Read voltage in V");
    reader_u32!(remaining_charge, 0x13b4, 0.001, "Read remaining charge in Ah");
    reader_u32!(capacity, 0x13b6, 0.001, "Read battery capacity in Ah");
    reader_u16!(cycle_number, 0x13b8, 1.0, "Read battery cycle number");
    reader_u16!(cell_voltage_1, 0x1389, 0.1, "Read cell voltage 1 in V");
    reader_u16!(cell_voltage_2, 0x138a, 0.1, "Read cell voltage 2 in V");
    reader_u16!(cell_voltage_3, 0x138b, 0.1, "Read cell voltage 3 in V");
    reader_u16!(cell_voltage_4, 0x138c, 0.1, "Read cell voltage 4 in V");
    reader_i16!(cell_temp_1, 0x139a, 0.1, "Read cell temperature 1 in deg C");
    reader_i16!(cell_temp_2, 0x139b, 0.1, "Read cell temperature 2 in deg C");
    reader_i16!(cell_temp_3, 0x139c, 0.1, "Read cell temperature 3 in deg C");
    reader_i16!(cell_temp_4, 0x139d, 0.1, "Read cell temperature 4 in deg C");
    reader_u16!(heater_level, 0x13ef, 0.3922, "Read heater level in percent");

    pub async fn test(&self) {
        let mut port = self.port.lock().await;
        port.ctx.set_slave(Slave(240));

    }

    pub async fn read_all(&self) -> Result<BatteryState, Box<dyn std::error::Error>> {
        Ok(BatteryState {
            current: self.current().await?,
            voltage: self.voltage().await?,
            remaining_charge: self.remaining_charge().await?,
            capacity: self.capacity().await?,
            cell_voltage_1: self.cell_voltage_1().await?,
            cell_voltage_2: self.cell_voltage_2().await?,
            cell_voltage_3: self.cell_voltage_3().await?,
            cell_voltage_4: self.cell_voltage_4().await?,
            cell_temp_1: self.cell_temp_1().await?,
            cell_temp_2: self.cell_temp_2().await?,
            cell_temp_3: self.cell_temp_3().await?,
            cell_temp_4: self.cell_temp_4().await?,
            heater_level: self.heater_level().await?,
        })
    }
}
