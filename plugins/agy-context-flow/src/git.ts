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

export function getGitRecentFiles(workspaceRoot?: string, commitCount = 15): Map<string, number> {
  const result = new Map<string, number>();
  if (!workspaceRoot) return result;
  try {
    const output = execSync(`git log --name-only -n ${commitCount} --pretty=format:`, {
      cwd: workspaceRoot,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
      timeout: 1500,
    });

    const lines = output.split("\n").map((l) => l.trim()).filter(Boolean);
    const counts = new Map<string, number>();
    for (const line of lines) {
      const normalized = line.replace(/\\/g, "/");
      counts.set(normalized, (counts.get(normalized) || 0) + 1);
    }

    const maxCount = Math.max(1, ...Array.from(counts.values()));
    for (const [file, cnt] of counts.entries()) {
      // Recency weight between 0.15 and 0.40 based on frequency in recent commits
      const score = Math.min(0.4, Number((0.15 + 0.25 * (cnt / maxCount)).toFixed(2)));
      result.set(file, score);
    }
    return result;
  } catch {
    return result;
  }
}

