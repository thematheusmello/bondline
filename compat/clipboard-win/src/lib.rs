//! Original Windows API adapter for Arboard; no Boost-licensed implementation is copied.

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

#[cfg(any(windows, test))]
fn decode_text(bytes: &[u8]) -> std::io::Result<String> {
    if !bytes.len().is_multiple_of(2) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "odd UTF-16 byte count",
        ));
    }
    let words: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|p| u16::from_le_bytes([p[0], p[1]]))
        .collect();
    let end = words.iter().position(|c| *c == 0).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "clipboard text lacks terminator",
        )
    })?;
    String::from_utf16(
        words
            .get(..end)
            .ok_or_else(|| std::io::Error::other("invalid UTF-16 range"))?,
    )
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

#[cfg(any(windows, test))]
fn html_range(data: &[u8]) -> std::io::Result<std::ops::Range<usize>> {
    let invalid = || {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid CF_HTML fragment offsets",
        )
    };
    let mut start = None;
    let mut end = None;
    for line in data
        .get(..data.len().min(4096))
        .ok_or_else(invalid)?
        .split(|b| *b == b'\r' || *b == b'\n')
    {
        for (prefix, slot) in [
            (b"StartFragment:".as_slice(), &mut start),
            (b"EndFragment:".as_slice(), &mut end),
        ] {
            if let Some(value) = line.strip_prefix(prefix) {
                if slot.is_some() {
                    return Err(invalid());
                }
                *slot = Some(
                    std::str::from_utf8(value)
                        .map_err(|_| invalid())?
                        .trim()
                        .parse::<usize>()
                        .map_err(|_| invalid())?,
                );
            }
        }
        if start.is_some() && end.is_some() {
            break;
        }
    }
    let range = start.ok_or_else(invalid)?..end.ok_or_else(invalid)?;
    data.get(range.clone()).ok_or_else(invalid)?;
    Ok(range)
}

#[cfg(test)]
mod tests {
    use super::{decode_text, html_range};

    #[test]
    fn unicode_text_is_terminated_and_checked() -> Result<(), Box<dyn std::error::Error>> {
        let bytes: Vec<u8> = "NaCl α"
            .encode_utf16()
            .chain(Some(0))
            .flat_map(u16::to_le_bytes)
            .collect();
        assert_eq!(decode_text(&bytes)?, "NaCl α");
        assert!(decode_text(&[0]).is_err());
        assert!(decode_text(&[0, 0xd8, 0, 0]).is_err());
        assert!(decode_text(&[65, 0]).is_err());
        Ok(())
    }

    #[test]
    fn html_fragment_offsets_are_byte_offsets_and_bounded() -> Result<(), Box<dyn std::error::Error>>
    {
        let data = b"StartFragment:0000000054\r\nEndFragment:0000000059\r\nABCDhello";
        assert_eq!(data.get(html_range(data)?), Some(b"hello".as_slice()));
        assert!(html_range(b"StartFragment:20\nEndFragment:1\n").is_err());
        assert!(html_range(b"StartFragment:1\nEndFragment:9999\n").is_err());
        assert!(html_range(b"no offsets").is_err());
        Ok(())
    }
}
