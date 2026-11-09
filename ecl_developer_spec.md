# 外部語意因果層（External Cognitive Layer, ECL）工程實施與開發說明書

> **文件版本**：v1.0.0  
> **面向對象**：資深前端/後端架構師、DevOps / Platform Engineer、AI Agent 基礎設施工程師  
> **核心定位**：專屬 AI Coding Agent 的外部結構化語意層與專案長期記憶基礎設施，將 Read-time 暴力探索運算轉移至 Write-time 增量編譯，實現精準手術刀式修改與防迴歸（Zero Regression）。

---

## 1. 系統架構哲學與核心原則（Architectural Philosophy）

### 1.1 傳統 AI Coding Agent 的三大結構性瓶頸
1. **探索期雜訊過大（Discovery Noise）**：面對抽象需求，Agent 仰賴全域文字搜尋（`grep`）與盲目開檔（`read_file`）。為了修改 15 行代碼，往往讀取數千行無關檔案，導致 80% 以上 Context Window 充斥雜訊，引發嚴重的 Token 浪費與模型注意力分散（Lost in the Middle）。
2. **抽象語意脫節（Semantic Mismatch）**：人類表達採用業務與產品語彙（如「播放鍵」、「計費防連點」），程式碼存放結構符號（`PlayControl.tsx`、`ThrottleGuard`）。兩者缺乏映射層，迫使 LLM 每次都耗費計算資源進行逆向工程猜測。
3. **AST 盲點與隱性因果遺失（Invisible Invariants & Couplings）**：靜態語法樹（AST）僅能捕捉顯性呼叫，無法感知 EventBus 廣播、跨模組副作用、時序約束、硬體相容性妥協及團隊歷史血淚教訓（Lesson Learned）。Agent 重構時極易「精簡」保命防衛代碼，引發非預期迴歸錯誤。

### 1.2 核心設計原則：依賴關係分區（Causal Closure），嚴禁檔案鏡像（No File-mirroring）
* **嚴禁檔案鏡像（File-centric Mirroring is Banned）**：代碼檔案結構（`components/`、`stores/`、`services/`）是人類為了封裝語法設計的，不是按業務因果流向設計的。若每個代碼檔對應一個說明檔，Agent 面對事件連動仍需橫跳翻閱多個檔案，系統將瞬間退化為無效冗餘文件。
* **因果閉環分區（Causal Cluster）**：將「會一起發生化學反應的代碼節點」打包在同一個視圖中。Agent 只要讀取這一個區塊，即可一次性獲取完整因果反射弧。
* **雙層實體結構**：
  * **第一層（檔案系統）**：按「業務領域（Domain）」劃分檔案，全專案僅維持 5～10 個領域檔案，由 `INDEX.yaml` 集中路由。
  * **第二層（檔案內容）**：依「因果鏈叢集（Causal Clusters）」分區，每個叢集內含進入點、副作用擴散清單及共同防護欄（Rules）。

---

## 2. 資料規格定義（Data Specification）

所有語意檔案統一存放於專案根目錄 `.agent/semantics/`，採用高密度、無冗餘之 YAML 格式。

### 2.1 全域入口地圖：`INDEX.yaml`（L0 全域路由）
嚴格控制在 200 Tokens 以內，負責業務意圖的第一層快速分流。

```yaml
# .agent/semantics/INDEX.yaml
version: 1.0
domains:
  - id: player
    file: player.yaml
    scope: 串流播放、解碼核心、UI 播放/暫停控制、進度條、系統通知面板同步
  - id: billing
    file: billing.yaml
    scope: 觀看計時心跳打點、第三方金流、扣款防重防抖、額度校驗
  - id: auth
    file: auth.yaml
    scope: 登入註冊、OAuth、JWT 自動刷新攔截器、路由權限守衛
```

### 2.2 領域因果子圖：`<domain>.yaml`（L1 領域子圖）
一個領域檔包含多個獨立的業務因果叢集（Causal Cluster）：

