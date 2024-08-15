use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::time::timeout;
use tokio_modbus::client::{Context, rtu};
use tokio_modbus::prelude::*;
use tokio_serial::SerialStream;

/// The baudrate of battery RS485 comms
const RENOGY_BAUDRATE: u32 = 9600;

pub struct Port {
    ctx: Context,
}

impl Port {
    pub fn new(dev: &str) -> Result<Self> {
        let serial = match SerialStream::open(
            &tokio_serial::new(dev, RENOGY_BAUDRATE).timeout(Duration::from_millis(400))
        ) {
            Ok(serial) => serial,
            Err(e) => match e.kind {
                tokio_serial::ErrorKind::Io(kind) => return Err(Error::Io(kind)),
                tokio_serial::ErrorKind::NoDevice => return Err(Error::NoDevice(e.description)),
                tokio_serial::ErrorKind::InvalidInput => return Err(Error::InvalidInput(e.description)),
                tokio_serial::ErrorKind::Unknown => return Err(Error::Unknown(e.description)),
            },
        };
        let ctx = rtu::attach(serial);
        Ok(Self { ctx })
    }
}

/// Represents all available battery stats
#[derive(Clone, Copy, Debug)]
pub struct BatteryState {
    pub current: f64,
    pub voltage: f64,
    pub remaining_charge: f64,
    pub capacity: f64,
    pub cycle_number: u16,
    pub cell_voltage_1: f64,
    pub cell_voltage_2: f64,
    pub cell_voltage_3: f64,
    pub cell_voltage_4: f64,
    pub cell_temp_1: f64,
    pub cell_temp_2: f64,
    pub cell_temp_3: f64,
    pub cell_temp_4: f64,
    pub heater_level: f64,
}


pub struct Battery {
    port: Arc<Mutex<Port>>,
    addr: u8,
}

#[derive(Debug)]
pub enum Error {
    Timeout,
    NoDevice(String),
    InvalidInput(String),
    Unknown(String),
    Io(std::io::ErrorKind),

}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value.kind())
    }
}

pub type Result<T> = std::result::Result<T, Error>;


#[repr(u16)]
enum RegAddr {
    Current = 0x13b2,
    Voltage = 0x13b3,
    RemainingCharge = 0x13b4,
    Capacity = 0x13b6,
    CycleNumber = 0x13b8,
    CellVoltage1 = 0x1389,
    CellVoltage2 = 0x138a,
    CellVoltage3 = 0x138b,
    CellVoltage4 = 0x138c,
    CellTemp1 = 0x139a,
    CellTemp2 = 0x139b,
    CellTemp3 = 0x139c,
    CellTemp4 = 0x139d,
    HeaterLevel = 0x13ef,
}

impl Battery {
    pub fn new(port: Arc<Mutex<Port>>, addr: u8) -> Self {
        Self { port, addr }
    }

    pub async fn read_register(&self, addr: u16, size: u16) -> Result<Vec<u16>> {
        const TIMEOUT: Duration = Duration::from_millis(200);
        let mut port = self.port.lock().await;
        port.ctx.set_slave(Slave(self.addr));
        println!("read_register {}", addr);

        std::thread::sleep(Duration::from_millis(10));
        match timeout(TIMEOUT, port.ctx.read_holding_registers(addr, size)).await {
            Ok(result) => result.map_err(|e| Error::Io(e.kind())),
            Err(_) => Err(Error::Timeout),
        }
    }

    /// Read a raw u16 value from a register
    pub async fn read_u16(&self, addr: u16) -> Result<u16> {
        let raw_value = self.read_register(addr, 1).await?;
        assert!(raw_value.len() == 1);
        Ok(raw_value[0])
    }

    /// Read a raw i16 value from a register
    pub async fn read_i16(&self, addr: u16) -> Result<i16> {
        let raw_value = self.read_register(addr, 1).await?;
        assert!(raw_value.len() == 1);
        Ok(raw_value[0] as i16)
    }

    /// Read a raw u32 value from a register
    pub async fn read_u32(&self, addr: u16) -> Result<u32> {
        let raw_value = self.read_register(addr, 2).await?;
        assert!(raw_value.len() == 2);
        Ok(raw_value[1] as u32 + ((raw_value[0] as u32) << 16))
    }

