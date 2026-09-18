import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";
import { createHash } from "node:crypto";
import type { StoragePaths } from "./types.js";

export function findWorkspaceRoot(startDir: string = process.cwd()): string | undefined {
  let current = path.resolve(startDir);
  let home = path.resolve(os.homedir());
  try {
    home = fs.realpathSync.native(home).toLowerCase();
  } catch {}

  while (true) {
    let resolvedCurrent = current.toLowerCase();
    try {
      resolvedCurrent = fs.realpathSync.native(current).toLowerCase();
    } catch {}

    if (resolvedCurrent !== home) {
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

export function computeWorkspaceHash(workspaceRoot?: string): string {
  if (!workspaceRoot) return "global";
  const normalized = path.resolve(workspaceRoot).toLowerCase().replace(/\\/g, "/");
  return createHash("sha256").update(normalized).digest("hex").slice(0, 12);
}

export function resolveStoragePaths(conversationId: string, startDir: string = process.cwd()): StoragePaths {
  const workspaceRoot = findWorkspaceRoot(startDir);
  const workspaceHash = computeWorkspaceHash(workspaceRoot);

  const baseCacheDir = path.join(os.homedir(), ".gemini", "observational-memory", workspaceHash);
  const sessionStoreDir = path.join(baseCacheDir, conversationId);
  const activeLedgerPath = path.join(baseCacheDir, "active_ledger.json");
  const activeProjectionPath = path.join(baseCacheDir, "active_projection.md");

  let projectReflectionsPath: string;
  if (workspaceRoot) {
    projectReflectionsPath = path.join(workspaceRoot, ".agents", "memory", "reflections.json");
  } else {
    projectReflectionsPath = path.join(baseCacheDir, "reflections.json");
  }

  return {
    projectReflectionsPath,
    sessionStoreDir,
    activeLedgerPath,
    activeProjectionPath,
    workspaceRoot,
    workspaceHash,
  };
}

export function getGlobalStoreDir(): string {
  return path.join(os.homedir(), ".gemini", "observational-memory");
}


