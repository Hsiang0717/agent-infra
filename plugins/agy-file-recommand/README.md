# agy-file-recommand

Pure background lifecycle hook plugin for behavioral episode tracking and context file recommendation in Antigravity.

## ✨ Highlights

- **Pure Lifecycle Hooks (No Skills)**: Runs silently in the background without polluting agent system prompts or tool declarations.
- **5-Dimensional Causal Logging**: Captures `Query`, `Read Files`, `Edited Files`, `Cited Files`, and `Active Files` via `PostInvocation`.
- **Pre-Turn Context Recommendations**: Injects ephemeral hints via `PreInvocation` with confidence scoring.
- **No-Git & No-Workspace Resilience**:
  - Automatically falls back to filesystem `mtime` scanning when Git is absent.
  - Automatically falls back from local `.agents/file-recommand/` to global `~/.gemini/file-recommand/` if workspace is missing.
- **Independent Feature Toggles**: Turn recording or recommendations on/off independently via local/global configuration.

## 📦 Directory Structure

```
plugins/agy-file-recommand/
├── bin/
│   └── recommand.cjs         # Single-file bundled executable
├── src/
│   ├── cli.ts                # CLI & Hook handlers
│   ├── config.ts             # Local-first config manager
│   ├── git.ts                # Git status & FS mtime fallback
│   ├── parser.ts             # Transcript extractor
│   ├── recommender.ts        # Intent gate & multi-route scoring
│   ├── store.ts              # Local/global store with fallback
│   └── types.ts              # Data contracts
├── tests/
│   └── recommand.test.ts     # Automated unit tests
├── build.mjs                 # esbuild bundler
├── hooks.json                # Lifecycle hook configuration
├── package.json
├── plugin.json
└── tsconfig.json
```

## ⚙️ Configuration & Commands

### 1. View Active Configuration
```bash
node bin/recommand.cjs config
```

### 2. Toggle Features Independently
```bash
# Disable recommendation injection (pure logging mode)
node bin/recommand.cjs config set recommend.enabled false

# Re-enable recommendation
node bin/recommand.cjs config set recommend.enabled true

# Adjust recommendation confidence threshold (default: 0.3)
node bin/recommand.cjs config set recommend.threshold 0.45

# Apply globally across all projects
node bin/recommand.cjs config set recommend.enabled false --global
```

### 3. Analytics & History
```bash
# View aggregated statistics
node bin/recommand.cjs stats

# View recent episodes
node bin/recommand.cjs list 10

# Test recommendation for a query
node bin/recommand.cjs recommend "<query>"
```
