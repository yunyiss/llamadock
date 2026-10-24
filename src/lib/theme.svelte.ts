// 主题状态：三态（跟随系统 / 深色 / 浅色），负责持久化、首帧对齐与切换动画。
//
// 首帧由 app.html 的内联脚本写入 <html data-theme>，这里只做状态同步与后续切换，
// 避免启动时先渲染深色再跳到浅色。

import { getConfig, saveTheme } from "$lib/ipc";

export type ThemeMode = "auto" | "light" | "dark";
export type Scheme = "light" | "dark";

const STORAGE_KEY = "llamadock-theme";
const TRANSITION_MS = 240;

const darkQuery =
  typeof window !== "undefined" ? window.matchMedia("(prefers-color-scheme: dark)") : null;

function isMode(value: unknown): value is ThemeMode {
  return value === "auto" || value === "light" || value === "dark";
}

function readStoredMode(): ThemeMode {
  try {
    const value = localStorage.getItem(STORAGE_KEY);
    return isMode(value) ? value : "auto";
  } catch {
    return "auto";
  }
}

export function systemScheme(): Scheme {
  return darkQuery?.matches ? "dark" : "light";
}

export function resolveScheme(mode: ThemeMode): Scheme {
  return mode === "auto" ? systemScheme() : mode;
}

export const theme = $state({
  mode: readStoredMode() as ThemeMode,
  scheme: "dark" as Scheme,
});

/** 把主题写到 <html> 上，并同步原生窗口外观（标题栏 / 菜单）。 */
export function applyScheme(scheme: Scheme): void {
  theme.scheme = scheme;
  if (typeof document !== "undefined") {
    document.documentElement.dataset.theme = scheme;
  }
  void syncWindowTheme(theme.mode === "auto" ? null : scheme);
}

/** 原生窗口外观：auto 传 null 表示交还给系统，避免强制覆盖。 */
async function syncWindowTheme(next: Scheme | null): Promise<void> {
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().setTheme(next);
  } catch {
    /* 浏览器环境或平台不支持时忽略 */
  }
}

/** 带过渡地应用一个主题（优先 View Transitions，不支持则用遮罩淡出）。 */
function transitionTo(scheme: Scheme): void {
  const startViewTransition = (
    document as unknown as { startViewTransition?: (callback: () => void) => unknown }
  ).startViewTransition;

  if (typeof startViewTransition === "function") {
    startViewTransition.call(document, () => applyScheme(scheme));
    return;
  }

  const veil = document.createElement("div");
  veil.className = "theme-veil";
  veil.style.background = getComputedStyle(document.body).backgroundColor;
  document.body.appendChild(veil);

  applyScheme(scheme);
  requestAnimationFrame(() => veil.classList.add("fade"));
  window.setTimeout(() => veil.remove(), TRANSITION_MS + 40);
}

/** 切换模式（auto → light → dark → auto），并持久化到本地缓存与配置文件。 */
export function cycleTheme(): void {
  const order: ThemeMode[] = ["auto", "light", "dark"];
  const next = order[(order.indexOf(theme.mode) + 1) % order.length];
  setMode(next);
}

export function setMode(next: ThemeMode): void {
  theme.mode = next;
  try {
    localStorage.setItem(STORAGE_KEY, next);
  } catch {
    /* 隐私模式等场景写入失败，忽略即可 */
  }
  void saveTheme(next).catch(() => {
    /* 浏览器预览时没有 IPC */
  });
  transitionTo(resolveScheme(next));
}

/** 启动时调用：与配置对齐，并监听系统主题变化。 */
export async function initTheme(): Promise<void> {
  theme.scheme = resolveScheme(theme.mode);
  applyScheme(theme.scheme);

  darkQuery?.addEventListener("change", () => {
    // 只有「跟随系统」模式才自动响应系统切换
    if (theme.mode !== "auto") return;
    transitionTo(resolveScheme("auto"));
  });

  try {
    const config = await getConfig();
    if (isMode(config.theme) && config.theme !== theme.mode) {
      // 配置文件是权威来源，localStorage 只是首帧缓存
      theme.mode = config.theme;
      try {
        localStorage.setItem(STORAGE_KEY, config.theme);
      } catch {
        /* 忽略 */
      }
      applyScheme(resolveScheme(theme.mode));
    }
  } catch {
    /* 浏览器预览时没有 IPC */
  }
}

/** 主题按钮的文案：当前状态 + 点击后的结果。 */
export function themeLabel(): string {
  const current =
    theme.mode === "auto"
      ? `跟随系统（当前${theme.scheme === "dark" ? "深色" : "浅色"}）`
      : theme.mode === "light"
        ? "浅色"
        : "深色";

  const order: ThemeMode[] = ["auto", "light", "dark"];
  const next = order[(order.indexOf(theme.mode) + 1) % order.length];
  const nextText = next === "auto" ? "跟随系统" : next === "light" ? "浅色" : "深色";
  // 跟随系统的解析结果可能与下一个固定主题同色，此时外观不会变化，注明避免误解
  const same = resolveScheme(next) === theme.scheme ? "（外观不变）" : "";

  return `主题：${current}，点击切换为${nextText}${same}`;
}

/** xterm 终端配色（CSS 变量对 canvas 无效，必须给出具体色值）。 */
export const TERMINAL_THEME: Record<Scheme, { background: string; foreground: string; cursor: string }> = {
  dark: { background: "#12161d", foreground: "#8b95a7", cursor: "#8b95a7" },
  light: { background: "#ffffff", foreground: "#24292f", cursor: "#24292f" },
};
