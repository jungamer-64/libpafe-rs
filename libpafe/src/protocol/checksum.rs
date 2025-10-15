// libpafe-rs/libpafe/src/protocol/checksum.rs

/// Compute Length Checksum (LCS) for FeliCa frame
/// LCS = 0x100 - length (mod 256)
pub fn lcs(len: u8) -> u8 {
    0u8.wrapping_sub(len)
}

/// Compute Data Checksum (DCS) for FeliCa frame
/// DCS = 0x100 - (sum(payload) & 0xff)
pub fn dcs(payload: &[u8]) -> u8 {
    let sum = payload.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
    0u8.wrapping_sub(sum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lcs_examples() {
        assert_eq!(lcs(3), 0xfd);
        assert_eq!(lcs(0), 0x00);
        assert_eq!(lcs(0xff), 0x01);
    }

    #[test]
    fn dcs_examples() {
        assert_eq!(dcs(&[0x01, 0x02, 0x03]), 0xfa);
        assert_eq!(dcs(&[]), 0x00);
    }
}
