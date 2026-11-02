# Epistemic Memory System (`.agents/memory/`)

> **跨工作階段的實證經驗沉澱錨點（Project Epistemic Memory）。**

---

## 🌟 定位與精神

1. **資訊高密度、零 Token 膨脹（Zero-Token-Bloat）**：
   - 拒絕冗長的會話總結與廢話紀錄。
   - 每筆紀錄為 1~3 行的超高密度精煉條目（Micro-card），格式為：
     `[領域/標籤]：非顯而易見的坑點與根本原因 ➔ 實證修復方案或不變量 (日期)`。
2. **只記錄「實證排除的非顯性陷阱」**：
   - 一般語法錯誤或已知標準 API 不記錄。
   - 專門記錄：環境怪異行為（Win32 編碼、Non-TTY 阻塞）、深層時序競爭、非顯性架構依賴。
3. **按需檢索（On-Demand Grounding）**：
   - 平時**不主動全量塞入 Prompt Context**。
   - Agent 在 Phase 1（探測階段）排查疑難雜症時，若發現相似症狀，以 `rg <關鍵字> .agents/memory/` 精準探測。

---

## 📂 目錄結構

```text
.agents/memory/
├── README.md        # 本說明文檔
└── pitfalls.md      # 實證陷阱與邊界避坑快照
```
