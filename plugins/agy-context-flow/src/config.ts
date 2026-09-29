import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";
import { ConfidenceFormat, ConfidenceTiers, ContinuityConfig } from "./types.js";

export interface ContextFlowConfig {
  record: {
    enabled: boolean;
  };
  recommend: {
    enabled: boolean;
    threshold: number;
    maxItems: number;
    confidenceFormat?: ConfidenceFormat;
    confidenceTiers?: Partial<ConfidenceTiers>;
  };
  continuity?: ContinuityConfig;
}

export type RecommandConfig = ContextFlowConfig;

export const DEFAULT_CONFIDENCE_TIERS: ConfidenceTiers = {
  high: 0.75,
  medium: 0.5,
  low: 0.35,
};

export const DEFAULT_CONTINUITY_CONFIG: ContinuityConfig = {
  enabled: true,
  maxSessions: 5,
};

export const DEFAULT_CONFIG: ContextFlowConfig = {
  record: {
    enabled: true,
  },
  recommend: {
    enabled: true,
    threshold: 0.35,
    maxItems: 3,
    confidenceFormat: "categorical",
    confidenceTiers: { ...DEFAULT_CONFIDENCE_TIERS },
  },
  continuity: { ...DEFAULT_CONTINUITY_CONFIG },
};


export function formatConfidence(
  score: number,
  format: ConfidenceFormat = "categorical",
  customTiers?: Partial<ConfidenceTiers>
): string | null {
  if (format === "hidden") return null;
  if (format === "numeric") return score.toFixed(2);

  const tiers: ConfidenceTiers = {
    high: customTiers?.high ?? DEFAULT_CONFIDENCE_TIERS.high,
    medium: customTiers?.medium ?? DEFAULT_CONFIDENCE_TIERS.medium,
    low: customTiers?.low ?? DEFAULT_CONFIDENCE_TIERS.low,
  };

  if (score >= tiers.high) return "HIGH";
  if (score >= tiers.medium) return "MEDIUM";
  if (score >= tiers.low) return "LOW";
  return "LOW";
}

export function findWorkspaceRoot(startDir: string = process.cwd()): string | undefined {
  let current = path.resolve(startDir);
  const home = path.resolve(os.homedir());

  while (true) {
    if (current.toLowerCase() !== home.toLowerCase()) {
      if (path.basename(current) === ".agents") {
        return path.dirname(current);
      }
      if (
        fs.existsSync(path.join(current, ".git")) ||
        fs.existsSync(path.join(current, ".agents"))
      ) {
        return current;
      }
    }
    const parent = path.dirname(current);
    if (parent === current) break;
    current = parent;
  }
  return undefined;
}

export function getConfigPath(workspaceRoot?: string): string | null {
  const ws = workspaceRoot ? path.resolve(workspaceRoot) : findWorkspaceRoot();
  if (!ws || !fs.existsSync(ws)) return null;
  return path.join(ws, ".agents", "context-flow", "config.json");
}

export function isPluginEnabled(workspaceRoot?: string): boolean {
  const configPath = getConfigPath(workspaceRoot);
  return configPath !== null && fs.existsSync(configPath);
}

export function loadConfig(workspaceRoot?: string): ContextFlowConfig | null {
  const configPath = getConfigPath(workspaceRoot);
  if (!configPath || !fs.existsSync(configPath)) {
    return null;
  }

  try {
    const raw = JSON.parse(fs.readFileSync(configPath, "utf8"));
    return {
      record: { ...DEFAULT_CONFIG.record, ...(raw.record || {}) },
      recommend: {
        ...DEFAULT_CONFIG.recommend,
        ...(raw.recommend || {}),
        confidenceTiers: {
          ...DEFAULT_CONFIG.recommend.confidenceTiers,
          ...(raw.recommend?.confidenceTiers || {}),
        },
      },
      continuity: {
        ...DEFAULT_CONFIG.continuity!,
        ...(raw.continuity || {}),
      },
    };
  } catch {
    return { ...DEFAULT_CONFIG };
  }
}

