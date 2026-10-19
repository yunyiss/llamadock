// 全局说明气泡状态：同一时刻只有一个气泡，且统一渲染在页面顶层，
// 避免被父级的裁剪 / 透明度 / 堆叠上下文影响。
export interface TipState {
  id: number;
  text: string;
  anchor: HTMLElement;
}

let current = $state<TipState | null>(null);
let counter = 0;

export function nextTipId(): number {
  counter += 1;
  return counter;
}

export function tipState(): TipState | null {
  return current;
}

export function openTip(tip: TipState): void {
  current = tip;
}

export function closeTip(): void {
  current = null;
}
