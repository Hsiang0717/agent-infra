# agy-enhancer (原生 Rust Agent 執行期增強核心)

為 Google Antigravity CLI 打造的高效能原生 Runtime Enhancer Core，以純 Rust 打造，在每次 Agent 推論前（`PreInvocation`）與工具調用前（`PreToolUse`）以 < 1ms 的極速提供主動治理、感知增強與自愈修復。

---

## 核心四大增強模組

1. **Context Posture Governance（工作記憶體姿態調度）**：
   * 透過 Hook `stdin` 傳入的 `transcriptPath`，以反向區塊 Seek（`SeekFrom::Start`）只掃描檔案末尾。
   * 解析出最新一輪 `PLANNER_RESPONSE` 的權威計數：`input_tokens` 與 `cache_read_tokens`。
   * 依據額度與比例動態注入三級語意姿態提示（`ephemeralMessage`）：
     - **NOMINAL**（正常）：`[Context Posture: NOMINAL]` 保持全速精準自主推論。
     - **TIGHT**（緊繃）：`[Context Posture: TIGHT]` 提示縮窄讀取切片，將發散研究委派給 `research` subagent。
     - **CRITICAL**（臨界）：`[Context Posture: CRITICAL]` 提示停止盲目探索，收斂進行中編輯並準備 `/handoff` 或 `/compact`。
   * **零數字干擾**：計算全留在 Rust 底層，絕不向 LLM 洩漏數字雜訊。
2. **Epistemic Memory Radar（專案實證記憶雷達）**：
   * 自動探測當前 Workspace 是否包含 `.agents/memory/`（如 `pitfalls.md`）。
   * 實時喚醒 Agent 記憶 awareness：
     - 若偵測到條目：注入 `[Project Memory: active (.agents/memory/pitfalls.md - N entries; query via rg)]`，引導 Agent 查閱避坑。
     - 若目錄存在但尚未沉澱：注入 `[Project Memory: ready (...; distill non-obvious traps)]`，提醒在 Phase 4 沉澱關鍵避坑。
     - 若專案未啟用 memory 目錄則保持完全靜默（Zero Noise）。
3. **Target Invariant Detector（代碼排版不變量感知）**：
   * 當前一步驟執行了 `read_file`，Hook 自動分析讀取切片中的微觀排版特徵：
     - **非典型縮排**：探測奇數空格（如 3 格、5 格）或 Tab 縮排，防止模型慣性套用 4 格縮排。
     - **跳脫字元**：探測程式碼中的雙反斜線（`\\` 如 LaTeX 或正則），提醒模型在 JSON 工具呼叫中加倍跳脫（`\\\\`）。
     - **行尾空白**：探測肉眼不可見的行尾空白，避免 `edit_file` 比對失敗。
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
    "nominal": "[Context Posture: NOMINAL]",
    "tight": "[Context Posture: TIGHT] Scope read_file directly to target edit sites; verify & converge.",
    "critical": "[Context Posture: CRITICAL] Halt new exploration; finish active atomic edit_file, verify, and invoke /compact || /handoff."
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
