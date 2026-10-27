use esp_idf_svc::io::{Read, ReadExactError};
use esp_idf_svc::sys::EspError;
use esp_idf_svc::tls::EspTls;
use esp_idf_svc::tls::{EspTls, ServerConfig, Socket, X509};
use std::io::{self, Error, Read};
use std::net::{TcpListener, TcpStream};
use std::os::fd::{AsRawFd, IntoRawFd};

pub enum Esp32Error {
    Wifi,
    Io(io::Error),
}

pub type Result<T> = std::result::Result<T, Esp32Error>;

impl From<io::Error> for Esp32Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<esp_idf_svc::sys::EspError> for Esp32Error {
    fn from(value: esp_idf_svc::sys::EspError) -> Self {
        Self::Wifi
    }
}
impl<T> From<ReadExactError<T>> for Esp32Error {
    fn from(value: ReadExactError<T>) -> Self {
        Self::Wifi
    }
}
pub struct TlsSocket(Option<TcpStream>);

impl TlsSocket {
    fn new(stream: TcpStream) -> Self {
        Self(Some(stream))
    }
}

impl Socket for TlsSocket {
    fn handle(&self) -> i32 {
        self.0.as_ref().unwrap().as_raw_fd()
    }
    fn release(&mut self) -> std::result::Result<(), esp_idf_svc::sys::EspError> {
        if let Some(stream) = self.0.take() {
            let _fd = stream.into_raw_fd();
        }
        Ok(())
    }
}

pub struct Connection {
    wifi: EspTls<TlsSocket>,
}

#[repr(u8)]
pub enum SendingOrder {
    HumanPresence = 1,
}
impl TryFrom<u8> for SendingOrder {
    type Error = Esp32Error;
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            1 => Ok(SendingOrder::HumanPresence),
            0 => Ok(SendingOrder::Off),
            _ => Err(Error::new(io::ErrorKind::BrokenPipe, "misisng match arm"))?,
        }
    }
}

impl Connection {
    pub fn new() -> Result<Self> {
        let listener = TcpListener::bind("0.0.0.0:8432")?;

        let (stream, _) = listener.accept()?;

        let tlssocket = TlsSocket::new(stream);

        let mut tls = EspTls::adopt(tlssocket)?;

        let config = ServerConfig {
            server_cert: Some(X509::pem_until_nul(include_bytes!("server.crt"))),
            server_key: Some(X509::pem_until_nul(include_bytes!("server.key"))),
            ..Default::default()
        };

        tls.negotiate_server(&config)?;

        Ok(Self { wifi: tls })
    }

    pub fn read(&mut self) -> Result<SendingOrder> {
        let mut bytes = [0u8];

        self.wifi.read_exact(&mut bytes)?;

        Ok(SendingOrder::try_from(bytes[0])?)
    }
}
