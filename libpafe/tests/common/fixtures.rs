// fixtures.rs — provides commonly used test payloads/frames

use libpafe::protocol::Frame;
use libpafe::types::{BlockData, Idm, Pmm, ServiceCode, SystemCode};

pub fn sample_idm_bytes() -> [u8; 8] {
    [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]
}

pub fn sample_pmm_bytes() -> [u8; 8] {
    [0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10]
}

pub fn sample_system_code() -> SystemCode {
    SystemCode::new(0x0A0B)
}

pub fn sample_idm() -> Idm {
    Idm::from_bytes(sample_idm_bytes())
}

pub fn sample_pmm() -> Pmm {
    Pmm::from_bytes(sample_pmm_bytes())
}

pub fn polling_payload() -> Vec<u8> {
    let mut payload = vec![0x01u8]; // response code for polling
    payload.extend_from_slice(&sample_idm_bytes());
    payload.extend_from_slice(&sample_pmm_bytes());
    payload.extend_from_slice(&sample_system_code().to_le_bytes());
    payload
}

pub fn polling_frame() -> Vec<u8> {
    Frame::encode(&polling_payload()).unwrap()
}

pub fn read_payload_with_block(block_data: &[u8; 16]) -> Vec<u8> {
    let mut payload = vec![0x07u8]; // response code for read
    payload.extend_from_slice(&sample_idm_bytes());
    payload.push(0); // status1
    payload.push(0); // status2
    payload.push(1); // block count
    payload.extend_from_slice(block_data);
    payload
}

pub fn read_frame_with_block(block_data: &[u8; 16]) -> Vec<u8> {
    Frame::encode(&read_payload_with_block(block_data)).unwrap()
}

pub fn write_response_frame_ok() -> Vec<u8> {
    let mut payload = vec![0x09u8];
    payload.extend_from_slice(&sample_idm_bytes());
    payload.push(0); // status1
    payload.push(0); // status2
    Frame::encode(&payload).unwrap()
}

pub fn write_response_frame_err(status1: u8, status2: u8) -> Vec<u8> {
    let mut payload = vec![0x09u8];
    payload.extend_from_slice(&sample_idm_bytes());
    payload.push(status1);
    payload.push(status2);
    Frame::encode(&payload).unwrap()
}

pub fn sample_blockdata(fill: u8) -> BlockData {
    BlockData::from_bytes([fill; 16])
}

pub fn sample_service_code() -> ServiceCode {
    ServiceCode::new(0x090f)
}
