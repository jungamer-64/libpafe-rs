// libpafe-rs/libpafe/src/device/models/s330/pn533.rs

//! PN533-specific helper utilities

/// Build a simple PN533 RF-ON payload. Kept minimal for unit tests and
/// for later expansion when more PN533 framing is needed.
pub fn build_rf_on() -> &'static [u8] {
    super::config::PN533_RF_ON
}

/// Build a PN533 GetVersion command payload.
pub fn build_get_version() -> &'static [u8] {
    super::config::PN533_GET_VERSION
}

/// Build a PN533 Deselect command payload.
pub fn build_deselect() -> &'static [u8] {
    super::config::PN533_DESELECT
}

/// Build an InListPassiveTarget command payload. The `brty` parameter
/// encodes the bit-rate / target type as per PN533/PN532 conventions
/// (0x00 = 106 kbps type A, 0x01 = 212/424 kbps, etc.).
pub fn build_in_list_passive_target(max_targets: u8, brty: u8) -> Vec<u8> {
    vec![0xD4, 0x4A, max_targets, brty]
}

/// Attempt to extract an inner FeliCa frame (wire frame starting with
/// preamble 0x00 0x00 0xFF) from a PN532 response. If the preamble is
/// not present but the expected FeliCa response code is found, build a
/// framed FeliCa packet from the trailing bytes. Returns None when no
/// plausible inner payload could be located.
pub fn extract_felica_from_pn532_response(raw: &[u8], expected_cmd: u8) -> Option<Vec<u8>> {
    use crate::protocol::Frame;

    // 1) If the raw contains a FeliCa frame preamble, extract and return
    // the slice starting at that preamble.
    if let Some(pos) = raw
        .windows(3)
        .position(|w| w == crate::constants::FELICA_PREAMBLE)
    {
        return Some(raw[pos..].to_vec());
    }

    // 2) If the raw begins with PN532 response prefix (0xD5) and contains
    // the expected FeliCa response code, build a FeliCa frame from the
    // subsequence starting at that code.
    if raw.get(0) == Some(&crate::constants::PN532_CMD_PREFIX_DEVICE) {
        let expected_response = expected_cmd.wrapping_add(1);
        // Search after the PN532 header fields so we match the actual
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

/// Extract all framed FeliCa frames contained in a PN532/PN533 response.
/// Returns a vector of full wire frames (including preamble & checksums)
/// when present. The `expected_cmd` is used as a heuristic for cases
/// where unframed payloads are returned.
pub fn extract_all_felica_frames_from_pn532_response(raw: &[u8], expected_cmd: u8) -> Vec<Vec<u8>> {
    use crate::protocol::Frame;
    let mut out = Vec::new();

    // Scan for explicit FeliCa frame preambles and extract complete
    // wire frames using the length byte at offset +3
    let mut i = 0usize;
    while i + 3 < raw.len() {
        if i + 3 < raw.len() && raw[i..].starts_with(&crate::constants::FELICA_PREAMBLE) {
            // Ensure length byte exists
            if i + 3 >= raw.len() {
                break;
            }
            let len = raw[i + 3] as usize;
            let total = 7usize.checked_add(len).unwrap_or(0);
            if total == 0 {
                break;
            }
            if i + total <= raw.len() {
                out.push(raw[i..i + total].to_vec());
                i += total;
                continue;
            } else {
                // Partial frame: stop scanning
                break;
            }
        }
        i += 1;
    }

    // If no explicit preamble frames found, try to interpret the
    // PN532 response as one or more D5-prefixed regions. Devices may
    // return multiple D5 response regions (multi-target), or return
    // unframed FeliCa payloads inside a D5 region. Scan per-region
    // and attempt to extract either explicit preamble frames or wrap
    // unframed payloads using the expected response code as a hint.
    if out.is_empty() {
        let expected_response = expected_cmd.wrapping_add(1);

        // Small helper: split the raw buffer into contiguous regions
        // that start with 0xD5. Each region typically represents a
        // single PN532 response sequence (useful for multi-target
        // replies).
        fn extract_d5_regions<'a>(raw: &'a [u8]) -> Vec<&'a [u8]> {
            let mut regions = Vec::new();
            let mut i = 0usize;
            while i < raw.len() {
                if raw[i] == crate::constants::PN532_CMD_PREFIX_DEVICE {
                    let start = i;
                    i += 1;
                    // Continue until next D5 or end-of-buffer
                    while i < raw.len() && raw[i] != crate::constants::PN532_CMD_PREFIX_DEVICE {
                        i += 1;
                    }
                    // Keep the region bytes intact (including trailing
                    // zeros). Trimming can accidentally remove the
                    // FeliCa postamble (0x00) when a region ends exactly
                    // on a frame; later extraction uses explicit
                    // preamble/length checks to slice full frames.
                    let end = i;
                    if end > start {
                        regions.push(&raw[start..end]);
                    }
                    continue;
                }
                i += 1;
            }
            regions
        }

        for region in extract_d5_regions(raw) {
            // 1) If the region contains a full FeliCa preamble, extract
            //    the complete wire frame found there.
            if let Some(mut rel_pos) = region
                .windows(3)
                .position(|w| w == crate::constants::FELICA_PREAMBLE)
            {
                let mut extracted = false;
                // Extract one or more concatenated FeliCa wire frames that
                // may be embedded within this single D5 region. Some
                // readers return multiple complete frames back-to-back.
                while rel_pos + 3 < region.len() {
                    let len = region[rel_pos + 3] as usize;
                    let total = 7usize.checked_add(len).unwrap_or(0);
                    if total == 0 || rel_pos + total > region.len() {
                        break;
                    }
                    out.push(region[rel_pos..rel_pos + total].to_vec());
                    extracted = true;
                    rel_pos += total;

                    // If next bytes start a new preamble, continue extracting;
                    // otherwise search forward for the next occurrence.
                    if rel_pos + 3 <= region.len()
                        && region[rel_pos..].starts_with(&crate::constants::FELICA_PREAMBLE)
                    {
                        continue;
                    }
                    if let Some(next) = region[rel_pos..]
                        .windows(3)
                        .position(|w| w == crate::constants::FELICA_PREAMBLE)
                    {
                        rel_pos += next;
                        continue;
                    }
                    break;
                }
                // If we extracted any frames from this region, skip other
                // heuristics and continue to the next region.
                if extracted {
                    continue;
                }
            }

            // 2) Otherwise, if this looks like an InListPassiveTarget
            //    response (D5 0x4B), use the reported target count to
            //    try to partition the trailing bytes into per-target
            //    payloads and wrap each into a FeliCa frame.
            if region.len() >= 3 && region[1] == crate::constants::PN532_RESP_INLIST_PASSIVE_TARGET
            {
                let ntg = region[2] as usize;
                let remainder = &region[3..];

                // Attempt to partition the remainder into `ntg` chunks
                // such that each chunk begins with the expected
                // response byte and can be encoded as a FeliCa frame.
                fn partition_unframed_targets<'a>(
                    rem: &'a [u8],
                    expected_response: u8,
                    targets: usize,
                ) -> Option<Vec<&'a [u8]>> {
                    use crate::protocol::Frame;

                    if targets == 0 {
                        return if rem.is_empty() {
                            Some(Vec::new())
                        } else {
                            None
                        };
                    }
                    if targets == 1 {
                        if !rem.is_empty()
                            && rem[0] == expected_response
                            && Frame::encode(rem).is_ok()
                        {
                            return Some(vec![rem]);
                        }
                        return None;
                    }

                    // Greedy/backtracking: prefer larger prefixes first to
                    // avoid trivial tiny splits; ensure at least 1 byte is
                    // left for each remaining target.
                    let max_len = rem.len().saturating_sub(targets.saturating_sub(1));
                    for len in (1..=max_len).rev() {
                        let candidate = &rem[..len];
                        if candidate.is_empty() || candidate[0] != expected_response {
                            continue;
                        }
                        if Frame::encode(candidate).is_ok() {
                            if let Some(mut rest) = partition_unframed_targets(
                                &rem[len..],
                                expected_response,
                                targets - 1,
                            ) {
                                let mut out = Vec::with_capacity(1 + rest.len());
                                out.push(candidate);
                                out.append(&mut rest);
                                return Some(out);
                            }
                        }
                    }
                    None
                }

                if ntg > 0 && !remainder.is_empty() {
                    if let Some(parts) =
                        partition_unframed_targets(remainder, expected_response, ntg)
                    {
                        for part in parts {
                            if let Ok(frame) = Frame::encode(part) {
                                out.push(frame);
                            }
                        }
                        continue;
                    }
                }

                // Fallback: look for the expected response code inside
                // the region and wrap the trailing payload into a
                // framed FeliCa packet.
                // Prefer finding the response code after the PN532 header
                // bytes (offset 3) so we don't accidentally match the NTG
                // / count field at index 2.
                if region.len() > 3 {
                    if let Some(pos) = region[3..].iter().position(|&b| b == expected_response) {
                        let idx = 3 + pos;
                        let payload = &region[idx..];
                        if let Ok(frame) = Frame::encode(payload) {
                            out.push(frame);
                            continue;
                        }
                    }
                }
            } else {
                // 2) Otherwise, look for the expected response code inside
                //    the region and wrap the trailing payload into a
                //    framed FeliCa packet.
                if let Some(pos) = region.iter().position(|&b| b == expected_response) {
                    let payload = &region[pos..];
                    if let Ok(frame) = Frame::encode(payload) {
                        out.push(frame);
                        continue;
                    }
                }
            }

            // 3) As a last resort, try to find any suffix of the region
            //    that can be interpreted as a valid FeliCa frame when
            //    wrapped. This handles corner-cases where offsets vary
            //    between devices.
            // As a last resort, attempt to encode suffixes that begin
            // after the PN532 header (offset 3). Avoid encoding slices
            // that include the PN532 D5/command header bytes as these
            // are not part of FeliCa payloads.
            if region.len() > 3 {
                for start in 3..region.len() {
                    if let Ok(frame) = Frame::encode(&region[start..]) {
                        out.push(frame);
                        break;
                    }
                }
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_rf_on_is_correct() {
        use crate::device::models::s330::config;
        assert_eq!(build_rf_on(), config::PN533_RF_ON);
    }

    #[test]
    fn build_get_version_is_correct() {
        use crate::device::models::s330::config;
        assert_eq!(build_get_version(), config::PN533_GET_VERSION);
    }

    #[test]
    fn build_in_list_passive_target_builds_vector() {
        let v = build_in_list_passive_target(1, 0x00);
        assert_eq!(v, vec![0xD4, 0x4A, 0x01, 0x00]);
    }

    #[test]
    fn extract_felica_from_pn532_response_finds_preamble() {
        use crate::protocol::Frame;
        let mut payload = vec![0x01];
        payload.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        payload.extend_from_slice(&[9, 10, 11, 12, 13, 14, 15, 16]);
        let frame = Frame::encode(&payload).unwrap();
        let mut pn_resp = vec![0xD5, 0x4B, 0x01];
        pn_resp.extend_from_slice(&frame);

        let extracted = super::extract_felica_from_pn532_response(&pn_resp, 0x00).unwrap();
        assert_eq!(extracted, frame);
    }

    #[test]
    fn extract_felica_from_pn532_response_wraps_payload_when_needed() {
        use crate::protocol::Frame;
        // Some PN532s may return unframed FeliCa payload bytes after 0xD5 0x4B
        let mut payload = vec![0x01];
        payload.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        payload.extend_from_slice(&[9, 10, 11, 12, 13, 14, 15, 16]);
        let mut pn_resp = vec![0xD5, 0x4B, 0x01];
        pn_resp.extend_from_slice(&payload);

        let extracted = super::extract_felica_from_pn532_response(&pn_resp, 0x00).unwrap();
        // Should be a framed payload (Frame::encode(payload) == extracted)
        let framed = Frame::encode(&payload).unwrap();
        assert_eq!(extracted, framed);
    }

    #[test]
    fn extract_all_felica_frames_from_pn532_response_returns_multiple() {
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

        let frames = extract_all_felica_frames_from_pn532_response(&pn, 0x00);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0], f1);
        assert_eq!(frames[1], f2);
    }

    #[test]
    fn extract_all_felica_frames_from_pn532_response_handles_multiple_d5_regions_unframed() {
        use crate::protocol::Frame;

        // Two unframed FeliCa payloads placed into separate D5 regions.
        let mut p1 = vec![0x01];
        p1.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        let mut p2 = vec![0x02];
        p2.extend_from_slice(&[21, 22, 23, 24, 25, 26, 27, 28]);

        let f1 = Frame::encode(&p1).unwrap();
        let f2 = Frame::encode(&p2).unwrap();

        let mut region1 = vec![0xD5, 0x4B, 0x01];
        region1.extend_from_slice(&p1);
        let mut region2 = vec![0xD5, 0x4B, 0x01];
        region2.extend_from_slice(&p2);

        // Combine regions with a separator byte to simulate noisy replies.
        let mut raw = Vec::new();
        raw.extend_from_slice(&region1);
        raw.push(0x00);
        raw.extend_from_slice(&region2);

        let frames = extract_all_felica_frames_from_pn532_response(&raw, 0x00);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0], f1);
        assert_eq!(frames[1], f2);
    }

    #[test]
    fn extract_all_felica_frames_from_pn532_response_handles_unframed_inlist_multi_target() {
        use crate::protocol::Frame;

        // Build two simple unframed per-target payloads that each
        // start with the expected response code (0x01) and will be
        // combined into a single D5 InListPassiveTarget region with
        // ntg == 2.
        let mut t1 = vec![0x01];
        t1.extend_from_slice(&[1, 2, 3, 4]);
        let mut t2 = vec![0x01];
        t2.extend_from_slice(&[5, 6, 7, 8]);

        let f1 = Frame::encode(&t1).unwrap();
        let f2 = Frame::encode(&t2).unwrap();

        let mut region = vec![0xD5, 0x4B, 0x02];
        region.extend_from_slice(&t1);
        region.extend_from_slice(&t2);

        let frames = extract_all_felica_frames_from_pn532_response(&region, 0x00);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0], f1);
        assert_eq!(frames[1], f2);
    }
}
