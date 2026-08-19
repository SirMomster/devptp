use std::io::{BufRead, BufReader};
use std::net::Ipv4Addr;
use std::path::Path;

/// TCP connection states as defined by Linux kernel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    Established = 0x01,
    SynSent = 0x02,
    SynRecv = 0x03,
    FinWait1 = 0x04,
    FinWait2 = 0x05,
    TimeWait = 0x06,
    Close = 0x07,
    CloseWait = 0x08,
    LastAck = 0x09,
    Listen = 0x0A,
    Closing = 0x0B,
}

impl TcpState {
    pub fn from_hex(hex: &str) -> Option<Self> {
        u8::from_str_radix(hex, 16)
            .ok()
            .and_then(|v| match v {
                0x01 => Some(Self::Established),
                0x02 => Some(Self::SynSent),
                0x03 => Some(Self::SynRecv),
                0x04 => Some(Self::FinWait1),
                0x05 => Some(Self::FinWait2),
                0x06 => Some(Self::TimeWait),
                0x07 => Some(Self::Close),
                0x08 => Some(Self::CloseWait),
                0x09 => Some(Self::LastAck),
                0x0A => Some(Self::Listen),
                0x0B => Some(Self::Closing),
                _ => None,
            })
    }

    pub fn to_hex(&self) -> String {
        format!("{:#02X}", *self as u8)
    }
}

/// A parsed entry from /proc/net/tcp
#[derive(Debug, Clone)]
pub struct TcpEntry {
    pub sl: u32,
    pub local_address: Ipv4Addr,
    pub local_port: u16,
    pub remote_address: Ipv4Addr,
    pub remote_port: u16,
    pub state: TcpState,
    pub tx_queue: u32,
    pub rx_queue: u32,
    pub timer_active: bool,
    pub timer_when: u32,
    pub timeout: u32,
    pub retransmits: u32,
    pub uid: u32,
    pub inode: u64,
}

/// Parse all TCP entries from /proc/net/tcp
pub fn parse_proc_net_tcp(path: &Path) -> std::io::Result<Vec<TcpEntry>> {
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);

    let mut entries = Vec::new();
    let mut first_line = true;

    for line in reader.lines() {
        let line = line?;
        if first_line {
            first_line = false;
            continue; // skip header
        }
        if let Some(entry) = parse_line(&line) {
            entries.push(entry);
        }
    }

    Ok(entries)
}

/// Parse a single line from /proc/net/tcp
fn parse_line(line: &str) -> Option<TcpEntry> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 12 {
        return None;
    }

    let sl_str = parts[0].strip_suffix(':')?;
    let sl = u32::from_str_radix(sl_str, 16).ok()?;

    let local_parts: Vec<&str> = parts[1].split(':').collect();
    // /proc/net/tcp stores IPs in little-endian byte order; swap_bytes() always reverses
    let local_ip = u32::from_str_radix(local_parts[0], 16).ok()?.swap_bytes();
    let local_port = u16::from_str_radix(local_parts[1], 16).ok()?;
    let local_address = Ipv4Addr::from(local_ip);

    let remote_parts: Vec<&str> = parts[2].split(':').collect();
    let remote_ip = u32::from_str_radix(remote_parts[0], 16).ok()?.swap_bytes();
    let remote_port = u16::from_str_radix(remote_parts[1], 16).ok()?;
    let remote_address = Ipv4Addr::from(remote_ip);

    let state = TcpState::from_hex(parts[3])?;

    let tx_queue_parts: Vec<&str> = parts[4].split(':').collect();
    let tx_queue = u32::from_str_radix(tx_queue_parts[0], 16).ok()?;
    let rx_queue = u32::from_str_radix(tx_queue_parts[1], 16).ok()?;

    let timer_parts: Vec<&str> = parts[5].split(':').collect();
    let timer_active = !timer_parts[0].chars().all(|c| c == '0');
    let timer_when = u32::from_str_radix(timer_parts[1], 16).ok()?;

    let retransmits = u32::from_str_radix(parts[6], 16).ok()?;
    let uid = u32::from_str_radix(parts[7], 16).ok()?;
    let timeout_field = u32::from_str_radix(parts[8], 16).ok()?;

    let inode = u64::from_str_radix(parts[9], 16).ok()?;

    Some(TcpEntry {
        sl,
        local_address,
        local_port,
        remote_address,
        remote_port,
        state,
        tx_queue,
        rx_queue,
        timer_active,
        timer_when,
        timeout: timeout_field,
        retransmits,
        uid,
        inode,
    })
}

/// Check if a given local port is in the LISTEN state on this system
pub fn is_listening_on(path: &Path, port: u16) -> std::io::Result<bool> {
    let entries = parse_proc_net_tcp(path)?;
    Ok(entries.iter().any(|e| e.state == TcpState::Listen && e.local_port == port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_state_from_hex() {
        assert_eq!(TcpState::from_hex("01"), Some(TcpState::Established));
        assert_eq!(TcpState::from_hex("0A"), Some(TcpState::Listen));
        assert_eq!(TcpState::from_hex("06"), Some(TcpState::TimeWait));
        assert_eq!(TcpState::from_hex("FF"), None);
    }

    #[test]
    fn test_tcp_state_to_hex() {
        assert_eq!(TcpState::Established.to_hex(), "0x1");
        assert_eq!(TcpState::Listen.to_hex(), "0xA");
    }

    #[test]
    fn test_parse_local_address() {
        // 0.0.0.0:32768 (0x8000) in little-endian hex
        let entry = parse_line(
            "   3: 00000000:8000 00000000:0000 0A 00000000:00000000 \
             00:00000000 00000000     0        0 2844311518 1 \
             000000004ade1a46 100 0 0 10 0",
        )
        .unwrap();
        assert_eq!(entry.local_address, Ipv4Addr::from([0, 0, 0, 0]));
        assert_eq!(entry.local_port, 0x8000); // 32768 (stored as hex in /proc/net/tcp)
        assert_eq!(entry.remote_address, Ipv4Addr::from([0, 0, 0, 0]));
        assert_eq!(entry.remote_port, 0);
        assert_eq!(entry.state, TcpState::Listen);
    }

    #[test]
    fn test_parse_established_connection() {
        // 192.168.5.157 (9D05A8C0 in LE) :8881 (0x22B1)
        let entry = parse_line(
            "   0: 9D05A8C0:22B1 00000000:0000 0A 00000000:00000000 \
             00:00000000 00000000     0        0 2844038225 1 \
             000000006a67644a 100 0 0 10 0",
        )
        .unwrap();
        assert_eq!(
            entry.local_address,
            Ipv4Addr::from([192, 168, 5, 157])
        );
        assert_eq!(entry.local_port, 0x22B1); // 8881
    }

    #[test]
    fn test_is_listening_on() {
        assert!(is_listening_on(Path::new("/proc/net/tcp"), 32768).unwrap());
        // Port 12345 is not listening on this system
        assert!(!is_listening_on(Path::new("/proc/net/tcp"), 12345).unwrap());
    }
}
