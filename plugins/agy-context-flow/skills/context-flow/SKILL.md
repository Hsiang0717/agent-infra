---
name: context-flow
description: >-
  Inspect, initialize, test, and manage the autonomous context flow engine,
  learning episodes, relevance stats, and configuration for agy-context-flow.
---

# Context Flow (`agy-context-flow`) Skill

This skill allows you and the user to initialize, configure, test, and monitor the autonomous context flow and file recommendation engine.

---

## ⚡ Available Commands & Actions

| Action | Command | Purpose |
| :--- | :--- | :--- |
| **Initialize Workspace** | `node plugins/agy-context-flow/bin/context-flow.cjs init` | Safely creates `.agents/context-flow/config.json` and registers hooks in `.agents/hooks.json` (preserving other plugins). |
| **View Active Config** | `node plugins/agy-context-flow/bin/context-flow.cjs config` | Displays active recording, recommendation threshold, and confidence format. |
| **View Learning Stats** | `node plugins/agy-context-flow/bin/context-flow.cjs stats` | Displays episode count, top read files, top edited files, and top cited files. |
| **List Recent Episodes** | `node plugins/agy-context-flow/bin/context-flow.cjs list 10` | Shows the last 10 recorded episodes with query and touched files. |
| **Simulate Recommendation** | `node plugins/agy-context-flow/bin/context-flow.cjs recommend "<query>"` | Simulates context recommendation scoring for any prompt. |
| **Toggle Recommendation** | `node plugins/agy-context-flow/bin/context-flow.cjs config set recommend.enabled <true\|false>` | Turns recommendation prompt injection on/off. |
| **Toggle Episode Recording** | `node plugins/agy-context-flow/bin/context-flow.cjs config set record.enabled <true\|false>` | Turns turn recording on/off. |
| **Adjust Recommendation Threshold** | `node plugins/agy-context-flow/bin/context-flow.cjs config set recommend.threshold <0.0-1.0>` | Fine-tunes recommendation strictness (default: 0.35). |
| **Adjust Confidence Format** | `node plugins/agy-context-flow/bin/context-flow.cjs config set recommend.confidenceFormat <categorical\|numeric\|hidden>` | Formats confidence tags in injected context hints. |
| **Toggle Session Continuity** | `node plugins/agy-context-flow/bin/context-flow.cjs config set continuity.enabled <true\|false>` | Enables/disables seamless session handover on the first turn of a new session. |
| **Set Max Recent Sessions** | `node plugins/agy-context-flow/bin/context-flow.cjs config set continuity.maxSessions <number>` | Adjusts how many distinct recent sessions to keep in LRU buffer (default: 5). |
| **View Recent Sessions** | `node plugins/agy-context-flow/bin/context-flow.cjs sessions` | Displays recent session ID snapshots and their last queries. |
| **Set Custom Message** | `node plugins/agy-context-flow/bin/context-flow.cjs message "<text>"` | Configures a persistent custom message/instructions injected on every turn. |
| **View Custom Message** | `node plugins/agy-context-flow/bin/context-flow.cjs message` | Displays the currently configured custom message. |
| **Clear Custom Message** | `node plugins/agy-context-flow/bin/context-flow.cjs message clear` | Removes `.agents/context-flow/custom_message.txt`. |

---

## 🛠️ Usage Guidelines

1. **When User asks to Initialize (`/context-flow init`)**:
   Run `node plugins/agy-context-flow/bin/context-flow.cjs init` to set up both config and hooks.

2. **When User asks for Stats or Recommendations (`/context-flow stats`)**:
   Run `node plugins/agy-context-flow/bin/context-flow.cjs stats` and present the top correlated files and learning progress.

3. **When User asks to set or view custom message**:
   Use `node plugins/agy-context-flow/bin/context-flow.cjs message "<text>"` or `node plugins/agy-context-flow/bin/context-flow.cjs message`.

4. **When User asks to tune sensitivity or enable/disable**:
   Use `node plugins/agy-context-flow/bin/context-flow.cjs config set <key> <value>`.
