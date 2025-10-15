// libpafe-rs/libpafe/src/device/models/s330/mod.rs

mod commands;
mod config;
mod pn533;

use crate::Result;

pub struct S330Model;

impl S330Model {
    pub fn new() -> Self {
        Self
    }
}

impl crate::device::models::DeviceModel for S330Model {
    fn initialize(&self, transport: &mut dyn crate::transport::Transport) -> Result<()> {
        // Prefer explicit vendor control transfers when available. Fall
        // back to control_write/read via the Transport defaults.
        // Best-effort RF-ON: some systems/devices may return a Pipe error
        // for this vendor transfer. Treat it as non-fatal and continue.
        let _ = transport.vendor_control_write(0x00, 0x0000, 0x0000, commands::pn533_rf_on());

        // Best-effort read of the RF-ON reply; ignore errors.
        let _ = transport.vendor_control_read(0x00, 0x0000, 0x0000, config::READ_TIMEOUT_MS);

        // Optionally attempt to query firmware version (best-effort).
        // Do not perform a blocking read for the version here because a
        // following operation (e.g. polling) may rely on queued
        // responses. Keep the write best-effort and avoid consuming the
        // transport response queue.
        let _ = transport.vendor_control_write(0x00, 0x0000, 0x0000, commands::pn533_get_version());

        Ok(())
    }

    fn wrap_command(&self, framed: &[u8], payload: &[u8]) -> Vec<u8> {
        // If caller already created a PN533-style packet, forward it.
        if !framed.is_empty() && framed[0] == 0xD4 {
            return framed.to_vec();
        }

        // If this is a Polling command (0x00), use InListPassiveTarget
        // (PN532 command 0x4A) as per S330 behavior.
        if !payload.is_empty() && payload[0] == 0x00 {
            let mut v = Vec::with_capacity(payload.len() + 4);
            v.push(0xD4);
            v.push(0x4A);
            v.push(0x01); // max targets
            v.push(0x01); // protocol: Felica-like
            v.extend_from_slice(payload);
            return v;
        }

        // Default PN533 data write envelope: D4 42 <len> <payload>
        let mut v = Vec::with_capacity(payload.len() + 3);
        v.push(0xD4);
        v.push(0x42);
        v.push((payload.len() + 1) as u8);
        v.extend_from_slice(payload);
        v
    }

    fn unwrap_response(&self, expected_cmd: u8, raw: &[u8]) -> Result<Vec<u8>> {
        use crate::protocol::Frame;

        // Use the PN533 helper to attempt to extract a FeliCa frame from the
        // PN532/PN533 response. If the helper fails to locate an inner
        // payload, fall back to returning the raw bytes unchanged so that
        // higher-level callers can decide how to handle unexpected formats.
        if let Some(inner) = pn533::extract_felica_from_pn532_response(raw, expected_cmd) {
            return Ok(inner);
        }

        Ok(raw.to_vec())
    }

