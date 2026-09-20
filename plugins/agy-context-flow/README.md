# agy-context-flow

Pure background lifecycle hook plugin for behavioral episode tracking and context flow / file recommendation in Antigravity.

## ✨ Highlights

- **Pure Lifecycle Hooks (No Skills)**: Runs silently in the background without polluting agent system prompts or tool declarations.
- **5-Dimensional Causal Logging**: Captures `Query`, `Read Files`, `Edited Files`, `Cited Files`, and `Active Files` via `PostInvocation`.
- **Pre-Turn Context Recommendations**: Injects ephemeral hints via `PreInvocation` with confidence scoring.
- **Git & Non-Git Resilience**:
  - Seamlessly integrates with Git status, with graceful fallback to filesystem `mtime` scanning when Git is absent.
  - Strictly scoped to local project workspace (`.agents/context-flow/`) without polluting user home directories.
- **Independent Feature Toggles**: Turn recording, recommendations, or session continuity on/off independently via configuration.

## 📦 Directory Structure

```
plugins/agy-context-flow/
├── bin/
│   └── context-flow.cjs      # Single-file bundled executable
├── src/
│   ├── cli.ts                # CLI & Hook handlers
│   ├── config.ts             # Local-first config manager
│   ├── continuity.ts         # Session continuity gating
│   ├── git.ts                # Git status & FS mtime fallback
│   ├── parser.ts             # Transcript extractor
│   ├── recommender.ts        # Intent gate & multi-route scoring
│   ├── store.ts              # Local/global store with fallback
│   ├── vector.ts             # PPMI-SVD & subword semantic vector engine
│   └── types.ts              # Data contracts
├── tests/
│   └── context-flow.test.ts  # Automated unit tests
├── build.mjs                 # esbuild bundler
├── hooks.json                # Lifecycle hook configuration
├── package.json
├── plugin.json
└── tsconfig.json
```

## ⚙️ Configuration & Commands

### 1. One-Click Initialization (Config + Hooks Merging)
```bash
# Safely initializes .agents/context-flow/config.json AND merges into .agents/hooks.json
# (Preserves any existing plugin hooks in hooks.json)
node plugins/agy-context-flow/bin/context-flow.cjs init
```

### 2. Slash Command Support in Session
You can invoke the skill directly in Antigravity chat sessions:
- `/context-flow` (or `/context-flow init`, `/context-flow stats`)

### 3. View Active Configuration
```bash
node plugins/agy-context-flow/bin/context-flow.cjs config
```

### 4. Toggle Features Independently
```bash
# Disable recommendation injection (pure logging mode)
node plugins/agy-context-flow/bin/context-flow.cjs config set recommend.enabled false

# Re-enable recommendation
node plugins/agy-context-flow/bin/context-flow.cjs config set recommend.enabled true

# Adjust recommendation confidence threshold (default: 0.35)
node plugins/agy-context-flow/bin/context-flow.cjs config set recommend.threshold 0.45

# Adjust confidence format (categorical, numeric, hidden)
node plugins/agy-context-flow/bin/context-flow.cjs config set recommend.confidenceFormat numeric
```

### 5. Analytics & History
```bash
# View aggregated statistics
node plugins/agy-context-flow/bin/context-flow.cjs stats

# View recent episodes
node plugins/agy-context-flow/bin/context-flow.cjs list 10

# Test recommendation for a query
node plugins/agy-context-flow/bin/context-flow.cjs recommend "<query>"
```