```yaml
# .agent/semantics/player.yaml
domain: player
description: 音訊串流載入、UI 播放控制與系統媒體面板聯動

clusters:
  # -------------------------------------------------------------
  # 叢集 1：播放觸發與計費生命週期鏈（完整的因果閉環）
  # -------------------------------------------------------------
  - cluster_id: playback_toggle_flow
    summary: 處理使用者點擊播放/暫停，協調 UI 狀態、後台計費打點與 OS 通知列
    
    # 【起點】：使用者與系統交互的進入點
    entrypoint:
      id: src/components/player/PlayControl.tsx#L42-65
      name: PlayPauseButton
      anchor: "export const PlayPauseButton: React.FC"

    # 【展開鏈】：此進入點在各檔案引發的連鎖反應節點
    chain:
      - node: src/stores/playerStore.ts#L80-92
        name: togglePlay
        effect: 樂觀更新前端 isPlaying 狀態
        
      - node: services/billing/heartbeat.ts#L15-35
        name: handlePlaybackTelemetry
        effect: 攔截 UI_PLAY_TOGGLE 事件，暫停/重啟後端觀看時數打點計費
        
      - node: native/mediaSession.ts#L10-28
        name: syncOSMetadata
        effect: 同步 Windows 托盤或 macOS Control Center 播放狀態

    # 【共同防護欄 / Lesson Learned】：修改此鏈條上任何代碼不可踩的雷
    rules:
      - 按鈕必須維持 300ms 節流，防範 heartbeat 併發重複計費
      - 嚴禁在 PlayControl 直接 import AudioEngine 實例
      - Safari 下首播必須由原生 click 事件同步喚醒
```

#### 欄位定義說明
* `cluster_id`：動賓結構之唯一識別碼（如 `playback_toggle_flow`）。
* `summary`：業務層級核心意圖摘要（說明 Why，不說明語法細節）。
* `entrypoint.id`：進入點物理路徑與行號區間（格式：`path#L{start}-{end}`）。
* `entrypoint.anchor`：該區塊首行代碼特徵指紋，供 Agent 驗證行號是否飄移。
* `chain[].node`：受波及的下游代碼節點路徑與行號區間。
* `chain[].effect`：該節點承擔的具體業務副作用。
* `rules`：團隊累積的歷史踩坑經驗（Lesson Learned）、技術天條與不可變約束（Invariants）。

---

## 3. 模組一：專案初始化自動化流水線（Bootstrap Engine）

專案冷啟動時，採用 **「純程式建骨架（75%）+ 批次小模型填語意（25%）」** 機制，全自動建置 `.agent/semantics/`。

```
[原始代碼庫] ──► (1) AST 靜態掃描 ──► (2) EventBus 虛擬邊縫合 ──► (3) 骨架 YAML ──► (4) 輕量 LLM 填空 ──► 驗收落盤
```

### 3.1 初始化腳本實作：`.agent/scripts/bootstrap.py`

```python
#!/usr/bin/env python3
"""
External Cognitive Layer - Bootstrap Engine
職責：
1. 靜態 AST 掃描 Entrypoints (UI組件、API Controller)
2. 全域掃描 EventBus / Action 派發與監聽，自動串接虛擬邊
3. 輸出包含物理座標與 anchor 的骨架 YAML
4. 呼叫輕量小模型（如 Gemini Flash / Claude Haiku）補齊 summary, effect, rules
"""

import os
import re
import json
from pathlib import Path

# --- 進入點與事件特徵 ---
ENTRYPOINT_PATTERNS = [
    (r"export\s+const\s+(\w+)\s*:\s*React\.FC", "REACT_COMPONENT"),
    (r"export\s+function\s+(\w+Controller|\w+Handler)", "API_HANDLER"),
    (r"@(?:Get|Post|Put|Delete)\(['\"]([^'\"]+)['\"]\)", "NEST_ROUTER")
]

EVENT_EMIT_PATTERN = r"(?:emit|dispatch)\(['\"]([A-Z0-9_]+)['\"]"
EVENT_LISTEN_PATTERN = r"(?:on|subscribe|case)\(['\"]([A-Z0-9_]+)['\"]"

def extract_project_topology(root_dir):
    topology = {"entrypoints": [], "events": {}}
    root = Path(root_dir)
    
    for p in root.glob("**/*.[tj]s*"):
        if any(ign in str(p) for ign in ["node_modules", "dist", ".git", ".agent"]):
            continue
        rel_path = str(p.relative_to(root))
        with open(p, "r", encoding="utf-8", errors="ignore") as f:
            lines = f.readlines()
            
        for idx, line in enumerate(lines, start=1):
            # 1. 進入點識別
            for pattern, ep_type in ENTRYPOINT_PATTERNS:
                m = re.search(pattern, line)
                if m:
                    topology["entrypoints"].append({
                        "id": f"{rel_path}#L{idx}-{min(idx+35, len(lines))}",
                        "name": m.group(1),
                        "anchor": line.strip()[:65],
                        "file": rel_path,
                        "line": idx
                    })
            # 2. 廣播事件識別
            for em in re.finditer(EVENT_EMIT_PATTERN, line):
                evt = em.group(1)
                topology["events"].setdefault(evt, {"emitters": [], "listeners": []})
                topology["events"][evt]["emitters"].append(f"{rel_path}#L{idx}-{min(idx+25, len(lines))}")
            # 3. 監聽事件識別
            for ls in re.finditer(EVENT_LISTEN_PATTERN, line):
                evt = ls.group(1)
                topology["events"].setdefault(evt, {"emitters": [], "listeners": []})
                topology["events"][evt]["listeners"].append(f"{rel_path}#L{idx}-{min(idx+25, len(lines))}")
                
    return topology

def build_skeleton_clusters(topology):
    """將進入點與事件關聯，產出待填補語意的骨架結構"""
    clusters = []
    for ep in topology["entrypoints"]:
        cluster = {
            "cluster_id": f"flow_{ep['name'].lower()}",
            "summary": "[TODO_LLM_SUMMARY]",
            "entrypoint": {
                "id": ep["id"],
                "name": ep["name"],
                "anchor": ep["anchor"]
            },
            "chain": [],
            "rules": ["[TODO_LLM_RULES]"]
        }
        # 尋找與該進入點檔案有事件廣播關聯的監聽者
        for evt, links in topology["events"].items():
            if any(ep["file"] in emitter for emitter in links["emitters"]):
                for listener in links["listeners"]:
                    cluster["chain"].append({
                        "node": listener,
                        "name": evt,
                        "effect": "[TODO_LLM_EFFECT]"
                    })
        clusters.append(cluster)
    return clusters

if __name__ == "__main__":
    import sys
    proj_dir = sys.argv[1] if len(sys.argv) > 1 else "."
    topo = extract_project_topology(proj_dir)
    skeletons = build_skeleton_clusters(topo)
    print(f"掃描完成！抓出 {len(topo['entrypoints'])} 個進入點，共構建 {len(skeletons)} 個因果閉環骨架。")
```