    fn list_passive_targets(
        &self,
        transport: &mut dyn crate::transport::Transport,
        system_code: crate::types::SystemCode,
        max_targets: u8,
        timeout_ms: u64,
    ) -> Result<Vec<crate::card::Card>> {
        // Build a Polling payload and wrap it in a PN533 InListPassiveTarget
        // command so the S330/PN533 performs target discovery.
        let payload = crate::protocol::commands::polling::encode_polling(system_code, 0, 0);
        let mut cmd = pn533::build_in_list_passive_target(max_targets, 0x00);
        cmd.extend_from_slice(&payload);

        // Use vendor_control transfers to send the PN533 command and read
        // the PN533 response (this will fall back to control_* for
        // transports that do not implement explicit vendor control).
        transport.vendor_control_write(0x00, 0x0000, 0x0000, &cmd)?;
        let raw = transport.vendor_control_read(0x00, 0x0000, 0x0000, timeout_ms)?;

        // Extract all inner FeliCa frames and decode them. We use the
        // polling command code (0x00) as the expected response type so
        // helper functions can locate unframed payloads when necessary.
        // No debug prints in normal test runs; extract frames silently.
        let frames = pn533::extract_all_felica_frames_from_pn532_response(&raw, 0x00);
        let mut out = Vec::new();
        // Expected command code for Polling (used by helper heuristics)
        let expected_cmd = 0x00u8;
        for frame in frames {
            match crate::protocol::codec::decode_response_frame(expected_cmd, &frame) {
                Ok(resp) => {
                    if let crate::protocol::Response::Polling {
                        idm,
                        pmm,
                        system_code,
                    } = resp
                    {
                        out.push(crate::card::Card::new(idm, pmm, system_code));
                    }
                }
                Err(e) => {
                    #[cfg(test)]
                    eprintln!(
                        "s330: initial decode_response_frame error: {:?}, frame: {:?}",
                        e, frame
                    );

                    // Recovery attempt #1: if the candidate looks like a PN532
                    // response region (starts with 0xD5), try to extract an
                    // inner FeliCa wire frame and decode that.
                    if frame.get(0) == Some(&crate::constants::PN532_CMD_PREFIX_DEVICE) {
                        if let Some(inner) =
                            pn533::extract_felica_from_pn532_response(&frame, expected_cmd)
                        {
                            if let Ok(resp2) =
                                crate::protocol::codec::decode_response_frame(expected_cmd, &inner)
                            {
                                if let crate::protocol::Response::Polling {
                                    idm,
                                    pmm,
                                    system_code,
                                } = resp2
                                {
                                    out.push(crate::card::Card::new(idm, pmm, system_code));
                                    continue;
                                }
                            }
                        }
                    }

                    // Recovery attempt #2: maybe the extractor returned an
                    // unframed payload (no preamble). Try building a proper
                    // FeliCa wire frame around the bytes and decode that.
                    if let Ok(rewrapped) = crate::protocol::Frame::encode(&frame) {
                        if let Ok(resp2) =
                            crate::protocol::codec::decode_response_frame(expected_cmd, &rewrapped)
                        {
                            if let crate::protocol::Response::Polling {
                                idm,
                                pmm,
                                system_code,
                            } = resp2
                            {
                                out.push(crate::card::Card::new(idm, pmm, system_code));
                                continue;
                            }
                        }
                    }

                    // If we reach here, decoding failed and recovery didn't
                    // succeed. Emit a test-only diagnostic with the error so
                    // the failing test run can capture the details.
                    #[cfg(test)]
                    eprintln!("s330: decode recovery failed: {:?}", e);
                }
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::models::DeviceModel;
    use crate::transport::mock::MockTransport;
    use crate::transport::traits::Transport;
    use crate::types::DeviceType;

    #[test]
    fn s330_model_sends_pn533_init() {
        let mut m = MockTransport::new(DeviceType::S330);
        m.push_response(vec![0x00]);
        let model = S330Model::new();
        model.initialize(&mut m).unwrap();
        // Initialization may perform multiple vendor/control writes
        assert!(m.sent.len() >= 1);
        assert_eq!(m.sent[0], vec![0xD4, 0x32, 0x01, 0x01]);
    }

    #[test]
    fn s330_commands_get_version_and_deselect_sent() {
        let mut m = MockTransport::new(DeviceType::S330);
        // direct control_write uses the transport default implementation
        m.control_write(super::commands::pn533_get_version())
            .unwrap();
        m.control_write(super::commands::pn533_deselect()).unwrap();

        assert_eq!(m.sent.len(), 2);
        assert_eq!(m.sent[0], vec![0xD4, 0x02]);
        assert_eq!(m.sent[1], vec![0xD4, 0x44, 0x01]);
    }

    #[test]
    fn s330_in_list_passive_target_builder() {
        let v = super::pn533::build_in_list_passive_target(1, 0x00);
        assert_eq!(v, vec![0xD4, 0x4A, 0x01, 0x00]);
        let mut m = MockTransport::new(DeviceType::S330);
        m.control_write(&v).unwrap();
        assert_eq!(m.sent.last().unwrap(), &v);
    }

    #[test]
    fn s330_model_uses_vendor_control_parameters() {
        let mut m = MockTransport::new(DeviceType::S330);
        m.push_response(vec![0xAA]);
        let model = S330Model::new();
        model.initialize(&mut m).unwrap();

        // Ensure vendor_control_write was invoked for RF-ON and get_version
        assert!(
            m.vendor_calls.len() >= 1,
            "expected at least one vendor call"
        );
        let (req, val, idx, data) = &m.vendor_calls[0];
        assert_eq!(*req, 0x00);
        assert_eq!(*val, 0x0000);
        assert_eq!(*idx, 0x0000);
        assert_eq!(data.as_slice(), commands::pn533_rf_on());
    }

    #[test]
    fn s330_list_passive_targets_returns_multiple_cards() {
        use crate::protocol::Frame;
        use crate::transport::mock::MockTransport;
        use crate::types::SystemCode;

        let mut m = MockTransport::new(crate::types::DeviceType::S330);
        // Init ack consumed by initialize()
        m.push_response(vec![0xAA]);

        // Build two polling payloads wrapped as FeliCa frames and append
        // them into a PN532 InListPassiveTarget response.
        let mut p1 = vec![0x01];
        p1.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        p1.extend_from_slice(&[9, 10, 11, 12, 13, 14, 15, 16]);
        p1.extend_from_slice(&SystemCode::new(0x0a0b).to_le_bytes());
        let f1 = Frame::encode(&p1).unwrap();

        let mut p2 = vec![0x01];
        p2.extend_from_slice(&[21, 22, 23, 24, 25, 26, 27, 28]);
        p2.extend_from_slice(&[29, 30, 31, 32, 33, 34, 35, 36]);
        p2.extend_from_slice(&SystemCode::new(0x1111).to_le_bytes());
        let f2 = Frame::encode(&p2).unwrap();

        let mut pn = vec![0xD5, 0x4B, 0x02];
        pn.extend_from_slice(&f1);
        pn.extend_from_slice(&f2);
        m.push_response(pn);

        let model = S330Model::new();
        model.initialize(&mut m).unwrap();
        let cards = model
            .list_passive_targets(&mut m, SystemCode::new(0x0a0b), 2, 1000)
            .unwrap();

        assert_eq!(cards.len(), 2);
        assert_eq!(cards[0].idm().as_bytes(), &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(cards[1].idm().as_bytes(), &[21, 22, 23, 24, 25, 26, 27, 28]);
    }

    #[test]
    fn s330_extracts_frames_from_vendor_control_read() {
        use crate::protocol::Frame;
        use crate::transport::mock::MockTransport;
        use crate::types::SystemCode;

        let mut m = MockTransport::new(crate::types::DeviceType::S330);
        // Init ack consumed by initialize()
        m.push_response(vec![0xAA]);

        // Build two polling payloads wrapped as FeliCa frames and append
        // them into a PN532 InListPassiveTarget response.
        let mut p1 = vec![0x01];
        p1.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        p1.extend_from_slice(&[9, 10, 11, 12, 13, 14, 15, 16]);
        p1.extend_from_slice(&SystemCode::new(0x0a0b).to_le_bytes());
        let f1 = Frame::encode(&p1).unwrap();

        let mut p2 = vec![0x01];
        p2.extend_from_slice(&[21, 22, 23, 24, 25, 26, 27, 28]);
        p2.extend_from_slice(&[29, 30, 31, 32, 33, 34, 35, 36]);
        p2.extend_from_slice(&SystemCode::new(0x1111).to_le_bytes());
        let f2 = Frame::encode(&p2).unwrap();

        let mut pn = vec![0xD5, 0x4B, 0x02];
        pn.extend_from_slice(&f1);
        pn.extend_from_slice(&f2);
        m.push_response(pn.clone());

        // Simulate the same vendor_control_read flow used in list_passive_targets.
        let raw = m.vendor_control_read(0x00, 0x0000, 0x0000, 1000).unwrap();
        let frames = super::pn533::extract_all_felica_frames_from_pn532_response(&raw, 0x00);

        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0], f1);
        assert_eq!(frames[1], f2);
    }

    #[test]
    fn s330_decode_extracted_frames_from_vendor_control_read() {
        use crate::protocol::codec;
        use crate::protocol::Frame;
        use crate::transport::mock::MockTransport;
        use crate::types::SystemCode;

        let mut m = MockTransport::new(crate::types::DeviceType::S330);
        m.push_response(vec![0xAA]);

        let mut p1 = vec![0x01];
        p1.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        p1.extend_from_slice(&[9, 10, 11, 12, 13, 14, 15, 16]);
        p1.extend_from_slice(&SystemCode::new(0x0a0b).to_le_bytes());
        let f1 = Frame::encode(&p1).unwrap();

        let mut p2 = vec![0x01];
        p2.extend_from_slice(&[21, 22, 23, 24, 25, 26, 27, 28]);
        p2.extend_from_slice(&[29, 30, 31, 32, 33, 34, 35, 36]);
        p2.extend_from_slice(&SystemCode::new(0x1111).to_le_bytes());
        let f2 = Frame::encode(&p2).unwrap();

        let mut pn = vec![0xD5, 0x4B, 0x02];
        pn.extend_from_slice(&f1);
        pn.extend_from_slice(&f2);
        m.push_response(pn);

        // Extract and then decode each frame using the real codec path.
        let raw = m.vendor_control_read(0x00, 0x0000, 0x0000, 1000).unwrap();
        let frames = super::pn533::extract_all_felica_frames_from_pn532_response(&raw, 0x00);

        let mut decoded = Vec::new();
        for frame in frames {
            let resp = codec::decode_response_frame(0x00, &frame).unwrap();
            decoded.push(resp);
        }

        assert_eq!(decoded.len(), 2);
        match &decoded[0] {
            crate::protocol::Response::Polling { idm, .. } => {
                assert_eq!(idm.as_bytes(), &[1, 2, 3, 4, 5, 6, 7, 8]);
            }
            other => panic!("unexpected response: {:?}", other),
        }
        match &decoded[1] {
            crate::protocol::Response::Polling { idm, .. } => {
                assert_eq!(idm.as_bytes(), &[21, 22, 23, 24, 25, 26, 27, 28]);
            }
            other => panic!("unexpected response: {:?}", other),
        }
    }
}
