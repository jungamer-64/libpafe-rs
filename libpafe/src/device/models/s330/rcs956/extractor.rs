// libpafe-rs/libpafe/src/device/models/s330/rcs956/extractor.rs

//! Single-frame FeliCa extraction from RCS956/PN532 responses

/// Attempt to extract an inner FeliCa frame (wire frame starting with
/// preamble 0x00 0x00 0xFF) from a RCS956/PN532 response. If the preamble is
/// not present but the expected FeliCa response code is found, build a
/// framed FeliCa packet from the trailing bytes. Returns None when no
/// plausible inner payload could be located.
pub fn extract_felica_from_pn532_response(raw: &[u8], expected_cmd: u8) -> Option<Vec<u8>> {
    use crate::protocol::Frame;

    // 1) If the raw contains a FeliCa frame preamble, extract and return
    // the slice starting at that preamble.
    // Search for an explicit FeliCa preamble but skip ACK-like
    // preambles which have a length byte of zero (these are RCS956/PN532
    // ACK frames and should not be interpreted as FeliCa payloads).
    let mut search_start = 0usize;
    while search_start + 3 <= raw.len() {
        if let Some(rel) = raw[search_start..]
            .windows(3)
            .position(|w| w == crate::constants::FELICA_PREAMBLE)
        {
            let pos = search_start + rel;
            // Ensure length byte exists
            if pos + 3 >= raw.len() {
                break;
            }
            let len = raw[pos + 3] as usize;
            // Skip ACK-like zero-length preambles
            if len == 0 {
                search_start = pos + 1;
                continue;
            }
            let total = 7usize.checked_add(len).unwrap_or(0);
            if total == 0 || pos + total > raw.len() {
                break;
            }
            let frame = &raw[pos..pos + total];
            if let Ok(payload) = Frame::decode(frame) {
                // If the payload begins with a RCS956/PN532 device TFI
                // (0xD5), search within it for the expected
                // FeliCa response code and wrap the trailing
                // payload as a proper FeliCa frame.
                if payload.get(0) == Some(&crate::constants::PN532_CMD_PREFIX_DEVICE) {
                    let expected_response = expected_cmd.wrapping_add(1);
                    if let Some(rel) = payload[1..].iter().position(|&b| b == expected_response) {
                        let idx = 1 + rel;
                        if let Ok(inner_frame) = Frame::encode(&payload[idx..]) {
                            return Some(inner_frame);
                        }
                    }
                } else {
                    // Payload looks like a direct FeliCa payload —
                    // return the original wire frame.
                    return Some(frame.to_vec());
                }
            }
        }
        break;
    }

    // 2) If the raw begins with RCS956/PN532 response prefix (0xD5) and contains
    // the expected FeliCa response code, build a FeliCa frame from the
    // subsequence starting at that code.
    if raw.get(0) == Some(&crate::constants::PN532_CMD_PREFIX_DEVICE) {
        let expected_response = expected_cmd.wrapping_add(1);
        // Search after the RCS956/PN532 header fields so we match the actual
        // embedded FeliCa response rather than command-specific counts.
        if raw.len() > 3 {
            if let Some(pos) = raw[3..].iter().position(|&b| b == expected_response) {
                let idx = 3 + pos;
                let payload = &raw[idx..];
                if let Ok(frame) = Frame::encode(payload) {
                    return Some(frame);
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_felica_from_pn532_response_finds_preamble() {
        use crate::protocol::Frame;
        let mut payload = vec![0x01];
        payload.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        payload.extend_from_slice(&[9, 10, 11, 12, 13, 14, 15, 16]);
        let frame = Frame::encode(&payload).unwrap();
        let mut pn_resp = vec![0xD5, 0x4B, 0x01];
        pn_resp.extend_from_slice(&frame);

        let extracted = extract_felica_from_pn532_response(&pn_resp, 0x00).unwrap();
        assert_eq!(extracted, frame);
    }

    #[test]
    fn extract_felica_from_pn532_response_wraps_payload_when_needed() {
        use crate::protocol::Frame;
        // Some RCS956/PN532 readers may return unframed FeliCa payload bytes after 0xD5 0x4B
        let mut payload = vec![0x01];
        payload.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        payload.extend_from_slice(&[9, 10, 11, 12, 13, 14, 15, 16]);
        let mut pn_resp = vec![0xD5, 0x4B, 0x01];
        pn_resp.extend_from_slice(&payload);

        let extracted = extract_felica_from_pn532_response(&pn_resp, 0x00).unwrap();
        // Should be a framed payload (Frame::encode(payload) == extracted)
        let framed = Frame::encode(&payload).unwrap();
        assert_eq!(extracted, framed);
    }
}
