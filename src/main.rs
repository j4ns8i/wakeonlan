use std::env;
use std::error::Error;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::process::{ExitCode, Termination};

const USAGE: &str = "Usage:

wakeonlan MACADDRESS
";

// #[derive(Debug)]
enum WOLError {
    Bind,
    InvalidMACAddress,
    InvalidUsage,
}

impl std::fmt::Debug for WOLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid usage\n{}", self)
    }
}

impl From<WOLError> for std::process::ExitCode {
    fn from(value: WOLError) -> Self {
        ExitCode::from(1)
        // match value {
        //     WOLError::InvalidMACAddress => ExitCode::from(1),
        //     WOLError::InvalidUsage => Exi,
        //     WOLError::Bind => todo!(),
        // }
    }
}

impl std::fmt::Display for WOLError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WOLError::InvalidMACAddress => write!(f, "Invalid MAC address"),
            WOLError::InvalidUsage => write!(f, "{}", USAGE),
            WOLError::Bind => write!(f, "Failed to bind to socket"),
        }
    }
}

impl std::error::Error for WOLError {}

impl Termination for WOLError {
    fn report(self) -> std::process::ExitCode {
        self.into()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // TODO convert the args to clap
    // TODO add arg for broadcast address
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        todo!();
        wake(&args[1]);
        return Ok(());
    }
    Err(WOLError::InvalidUsage.into())
}

fn form_payload(addr: impl AsRef<str>) -> Result<Vec<u8>, WOLError> {
    let mut payload = Vec::with_capacity(102);

    // Add 6 bytes of 255 (0xFF)
    payload.extend([0xFF; 6].iter());

    // Parse MAC address
    let mac_address: Vec<u8> = addr
        .as_ref()
        .split(':')
        .map(|s| u8::from_str_radix(s, 16).map_err(|_| WOLError::InvalidMACAddress))
        .collect::<Result<Vec<u8>, WOLError>>()?;

    if mac_address.len() != 6 {
        return Err(WOLError::InvalidMACAddress);
    }

    // Add 16 repetitions of the MAC address
    for _ in 0..16 {
        payload.extend(&mac_address);
    }

    Ok(payload)
}

fn send_payload(addr: impl AsRef<str>, payload: Vec<u8>) -> Result<(), WOLError> {
    let socket = UdpSocket::bind("127.0.0.1:64000").map_err(|_| WOLError::Bind)?;

    // Enable broadcast
    socket.set_broadcast(true).map_err(|_| WOLError::Bind)?;

    // Convert the MAC address to a socket address
    // TODO receive the broadcast address as an argument
    let target_addr: SocketAddr = format!("{}:9", addr.as_ref())
        .to_socket_addrs()
        .next()
        .ok_or(WOLError::InvalidMACAddress)?;

    // Send the payload
    socket
        .send_to(&payload, target_addr)
        .map_err(|_| WOLError::Bind)?;

    Ok(())
}

fn wake(addr: impl AsRef<str>) -> Result<(), WOLError> {
    let payload = form_payload(addr)?;
    send_payload(payload)
}
