#[path = "../common/mod.rs"]
mod common;

use libpafe::protocol::{dcs, lcs};

#[test]
fn lcs_and_dcs_examples() {
    assert_eq!(lcs(3), 0xfd);
    assert_eq!(lcs(0), 0x00);
    assert_eq!(lcs(0xff), 0x01);

    assert_eq!(dcs(&[0x01, 0x02, 0x03]), 0xfa);
    assert_eq!(dcs(&[]), 0x00);
}
