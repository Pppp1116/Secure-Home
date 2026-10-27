use crate::connection::{Connection, SendingOrder};
use crate::Connection::Result;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::peripherals::Peripherals;
use std::io;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

mod actions;
mod connection;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().map_err(|_| {
        io::Error::new(
            io::ErrorKind::ConnectionRefused,
            "coulndt start peripherals",
        )
    })?;

    let motion = PinDriver::input(
        peripherals.pins.gpio14,
        esp_idf_svc::hal::gpio::Pull::Floating,
    )
    .map_err(|_| io::Error::new(io::ErrorKind::AddrNotAvailable, "couldnt lauch motion pin"))?;

    let mut connection = Connection::new()?;

    let mut previous = false;
    let mut detections = 0;

    loop {
        let order = connection.read()?;

        if motion.is_high() && !previous {
            detections += 1;
            previous = true;
        }
        if !motion.is_high() {
            previous = false;
        }
    }
}
