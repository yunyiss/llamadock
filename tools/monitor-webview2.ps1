<#
  采样 LlamaDock 的 WebView2 renderer 私有内存，输出趋势与斜率。

  用法（在仓库根目录）：
    pwsh -File tools/monitor-webview2.ps1 -Minutes 10 -IntervalSeconds 10

  判读方式：只看「是否持续爬升」——平均每点增量接近 0 即合格。
  注意：V8 不会立刻把释放的内存还给系统，绝对值不下降是正常的，别因此误判。
#>
param([int]$Minutes = 10, [int]$IntervalSeconds = 10)

function Get-DockRenderer {
  Get-CimInstance Win32_Process -Filter "Name='msedgewebview2.exe'" -ErrorAction SilentlyContinue |
    Where-Object { $_.CommandLine -like "*--type=renderer*" -and $_.CommandLine -match "llamadock" } |
    Select-Object -First 1
}

$samples = [System.Collections.Generic.List[double]]::new()
$deadline = (Get-Date).AddMinutes($Minutes)

while ((Get-Date) -lt $deadline) {
  $proc = Get-DockRenderer
  if (-not $proc) {
    Write-Host "未找到 LlamaDock 的 renderer（应用没开？）"
    Start-Sleep -Seconds $IntervalSeconds
    continue
  }
  $p = Get-Process -Id $proc.ProcessId -ErrorAction SilentlyContinue
  if ($p) {
    $mb = [math]::Round($p.PrivateMemorySize64 / 1MB, 1)
    $samples.Add($mb)
    Write-Host ("{0}  {1,8:N1} MB  (pid {2})" -f (Get-Date -Format "HH:mm:ss"), $mb, $proc.ProcessId)
  }
  Start-Sleep -Seconds $IntervalSeconds
}

if ($samples.Count -lt 3) {
  Write-Host "采样点不足，无法评估趋势"
  exit 1
}

$first = $samples[0]
$last = $samples[$samples.Count - 1]
$min = ($samples | Measure-Object -Minimum).Minimum
$max = ($samples | Measure-Object -Maximum).Maximum
$slope = [math]::Round(($last - $first) / ($samples.Count - 1), 2)

Write-Host ""
Write-Host ("采样 {0} 点 | min {1:N1} MB | max {2:N1} MB | 首 {3:N1} → 末 {4:N1}" -f `
    $samples.Count, $min, $max, $first, $last)
Write-Host ("平均每点增量 {0:N2} MB（接近 0 表示已稳定）" -f $slope)

if ([math]::Abs($slope) -le 0.5) {
  Write-Host "结论：内存稳定"
} else {
  Write-Host "结论：仍在爬升"
  exit 2
}
