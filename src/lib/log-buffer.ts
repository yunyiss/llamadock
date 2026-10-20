// 日志缓冲：按脚本分桶、以「块数组」存储，带单脚本 / 全局字符预算与 LRU 淘汰。
//
// 为什么不用一个大字符串：每次追加都做「老串 + 新块」会整体复制，超过上限截断时
// 还要再复制一次——40ms 一批就相当于每秒数 MB 垃圾。块数组把追加变成 O(1) 的 push。
//
// 本模块刻意不依赖 Svelte（便于将来单独测试）；响应式触发由 store 的信号负责。

/** 单脚本字符预算 */
export const PER_SCRIPT_LIMIT = 512 * 1024;
/** 全局字符预算 */
export const TOTAL_LIMIT = 4 * 1024 * 1024;

interface Bucket {
  /** 保存的文本块（按时间顺序） */
  chunks: string[];
  chars: number;
  /** 已从头部淘汰的块数：游标用绝对块序号，靠它换算成当前数组下标 */
  dropped: number;
  /** 最后一次写入时间（全局 LRU 依据） */
  at: number;
}

/** 消费游标：下一个待读取的绝对块序号。 */
export interface LogCursor {
  chunk: number;
}

export function newCursor(): LogCursor {
  return { chunk: 0 };
}

export class LogStore {
  private buckets = new Map<string, Bucket>();
  private totalChars = 0;

  /** 追加一段日志；`pinned` 返回 true 的脚本在全局超预算时不会被淘汰。 */
  append(path: string, text: string, pinned: (path: string) => boolean): void {
    if (!text) return;

    let bucket = this.buckets.get(path);
    if (!bucket) {
      bucket = { chunks: [], chars: 0, dropped: 0, at: 0 };
      this.buckets.set(path, bucket);
    }
    bucket.chunks.push(text);
    bucket.chars += text.length;
    bucket.at = Date.now();
    this.totalChars += text.length;

    // 单脚本超预算：从最旧的块开始整块丢弃（不做整串复制）
    while (bucket.chars > PER_SCRIPT_LIMIT && bucket.chunks.length > 1) {
      const head = bucket.chunks.shift()!;
      bucket.chars -= head.length;
      bucket.dropped += 1;
      this.totalChars -= head.length;
    }

    if (this.totalChars > TOTAL_LIMIT) this.evictByLru(pinned);
  }

  /** 读取自游标以来尚未消费的日志，返回新游标。 */
  read(path: string, cursor: LogCursor): { text: string; cursor: LogCursor } {
    const bucket = this.buckets.get(path);
    if (!bucket) return { text: "", cursor: newCursor() };

    // 游标早于现存最旧块（中间那段已被淘汰）：从最旧块继续，不重复也不错位
    const start = Math.max(0, cursor.chunk - bucket.dropped);
    const end = bucket.dropped + bucket.chunks.length;
    if (start >= bucket.chunks.length) return { text: "", cursor: { chunk: end } };

    let text = "";
    for (let i = start; i < bucket.chunks.length; i += 1) text += bucket.chunks[i];
    return { text, cursor: { chunk: end } };
  }

  /**
   * 清空某个脚本的日志。
   *
   * 保留桶并累加 `dropped`，让清空之后新日志的绝对块序号继续前进，
   * 这样终端里已经推进的游标不会错位。
   */
  clear(path: string): void {
    const bucket = this.buckets.get(path);
    if (!bucket) return;
    this.totalChars -= bucket.chars;
    bucket.dropped += bucket.chunks.length;
    bucket.chunks = [];
    bucket.chars = 0;
    bucket.at = Date.now();
  }

  /** 彻底丢弃某个脚本的日志桶（脚本被删除 / 切换工作目录时调用）。 */
  drop(path: string): void {
    const bucket = this.buckets.get(path);
    if (!bucket) return;
    this.totalChars -= bucket.chars;
    this.buckets.delete(path);
  }

  /** 全局超预算：按最久未写入的顺序整桶淘汰（运行中 / 选中脚本除外）。 */
  private evictByLru(pinned: (path: string) => boolean): void {
    const victims = [...this.buckets.entries()]
      .filter(([path]) => !pinned(path))
      .sort((a, b) => a[1].at - b[1].at);
    for (const [path] of victims) {
      if (this.totalChars <= TOTAL_LIMIT) break;
      this.drop(path);
    }
  }
}
