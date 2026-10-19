//! 文本编码探测与转换：脚本文件可能是 UTF-8，也可能是中文 Windows 的 GBK。

use encoding_rs::GBK;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEncoding {
    Utf8,
    Gbk,
}

/// 探测并解码文本：合法 UTF-8 用 UTF-8，否则按 GBK 处理。
pub fn decode(bytes: &[u8]) -> (String, TextEncoding) {
    if let Ok(text) = std::str::from_utf8(bytes) {
        return (text.to_string(), TextEncoding::Utf8);
    }
    let (text, _, _) = GBK.decode(bytes);
    (text.into_owned(), TextEncoding::Gbk)
}

/// 按原编码写回，避免破坏中文注释。
pub fn encode(text: &str, enc: TextEncoding) -> Vec<u8> {
    match enc {
        TextEncoding::Utf8 => text.as_bytes().to_vec(),
        TextEncoding::Gbk => {
            let (bytes, _, _) = GBK.encode(text);
            bytes.into_owned()
        }
    }
}
