// libpafe-rs/libpafe/src/device/models/s330/mod.rs

mod commands;
mod config;
mod rcs956;

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
        let _ = transport.vendor_control_write(0x00, 0x0000, 0x0000, commands::rcs956_rf_on());

        // Best-effort read of the RF-ON reply; ignore errors.
        let _ = transport.vendor_control_read(0x00, 0x0000, 0x0000, config::READ_TIMEOUT_MS);

        // Optionally attempt to query firmware version (best-effort).
        // Do not perform a blocking read for the version here because a
        // following operation (e.g. polling) may rely on queued
        // responses. Keep the write best-effort and avoid consuming the
        // transport response queue.
        let _ = transport.vendor_control_write(0x00, 0x0000, 0x0000, commands::rcs956_get_version());

        Ok(())
    }

    fn wrap_command(&self, framed: &[u8], payload: &[u8]) -> Vec<u8> {
        // If caller already created a RCS956/PN533-style packet, forward it.
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

                // Default RCS956/PN533 data write envelope: D4 42 <len> <payload>
        let mut v = Vec::with_capacity(3 + payload.len());
        v.push(0xD4);
        v.push(0x42);
        v.push(payload.len() as u8);
        v.extend_from_slice(payload);
        v
    }

    fn unwrap_response(&self, expected_cmd: u8, raw: &[u8]) -> Result<Vec<u8>> {
        // Use the RCS956 helper to attempt to extract a FeliCa frame from the
        // RCS956/PN532/PN533 response. If the helper fails to locate an inner
        // payload, fall back to returning the raw bytes unchanged so that
        // higher-level callers can decide how to handle unexpected formats.
        if let Some(inner) = rcs956::extract_felica_from_pn532_response(raw, expected_cmd) {
            return Ok(inner);
        }

        Ok(raw.to_vec())
    }

    fn list_passive_targets(
        &self,
        transport: &mut dyn crate::transport::Transport,
        card_type: crate::types::CardType,
        system_code: crate::types::SystemCode,
        max_targets: u8,
        timeout_ms: u64,
    ) -> Result<Vec<crate::card::Card>> {
        use crate::types::CardType;

        // Determine brty (bit rate/type) parameter based on card type
        let brty = match card_type {
            CardType::TypeA => 0x00,  // 106 kbps Type A
            CardType::TypeB => 0x03,  // 106 kbps Type B
            CardType::TypeF => 0x01,  // 212/424 kbps FeliCa
        };

        let mut cmd = rcs956::build_in_list_passive_target(max_targets, brty);

        // For FeliCa (Type F), add the polling payload
        if card_type == CardType::TypeF {
            let payload = crate::protocol::commands::polling::encode_polling(system_code, 0, 0);
            cmd.extend_from_slice(&[0xFF, 0xFF, 0x00, 0x00]);
            cmd.extend_from_slice(&payload);
        }

        // Use vendor_control transfers to send the RCS956/PN533 command and read
        // the RCS956/PN533 response (this will fall back to control_* for
        // transports that don't support vendor_control).
        transport.vendor_control_write(0x00, 0x0000, 0x0000, &cmd)?;
        let raw = transport.vendor_control_read(0x00, 0x0000, 0x0000, timeout_ms)?;

        let mut out = Vec::new();

        // Handle Type B/A differently from FeliCa
        if card_type == CardType::TypeB || card_type == CardType::TypeA {
            // Parse PN532 InListPassiveTarget response for Type B/A
            // Response format: D5 4B [NbTg] [Tg] [SENS_RES/ATQB] [UID...]
            if raw.len() > 4 && raw[0] == 0xD5 && raw[1] == 0x4B {
                let nb_tg = raw[2]; // Number of targets found
                if nb_tg > 0 {
                    let mut pos = 3; // Start after D5 4B NbTg
                    for _ in 0..nb_tg {
                        if pos >= raw.len() {
                            break;
                        }
                        let _tg = raw[pos]; // Target number
                        pos += 1;

                        if card_type == CardType::TypeB {
                            // Type B: ATQB (12 bytes) + ATTRIB_RES_LEN (1 byte) + optional ATTRIB_RES + UID (4 bytes)
                            if pos + 12 <= raw.len() {
                                let mut atqb_bytes = [0u8; 12];
                                atqb_bytes.copy_from_slice(&raw[pos..pos + 12]);
                                let atqb = crate::types::Atqb::from_bytes(atqb_bytes);
                                // UID is in PUPI (bytes 1-4 of ATQB)
                                let uid = crate::types::Uid::from_bytes(atqb_bytes[1..5].to_vec());
                                out.push(crate::card::Card::new_type_b(uid, atqb));
                                pos += 12;
                                // Skip ATTRIB_RES if present
                                if pos < raw.len() {
                                    let attrib_len = raw[pos] as usize;
                                    pos += 1 + attrib_len;
                                }
                            }
                        } else {
                            // Type A: SENS_RES (2 bytes) + SEL_RES (1 byte) + UID_LEN + UID
                            if pos + 3 < raw.len() {
                                pos += 3; // Skip SENS_RES and SEL_RES
                                let uid_len = raw[pos] as usize;
                                pos += 1;
                                if pos + uid_len <= raw.len() {
                                    let uid = crate::types::Uid::from_bytes(raw[pos..pos + uid_len].to_vec());
                                    out.push(crate::card::Card::new_type_a(uid));
                                    pos += uid_len;
                                }
                            }
                        }
                    }
                }
            }
            return Ok(out);
        }

        // For FeliCa (Type F), extract and decode frames
        let frames = rcs956::extract_all_felica_frames_from_pn532_response(&raw, 0x00);
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
                #[cfg_attr(not(test), allow(unused_variables))]
                Err(e) => {
                    #[cfg(test)]
                    eprintln!(
                        "s330: initial decode_response_frame error: {e:?}, frame: {frame:?}"
                    );

                    // Recovery attempt #1: if the candidate looks like a PN532
                    // response region (starts with 0xD5), try to extract an
                    // inner FeliCa wire frame and decode that.
                    if frame.get(0) == Some(&crate::constants::PN532_CMD_PREFIX_DEVICE) {
                        if let Some(inner) =
                            rcs956::extract_felica_from_pn532_response(&frame, expected_cmd)
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
                    eprintln!("s330: decode recovery failed: {e:?}");
                }
            }
        }
        Ok(out)
    }

    fn extract_candidate_frames(&self, raw: &[u8], expected_cmd: u8) -> Vec<Vec<u8>> {
        rcs956::extract_all_felica_frames_from_pn532_response(raw, expected_cmd)
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
    fn s330_model_sends_rcs956_init() {
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
        m.control_write(super::commands::rcs956_get_version())
            .unwrap();
        m.control_write(super::commands::rcs956_deselect()).unwrap();

        assert_eq!(m.sent.len(), 2);
        assert_eq!(m.sent[0], vec![0xD4, 0x02]);
        assert_eq!(m.sent[1], vec![0xD4, 0x44, 0x01]);
    }

    #[test]
    fn s330_in_list_passive_target_builder() {
        let v = super::rcs956::build_in_list_passive_target(1, 0x00);
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
        assert_eq!(data.as_slice(), commands::rcs956_rf_on());
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
            .list_passive_targets(
                &mut m,
                crate::types::CardType::TypeF,
                SystemCode::new(0x0a0b),
                2,
                1000,
            )
            .unwrap();

        assert_eq!(cards.len(), 2);
        assert_eq!(
            cards[0].idm().unwrap().as_bytes(),
            &[1, 2, 3, 4, 5, 6, 7, 8]
        );
        assert_eq!(
            cards[1].idm().unwrap().as_bytes(),
            &[21, 22, 23, 24, 25, 26, 27, 28]
        );
    }

    #[test]
    fn s330_extracts_frames_from_vendor_control_read() {
        use crate::protocol::Frame;
        use crate::types::SystemCode;

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

        // Test direct extraction without transport layer
        let frames = rcs956::extract_all_felica_frames_from_pn532_response(&pn, 0x00);

        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0], f1);
        assert_eq!(frames[1], f2);
    }

    #[test]
    fn s330_decode_extracted_frames_from_vendor_control_read() {
        use crate::protocol::codec;
        use crate::protocol::Frame;
        use crate::types::SystemCode;

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

        // Extract and then decode each frame using the real codec path.
        let frames = rcs956::extract_all_felica_frames_from_pn532_response(&pn, 0x00);

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
            other => panic!("unexpected response: {other:?}"),
        }
        match &decoded[1] {
            crate::protocol::Response::Polling { idm, .. } => {
                assert_eq!(idm.as_bytes(), &[21, 22, 23, 24, 25, 26, 27, 28]);
            }
            other => panic!("unexpected response: {other:?}"),
        }
    }
}
