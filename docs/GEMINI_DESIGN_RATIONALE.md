# GEMINI.md 設計哲學與架構演進史 (Architecture & Design Rationale)

本文檔記錄 [agents/GEMINI.md](agents/GEMINI.md) 的核心設計哲學、演化歷程與邊界決策。**未來任何針對該協議的迭代，皆應立足於此核心理念進行延續與微調，避免重新退化回過度防禦的「拘束器模型」。**

---

## 🧭 一、核心工程哲學 (Core Philosophy)

> **Autonomous, Empirical, Minimal.**
> （自主掌舵、崇尚實證、極致克制）

Antigravity (AGY) 是定位於 **Staff Software Engineer（資深主任軟體工程師）** 的自主 Pair Programmer。
這份協議的目的**不是當防呆的手銬**，而是建立一套**高階工程師的心智模型（Cognitive Lifecycle）**：

1. **擁有問題（Own the Problem End-to-End）**：
   - 杜絕「每走一步都要問主人的實習生行為」。
   - 高置信度下主動排查、主動驗證、端到端閉環；唯有面對真實的架構分歧或破壞性決策時，才精準對齊。
2. **實證高於臆測（Ground Truth over Simulation）**：
   - 不憑記憶盲猜代碼結構或第三方套件版本。
   - 拒絕「只跑 Mock 通過就自欺欺人宣布完成」，必須在終端看到真實命令的 stdout/stderr 與執行成效。
3. **上下文經濟學（Context & Token Economics）**：
   - 珍視 Working Memory。嚴禁粗暴全檔讀取或在同一檔案裡無腦 Read-Edit 乒乓循環。
   - 大檔綱要先行、外科手術切片、子代理充當 Context 防火牆。

---

## 🔄 二、歷史演進與血淚教訓 (Evolutionary Lessons)

在 Git 歷程中（從 `1501191`、`6ce52f6` 到 `8667af3`），GEMINI.md 曾經歷以下演變與陷阱：

### 1. 「基準測試過擬合」的陷阱 (Overfitting to Benchmarks)
- **歷史教訓**：在過往版本中，為了通過特定的 Benchmark 或消除 Linter findings，協議被塞滿了大量死板的閥值與長括號例外（例如 `>300 lines / >15KB`、`>10 Results Threshold`、`*(Exemptions: 1... 2... 3...)`）。
- **帶來的後遺症**：否定句與懲罰語態過多（`STRICTLY FORBIDDEN`, `HALT`），引發了模型的「風險規避退縮（Risk Aversion）」，導致模型注意力全花在「檢查自己有沒有違規」，失去了大局觀與主動推進的魄力。
- **演進決策**：**拔除評測補丁，改為原則驅動（Principle-Driven）**。將死板數字昇華為認知感知（例如雜訊感知、大檔預警雷達）。

### 2. 「失敗兩次強制停機」的盲點 (Failure Budget Calibration)
- **歷史教訓**：原版規定「任何修復連續失敗 2 次立即強制停手 (HALT)」，但在真實環境中，正常的日誌探測與環境變數排查經常需要 2~3 次交互，硬性計數器會「誤殺」正常的探索流程。
- **演進決策**：改為 **「假說升級原則（Hypothesis Escalation）」**。
  - 核心精神不變：**禁止在同一個點同維度盲目刮痧微調（No Repetitive Micro-Patching）**。
  - 但給予升級路徑：一旦初步假設無效，強制跳脫局部視角，審查環境變數、構建配置、上下游調用等系統邊界。

---

## 🧩 三、四大階段生命週期與邊界決策 (The 4-Phase Lifecycle)

協議採用符合現代 LLM 思維鏈（Chain-of-Thought）的時序架構：

```
Phase 1: Grounding & Discovery       --> 事實查證、大綱切片、漏斗縮圈
Phase 2: Deliberate Design           --> YAGNI 克制、置信度推進、先抽後建
Phase 3: Surgical Implementation     --> 外科手術替換、錨定視窗、同輪原子變更
Phase 4: Empirical Verification      --> 真機運作、多模態視覺確認、假說升級
```

### 關鍵情境對應策略 (Stress Scenarios & Playbook)：

| 情境 / 挑戰 | 過去的錯誤行為 (Anti-Pattern) | 現在的標準行為 (Target Pattern) |
| :--- | :--- | :--- |
| **模糊需求 (如「清單有 Bug」)** | 連代碼都沒看就彈出問卷打斷人類 | 先看 Outline、切片探測與跑日誌。高信心直接修好；業務分歧時帶著模組選項精準發問。 |
| **1000+ 行單檔加新功能** | 盲目在千行檔案裡硬塞複雜狀態，代碼越寫越爛 | **先抽後建（Extract-Before-Extend）**：先將獨立職責抽成純函式/子模組，再乾淨實作，節省 Token 並保障品質。 |
| **UI / 畫面佈局有問題** | 盯著 CSS/DOM 瞎猜，改完憑空想像修好 | **多模態閉環**：調用 `screen-capture` 截圖並查看，對照畫面除錯，修復後再截圖當交件證據。 |
| **天馬行空的想法討論** | 把探討當成需求未指明而報警，或直接亂改代碼 | **架構師辯證（Yes, and...）**：純文字探討利弊與隱性代價，守住零寫入邊界，提供低成本沙盒 PoC 選項。 |
| **修改檔案反覆 Read-Edit** | 改一行 -> 讀全檔 -> 報錯 -> 又讀全檔（Context 暴增） | **低開銷修改**：錨定視窗看足閉合上下文、改完禁止冗餘重讀、依賴編譯器錯誤坐標精確微調。 |
| **長對話多輪未修好** | 陷入沉沒成本，在錯誤的局部反覆打補丁 | **意圖校準（Intent Recovery）**：停止局部修補，跳回初心（Root Intent），坦誠診斷架構瓶頸並提出升級方案。 |

---

## 🏛️ 四、系統架構分離原則 (Separation of Concerns)

專案嚴格遵循「硬環境歸 Rules，心智模型歸 Agent」的分離原則：

1. **`rules/cli_toolchain.md` (全域硬規則，`trigger: always_on`)**：
   - 定義 Windows `pwsh` 環境規範。
   - 強制推行跨平台 Agent-Native 二進制工具（`rg`, `fd`, `jq`）與 PowerShell Profile 壓縮輔助函式（`lsj`, `outline`）。
2. **`agents/GEMINI.md` (Agent 專屬心智綱領)**：
   - 專注於 **AAEP 工程推演協議**，不重複寫落落長的環境設定，專注於端到端解決問題的認知與架構素養。

---

> 💡 **維護提醒**：未來若發現 Agent 在特定情境出現摩擦，**請優先調整 Phase 內的權衡語意或補充目標模式（Target Pattern），切勿重新倒退回引入禁止性硬計數器或審判式字眼。**
