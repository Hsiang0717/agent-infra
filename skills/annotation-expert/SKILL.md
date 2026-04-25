---
name: annotation-expert
description: 建立並執行「LLM/Agent 優先」的跨語言註解規範。可用於分析專案環境、動態生成標註協定，並自動化治理、校對或重構註解以提升 AI 推理能力。
---

# Skill: annotation-expert

## Instructions

### 1. 環境偵測 (Discovery)
- 優先掃描 `package.json`, `pyproject.toml`, `go.mod`, `Cargo.toml` 等配置文件。
- 識別專案語言與現有的 Linting 規則。

### 2. 核心哲學：LLM-First
註解應被視為「結構化數據 (RAG-Friendly)」：
- **Agent Logic (High Priority)**: 標註 **Side Effects** (`@modifies`, `@calls`), **Preconditions**, **Invariants**。
- **LSP Precision**: 使用 `{@link}` 與 `@see` 建立語意鏈結。
- **Human Semantics (Low Priority)**: 解釋「Why」動機，避免解釋「How」實作。

### 3. 三大維度標註 (3-Pillar Protocol)
1. **Identity (身分)**: 使用 `@module` 定義模組在架構中的定位。
2. **Contract (契約)**: 明確 `@param`, `@returns`, `@throws` 語意。
3. **Context (脈絡)**: 紀錄隱性知識與跨層連動 (Cross-Reference)。

### 4. 自動化治理流程 (Governance)
- **Audit Mode**: 掃描檔案並識別缺失的標註標籤，產出 「LLM-Readability Report」。
- **Refactor Mode**: 使用內建工具自動增強註解：
    - **Step 1**: 執行 `node annotation-expert/tools/auto-annotate.js <file>`。
    - **Step 2**: 腳本會自動補全 `@referenced_by`（引用來源）。
    - **Step 3**: 手動校對 `@modifies` 與 `@calls` 等邏輯標記。

### 5. 治理工具 (Governance Tools)
本技能內建一個多語言自動標註工具，支援 TS/JS, Python, Go：
- **位置**: `annotation-expert/tools/auto-annotate.js`
- **能力**: 
    - 全域搜尋引用來源並標註 `@referenced_by`。
- **用法**: `node annotation-expert/tools/auto-annotate.js [file_path]`

### 6. 最佳實踐參考 (Best Practices)
偵測環境後，讀取對應語言的具體標註格式：
- **TypeScript/JS**: [references/ts.md](references/ts.md)
- **Python**: [references/py.md](references/py.md)
- **Go**: [references/go.md](references/go.md)

### 6. 記憶鎖定 (Memory)
1. 產生專案專屬註解範本。
2. 擬定 `project memory` Fact 字串並詢問使用者是否儲存。

### 7. 提示詞匯出 (Prompt Export / AI-Context Integration)
當專案開始或需要同步規範給其他 AI 時，應生成一份「AI 註解指令集 (AI Annotation Rule Set)」供使用者寫入專案根目錄的指令檔案 (如 `gemini.md`, `agent.md`, `claude.md`)：

- **指令集內容必須包含**：
    - **Protocol Label**: 聲明使用 `LLM-First Annotation Protocol`。
    - **Pillar Definitions**: 簡述 Identity, Contract, Context 的要求。
    - **Forbidden Patterns**: 禁止無意義的冗餘註解 (如解釋代碼如何實作)。
    - **Example Block**: 提供一個該語言的最簡標籤範例。
- **格式化建議**：
    - 使用 `<annotation_rules>` 標籤封裝，以便 AI 精確識別。

## Success Criteria
- 註解內容能讓 LSP 彈窗顯示完整開發上下文。
- 生成的「AI 指令集」能讓其他 AI 助手在無需人工干預下直接正確標註程式碼。
- 代碼對未來接手的 Code Agent 具有 100% 的邏輯透明度。
