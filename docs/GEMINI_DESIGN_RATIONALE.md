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

### 3. 「跨模型注意力落差與 CLI Schema 漂移」的校準 (Flash Attention & CLI Schema Alignment)
- **歷史教訓**：高階推理模型（如 Gemini 4）具備隱式語意橋接能力，能自動將舊版 IDE 工具名稱（`view_file`、`replace_file_content`、`write_to_file`）映射至 Antigravity CLI 宣告的原生工具（`read_file`、`edit_file`、`write_file`）。然而輕量高速模型（如 Gemini 3.8 Flash）高度依賴 Literal Token Matching 與首位規則錨定（Primacy Bias）：
  1. 當規則中未明確攔截 `Get-ChildItem`，Flash 首輪預設執行 `Get-ChildItem -Recurse | Select-Object FullName, Length`，導致長路徑觸發 PowerShell 表格寬度截斷（將 `13165` bytes 截斷顯示為 `…65`），進而誤判大檔體積並跳過 `outline` 與切片讀取。
  2. 在 CLI 非互動式 `pwsh` 管道中，若執行 `rg -n <pattern>` 未帶 `<path>` 參數，`rg` 會因偵測到非 TTY stdin 而無限掛起等待標準輸入。
- **演進決策**：
  - **全面統整為 CLI 原生工具名**：環境已全面收斂至純 `agy cli`，協議與外掛中全面廢除舊版 IDE 工具名稱（`view_file`、`replace_file_content`、`write_to_file`），100% 統一為 CLI 宣告的原生工具（`read_file`、`edit_file`、`write_file`），徹底消除別名干擾。
  - **漏斗探索首位錨定**：於 `rules/cli_toolchain.md` 第一條明確將 `lsj [path]` / `topj` 置於目錄探索首位，並將 `Get-ChildItem`、`dir`、`ls`、`tree`、`Select-String` 列入明確禁令；同時規範 `rg -n <pattern> <path>` 必填搜尋路徑以杜絕非 TTY 掛起。
  - **編輯前 CRLF 物理正規化與切片完整性**：針對 Windows 下外部生成的 CRLF (`\r\n`) 檔案導致 `edit_file` 匹配失敗，於 `agy-enhancer` 掛載 `PreToolUse` (`edit_file`) 自動正規化與自愈修復 Hook，並於 `GEMINI.md` 強調 `Target` 必須取自單一連續 `read_file` 切片（嚴禁由搜尋片段拼接）。

### 4. 「字串替換鐘擺陷阱」的校準 (The Edit Pendulum: Oversized Target vs. Global Mass-Corruption)
- **歷史教訓**：模型在遭遇 `edit_file` 比對失敗時，常陷入雙重極端：
  1. *極端 A（貪多大包）*：企圖替換包含 15~20 行未經驗證的程式碼區塊，只要空格、引號或換行微幅不吻合即報錯；甚至跨越 `read_file` 切片邊界憑空拼接未讀行。
  2. *極端 B（過激無錨）*：失敗後矯枉過正，改以極短、無特徵的通用語句（如純縮排空白 + `errors.append(`）作 Target。因 `edit_file` 具備「全文所有匹配項全數替換」的合約特性，引爆全檔 7 處合法程式碼被錯誤凸排，衍生 10 個語法災難。
- **演進決策**：確立 **「寬讀窄錨（Wide-Read, Micro-Anchor）」**。讀取 15~20 行掌握上下文全貌，但 `Target` 必須且僅包含具備結構唯一性的控制語句（如特定 `elif <cond>:`），嚴禁泛用無錨替換，且嚴格鎖定於最新切片邊界內。

### 5. 「自動化 TTY 終端死鎖」與版控演進 (Headless Rebase vs. Soft-Reset Consolidation)
- **歷史教訓**：在版控整頓時，直覺呼叫 `git rebase -i origin/main` 試圖 squash 零散 commit。但在 Agent 無人值守的背景非交互式 session 中，Git 強制呼叫文字編輯器等待用戶輸入，導致 subshell 永久死鎖掛起。
- **演進決策**：確立 **「Soft-Reset Consolidation（`git reset --soft origin/main`）」** 作為 Agent 原生的 Squash 平替方案，配合 `main` 主幹保護與自適應風格探測，兼顧線性歷史與無人值守穩定性。

### 6. 「記憶膨脹與跨工作階段經驗沉澱」的架構平衡 (Epistemic Memory & Context Firewall)
- **歷史教訓**：過往試圖將過多工作總結灌入全域 Prompt，導致 Context Window 嚴重膨脹；但完全無狀態又造成相同環境陷阱重複踩坑。
- **演進決策**：實施 **「實證陷阱快照庫（`.agents/memory/pitfalls.md`）」** 與 **「Context 防火牆」**：
  1. 每條踩坑經驗嚴格限制在 3 行以內高密度條目，平時不塞進 Prompt，僅在排查疑難時以 `rg` 實證探測。
  2. 當系統處於 `TIGHT` 姿態時，將大範圍探測或日誌審計外包給 `research` subagent，依據嚴格輸出合約僅傳回結論座標，保證主工作記憶體整潔。