### 3.2 批次 LLM 填補 Prompt 模板（填補 25% 語意）

```markdown
[System Instruction]
你是一位專業的代碼架構分析師。我會提供由 AST 靜態分析產出的「因果閉環骨架」以及對應的原始碼切片。
請嚴格填補以下欄位，切勿更動已存在的 id、node 與 anchor 物理座標：

1. cluster_id: 以動賓英文字串命名（如 playback_toggle_flow, checkout_submit_flow）。
2. summary: 一句話概括業務核心職責與協調的跨模組狀態（40 字以內，說明 Why 而非語法 How）。
3. chain[].effect: 該節點產生的業務副作用（如：樂觀更新 UI、暫停打點計費、同步系統面板）。
4. rules: 從代碼防衛邏輯（節流防抖、try-catch、特殊相容性判斷、註解中的 HACK）中提煉 2~3 條不可違反的架構軍規（Lesson Learned）。
```

---

## 4. 模組二：Git 提交與 CI 自動維護流水線（Continuous Maintenance）

維護機制切分為**確定性數學計算**與**語意差異蒸餾**兩軌，確保日常提交零 AI 成本。

```
git commit
    │
    ▼
[軌道 1：本地 Pre-commit Hook] (0 Token，耗時 < 50ms)
    解析 git diff hunk 首部 (@@ -L,S +L,S @@) ──► 數學校準平移 YAML 中的 #L{start}-{end}
    │
    ▼ (git push / PR 流程)
[軌道 2：CI 增量語意檢查] (僅在破壞因果時觸發小模型)
    檢查 Diff 是否增刪 EventBus 字串、新增/刪除 export 或改動公共 API？
    ├── [否（常規邏輯修復/排版/註解）] ──► CI Pass（0 消耗）
    └── [是（結構性因果變更）] ──► 呼叫輕量 LLM 增量 Patch 該特定 cluster ──► 產出 PR 建議
```

### 4.1 本地 Pre-commit Hook 腳本：`.agent/scripts/git_hook_calibrator.py`
掛載至 `.git/hooks/pre-commit`，提交時自動更新所有行號：

