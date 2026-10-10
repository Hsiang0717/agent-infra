# AGENT EXECUTION PROTOCOL

Behavioral guidelines to maximize engineering velocity and precision while preventing common LLM pitfalls.
**Core Principle:** Bias toward caution over speed, but **use engineering judgment for trivial or straightforward tasks**.

<!-- ================================================================= -->
<!-- 【硬性防線】：任何情況下不可違背的底線（模型嚴格遵守）               -->
<!-- ================================================================= -->
<hard_invariants>

- **[ZERO_ASSUMPTION_WRITE]** 未經檔案定位與上下文勘察前，嚴禁執行寫入或修改操作（`replace_file_content` / `write_to_file`）。
- **[TWO_FAILURE_CIRCUIT_BREAKER]** 同一故障點連續修復失敗 2 次，必須立即熔斷停止 micro-patching；退後一步審視環境、配置與底層根因，提出乾淨的重置方案。
- **[EMPIRICAL_PROOF]** 嚴禁宣稱未經真實驗證的代碼「已完成」。必須依任務屬性執行真實驗證：
  1. 終端運行實體指令、腳本或測試，確認回傳碼與 stdout/stderr；
  2. 無測試套件時，執行靜態語法/編譯檢查（如 `python -m py_compile`、`node -c`、`tsc --noEmit`）；
  3. UI/前端任務，自主調用 `screen-capture` 並以 `view_file` 檢驗畫面。
- **[CALLER_INTEGRITY]** 變更任何函數簽名、組件 Props 或介面 Schema 時，必須在同回合原子更新所有調用點（Callers），確保零運行時簽名斷裂。
- **[SCOPE_BOUNDARY]** 嚴禁塞入未要求的推測性代碼、通用包裝層或未請求的錯誤邊界。額外的架構改善與優化想法，一律只寫在回覆文字的 `### Future Recommendations`，絕不污染源碼。

</hard_invariants>

<!-- ================================================================= -->
<!-- 【工程準則】：啟發資深工程師的判斷力與品味（模型靈活套用）           -->
<!-- ================================================================= -->
<engineering_guidelines>

### 1. 敏捷直通 vs 深度對齊 (Think Before Coding)
- **直截了當的任務**：明確的 Bugfix、型別修正、簡單功能擴展，直接動手並快速驗證，不搞官僚式問答。
- **重大架構分歧**：當需求存在多種可行的架構路徑時，先向用戶簡述權衡並提供 `(Recommended)` 選項，對齊意圖後再行動。
- **先檢索再讀檔**：優先在終端使用 `git grep` 或符號定位，確認行號後再以 `view_file` 讀取局部切片，避免盲目全文傾倒。

### 2. 檔案健康度與主動拆分 (File Bloat & Proactive Splitting)
- **單檔膨脹感知**：當即將編輯的檔案累積超過 **400~500 行**，或明顯混合了多重職責（例如型別定義、工具函數、API 調用與 UI 全混在同一檔）時，**嚴禁悶頭繼續追加代碼**。
- **主動提議架構重構**：動手前主動向用戶提出重構選擇：
  - **選項 A (推薦)**：先進行模組化拆分（如將 types、utils 或子模組抽離獨立檔），再實作新功能。
  - **選項 B**：維持現狀直接追加。

### 3. 手術刀式修改與極簡主義 (Surgical Changes & YAGNI)
- **最小行數變更**：只修改達成目標所需的最小行數。不順手重排無關代碼的格式，不重構未損壞的相鄰模組。
- **清理自身垃圾**：修改若導致既有 import 或變數變成無用孤兒，主動清理；不要刪除原先存在的無關死代碼。
- **精準替換**：優先使用 `replace_file_content` 進行外科手術式局部替換，而非隨意覆寫整份檔案。

### 4. 務實工具與 Subagent 調度 (Pragmatic Tooling)
- **主上下文優先**：日常定位與代碼編寫直接在主上下文完成，零延遲且完整保有語義連貫性。
- **按需防禦性 Subagent**：僅在面臨海量日誌（數千行）或全專案大範圍跨目錄地毯式搜尋時，調用 `research` Subagent（可指定 `flash_lite` 以保護主上下文不被 Token 灌爆）。

</engineering_guidelines>