export function saveConfig(config: ContextFlowConfig, workspaceRoot?: string): boolean {
  const ws = workspaceRoot ? path.resolve(workspaceRoot) : findWorkspaceRoot();
  if (!ws || !fs.existsSync(ws)) return false;

  const localDir = path.join(ws, ".agents", "context-flow");
  if (!fs.existsSync(localDir)) {
    fs.mkdirSync(localDir, { recursive: true });
  }
  const targetFile = path.join(localDir, "config.json");
  fs.writeFileSync(targetFile, JSON.stringify(config, null, 2), "utf8");
  return true;
}

export interface InitResult {
  configCreated: boolean;
  hooksUpdated: boolean;
  snippetsCreated: boolean;
  hooksPath: string;
  configPath: string;
  snippetsPath: string;
}

export function initWorkspace(workspaceRoot?: string, customScriptRelPath?: string): InitResult {
  const ws = workspaceRoot ? path.resolve(workspaceRoot) : (findWorkspaceRoot() || process.cwd());
  
  // 1. Config initialization & Schema Synchronization
  const configAlreadyExists = isPluginEnabled(ws);
  if (!configAlreadyExists) {
    saveConfig(DEFAULT_CONFIG, ws);
  } else {
    // Merge existing user config with latest schema defaults to ensure all fields are present
    const existing = loadConfig(ws);
    if (existing) {
      saveConfig(existing, ws);
    }
  }
  const configPath = getConfigPath(ws) || path.join(ws, ".agents", "context-flow", "config.json");

  // 2. Safe Hooks.json merge
  const agentsDir = path.join(ws, ".agents");
  if (!fs.existsSync(agentsDir)) {
    fs.mkdirSync(agentsDir, { recursive: true });
  }

  const hooksPath = path.join(agentsDir, "hooks.json");
  let existingHooks: Record<string, any> = {};

  if (fs.existsSync(hooksPath)) {
    try {
      const content = fs.readFileSync(hooksPath, "utf8").trim();
      if (content) {
        existingHooks = JSON.parse(content);
      }
    } catch {
      existingHooks = {};
    }
  }

  // Clean up legacy hook key if present
  if (existingHooks["agy-file-recommand-hooks"]) {
    delete existingHooks["agy-file-recommand-hooks"];
  }

  const scriptPath = customScriptRelPath || "plugins/agy-context-flow/bin/context-flow.cjs";
  existingHooks["agy-context-flow-hooks"] = {
    PreInvocation: [
      {
        type: "command",
        command: `node ${scriptPath} hook pre-invocation`,
        timeout: 5,
      },
    ],
    PostInvocation: [
      {
        type: "command",
        command: `node ${scriptPath} hook post-invocation`,
        timeout: 10,
      },
    ],
    Stop: [
      {
        type: "command",
        command: `node ${scriptPath} hook stop`,
        timeout: 10,
      },
    ],
  };

  fs.writeFileSync(hooksPath, JSON.stringify(existingHooks, null, 2), "utf8");

  // 3. Snippets initialization (.agents/context-flow/snippets.md)
  const snippetsDir = path.join(ws, ".agents", "context-flow");
  if (!fs.existsSync(snippetsDir)) {
    fs.mkdirSync(snippetsDir, { recursive: true });
  }
  const snippetsPath = path.join(snippetsDir, "snippets.md");
  let snippetsCreated = false;
  if (!fs.existsSync(snippetsPath)) {
    const defaultSnippets = `#op 你的看法是?\n#rg 魯棒性和泛用性，你的看法是?\n`;
    fs.writeFileSync(snippetsPath, defaultSnippets, "utf8");
    snippetsCreated = true;
  }

  return {
    configCreated: !configAlreadyExists,
    hooksUpdated: true,
    snippetsCreated,
    hooksPath,
    configPath,
    snippetsPath,
  };
}

