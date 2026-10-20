// 日志缓冲逻辑校验：直接跑 LogStore 的预算 / 淘汰 / 游标语义，不依赖任何测试框架。
//
// 运行：node tools/log-buffer-check.mts
// （Node 23.6+ 原生支持 TS 类型剥离，不需要先构建）

import {
  LogStore,
  newCursor,
  PER_SCRIPT_LIMIT,
  TOTAL_LIMIT,
} from "../src/lib/log-buffer.ts";

let failed = 0;
function check(name: string, ok: boolean, extra = "") {
  console.log(`  ${ok ? "ok  " : "FAIL"} ${name}${ok ? "" : `  → ${extra}`}`);
  if (!ok) failed += 1;
}

const KB = "x".repeat(1024);
const neverPinned = () => false;
const alwaysPinned = () => true;

console.log("1) 单脚本预算");
{
  const store = new LogStore();
  for (let i = 0; i < 1000; i += 1) store.append("a.cmd", KB, neverPinned);
  const { text } = store.read("a.cmd", newCursor());
  check("不超过 512KB", text.length <= PER_SCRIPT_LIMIT, `实得 ${text.length}`);
  check("至少保留一个块", text.length > 0, `实得 ${text.length}`);
}

console.log("2) 游标增量语义");
{
  const store = new LogStore();
  for (let i = 0; i < 100; i += 1) store.append("a.cmd", `块${i};`, neverPinned);
  const first = store.read("a.cmd", newCursor());
  check("首读拿到全部块", first.text.includes("块0;") && first.text.includes("块99;"));

  for (let i = 100; i < 300; i += 1) store.append("a.cmd", `块${i};`, neverPinned);
  const second = store.read("a.cmd", first.cursor);
  check(
    "二次读只拿到新增、无重复",
    !second.text.includes("块0;") && second.text.includes("块299;"),
    JSON.stringify(second.text.slice(0, 40)),
  );
}

console.log("3) 清空后游标不错位");
{
  const store = new LogStore();
  for (let i = 0; i < 10; i += 1) store.append("a.cmd", `A${i};`, neverPinned);
  const consumed = store.read("a.cmd", newCursor()).cursor;
  store.clear("a.cmd");
  store.append("a.cmd", "B0;", neverPinned);
  const { text } = store.read("a.cmd", consumed);
  check("清空后新日志可读且不重复", text === "B0;", JSON.stringify(text));
}

console.log("4) 严格全局上限（全部脚本都在运行）");
{
  const store = new LogStore();
  const scripts = Array.from({ length: 20 }, (_, i) => `s${i}.cmd`);
  for (let round = 0; round < 400; round += 1) {
    for (const s of scripts) store.append(s, KB, alwaysPinned);
  }
  let total = 0;
  for (const s of scripts) total += store.read(s, newCursor()).text.length;
  check("总量不超过 4MB", total <= TOTAL_LIMIT, `实得 ${total}`);
  check("没有把日志清空（保住最新内容）", total > 0, `实得 ${total}`);
}

console.log("5) 全局超预算时优先整桶淘汰非运行中脚本（不动运行中的）");
{
  const store = new LogStore();
  const live = (p: string) => p === "live.cmd";
  for (let i = 0; i < 500; i += 1) store.append("live.cmd", KB, live);
  const olds = Array.from({ length: 12 }, (_, i) => `old${i}.cmd`);
  for (const s of olds) {
    for (let i = 0; i < 500; i += 1) store.append(s, KB, live);
  }

  const liveLen = store.read("live.cmd", newCursor()).text.length;
  const dropped = olds.filter((s) => store.read(s, newCursor()).text.length === 0).length;
  check("运行中脚本完整保留", liveLen === 500 * 1024, `实得 ${liveLen}`);
  check("非运行中脚本有被整桶淘汰", dropped > 0, `淘汰 ${dropped} 个`);
}

console.log(failed === 0 ? "\n全部通过" : `\n${failed} 项失败`);
process.exit(failed === 0 ? 0 : 1);
