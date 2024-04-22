use clap::{ColorChoice, Parser};
use wakeonlan::{wake, WOLError};

#[derive(Parser, Debug)]
#[command(version, about, color = ColorChoice::Never)]
struct Args {
    /// Target MAC Address to wake
    mac_addr: String
}

fn main() -> Result<(), WOLError> {
    let args = Args::parse();
    wake(args.mac_addr)
}
