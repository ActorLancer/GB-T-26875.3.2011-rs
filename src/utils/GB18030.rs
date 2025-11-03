use encoding_rs::GB18030;

pub fn decode_gb18030(input: &[u8]) -> Option<String> {
    let (gb18030_bytes, _encoding, is_error) = GB18030.decode(input);

    if is_error {
        return None;
    }

    Some(gb18030_bytes.into_owned())
}

pub fn encode2gb18030(input: &str) -> Option<Vec<u8>> {
    let (gb18030_bytes, _encoding, is_error) = GB18030.encode(input);

    if is_error {
        return None;
    }

    Some(gb18030_bytes.into_owned())
}

