<script lang="ts">
  // 原始脚本编辑器（CodeMirror 6，shell 语法高亮），改动去抖上报。
  import { onDestroy, onMount } from "svelte";
  import { EditorView, basicSetup } from "codemirror";
  import { EditorState } from "@codemirror/state";
  import { StreamLanguage } from "@codemirror/language";
  import { shell } from "@codemirror/legacy-modes/mode/shell";
  import { oneDark } from "@codemirror/theme-one-dark";

  let { initial, onChange }: { initial: string; onChange: (text: string) => void } = $props();

  let host: HTMLDivElement;
  let view: EditorView | undefined;
  let timer: ReturnType<typeof setTimeout> | null = null;

  onMount(() => {
    view = new EditorView({
      state: EditorState.create({
        doc: initial,
        extensions: [
          basicSetup,
          StreamLanguage.define(shell),
          oneDark,
          EditorView.theme({
            "&": { height: "100%" },
            ".cm-scroller": { overflow: "auto" },
          }),
          EditorView.updateListener.of((update) => {
            if (!update.docChanged) return;
            const text = update.state.doc.toString();
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
    font-size: 12px;
  }
</style>
