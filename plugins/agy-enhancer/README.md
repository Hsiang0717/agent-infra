# agy-enhancer (Agent 執行期增強旗艦套件 / AAEP Runtime Suite)

為 Google Antigravity CLI 打造的頂級旗艦增強套件，集結 **AAEP 自主工程協議大腦**、**Agent-Native CLI 工具鏈規則（含自愈安裝）** 與 **< 1ms Rust 原生神經中樞**。一鍵安裝即可讓 Agent 具備動態姿態調度、排版不變量感知、專案實證記憶雷達、工具調用自愈修復與環境自動配置能力。

---

## 套件結構組成 (Components)

* **Rules (2 組內建硬規則)**：
  * `rules/AAEP.md`：**自主工程代理協議 (AAEP)** 四階段生命週期（事實查證、先抽後建、外科編輯、實證閉環與記憶蒸餾）。
  * `rules/CLI_TOOLCHAIN.md`：**Windows pwsh 工具鏈與自愈安裝協議**（優先使用 `lsj` / `topj` / `outline`，遭遇未安裝時主動執行環境自愈安裝，絕不低效降級）。
* **Hooks (1 組原生 Rust 二進位中樞)**：
  * `PreInvocation`：反向解析 Transcript 注入 `[Context Posture]`、`[Target Invariant]` 與 `[Project Memory]`。
  * `PreToolUse` (`edit_file`)：CRLF 實體正規化、行號前綴過濾、空行容錯與 `overwrite` 參數修復。

---

## 核心四大增強模組

1. **Context Posture Governance（200k 零噪音工作記憶體調度與近因重錨定）**：
   * 透過 Hook `stdin` 傳入的 `transcriptPath`，以反向區塊 Seek（`SeekFrom::Start`）只掃描檔案末尾。
   * 解析出最新一輪 `PLANNER_RESPONSE` 的權威計數：`input_tokens` 與 `cache_read_tokens`。
   * 以 **200k tokens** 為最佳注意力基準，動態執行「平時靜默 ＋ 長文本近因重錨定（Recency Re-Anchoring）」：
     - **NOMINAL**（`< 60%` / `< 120k`）：**完全靜默（Zero Noise）**，不輸出任何冗餘標籤，避免日常回合產生注意力疲勞。
     - **TIGHT**（`60% ~ 85%` / `120k ~ 170k`）：注入 `[Context Posture: TIGHT]` 近因重錨定，提醒使用 `outline`/`rg` 精準切片、將跨檔檢索委派給 `research` subagent，並在修復失敗時追查跨模組因果鏈而非重複局部微調。
     - **CRITICAL**（`>= 85%` / `>= 170k`）：注入 `[Context Posture: CRITICAL]` 啟動嚴格 Context Firewall，將剩餘探索、Log 排查或獨立子任務全面委派給 `research`/`self` subagent，主對話僅執行 `rg` 錨點編輯並直接完成驗證閉環（絕不催促使用者 `/compact` 或 `/handoff`）。
   * **零數字干擾**：計算全留在 Rust 底層，絕不向 LLM 洩漏數字雜訊。
2. **Epistemic Memory Radar（專案實證記憶雷達 — 64KB 視窗冷卻防刷屏）**：
   * 自動探測當前 Workspace 是否包含 `.agents/memory/`（如 `pitfalls.md`）。
   * 採 **Session 開局喚醒 ＋ 64KB 滾動視窗去重**（避免每輪工具呼叫重複刷屏造成橫幅麻痺）：
     - 若偵測到條目且近期 64KB Transcript 內尚未提醒：注入 `[Project Memory: active (.agents/memory/pitfalls.md - N entries; query via rg)]`。
     - 若目錄存在但尚未沉澱：注入 `[Project Memory: ready (...; distill non-obvious traps)]`。
     - 若專案未啟用 memory 目錄或近期已提醒過，則保持完全靜默（Zero Noise）。
3. **Target Invariant Detector（多檔平行讀取支援與排版不變量感知）**：
   * 當前一回合執行了 `read_file`（支援同一回合平行讀取多個檔案合併掃描），Hook 自動分析讀取切片中的微觀排版特徵：
     - **非典型縮排**：探測奇數空格（如 3 格、5 格，自動排除 ` * ` 多行註解星號行）或 Tab 縮排，防止模型慣性套用 4 格縮排。
     - **跳脫字元**：探測程式碼中的雙反斜線（`\\` 如 LaTeX 或正則），提醒模型在 JSON 工具呼叫中加倍跳脫（`\\\\`）。
     - （註：行尾空白已交由 `PreToolUse` 無感自愈對齊，不再拋出警告消耗模型注意力。）
   * 輸出如 `[Target Invariant: 3-space indent; \\ escape; match verbatim]`，達成零失誤編輯。
4. **PreToolUse Active Interception & Self-Healing（編輯攔截與自愈修復）**：
   * 掛載於 `PreToolUse` (`edit_file`)，主動執行安全檢查：
     - **CRLF 物理正規化**：將目標檔案統一把關為乾淨換行。
     - **行號前綴自動剝除**：自動清除模型意外複製的 `12: ` 檢視行號。
     - **空白行容錯比對**：消除空行差異導致的比對失敗。
     - **參數重寫覆蓋（`overwrite`）**：直接在工具調用前修正 Target 與 Replacement，拯救潛在失敗。

---

## 建置與安裝

### 1. 本地編譯與部署

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File .\plugins\agy-enhancer\install.ps1 -Build
```

### 2. 全域安裝（套用至所有 Antigravity 專案）

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File .\plugins\agy-enhancer\install.ps1 -Global
```

---

## 自訂配置 (`config.json`)

支援熱讀取配置，修改後下一輪對話立即生效，無需重新編譯：

* **本機配置**：`plugins/agy-enhancer/config.json`
* **全域配置**：`~/.gemini/config/plugins/agy-enhancer/config.json`
* **專案覆寫**：`.agents/enhancer.json`

```json
{
  "context_limit": 200000,
  "thresholds": {
    "tight": 0.60,
    "critical": 0.85
  },
  "messages": {
    "nominal": "",
    "tight": "[Context Posture: TIGHT] Avoid blind full-file dumps (use outline/rg; offload broad cross-file audits to research subagent). If a fix failed or spans modules, do NOT repeat local tweaks—trace paired channels & align Where/Who/When first.",
    "critical": "[Context Posture: CRITICAL] Working memory near ceiling. Offload remaining discovery, log forensics, or isolated subtasks to research/self subagents; keep primary reads strictly rg-anchored, suppress verbose CLI output, and converge directly on verified completion."
  },
  "memory_radar": {
    "enabled": true
  }
}
```

---

## 命令列手動診斷

```powershell
# 檢視幫助
.\plugins\agy-enhancer\bin\agy-enhancer.exe --help

# 檢視當前載入之配置
.\plugins\agy-enhancer\bin\agy-enhancer.exe --config

# 探測 Transcript 與代碼不變量
.\plugins\agy-enhancer\bin\agy-enhancer.exe --probe "C:\path\to\transcript.jsonl"

# 探測專案 Memory 狀態
.\plugins\agy-enhancer\bin\agy-enhancer.exe --memory "."

# 模擬 Token 水位姿態
.\plugins\agy-enhancer\bin\agy-enhancer.exe --simulate 50000
.\plugins\agy-enhancer\bin\agy-enhancer.exe --simulate 180000
.\plugins\agy-enhancer\bin\agy-enhancer.exe --simulate 240000
```
