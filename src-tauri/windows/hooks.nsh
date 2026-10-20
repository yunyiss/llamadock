; Tauri NSIS 卸载钩子。
;
; WebView2（Chromium 内核）会在 %LOCALAPPDATA%\<bundle identifier> 下创建自己的数据目录
; （EBWebView：GPUCache / Code Cache / Local Storage 等，实测 100MB+）。它不是应用代码写的，
; 但卸载时应一并清理，避免用户目录残留 —— 这正是「开箱即用、卸载无残留」要处理的唯一一处。
;
; 注意：若将来修改 tauri.conf.json 的 identifier，这里必须同步修改。

!macro NSIS_HOOK_POSTUNINSTALL
  ; 先尝试常规清理；被占用而删不掉的会在重启后由系统回收，不影响后续安装
  RMDir /r "$LOCALAPPDATA\com.wbai.llamadock"
!macroend

; 某些模板在卸载前调用，一并定义（两个宏都是幂等的 RMDir /r）
!macro NSIS_HOOK_PREUNINSTALL
  RMDir /r "$LOCALAPPDATA\com.wbai.llamadock"
!macroend
