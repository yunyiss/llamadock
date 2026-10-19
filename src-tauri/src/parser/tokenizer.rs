//! 分词：按引号规则切分一行命令，记录每个 token 在原行中的字节区间（供最小差异重建使用）。
//!
//! 说明：MVP 采用 cmd/sh 共用的简化规则，不处理 `^` 转义；
//! 多行续行的拼接（cmd `^` / sh `\` / PowerShell 反引号）在 locate 里按脚本类型处理。

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// 去掉外层引号后的值
    pub text: String,
    /// 在行内的字节起点
    pub start: usize,
    /// 在行内的字节终点（不含）
    pub end: usize,
    /// 是否出现过引号
    pub quoted: bool,
}

pub fn tokenize(line: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = line.char_indices().peekable();

    while let Some(&(start, c)) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }

        let mut text = String::new();
        let mut quoted = false;
        let mut end = start;
        let mut in_double = false;
        let mut in_single = false;

        while let Some(&(idx, ch)) = chars.peek() {
            let width = ch.len_utf8();

            if in_double {
                chars.next();
                end = idx + width;
                if ch == '"' {
                    in_double = false;
                    quoted = true;
                } else {
                    text.push(ch);
                }
                continue;
            }
            if in_single {
                chars.next();
                end = idx + width;
                if ch == '\'' {
                    in_single = false;
                    quoted = true;
                } else {
                    text.push(ch);
                }
                continue;
            }
            if ch == '"' {
                chars.next();
                end = idx + width;
                in_double = true;
                quoted = true;
                continue;
            }
            if ch == '\'' {
                chars.next();
                end = idx + width;
                in_single = true;
                quoted = true;
                continue;
            }
            if ch.is_whitespace() {
                break;
            }
            chars.next();
            end = idx + width;
            text.push(ch);
        }

        tokens.push(Token {
            text,
            start,
            end,
            quoted,
        });
    }

    tokens
}
