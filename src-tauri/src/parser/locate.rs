//! 定位启动行：按脚本类型把续行拼成一条逻辑命令（cmd `^` / sh `\` / PowerShell 反引号），
//! 再判断它是否直接调用 llama.cpp 可执行文件。
//!
//! 边界（宁可少覆盖，不猜）：含 `if` / `for` / `start` 的脚本不做结构化解析。

use super::tokenizer::{self, Token};

/// 脚本类型：不同 shell 的续行符不同，必须区分处理，否则会把普通字符误判成续行。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptKind {
    /// Windows 批处理（`.cmd` / `.bat`）：行尾 `^` 续行
    Cmd,
    /// POSIX shell（`.sh` / `.bash` 等）：行尾 `\` 续行
    Sh,
    /// PowerShell（`.ps1` / `.psm1`）：行尾反引号续行
    PowerShell,
}

impl ScriptKind {
    /// 由文件路径推断脚本类型；未知扩展名按 cmd 处理。
    pub fn from_path(path: &str) -> Self {
        let ext = path.rsplit('.').next().unwrap_or_default().to_lowercase();
        match ext.as_str() {
            "sh" | "bash" | "zsh" | "ksh" | "dash" => ScriptKind::Sh,
            "ps1" | "psm1" => ScriptKind::PowerShell,
            _ => ScriptKind::Cmd,
        }
    }

    /// 该类型的续行符（删除行首参数时也要吞掉上一行的这个字符）。
    pub fn continuation_byte(self) -> u8 {
        match self {
            ScriptKind::Cmd => b'^',
            ScriptKind::Sh => b'\\',
            ScriptKind::PowerShell => b'`',
        }
    }

    /// 行尾是否为续行符。
    ///
    /// 必须紧贴行尾，后面不能有空格：cmd 的 `^ ` 转义的是那个空格（换行不会被吃掉），
    /// sh 的 `\ ` 是转义空格，PowerShell 的反引号后跟空格同样不续行。
    /// 宽松判断会把实际不续行的两行拼成一条，面板里就会出现运行时并不生效的参数。
    pub fn ends_with_continuation(self, line: &str) -> bool {
        line.ends_with(self.continuation_byte() as char)
    }

    /// 去掉行尾的续行符与多余空白。
    pub fn drop_continuation<'a>(self, line: &'a str) -> &'a str {
        let trimmed = line.trim_end();
        trimmed
            .strip_suffix(self.continuation_byte() as char)
            .unwrap_or(trimmed)
            .trim_end()
    }
}

#[derive(Debug, Clone)]
pub struct LaunchLine {
    /// 首行行号（从 0 开始）
    pub line_index: usize,
    pub tokens: Vec<Token>,
    /// 命令在原文中的字节终点（不含行尾换行）
    pub end: usize,
}

/// 可执行文件名（不含扩展名）白名单
const EXE_NAMES: [&str; 3] = ["llama-server", "llama-cli", "main"];

struct Line {
    no: usize,
    /// 行内容在原文中的起点
    start: usize,
    /// 行内容长度（不含行尾换行）
    len: usize,
}

fn split_lines(text: &str) -> Vec<Line> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    for (no, raw) in text.split_inclusive('\n').enumerate() {
        let mut len = raw.len();
        if len > 0 && raw.as_bytes()[len - 1] == b'\n' {
            len -= 1;
            if len > 0 && raw.as_bytes()[len - 1] == b'\r' {
                len -= 1;
            }
        }
        out.push(Line { no, start: offset, len });
        offset += raw.len();
    }
    out
}

fn is_exe(token: &str) -> bool {
    let base = token.rsplit(['/', '\\']).next().unwrap_or(token);
    let stem = base.strip_suffix(".exe").unwrap_or(base);
    EXE_NAMES.contains(&stem) || stem.contains("llama")
}

fn is_comment(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with('#')
        || trimmed.starts_with("::")
        || trimmed.to_lowercase().starts_with("rem ")
}

/// 把拼接串里的偏移映射回原文偏移。
fn translate(map: &[(usize, usize, usize)], offset: usize) -> Option<usize> {
    for &(joined_start, orig_start, len) in map {
        if offset >= joined_start && offset <= joined_start + len {
            return Some(orig_start + (offset - joined_start));
        }
    }
    None
}

pub fn locate(text: &str, kind: ScriptKind) -> Option<LaunchLine> {
    let lines = split_lines(text);
    let mut i = 0;

    while i < lines.len() {
        if is_comment(&text[lines[i].start..lines[i].start + lines[i].len]) {
            i += 1;
            continue;
        }

        // 收集续行组（按脚本类型使用对应的续行符）
        let mut group = vec![i];
        while kind.ends_with_continuation(&text[lines[i].start..lines[i].start + lines[i].len])
            && i + 1 < lines.len()
        {
            i += 1;
            group.push(i);
        }

        // 拼接成一条逻辑命令，并记录每段与原文的对应关系
        let mut joined = String::new();
        let mut map: Vec<(usize, usize, usize)> = Vec::new();
        for (k, &li) in group.iter().enumerate() {
            let line = &lines[li];
            let segment = kind.drop_continuation(&text[line.start..line.start + line.len]);
            if k > 0 {
                joined.push(' ');
            }
            map.push((joined.len(), line.start, segment.len()));
            joined.push_str(segment);
        }

        let mut tokens = tokenizer::tokenize(&joined);
        for token in tokens.iter_mut() {
            if let (Some(s), Some(e)) = (translate(&map, token.start), translate(&map, token.end)) {
                token.start = s;
                token.end = e;
            }
        }

        // PowerShell 用 `&`（调用运算符）执行带引号的路径，这里先摘掉，
        // 让启动行结构与 cmd / sh 保持一致（参数从第一个 token 之后开始）。
        if kind == ScriptKind::PowerShell
            && tokens.first().is_some_and(|t| t.text == "&" || t.text == ".")
        {
            tokens.remove(0);
        }

        if tokens.first().is_some_and(|t| is_exe(&t.text)) {
            let last = &lines[*group.last().unwrap()];
            return Some(LaunchLine {
                line_index: lines[group[0]].no,
                tokens,
                end: last.start + last.len,
            });
        }

        i += 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caret_at_line_end_continues() {
        let text = "llama-server.exe ^\n  --port 8080\n";
        let launch = locate(text, ScriptKind::Cmd).expect("应识别到启动行");
        assert!(launch.tokens.iter().any(|t| t.text == "--port"));
    }

    #[test]
    fn caret_with_trailing_space_does_not_continue() {
        // cmd 里 `^ ` 转义的是那个空格，换行没有被吃掉：第二行是另一条命令
        let text = "llama-server.exe -m a.gguf ^ \n  --port 8080\n";
        let launch = locate(text, ScriptKind::Cmd).expect("应识别到启动行");
        assert!(launch.tokens.iter().all(|t| t.text != "--port"));
    }

    #[test]
    fn backslash_with_trailing_space_does_not_continue() {
        // sh 里 `\ ` 是转义空格，不是续行
        let text = "llama-server -m a.gguf \\ \n  --port 8080\n";
        let launch = locate(text, ScriptKind::Sh).expect("应识别到启动行");
        assert!(launch.tokens.iter().all(|t| t.text != "--port"));
    }

    #[test]
    fn backtick_at_line_end_continues() {
        let text = "llama-server.exe `\n  --port 8080\n";
        let launch = locate(text, ScriptKind::PowerShell).expect("应识别到启动行");
        assert!(launch.tokens.iter().any(|t| t.text == "--port"));
    }
}