    /// Get the battery current in Amps
    ///
    /// Current is negative when discharging, positive when charging
    pub async fn current(&self) -> Result<f64> {
        // This register has opposite endianness of other registers for some reason
        let raw = i16::swap_bytes(self.read_i16(RegAddr::Current as u16).await?);
        Ok(raw as f64 * 0.01)
    }

    /// Get the battery voltage in Volts
    pub async fn voltage(&self) -> Result<f64> {
        let raw = self.read_u16(RegAddr::Voltage as u16).await?;
        Ok(raw as f64 * 0.1)
    }

    /// Get state of charge
    ///
    /// Returns the estimated remaining charge in Ah
    pub async fn remaining_charge(&self) -> Result<f64> {
        let raw = self.read_u32(RegAddr::RemainingCharge as u16).await?;
        Ok(raw as f64 * 0.001)
    }

    /// Get the total battery capacity
    ///
    /// Returns the battery capacity (when fully charged) in Ah
    pub async fn capacity(&self) -> Result<f64> {
        let raw = self.read_u32(RegAddr::Capacity as u16).await?;
        Ok(raw as f64 * 0.001)
    }

    /// Get the battery cycle counter value
    pub async fn cycle_number(&self) -> Result<u16> {
        Ok(self.read_u16(RegAddr::CycleNumber as u16).await?)
    }

    /// Get individual cell voltage 1 in V
    pub async fn cell_voltage_1(&self) -> Result<f64> {
        let raw = self.read_u16(RegAddr::CellVoltage1 as u16).await?;
        Ok(raw as f64 * 0.1)
    }

    /// Get individual cell voltage 2 in V
    pub async fn cell_voltage_2(&self) -> Result<f64> {
        let raw = self.read_u16(RegAddr::CellVoltage2 as u16).await?;
        Ok(raw as f64 * 0.1)
    }

    /// Get individual cell voltage 3 in V
    pub async fn cell_voltage_3(&self) -> Result<f64> {
        let raw = self.read_u16(RegAddr::CellVoltage3 as u16).await?;
        Ok(raw as f64 * 0.1)
    }

    /// Get individual cell voltage 4 in V
    pub async fn cell_voltage_4(&self) -> Result<f64> {
        let raw = self.read_u16(RegAddr::CellVoltage4 as u16).await?;
        Ok(raw as f64 * 0.1)
    }

    /// Read individual cell temperature 1 in deg C
    pub async fn cell_temp_1(&self) -> Result<f64> {
        let raw = self.read_i16(RegAddr::CellTemp1 as u16).await?;
        Ok(raw as f64 * 0.1)
    }

    /// Read individual cell temperature 2 in deg C
    pub async fn cell_temp_2(&self) -> Result<f64> {
        let raw = self.read_i16(RegAddr::CellTemp2 as u16).await?;
        Ok(raw as f64 * 0.1)
    }

    /// Read individual cell temperature 3 in deg C
    pub async fn cell_temp_3(&self) -> Result<f64> {
        let raw = self.read_i16(RegAddr::CellTemp3 as u16).await?;
        Ok(raw as f64 * 0.1)
    }

    /// Read individual cell temperature 4 in deg C
    pub async fn cell_temp_4(&self) -> Result<f64> {
        let raw = self.read_i16(RegAddr::CellTemp4 as u16).await?;
        Ok(raw as f64 * 0.1)
    }

    /// Read heater level in percent
    pub async fn heater_level(&self) -> Result<f64> {
        let raw = self.read_u16(RegAddr::HeaterLevel as u16).await?;
        Ok(raw as f64 * 0.3922)
    }


    pub async fn test(&self) {
        let mut port = self.port.lock().await;
        port.ctx.set_slave(Slave(240));

    }

    pub async fn read_all(&self) -> Result<BatteryState> {
        Ok(BatteryState {
            current: self.current().await?,
            voltage: self.voltage().await?,
            remaining_charge: self.remaining_charge().await?,
            capacity: self.capacity().await?,
            cell_voltage_1: self.cell_voltage_1().await?,
            cell_voltage_2: self.cell_voltage_2().await?,
            cell_voltage_3: self.cell_voltage_3().await?,
            cell_voltage_4: self.cell_voltage_4().await?,
            cycle_number: self.cycle_number().await?,
            cell_temp_1: self.cell_temp_1().await?,
            cell_temp_2: self.cell_temp_2().await?,
            cell_temp_3: self.cell_temp_3().await?,
            cell_temp_4: self.cell_temp_4().await?,
            heater_level: self.heater_level().await?,
        })
    }
}
