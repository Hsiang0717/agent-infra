# agy-context-monitor (原生 Rust Context 監控 Hook)

為 Google Antigravity CLI 打造的極致輕量原生 Context 監控 Hook，以純 Rust 打造，在每次 Agent 推論前（`PreInvocation`）以 < 1ms 的極速探測當前 Session 的真實 Token 吞吐，並依據上限（256k tokens）注入暫態語意狀態提示（`ephemeralMessage`）。

---

## 核心設計亮點

1. **極速探測（Zero-Overhead Session Probe）**：
   * 透過 Hook `stdin` 傳入的 `transcriptPath`，以反向區塊 Seek（`SeekFrom::Start(chunk_start)`）只掃描檔案末尾。
   * 解析出最新一輪 `PLANNER_RESPONSE` 的權威計數：`input_tokens` 與 `cache_read_tokens`。
   * 執行耗時 < 1ms，二進位檔體積僅 ~220KB，記憶體佔用近乎為零。
2. **固定 256k 額度與三級語意約束**：
   * 上限固定為 `256,000` tokens。
   * **NOMINAL**（用量 < 60%）：廣播 `[Context Posture: NOMINAL] Working memory is abundant. Proceed with full surgical precision and end-to-end autonomy.`
   * **TIGHT**（用量 60% ~ 90%）：廣播 `[Context Posture: TIGHT] Working memory is tightening. Constrain read slices and offload expansive exploration to the research subagent.`
   * **CRITICAL**（用量 >= 90%）：廣播 `[Context Posture: CRITICAL] Working memory near ceiling. Cease broad exploration, converge on active atomic changes, and prepare /handoff.`
   * **零數字干擾**：所有 256k 與百分比計算留在底層，絕不向 LLM 洩漏數字雜訊，落實原則驅動。
3. **完全遵循 Antigravity Hook 協議**：
   * 透過標準輸出輸出 `{"injectSteps": [{"ephemeralMessage": "..."}]}`。
   * 不寫入持久歷史，僅作為模型每次推論前的引導參照。

---

## 建置與安裝

### 1. 本地編譯與部署

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File .\plugins\agy-context-monitor\install.ps1 -Build
```

### 2. 全域安裝（套用至所有 Antigravity 專案）

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File .\plugins\agy-context-monitor\install.ps1 -Global
```

---

## 自訂配置 (`config.json`)

Hook 支援熱讀取配置，修改後下一輪對話立即生效，無需重新編譯：

* **本機配置**：`plugins/agy-context-monitor/config.json`
* **全域配置**：`~/.gemini/config/plugins/agy-context-monitor/config.json`
* **專案覆寫**：`.agents/context-monitor.json`

```json
{
  "context_limit": 256000,
  "thresholds": {
    "tight": 0.60,
    "critical": 0.90
  },
  "messages": {
    "nominal": "[Context Posture: NOMINAL] Working memory is abundant. Proceed with full surgical precision and end-to-end autonomy.",
    "tight": "[Context Posture: TIGHT] Working memory is tightening. Constrain read slices and offload expansive exploration to the research subagent.",
    "critical": "[Context Posture: CRITICAL] Working memory near ceiling. Cease broad exploration, converge on active atomic changes, and prepare /handoff."
  }
}
```

### 支援之動態模板變數（選用）
若希望在自訂語意中帶入實體數值，可自由在 messages 中使用下列變數（若未填入則為純文字輸出，零數字干擾）：
* `{tier}`：當前狀態名稱（如 `NOMINAL`, `TIGHT`, `CRITICAL`）
* `{pct}`：已使用百分比（如 `42.5%`）
* `{used}`：格式化已用 Token（如 `108.8K`）
* `{limit}`：格式化上限 Token（如 `256.0K`）
* `{remain}`：格式化剩餘 Token（如 `147.2K`）

---

## 命令列手動診斷

```powershell
# 檢視幫助
.\plugins\agy-context-monitor\bin\agy-context-monitor.exe --help

# 檢視當前生效的配置內容與路徑
.\plugins\agy-context-monitor\bin\agy-context-monitor.exe --config

# 直接探測指定對話 transcript 檔案
.\plugins\agy-context-monitor\bin\agy-context-monitor.exe --probe "C:\path\to\transcript.jsonl"

# 模擬不同 Token 用量下的狀態語意輸出
.\plugins\agy-context-monitor\bin\agy-context-monitor.exe --simulate 50000
.\plugins\agy-context-monitor\bin\agy-context-monitor.exe --simulate 180000
.\plugins\agy-context-monitor\bin\agy-context-monitor.exe --simulate 240000

# 模擬 Hook stdin 管道呼叫
'{"transcriptPath": "C:\\path\\to\\transcript.jsonl"}' | .\plugins\agy-context-monitor\bin\agy-context-monitor.exe
```
