import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";

export interface RecommandConfig {
  record: {
    enabled: boolean;
  };
  recommend: {
    enabled: boolean;
    threshold: number;
    maxItems: number;
  };
}

export const DEFAULT_CONFIG: RecommandConfig = {
  record: {
    enabled: true,
  },
  recommend: {
    enabled: true,
    threshold: 0.35,
    maxItems: 3,
  },
};

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
  return path.join(ws, ".agents", "file-recommand", "config.json");
}

export function isPluginEnabled(workspaceRoot?: string): boolean {
  const configPath = getConfigPath(workspaceRoot);
  return configPath !== null && fs.existsSync(configPath);
}

export function loadConfig(workspaceRoot?: string): RecommandConfig | null {
  const configPath = getConfigPath(workspaceRoot);
  if (!configPath || !fs.existsSync(configPath)) {
    return null;
  }

  try {
    const raw = JSON.parse(fs.readFileSync(configPath, "utf8"));
    return {
      record: { ...DEFAULT_CONFIG.record, ...(raw.record || {}) },
      recommend: { ...DEFAULT_CONFIG.recommend, ...(raw.recommend || {}) },
    };
  } catch {
    return { ...DEFAULT_CONFIG };
  }
}

export function saveConfig(config: RecommandConfig, workspaceRoot?: string): boolean {
  const ws = workspaceRoot ? path.resolve(workspaceRoot) : findWorkspaceRoot();
  if (!ws || !fs.existsSync(ws)) return false;

  const localDir = path.join(ws, ".agents", "file-recommand");
  if (!fs.existsSync(localDir)) {
    fs.mkdirSync(localDir, { recursive: true });
  }
  const targetFile = path.join(localDir, "config.json");
  fs.writeFileSync(targetFile, JSON.stringify(config, null, 2), "utf8");
  return true;
}
