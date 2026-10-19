<script lang="ts">
  // 保存前差异确认。
  import { diffLines } from "diff";

  let {
    oldText,
    newText,
    onConfirm,
    onCancel,
  }: {
    oldText: string;
    newText: string;
    onConfirm: () => void;
    onCancel: () => void;
  } = $props();

  type Row = { kind: "add" | "del" | "same"; text: string };

  const rows = $derived.by((): Row[] => {
    const out: Row[] = [];
    for (const part of diffLines(oldText, newText)) {
      const kind: Row["kind"] = part.added ? "add" : part.removed ? "del" : "same";
      const lines = part.value.split("\n");
      if (lines.length > 0 && lines[lines.length - 1] === "") lines.pop();
      for (const line of lines) out.push({ kind, text: line });
    }
    return out;
  });

  const addedCount = $derived(rows.filter((r) => r.kind === "add").length);
  const removedCount = $derived(rows.filter((r) => r.kind === "del").length);
</script>

<div class="overlay">
  <div class="dialog">
    <header>
      <span class="title">确认保存改动</span>
      <span class="stat">
        <span class="add">+{addedCount}</span>
        <span class="del">-{removedCount}</span>
      </span>
    </header>

    <div class="diff">
      {#each rows as r}
        <div class="line {r.kind}">
          <span class="sign">{r.kind === "add" ? "+" : r.kind === "del" ? "-" : " "}</span>{r.text}
        </div>
      {/each}
    </div>

    <footer>
      <button class="ghost" type="button" onclick={onCancel}>取消</button>
      <button class="btn" type="button" onclick={onConfirm}>确认保存</button>
    </footer>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.55);
    z-index: 10;
  }
  .dialog {
    width: min(720px, 90vw);
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    overflow: hidden;
  }
  header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
  }
  .title {
    font-weight: 600;
    font-size: 13px;
  }
  .stat {
    margin-left: auto;
    font-family: var(--mono);
    font-size: 12px;
  }
  .stat .add {
    color: var(--brand);
  }
  .stat .del {
    color: var(--danger);
  }
  .diff {
    flex: 1;
    overflow: auto;
    padding: 8px 0;
    font-family: var(--mono);
    font-size: 12px;
    line-height: 18px;
  }
  .line {
    white-space: pre-wrap;
    word-break: break-all;
    padding: 0 16px;
    color: var(--muted);
  }
  .line .sign {
    display: inline-block;
    width: 14px;
    color: var(--faint);
  }
  .line.add {
    color: var(--brand);
    background: rgba(63, 185, 80, 0.08);
  }
  .line.del {
    color: var(--danger);
    background: rgba(255, 123, 114, 0.08);
  }
  footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid var(--border);
  }
</style>
