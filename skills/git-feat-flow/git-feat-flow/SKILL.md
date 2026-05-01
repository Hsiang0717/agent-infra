---
name: git-feat-flow
description: 專為 Feature Branch 設計的 Git 工作流。支援按主題自動分批提交 (Conventional Commits) 以及具備衝突分析建議的智能 Rebase 管理。
---

# Git Feature Flow

本 Skill 旨在提升 Git 提交的專業度與 Rebase 的安全性。它將 Agent 轉化為您的 Git 諮詢顧問。

## 核心功能

### 1. 顆粒化提交 (Granular Commits)
當您完成一段開發任務，使用此功能來分批提交。

**流程：**
1. **掃描變更**：分析 `git status` 與 `git diff`。
2. **主題分類**：
   - `agents/` -> `feat(agents)` 或 `refactor(agents)`
   - `extensions/` -> `feat(ext)` 或 `fix(ext)`
   - `docs/` -> `docs`
   - `.gitignore`, `package.json` -> `chore`
3. **混合檔案偵測**：若單一檔案（如 `README.md`）跨主題，主動詢問：
   - 「偵測到混合變更，是否需要我進行區塊分析 (git add -p)？」
4. **生成建議**：為每個批次生成符合慣例的提交訊息。

### 2. 智能同步 (Safe Rebase)
當需要將 `main` 的最新變更合併進來時。

**流程：**
1. **預檢**：確認當前分支非 `main`，且工作區已清理 (Stashed/Committed)。
2. **執行**：啟動 `git rebase main`。
3. **衝突處理模式**：
   - **自動分析**：若發生衝突，提取衝突區塊。
   - **提案 (選項 B)**：分析上下文，給出「建議修復版本」。
   - **引導 (選項 A)**：若建議不適用，轉向手動模式，等待使用者修改後執行 `rebase --continue`。

## 指令參考

- **"幫我分批提交此次變更"**：啟動顆粒化提交邏輯。
- **"同步 main 的變更"**：啟動 Rebase 同步邏輯。
- **"處理當前衝突"**：在 Rebase 中斷時呼叫，啟動衝突分析建議。

## 提交規範參考 (Conventional Commits)
- `feat`: 新功能
- `fix`: 修補錯誤
- `docs`: 只有文件變更
- `style`: 不影響程式碼含義的變更 (空格, 格式, 漏掉的分號等)
- `refactor`: 重構（既不是修復錯誤也不是添加功能）
- `perf`: 提高性能的變更
- `test`: 添加遺漏的測試或更正現有測試
- `chore`: 構建過程或輔助工具的變更
