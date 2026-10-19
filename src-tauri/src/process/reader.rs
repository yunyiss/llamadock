//! stdout / stderr 读取：按时间窗批量合并后推送事件，避免高频小包压垮前端。
//!
//! 说明：读取会切断多字节字符，因此尾部未完成序列会留到下一批；整段非 UTF-8 时按 GBK 兜底。

use std::io::Read;
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::fsutil::encoding;

#[derive(Clone, serde::Serialize)]
pub struct LogPayload {
    pub stream: String,
    pub data: String,
}

/// 合并时间窗
const FLUSH_INTERVAL: Duration = Duration::from_millis(40);
/// 累积到该字节数立即推送
const FLUSH_BYTES: usize = 4096;

/// 启动读取 + 推送线程：读取线程投递分片，合并线程按时间窗批量 emit。
pub fn pump<R: Read + Send + 'static>(app: AppHandle, stream: &'static str, reader: R) {
    let (tx, rx) = channel::<Vec<u8>>();

    std::thread::spawn(move || {
        let mut reader = reader;
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if tx.send(buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
            }
        }
    });

    std::thread::spawn(move || {
        let mut carry: Vec<u8> = Vec::new();
        loop {
            match rx.recv_timeout(FLUSH_INTERVAL) {
                Ok(chunk) => {
                    carry.extend_from_slice(&chunk);
                    if carry.len() >= FLUSH_BYTES {
                        flush(&app, stream, &mut carry, false);
                    }
                }
                Err(RecvTimeoutError::Timeout) => flush(&app, stream, &mut carry, false),
                Err(RecvTimeoutError::Disconnected) => {
                    flush(&app, stream, &mut carry, true);
                    break;
                }
            }
        }
    });
}

fn flush(app: &AppHandle, stream: &str, carry: &mut Vec<u8>, final_flush: bool) {
    if carry.is_empty() {
        return;
    }
    let hold = if final_flush { 0 } else { incomplete_tail(carry) };
    let take = carry.len().saturating_sub(hold);
    if take == 0 {
        return;
    }

    let text = encoding::decode(&carry[..take]).0;
    carry.drain(..take);
    if text.is_empty() {
        return;
    }

    let _ = app.emit(
        "process://log",
        LogPayload {
            stream: stream.to_string(),
            data: text,
        },
    );
}

/// 尾部未完成的多字节 UTF-8 序列长度（需留到下一批再解码）。
fn incomplete_tail(bytes: &[u8]) -> usize {
    let n = bytes.len();
    let scan = n.min(4);
    let mut i = n;
    while i > n - scan {
        i -= 1;
        let b = bytes[i];
        if b & 0xC0 == 0x80 {
            continue; // 连续字节，继续往前找起始字节
        }
        let need = match b {
            0xC0..=0xDF => 2,
            0xE0..=0xEF => 3,
            0xF0..=0xF7 => 4,
            _ => 1,
        };
        let have = n - i;
        return if have < need { have } else { 0 };
    }
    0
}

#[cfg(test)]
mod tests {
    use super::incomplete_tail;

    #[test]
    fn keeps_incomplete_multibyte_tail() {
        // "中" = E4 B8 AD：只收到前两个字节
        assert_eq!(incomplete_tail(&[0xE4, 0xB8]), 2);
        assert_eq!(incomplete_tail("中文".as_bytes()), 0);
        assert_eq!(incomplete_tail(b"hello"), 0);
    }
}
