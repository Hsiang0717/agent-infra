import * as fs from "node:fs";
import * as path from "node:path";
import { Episode, LastSessionState } from "./types.js";
import { findWorkspaceRoot } from "./config.js";

export function getStoreDir(workspaceRoot?: string): string | null {
  const ws = workspaceRoot ? path.resolve(workspaceRoot) : findWorkspaceRoot();
  if (!ws || !fs.existsSync(ws)) return null;
  const dir = path.join(ws, ".agents", "context-flow");
  if (!fs.existsSync(dir)) {
    try {
      fs.mkdirSync(dir, { recursive: true });
    } catch {
      return null;
    }
  }
  return dir;
}

export function getStorePath(workspaceRoot?: string): string | null {
  const dir = getStoreDir(workspaceRoot);
  if (!dir) return null;
  return path.join(dir, "episodes.jsonl");
}

export function getLastSessionPath(workspaceRoot?: string): string | null {
  const dir = getStoreDir(workspaceRoot);
  if (!dir) return null;
  return path.join(dir, "last_session.json");
}

export function getCustomMessagePath(workspaceRoot?: string): string | null {
  const dir = getStoreDir(workspaceRoot);
  if (!dir) return null;
  return path.join(dir, "custom_message.txt");
}

export function loadCustomMessage(workspaceRoot?: string): string | null {
  const p = getCustomMessagePath(workspaceRoot);
  if (!p || !fs.existsSync(p)) return null;
  try {
    const content = fs.readFileSync(p, "utf8").trim();
    return content.length > 0 ? content : null;
  } catch {
    return null;
  }
}

export function saveCustomMessage(workspaceRoot: string | undefined, message: string): void {
  const p = getCustomMessagePath(workspaceRoot);
  if (!p) return;
  try {
    if (message.trim().length === 0) {
      if (fs.existsSync(p)) fs.unlinkSync(p);
    } else {
      fs.writeFileSync(p, message.trim(), "utf8");
    }
  } catch {
    // Ignore if unwritable
  }
}

export function loadRecentSessions(workspaceRoot?: string): SessionSnapshot[] {
  const lastSessionPath = getLastSessionPath(workspaceRoot);
  if (!lastSessionPath || !fs.existsSync(lastSessionPath)) return [];
  try {
    const raw = JSON.parse(fs.readFileSync(lastSessionPath, "utf8"));
    if (Array.isArray(raw)) {
      return raw.filter((item): item is SessionSnapshot => Boolean(item && item.conversationId));
    }
    if (raw && typeof raw === "object" && raw.conversationId) {
      return [raw as SessionSnapshot];
    }
    return [];
  } catch {
    return [];
  }
}

export function loadLastSessionState(workspaceRoot?: string): SessionSnapshot | null {
  const list = loadRecentSessions(workspaceRoot);
  return list.length > 0 ? list[0] : null;
}

export function saveSessionSnapshot(
  workspaceRoot: string | undefined,
  snapshot: SessionSnapshot,
  maxSessions = 5
): void {
  const lastSessionPath = getLastSessionPath(workspaceRoot);
  if (!lastSessionPath) return;

  const currentList = loadRecentSessions(workspaceRoot);
  const filtered = currentList.filter((s) => s.conversationId !== snapshot.conversationId);
  const updated = [snapshot, ...filtered].slice(0, Math.max(1, maxSessions));

  try {
    fs.writeFileSync(lastSessionPath, JSON.stringify(updated, null, 2), "utf8");
  } catch {
    // Ignore if unwritable
  }
}

export function saveLastSessionState(workspaceRoot: string | undefined, state: LastSessionState): void {
  if (Array.isArray(state)) {
    const lastSessionPath = getLastSessionPath(workspaceRoot);
    if (!lastSessionPath) return;
    try {
      fs.writeFileSync(lastSessionPath, JSON.stringify(state, null, 2), "utf8");
    } catch {
      // Ignore if unwritable
    }
  } else if (state && state.conversationId) {
    saveSessionSnapshot(workspaceRoot, state);
  }
}

export function saveEpisode(workspaceRoot: string | undefined, episode: Episode): void {
  const storePath = getStorePath(workspaceRoot);
  if (!storePath) return;

  const line = JSON.stringify(episode) + "\n";
  try {
    fs.appendFileSync(storePath, line, "utf8");
  } catch {
    // Ignore if unwritable
  }
}

export function loadEpisodes(workspaceRoot?: string, limit = 100): Episode[] {
  const storePath = getStorePath(workspaceRoot);
  if (!storePath || !fs.existsSync(storePath)) return [];

  try {
    const content = fs.readFileSync(storePath, "utf8");
    const lines = content.split("\n").filter((l) => l.trim().length > 0);
    const episodes: Episode[] = [];

    for (let i = lines.length - 1; i >= 0; i--) {
      try {
        const ep = JSON.parse(lines[i]);
        episodes.push(ep);
        if (episodes.length >= limit) break;
      } catch {
        // Ignore corrupted lines
      }
    }

    return episodes.reverse();
  } catch {
    return [];
  }
}

export interface ContextFlowStats {
  totalEpisodes: number;
  topReadFiles: Array<{ file: string; count: number }>;
  topEditedFiles: Array<{ file: string; count: number }>;
  topCitedFiles: Array<{ file: string; count: number }>;
  avgReadFilesPerTurn: number;
  avgEditedFilesPerTurn: number;
}

export type RecommandStats = ContextFlowStats;

export function getStats(workspaceRoot?: string): ContextFlowStats {
  const episodes = loadEpisodes(workspaceRoot, 1000);
  const readCount: Record<string, number> = {};
  const editCount: Record<string, number> = {};
  const citeCount: Record<string, number> = {};

  let totalRead = 0;
  let totalEdit = 0;

  for (const ep of episodes) {
    for (const f of ep.readFiles) {
      readCount[f] = (readCount[f] || 0) + 1;
      totalRead++;
    }
    for (const f of ep.editedFiles) {
      editCount[f] = (editCount[f] || 0) + 1;
      totalEdit++;
    }
    for (const f of ep.citedFiles) {
      citeCount[f] = (citeCount[f] || 0) + 1;
    }
  }

  const toSortedArray = (map: Record<string, number>) =>
    Object.entries(map)
      .map(([file, count]) => ({ file, count }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 10);

  return {
    totalEpisodes: episodes.length,
    topReadFiles: toSortedArray(readCount),
    topEditedFiles: toSortedArray(editCount),
    topCitedFiles: toSortedArray(citeCount),
    avgReadFilesPerTurn: episodes.length > 0 ? Number((totalRead / episodes.length).toFixed(2)) : 0,
    avgEditedFilesPerTurn: episodes.length > 0 ? Number((totalEdit / episodes.length).toFixed(2)) : 0,
  };
}

