// 重型组件的懒加载入口。
//
// CodeMirror 只在用户切到「原始脚本」时才用得到，但它是启动包里体积最大的依赖之一，
// 所以这里延迟到首次需要时才 import；模块系统会缓存，重复调用只解析一次。
import type { Component } from "svelte";

type RawEditorProps = {
  initial: string;
  onChange: (text: string) => void;
};

let rawEditor: Promise<Component<RawEditorProps>> | null = null;

/** 懒加载原始脚本编辑器（CodeMirror）。 */
export function loadRawEditor(): Promise<Component<RawEditorProps>> {
  rawEditor ??= import("$lib/components/RawEditor.svelte").then(
    (m) => m.default as Component<RawEditorProps>,
  );
  return rawEditor;
}
