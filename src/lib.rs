use std::net::{Ipv4Addr, SocketAddr, UdpSocket};

pub fn wake(mac_addr: impl AsRef<str>) -> Result<(), WOLError> {
    let payload = build_payload(mac_addr.as_ref())?;
    broadcast_payload(payload)
}

pub enum WOLError {
    Bind,
    SetBroadcast,
    SendPacket,
    InvalidMACAddress,
}

impl std::fmt::Debug for WOLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WOLError::InvalidMACAddress => write!(f, "invalid MAC address"),
            WOLError::Bind => write!(f, "failed to bind to socket"),
            WOLError::SetBroadcast => write!(f, "failed to set broadcast mode"),
            WOLError::SendPacket => write!(f, "failed to send magic packet"),
        }
    }
}

fn build_payload(mac_addr: impl AsRef<str>) -> Result<Vec<u8>, WOLError> {
    let mut payload = Vec::with_capacity(102);

    // Add 6 bytes of 255 (0xFF)
    payload.extend([0xFF; 6].iter());

    // Add 16 repetitions of the MAC address
    let mac_address = parse_mac_address(mac_addr)?;
    for _ in 0..16 {
        payload.extend(&mac_address);
    }

    Ok(payload)
}

fn parse_mac_address(mac_addr: impl AsRef<str>) -> Result<[u8; 6], WOLError> {
    let mac_addr = mac_addr.as_ref();
    if mac_addr.len() != 17 {
        return Err(WOLError::InvalidMACAddress);
    }

    let mut result = [0; 6];
    for i in 0..6 {
        let n = i * 2 + i;
        let first = &mac_addr[n..(n + 1)];
        let second = &mac_addr[(n + 1)..(n + 2)];
        result[i] += u8::from_str_radix(first, 16).map_err(|_| WOLError::InvalidMACAddress)? << 4;
        result[i] += u8::from_str_radix(second, 16).map_err(|_| WOLError::InvalidMACAddress)?;
    }
    Ok(result)
}

fn broadcast_payload(payload: Vec<u8>) -> Result<(), WOLError> {
    let socket_addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0));
    let socket = UdpSocket::bind(socket_addr).map_err(|_| WOLError::Bind)?;

    socket.set_broadcast(true).map_err(|_| WOLError::SetBroadcast)?;

    let target_addr = SocketAddr::from((Ipv4Addr::BROADCAST, 9));
    socket
        .send_to(&payload, target_addr)
        .map_err(|_| WOLError::SendPacket)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn validate_magic_packet(mac_addr: &str, b: &[u8]) {
        assert_eq!(b.len(), 102);
        assert_eq!(&b[..6], &[0xFF; 6]);
        let parsed = parse_mac_address(mac_addr).expect(mac_addr);
        for i in 0..16 {
            // After the initial 6 bytes of FFFFFFFFFFFF, there must be 16 repetitions of the MAC
            // Address to wake up
            let idx = 6 + (6 * i);
            let mac_addr_slice = &b[idx..idx + 6];
            assert_eq!(mac_addr_slice, parsed);
        }
    }

    #[test]
    fn test_form_payload() {
        let mac_addr = "01:23:45:67:89:AB";
        let payload = build_payload(mac_addr).expect(mac_addr);
        validate_magic_packet(mac_addr, &payload);
    }

    #[test]
    fn test_form_payload_zeros() {
        let mac_addr = "00:00:00:00:00:00";
        let payload = build_payload(mac_addr).expect(mac_addr);
        validate_magic_packet(mac_addr, &payload);
    }

    #[test]
    fn test_form_payload_max() {
        let mac_addr = "FF:FF:FF:FF:FF:FF";
        let payload = build_payload(mac_addr).expect(mac_addr);
        validate_magic_packet(mac_addr, &payload);
    }

    #[test]
    fn test_parse_mac_address() {
        let mac_addr = "01:23:45:67:89:AB";
        let mac = parse_mac_address(mac_addr).expect(mac_addr);
        assert_eq!(mac, [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB]);
    }

    #[test]
    fn test_parse_mac_address_zeros() {
        let mac_addr = "00:00:00:00:00:00";
        let mac = parse_mac_address(mac_addr).expect(mac_addr);
        assert_eq!(mac, [0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_parse_mac_address_max() {
        let mac_addr = "FF:FF:FF:FF:FF:FF";
        let mac = parse_mac_address(mac_addr).expect(mac_addr);
        assert_eq!(mac, [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    }
}