```python
#!/usr/bin/env python3
"""
ECL Zero-Token Line Calibrator
解析 git diff --cached 獲取位移量，原地更新 .agent/semantics/*.yaml 中的座標
"""

import subprocess
import re
from pathlib import Path

SEMANTICS_DIR = Path(".agent/semantics")
HUNK_PATTERN = r"@@ -(\d+)(?:,\d+)? \+(\d+)(?:,(\d+))? @@"

def get_staged_diff_offsets():
    diff_output = subprocess.check_output(
        ["git", "diff", "--cached", "--unified=0"], text=True
    )
    offsets = {}
    current_file = None
    
    for line in diff_output.splitlines():
        if line.startswith("+++ b/"):
            current_file = line[6:]
            offsets[current_file] = []
        elif line.startswith("@@ ") and current_file:
            m = re.search(HUNK_PATTERN, line)
            if m:
                old_start = int(m.group(1))
                new_start = int(m.group(2))
                new_len = int(m.group(3)) if m.group(3) is not None else 1
                offset = (new_start + new_len) - old_start
                offsets[current_file].append((old_start, offset))
    return offsets

def apply_offsets(offsets):
    coord_regex = r"(id|node):\s*([^\s#]+)#L(\d+)-(\d+)"
    
    for yaml_path in SEMANTICS_DIR.glob("*.yaml"):
        with open(yaml_path, "r", encoding="utf-8") as f:
            content = f.read()
            
        def replacer(match):
            key = match.group(1)
            filepath = match.group(2)
            start = int(match.group(3))
            end = int(match.group(4))
            
            if filepath in offsets:
                for change_line, offset in offsets[filepath]:
                    if start >= change_line:
                        start += offset
                        end += offset
            return f"{key}: {filepath}#L{start}-{end}"
            
        new_content = re.sub(coord_regex, replacer, content)
        if new_content != content:
            with open(yaml_path, "w", encoding="utf-8") as f:
                f.write(new_content)
            subprocess.run(["git", "add", str(yaml_path)])

if __name__ == "__main__":
    diff_offsets = get_staged_diff_offsets()
    if diff_offsets:
        apply_offsets(diff_offsets)
```

---

## 5. 模組三：Agent LLM 執行期操作手冊（Runtime Protocol）

這份指引需置於專案根目錄之 `.cursorrules`、`CLAUDE.md` 或系統提示詞中，強制約束 Coding Agent 行為：

```markdown
# 外部語意因果層（External Cognitive Layer）執行協定

你正在配置有「外部語意因果層（ECL）」的專案中工作。
【硬性約束】：嚴禁執行全專案全域文字搜尋（grep / find）或盲目開啟完整大檔案。所有代碼定位必須遵循三級漸進式協議：

### 執行協議流水線
1. [L0 路由]：收到任務後，呼叫 view_file 讀取 .agent/semantics/INDEX.yaml（~100 Tokens），比對意圖鎖定唯一目標 domain。
2. [L1 子圖]：呼叫 view_file 讀取對應的 .agent/semantics/<domain>.yaml（~150 Tokens），在 clusters 清單中鎖定符合業務目標的因果閉環，一次性獲取 entrypoint、chain 與 rules。
3. [L2 緩衝切片]：嚴禁開全檔！依據節點記載之 id: <path>#L{start}-{end} 計算安全切片：
   - start = max(1, range.start - 5)
   - count = (range.end - range.start) + 10
   - 呼叫 view_file(path, StartLine=start, EndLine=start+count-1)。
4. [錨點核對]：確認切片內包含 anchor 特徵字串。
   - 若相符：代表定位準確，直接在切片代碼上思考與編輯。
   - 若不符：僅在該單一檔案內搜尋 anchor 取得最新行號後重新切片。

### 重構天條（Invariants）
- 修改產出的代碼必須嚴格遵守該 Cluster 聲明的所有 rules。
- 涉及跨模組變更時，僅得針對 chain 中列出的受波及節點進行修改，不得任意發散至未宣告的檔案。
```

---

## 6. Lesson Learned 與專案記憶閉環（Institutional Memory Loop）

本架構是專案長期記憶的承載體。當線上發生缺陷（Bug）或踩雷時，團隊必須遵循此標準記憶沉澱流程：

```
線上發生缺陷 / 踩坑事故
          │
          ▼ 步驟 1：修復代碼並定位事故根因
工程師完成代碼修復與驗證
          │
          ▼ 步驟 2：記憶寫入（Memory Commit）
在對應的 .agent/semantics/<domain>.yaml 找到該 Cluster，將該次事故防禦規範增補至 rules：
  rules:
    - 原生點擊事件才可喚醒播放（預防 Safari 播放中斷事故 #302）
          │
          ▼ 步驟 3：全專案 Agent 共享記憶
下一次任何 Agent 觸及該功能，在 L1 階段便強制感知該條規則，永久杜絕同類錯誤重複發生。
```

---

## 7. 驗收指標與測試基線（Evaluation Baseline）

| 評估維度 | 衡量方式 | 驗收合格基準 |
|---|---|---|
| **Token 消耗效率** | 記錄探索階段 Prompt + Completion Tokens | 相比原生 grep 方式**降低 80% 以上**（單次 < 800 Tokens） |
| **定位精確度** | 首次呼叫 `view_file` 命中正確代碼區間比例 | **命中率 > 90%** |
| **防護欄遵守率** | 生成代碼是否遵守 Cluster 之 `rules` | **合規率 100%** |
| **物理行號穩定度** | 經 20 次無關代碼提交後，YAML 記載座標之有效性 | **錨點核對成功率 > 95%** |
| **維護零成本率** | 日常常規 Commit 觸發 AI Token 消耗次數 | **Token 消耗為 0**（由 Git Hook 完全接管） |