### 7. 「吸管視野與長對話收斂緊箍咒」的校準 (Causal-Chain Completeness vs. Myopic Slicing)
- **歷史教訓**：在複雜非同步或狀態機專案（如 `rs-yt-player`）的長對話除錯中，輕量模型（Flash）高度服從機械式約束（如「每次只讀 30–80 行」與 `TIGHT` 姿態下的「直接鎖定目標修改點並收斂」）。這導致兩個嚴重副作用：
  1. *吸管視野（Straw-View Myopia）*：為湊足 30–80 行限制而將完整函式或狀態流轉腰斬，只見報錯點（Symptom）不見上下游因果鏈（Causal Chain）。
  2. *長對話收斂陷阱（Premature Convergence under TIGHT）*：當對話超過 60% 觸發 `TIGHT` 時，舊版提示命令模型「只讀目標修改點並收斂」，使 Flash 在修改失敗後不敢跨檔案追查上游根因，陷入在同一處反覆微調（Local Ping-Pong）的死胡同。
- **演進決策**：將 Argon 等級的「系統因果推理」顯性化植入協議：
  1. **完整邏輯單元切片（Logical-Unit Slicing）**：大檔（`>300` 行）仍以 `outline` 先行，但切片大小以**完整涵蓋整個邏輯單元（函式、狀態處理器、結構體實作，通常 40–120 行；`<300` 行小檔可直接讀取）**為準，嚴禁腰斬控制流。
  2. **因果鏈追蹤優先（Causal-Chain Tracing: Symptom ≠ Root Cause）**：動刀修 Bug 前，必須先確認 `Upstream Producer / Caller ➔ State / Channel / Lifecycle ➔ Downstream Consumer / Callback` 的完整流向，確認狀態在何處被初始化、變更或非同步覆寫。
  3. **2-Strike 第一性原理重置（First-Principles Reset）與 `TIGHT` 解綁**：將 `TIGHT` 定義為「避免無腦全檔傾倒（善用 `outline`/`rg` 與 `research` subagent），但絕不為省 Token 犧牲根因追查」；一旦連續 1–2 次局部修改未解決問題，強制停止在原處刮痧，先質疑既有假設（是否有其他執行緒/回呼/生命週期覆寫）並擴大上下游排查半徑。

### 8. 「從 3.8 Flash 到 4 Hydrogen」的認知與編排校準 (Reasoning-Model Calibration)
- **歷史教訓**：針對 3.8 Flash 設計的協議側重補償「思考不足與字面匹配缺陷」（逐行步驟、編輯防呆範例、硬性行數數字與單一案例補丁）。當主力模型升級至高推理模型（Gemini 4 Hydrogen）後，失效模式反轉為「過度思考、過度工程（順手重構無關程式碼）、過度編排開銷，以及為錯誤因果假設編寫複雜合理化補丁」；同時 `agy-enhancer` 的 `200k` 固定 `CRITICAL` 門檻在較長對話中過早催促收尾。
- **演進決策**：
  1. **拔除機械拐杖，保留物理不變量**：精簡已被 `sanitizer.rs` / `invariants.rs` 執行期吸收的排版教學與重複切片數字，僅保留真正無法靠推理繞過的物理契約（`edit_file` 全文多處替換風險、`rg` 必帶路徑、非交互式 Git 紀律）。
  2. **比例原則與 Diff 最小化**：導入「快慢路徑分流（Proportionality）」（單模組小改直達外科修改，跨模組/有狀態變更才啟用完整 `Where/Who/When` 拓樸與 `[ECL Shield]` 檢查）以及「改動行可追溯至任務（Diff Minimality）」護欄。
  3. **假說證偽與編排經濟學**：將 `2-Strike` 明確定義為「證偽因果心理模型（Falsification）」，並補上 `Direct` vs. `invoke_subagent` vs. `swarm` 的最小編排開銷準則。
  4. **模型感知 Posture（`model_profiles`）**：啟用 `agy-enhancer` `HookInput.modelName` 比對，針對 Hydrogen 提供較寬鬆的注意力預算（`300k` / `0.65` / `0.88`）並移除催促提前收斂的字眼，同時保留零噪音與 Context Firewall。

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
| **修改檔案反覆 Read-Edit 或全域誤傷** | 改一行 -> 讀全檔 -> 報錯 -> 又讀全檔；或使用無特徵短字串引發全域替換連鎖災難 | **寬讀窄錨與微錨點**：讀足閉合上下文但 Target 僅取最小唯一特徵行；嚴禁無特徵泛用語句；切片外嚴禁憑空拼接。 |
| **主題分支零散 Commit 需整頓** | 在背景直覺呼叫 `git rebase -i` 導致終端因 `$EDITOR` 永久掛死 | **非交互式軟重置（Soft-Reset）**：以 `git reset --soft origin/main` 將改動整頓回暫存區，重新執行原子化 Conventional Commits。 |
| **長對話多輪未修好** | 陷入沉沒成本，在錯誤的局部反覆打補丁 | **意圖校準（Intent Recovery）**：停止局部修補，跳回初心（Root Intent），坦誠診斷架構瓶頸並提出升級方案。 |
| **結構化資料解析與測試處理** | 偷懶手刻脆弱正則偽解析，或對特定測試特例硬編碼（Overfitting） | **Parser-First 與通用邏輯**：結構化資料（HTML/JSON/AST/代碼）嚴格使用標準 Parser；解決根本問題而非特例硬編碼。 |

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
