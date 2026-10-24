<script lang="ts">
  // 原始脚本编辑器（CodeMirror 6，shell 语法高亮），改动去抖上报。
  import { onDestroy, onMount } from "svelte";
  import { EditorView, basicSetup } from "codemirror";
  import { Compartment, EditorState } from "@codemirror/state";
  import { StreamLanguage } from "@codemirror/language";
  import { shell } from "@codemirror/legacy-modes/mode/shell";
  import { oneDark } from "@codemirror/theme-one-dark";
  import { theme, type Scheme } from "$lib/theme.svelte";

  let { initial, onChange }: { initial: string; onChange: (text: string) => void } = $props();

  let host: HTMLDivElement;
  let view: EditorView | undefined;
  let timer: ReturnType<typeof setTimeout> | null = null;
  // 程序化写入（外部同步）期间为 true：这不算用户编辑，不能上报 onChange
  let syncing = false;

  /**
   * 保留原文件的换行风格。
   *
   * CodeMirror 默认按 "\n" 序列化文档，CRLF 脚本（cmd / ps1 模板都是 CRLF）
   * 一进编辑器就会被改成 LF，于是「只打开原始脚本、什么都没改」也会被判成有改动，
   * 切换脚本时就会误弹「保留改动」，保存还会把行尾全改掉。
   */
  function detectLineBreak(text: string): string {
    if (text.includes("\r\n")) return "\r\n";
    if (text.includes("\r")) return "\r";
    return "\n";
  }

  // 主题用 Compartment 动态重配置：浅色去掉 oneDark 并用现有 token 着色
  const themeCompartment = new Compartment();

  const lightTheme = EditorView.theme({
    "&": { backgroundColor: "var(--surface)", color: "var(--text)" },
    ".cm-content": { caretColor: "var(--text)" },
    ".cm-cursor, .cm-dropCursor": { borderLeftColor: "var(--text)" },
    ".cm-gutters": {
      backgroundColor: "var(--surface)",
      color: "var(--faint)",
      borderRight: "1px solid var(--border)",
    },
    ".cm-activeLine": { backgroundColor: "var(--surface-2)" },
    ".cm-activeLineGutter": { backgroundColor: "var(--surface-2)" },
    ".cm-selectionBackground, &.cm-focused .cm-selectionBackground, ::selection": {
      backgroundColor: "var(--surface-3)",
    },
    ".cm-panels": { backgroundColor: "var(--surface-2)", color: "var(--text)" },
  });

  function themeExtension(scheme: Scheme) {
    return scheme === "dark" ? oneDark : lightTheme;
  }

  onMount(() => {
    view = new EditorView({
      state: EditorState.create({
        doc: initial,
        extensions: [
          basicSetup,
          StreamLanguage.define(shell),
          // 按原文件的换行风格序列化，避免 CRLF 被悄悄改成 LF
          EditorState.lineSeparator.of(detectLineBreak(initial)),
          themeCompartment.of(themeExtension(theme.scheme)),
          EditorView.theme({
            "&": { height: "100%" },
            ".cm-scroller": { overflow: "auto" },
          }),
          EditorView.updateListener.of((update) => {
            if (!update.docChanged || syncing) return;
            // sliceDoc 走 lineSeparator facet，保留 CRLF；doc.toString() 会强制用 LF
            const text = update.state.sliceDoc();
            if (timer) clearTimeout(timer);
            timer = setTimeout(() => onChange(text), 250);
          }),
        ],
      }),
      parent: host,
    });
  });

  onDestroy(() => {
    if (timer) clearTimeout(timer);
    view?.destroy();
  });

  // 外部文本变化（切换脚本、或参数改动重建草稿）时同步到编辑器
  $effect(() => {
    const next = initial;
    if (!view) return;
    // 用 sliceDoc 比较：它按原换行风格输出，不会把 CRLF 看成「内容变了」
    const current = view.state.sliceDoc();
    if (current === next) return;
    syncing = true;
    try {
      view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: next } });
    } finally {
      syncing = false;
    }
  });

  // 主题切换时重配置编辑器配色
  $effect(() => {
    const scheme = theme.scheme;
    if (!view) return;
    view.dispatch({ effects: themeCompartment.reconfigure(themeExtension(scheme)) });
  });
</script>

<div class="editor" bind:this={host}></div>

<style>
  .editor {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
  .editor :global(.cm-editor) {
    height: 100%;
    font-size: 13px;
  }
</style>
