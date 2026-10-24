//! 文本编码探测与转换：脚本文件可能是 UTF-8、中文 Windows 的 GBK，
//! 也可能是 PowerShell 默认保存的 UTF-16（带 BOM 或不带）。
//!
//! 关键点：解码必须能识别 UTF-16，否则 UTF-16 的 .ps1 会被当成 GBK 解出乱码，
//! 保存时再按 GBK 写回，原文件就被破坏了。

use encoding_rs::{GBK, UTF_16BE, UTF_16LE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEncoding {
    Utf8,
    Gbk,
    Utf16Le,
    Utf16Be,
}

/// 探测并解码文本：UTF-16（BOM 或 NUL 特征）> 合法 UTF-8 > GBK。
pub fn decode(bytes: &[u8]) -> (String, TextEncoding) {
    if let Some((text, enc)) = decode_utf16_bom(bytes) {
        return (text, enc);
    }
    if let Some(enc) = detect_utf16_without_bom(bytes) {
        let encoding = match enc {
            TextEncoding::Utf16Le => UTF_16LE,
            _ => UTF_16BE,
        };
        let (text, _, _) = encoding.decode(bytes);
        return (text.into_owned(), enc);
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        return (text.to_string(), TextEncoding::Utf8);
    }
    let (text, _, _) = GBK.decode(bytes);
    (text.into_owned(), TextEncoding::Gbk)
}

/// 按原编码写回，避免破坏中文注释；UTF-16 统一补回 BOM。
pub fn encode(text: &str, enc: TextEncoding) -> Vec<u8> {
    match enc {
        TextEncoding::Utf8 => text.as_bytes().to_vec(),
        TextEncoding::Gbk => {
            let (bytes, _, _) = GBK.encode(text);
            bytes.into_owned()
        }
        TextEncoding::Utf16Le => utf16_bytes(text, true, &[0xFF, 0xFE]),
        TextEncoding::Utf16Be => utf16_bytes(text, false, &[0xFE, 0xFF]),
    }
}

fn decode_utf16_bom(bytes: &[u8]) -> Option<(String, TextEncoding)> {
    let (encoding, enc) = match bytes {
        [0xFF, 0xFE, ..] => (UTF_16LE, TextEncoding::Utf16Le),
        [0xFE, 0xFF, ..] => (UTF_16BE, TextEncoding::Utf16Be),
        _ => return None,
    };
    let (text, _) = encoding.decode_with_bom_removal(bytes);
    Some((text.into_owned(), enc))
}

/// 无 BOM 的 UTF-16：按 ASCII 文本高位字节为 0 的特征判断字节序。
///
/// 只在 NUL 占比很高且只集中在同一奇偶位时判定，普通 UTF-8 / GBK 文本不会命中。
fn detect_utf16_without_bom(bytes: &[u8]) -> Option<TextEncoding> {
    if bytes.len() < 8 || bytes.len() % 2 != 0 {
        return None;
    }
    let pairs = bytes.len() / 2;
    let mut even_zero = 0usize;
    let mut odd_zero = 0usize;
    for i in 0..pairs {
        if bytes[i * 2] == 0 {
            even_zero += 1;
        }
        if bytes[i * 2 + 1] == 0 {
            odd_zero += 1;
        }
    }

    // UTF-16LE：低字节在前，ASCII 的高位（奇数位）全为 0
    if odd_zero * 10 >= pairs * 6 && even_zero * 20 < pairs {
        return Some(TextEncoding::Utf16Le);
    }
    // UTF-16BE：高字节在前，ASCII 的低位（偶数位）全为 0
    if even_zero * 10 >= pairs * 6 && odd_zero * 20 < pairs {
        return Some(TextEncoding::Utf16Be);
    }
    None
}

/// 编码成 UTF-16 并补回 BOM。
///
/// 不用 `Encoding::encode`：encoding_rs 面向 Web，UTF-16 只能解码、不能作为输出编码
/// （`UTF_16LE.encode` 实际返回 UTF-8 字节），这里直接按 UTF-16 码元写出。
fn utf16_bytes(text: &str, le: bool, bom: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bom.len() + text.len() * 2);
    out.extend_from_slice(bom);
    for unit in text.encode_utf16() {
        let bytes = if le { unit.to_le_bytes() } else { unit.to_be_bytes() };
        out.extend_from_slice(&bytes);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_le_with_bom_roundtrip() {
        let original = "@echo off\n# 中文注释\nllama-server --port 8080\n";
        let mut bytes = vec![0xFF, 0xFE];
        for unit in original.encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }

        let (text, enc) = decode(&bytes);
        assert_eq!(text, original);
        assert_eq!(enc, TextEncoding::Utf16Le);
        assert_eq!(encode(&text, enc), bytes);
    }

    #[test]
    fn utf16_be_with_bom_roundtrip() {
        let original = "# 中文注释\nllama-server\n";
        let mut bytes = vec![0xFE, 0xFF];
        for unit in original.encode_utf16() {
            bytes.extend_from_slice(&unit.to_be_bytes());
        }
        // BE 解码时 BOM 已剥离，写回会补回 BOM
        let (text, enc) = decode(&bytes);
        assert_eq!(text, original);
        assert_eq!(enc, TextEncoding::Utf16Be);
        assert_eq!(encode(&text, enc), bytes);
    }

    #[test]
    fn utf16_without_bom_detected() {
        let mut bytes = Vec::new();
        for unit in "llama-server -c 4096\n".encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        let (text, enc) = decode(&bytes);
        assert_eq!(enc, TextEncoding::Utf16Le);
        assert_eq!(text, "llama-server -c 4096\n");
    }

    #[test]
    fn plain_text_stays_utf8_or_gbk() {
        let (text, enc) = decode(b"llama-server -c 4096\n");
        assert_eq!(enc, TextEncoding::Utf8);
        assert_eq!(text, "llama-server -c 4096\n");

        // GBK 编码的「中文」：D6 D0 CE C4
        let gbk = [0xD6u8, 0xD0, 0xCE, 0xC4];
        let (text, enc) = decode(&gbk);
        assert_eq!(enc, TextEncoding::Gbk);
        assert_eq!(text, "中文");
        assert_eq!(encode(&text, enc), gbk);
    }
}
