//! 工作目录文件监控：基于 notify 侦听脚本目录变化，去抖后推送事件（见技术方案 §4.5）。

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State};

/// 同一时刻只监听一个目录。
#[derive(Default)]
pub struct WatcherState {
    inner: Mutex<Option<RecommendedWatcher>>,
}

/// 开始监听指定目录（重复调用会替换旧监听）。
#[tauri::command]
pub fn watch_dir(app: AppHandle, state: State<WatcherState>, path: String) -> Result<(), String> {
    let mut guard = state.inner.lock().map_err(|_| "监控器加锁失败")?;
    *guard = None;

    let last = Arc::new(Mutex::new(Instant::now()));
    let handle = app.clone();

    let mut watcher =
        notify::recommended_watcher(move |result: Result<notify::Event, notify::Error>| {
            if result.is_err() {
                return;
            }
            let Ok(mut stamp) = last.lock() else {
                return;
            };
            // 去抖：短时间内只提示一次
            if stamp.elapsed() < Duration::from_millis(400) {
                return;
            }
            *stamp = Instant::now();
            let _ = handle.emit("scripts://changed", ());
        })
        .map_err(|e| e.to_string())?;

    watcher
        .watch(Path::new(&path), RecursiveMode::NonRecursive)
        .map_err(|e| e.to_string())?;

    *guard = Some(watcher);
    Ok(())
}
