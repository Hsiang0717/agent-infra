import { execSync } from "node:child_process";

export function getGitModifiedFiles(workspaceRoot?: string): string[] {
  if (!workspaceRoot) return [];
  try {
    const output = execSync("git status --porcelain", {
      cwd: workspaceRoot,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
      timeout: 1500,
    });

    const files: string[] = [];
    for (const line of output.split("\n")) {
      const trimmed = line.trim();
      if (!trimmed) continue;
      const parts = trimmed.split(/\s+/);
      if (parts.length >= 2) {
        const filePath = parts[parts.length - 1].replace(/^"|"$/g, "");
        const normalized = filePath.replace(/\\/g, "/");
        if (!files.includes(normalized)) {
          files.push(normalized);
        }
      }
    }
    return files;
  } catch {
    return [];
  }
}

export function getGitTrackedFiles(workspaceRoot?: string, limit = 500): string[] {
  if (!workspaceRoot) return [];
  try {
    const output = execSync("git ls-files", {
      cwd: workspaceRoot,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
      timeout: 1500,
    });

    const files: string[] = [];
    for (const line of output.split("\n")) {
      const trimmed = line.trim();
      if (!trimmed) continue;
      const normalized = trimmed.replace(/\\/g, "/");
      files.push(normalized);
      if (files.length >= limit) break;
    }
    return files;
  } catch {
    return [];
  }
}
