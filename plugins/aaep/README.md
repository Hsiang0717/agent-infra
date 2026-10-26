# AAEP Plugin (`plugins/aaep`)

> **Autonomous Agent Engineering Protocol (AAEP) Operational Calibration & Protocol Reminder Skill.**

本外掛專為 Antigravity (AGY) Agent 與開發者設計，無任何外部二進位工具或額外環境相依，純粹以高資訊密度的 Prompt 實現 Agent 核心工程協議的即時喚醒與自我校準。

---

## 核心理念與價值

當對話歷程增長、工作記憶收緊，或任務出現多輪迭代時，LLM 容易產生以下典型漂移行為：
1. **憑空猜測**：跳過實際讀檔/檢索，直接推論不存在的 API 或目錄結構。
2. **過度設計（YAGNI 違規）**：加入未被要求的抽象層、通用封裝或過度防禦代碼。
3. **粗暴重寫**：不使用外科手術式的精確替換，而是大範圍整檔重寫導致遺失現有邏輯或註解。
4. **合成模擬假象**：僅憑 Mock 或推測宣告完成，未在真實終端執行實證命令與檢查 Exit Code。
5. **死胡同迴圈（Micro-patching Loop）**：針對同一錯誤反覆嘗試盲目包裝 `try/catch` 或重複的 null-check。

`/aaep` 提供即時的心智重錨（Cognitive Re-anchoring），讓 Agent 在任何階段一鍵校準至「**自主掌舵、崇尚實證、極致克制**」四大生命週期。

---

## 指令與使用情境

### 觸發方式：
```text
/aaep
```
或附帶目前碰到的困境或任務上下文：
```text
/aaep Refactor in phase 3 encountered type friction, recalibrating posture.
```

### 內建模擬自檢清單（Pre-Action Gate）：
* **[Grounding]**：我當前的推論是否有真實程式碼/終端輸出作為依據？還是正在盲猜？
* **[Scope / YAGNI]**：此變更是否為使用者所要求的最小解？是否有未經要求的過度封裝？
* **[Surgical]**：我是否採用了 `edit_file` 搭配 Unique Anchor Window？是否避免了無謂的 `write_file` 整檔覆寫與改後重讀（`read_file`）？
* **[Verification]**：我是否已透過實際命令真實執行並檢驗 Exit Code 與 stderr？
* **[Posture]**：當前 Context 姿態（NOMINAL / TIGHT / CRITICAL）為何？我是否已收斂探索範圍？

---

## 一鍵安裝至全域

透過專案隨附的安裝腳本，將本 Plugin 部署至 Antigravity 全域外掛目錄（`~/.gemini/config/plugins/aaep`）：

```powershell
# 模式 A: 建立 Junction 連結（開發推薦：即時同步，修改本倉庫檔案全域立即生效）
pwsh -NoProfile -ExecutionPolicy Bypass -File .\plugins\aaep\install.ps1

# 模式 B: 實體複製模式（分發給非開發環境）
pwsh -NoProfile -ExecutionPolicy Bypass -File .\plugins\aaep\install.ps1 -Copy

# 解除安裝
pwsh -NoProfile -ExecutionPolicy Bypass -File .\plugins\aaep\install.ps1 -Uninstall
```

---

## 外掛目錄架構

```text
plugins/aaep/
├── README.md               # 外掛說明文件
├── plugin.json             # Antigravity 外掛清單宣告
├── install.ps1             # 一鍵部署/連結腳本
└── skills/
    └── aaep/
        └── SKILL.md        # /aaep 核心協議提醒與心智自檢 Prompt
```
