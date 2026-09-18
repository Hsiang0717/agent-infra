# Antigravity Observational Memory (`agy-observational-memory`)

> **極簡 Skill-First 雙層記憶架構（Observations + Reflections），提供確定性 12 碼追溯、零延遲紀錄與對話內反思。**

---

## 🌟 核心理念（Core Concepts）

1. **雙層記憶結構 (Dual-Layer Representation)**：
   - **Observations（具體觀察）**：記錄帶有時戳、重要度評級（low/medium/high/critical）與 12 字元 Hex ID（例如 `[d4e5f6a1b2c3]`）的原子事件。
   - **Reflections（持久事實反思）**：自觀察提煉出的全局不變約束（技術架構、專案限制、開發偏好），存放於 `.agents/memory/reflections.json`。

2. **確定性精準溯源 (Deterministic Traceability)**：
   - 任何記憶條目均標註 12 碼 ID。
   - 執行 `node bin/om.cjs recall <id>` 精準調取原始時間戳、評級、證據鏈與完整文字。

3. **Workspace 級活躍記憶流 (Active Working Stream)**：
   - Observations 提升至 Workspace 維度共享，使用者即使常態執行 `/clear` 重置對話，新 Session 仍無縫繼承當前最新工作狀態。
   - 內建滑動視窗防護（上限 20 筆，優先保留 high/critical），防止 Context 膨脹。

4. **樂觀鎖並行控制 (Optimistic Concurrency Control)**：
   - 採用 Version 版號與自動重試合併（Retry-Merge）機制，完美支援同 Workspace 內多終端、多視窗或 Subagents 並行寫入，避免 Lost Update 與死鎖。

5. **Skill-First 極簡設計**：
   - **無 MCP Daemon**：無需啟動背景 JSON-RPC 伺服器，直接透過 CLI 執行。
   - **無多餘 Model 設計**：對話內的 Antigravity Agent 本身就是高階 LLM，直接負責反思提煉與修剪。
   - **零設定（Zero-Config）**：自動辨識 Workspace 根目錄並劃分「專案反思庫」與「Workspace 活躍記憶庫」。

---

## ⚡ Slash 指令與工作流（Slash Commands）

| 對話指令 | 對應命令列 | 說明 |
| :--- | :--- | :--- |
| `/om status` | `om status` | 檢視當前活躍 Session、觀察數量、持久反思數與儲存目錄。 |
| `/om view` | `om view` | 輸出當前活躍的折疊記憶 Markdown 投影。 |
| `/om record "<內容>" [-r <等級>]` | `om record "<內容>" -r high` | 即時記錄原子觀察（支援 low / medium / high / critical），零延遲。 |
| `/om pin "<內容>"` | `om pin "<內容>"` | 即時將全局架構或開發規範釘選至專案基準反思庫（`.agents/memory/reflections.json`）。 |
| `/om recall <id>` | `om recall <id>` | 輸入 12 碼 Hex ID，精準調出該筆記憶的完整時間與內容。 |
| `/om drop <id1> [id2 ...]` | `om drop <id1> ...` | 安全修剪或淘汰已被反思涵蓋的舊觀察。 |
| `/om compact` | *(Agent workflow)* | Agent 自主審視 `om view`，提煉出反思後調用 `om pin`，並以 `om drop` 修剪舊觀察。 |
| `/om clear` | `om clear` | 清空當前 Session 觀察（保留專案反思）。加上 `--all` 徹底清空。 |

---

## 🛠️ 極簡目錄結構

```text
agy-observational-memory/
├── bin/
│   └── om.cjs                    # 高效能獨立 CLI 執行檔（支援 status, view, record, pin, recall, drop, hook, clear）
├── hooks.json                    # Antigravity 生命週期 Hook（PreInvocation 自動瞬態記憶注入）
├── skills/
│   └── om/
│       └── SKILL.md              # /om 技能定義與 Slash 指令工作流
├── src/                          # TypeScript 原始碼
│   ├── cli.ts                    # CLI 入口與 PreInvocation Hook 處理
│   ├── config.ts                 # 零配置路徑解析
│   ├── ids.ts                    # 12-char Hex ID 生成
│   ├── storage.ts                # 儲存引擎、Atomic 寫入與投影渲染
│   ├── tokens.ts                 # Token 估算
│   └── types.ts                  # 核心介面定義
├── tests/
│   └── om.test.ts                # 單元測試套件
├── build.mjs                     # esbuild 建置腳本
├── package.json                  # 零生產依賴 (純 TS/esbuild 開發依賴)
├── plugin.json                   # 外掛清單
└── README.md
```

---

## 🧪 驗證與測試

```bash
npm test
```
