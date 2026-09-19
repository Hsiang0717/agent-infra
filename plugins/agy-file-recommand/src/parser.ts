import * as fs from "node:fs";
import * as path from "node:path";
import { TranscriptStep } from "./types.js";

export function cleanUserQuery(rawContent: string): string {
  if (!rawContent) return "";
  let text = rawContent;
  const userRequestMatch = /<USER_REQUEST>([\s\S]*?)<\/USER_REQUEST>/i.exec(text);
  if (userRequestMatch) {
    text = userRequestMatch[1];
  }
  text = text.replace(/<[A-Z_]+>[\s\S]*?<\/[A-Z_]+>/gi, "");
  text = text.replace(/<[^>]+>/g, "");
  return text.trim();
}

export function normalizePath(filePath: string, workspaceRoot: string): string {
  let cleaned = filePath.replace(/^file:\/\/\/?/i, "").replace(/[`'"]/g, "").trim();
  if (path.isAbsolute(cleaned) || /^[a-zA-Z]:/.test(cleaned)) {
    const rel = path.relative(workspaceRoot, cleaned);
    cleaned = rel;
  }
  return cleaned.replace(/\\/g, "/").replace(/^\.\//, "").trim();
}

export function extractCitedFiles(text: string, workspaceRoot: string): string[] {
  const cited = new Set<string>();

  // 1. Match file:/// links
  const fileUriRegex = /file:\/\/\/([^\s\)"'#]+)/g;
  let match: RegExpExecArray | null;
  while ((match = fileUriRegex.exec(text)) !== null) {
    const norm = normalizePath(decodeURIComponent(match[1]), workspaceRoot);
    if (norm && !norm.startsWith("..")) {
      cited.add(norm);
    }
  }

  // 2. Match markdown link targets: [text](path/to/file.ext)
  const mdLinkRegex = /\[[^\]]+\]\(([^:\)\s]+\.[a-zA-Z0-9_-]+)(?:#[^\)]*)?\)/g;
  while ((match = mdLinkRegex.exec(text)) !== null) {
    const norm = normalizePath(match[1], workspaceRoot);
    if (norm && !norm.startsWith("..") && !norm.startsWith("http")) {
      cited.add(norm);
    }
  }

  // 3. Match backtick paths: `src/foo/bar.ts`
  const backtickRegex = /`([a-zA-Z0-9_\-\.\/]+\.[a-zA-Z0-9_-]+)`/g;
  while ((match = backtickRegex.exec(text)) !== null) {
    const candidate = match[1];
    if (candidate.includes("/") || candidate.includes("\\")) {
      const norm = normalizePath(candidate, workspaceRoot);
      if (norm && !norm.startsWith("..")) {
        cited.add(norm);
      }
    }
  }

  return Array.from(cited);
}

export const STOP_WORDS = new Set([
  "the", "this", "that", "with", "from", "have", "will", "shall", "should",
  "could", "would", "about", "after", "before", "their", "there", "what",
  "which", "when", "where", "user", "want", "need", "also", "then", "just",
  "into", "some", "only", "first", "next", "last", "more", "most", "been",
  "being", "does", "done", "make", "made", "like", "well", "take", "took",
]);

export function extractThinkingTokens(thinking: string): string[] {
  if (!thinking) return [];
  const tokens = new Set<string>();

  // 1. Extract paths and filenames
  const pathRegex = /[a-zA-Z0-9_\-\./]+\.[a-zA-Z0-9_-]+/g;
  let match: RegExpExecArray | null;
  while ((match = pathRegex.exec(thinking)) !== null) {
    tokens.add(match[0].toLowerCase().replace(/\\/g, "/"));
  }

  // 2. Extract words and split camelCase / snake_case
  const words = thinking
    .replace(/[^\w\s\.-]/g, " ")
    .split(/\s+/)
    .filter((w) => w.length >= 3);

  for (const rawWord of words) {
    // CamelCase split: "loginTimeout" -> "login", "timeout"
    const subWords = rawWord
      .replace(/([a-z])([A-Z])/g, "$1 $2")
      .replace(/[_\.-]+/g, " ")
      .toLowerCase()
      .split(/\s+/)
      .filter((w) => w.length >= 3);

    for (const sw of subWords) {
      if (!STOP_WORDS.has(sw) && !/^\d+$/.test(sw)) {
        tokens.add(sw);
      }
    }
  }

  return Array.from(tokens).slice(0, 50); // Cap to top 50 salient tokens
}

export function parseLastTurn(
  transcriptPath: string,
  workspaceRoot: string
): {
  query: string;
  thinkingTokens: string[];
  readFiles: string[];
  editedFiles: string[];
  citedFiles: string[];
} | null {
  if (!fs.existsSync(transcriptPath)) return null;

  const content = fs.readFileSync(transcriptPath, "utf8");
  const lines = content.split("\n").filter((l) => l.trim().length > 0);

  const steps: TranscriptStep[] = [];
  for (const line of lines) {
    try {
      steps.push(JSON.parse(line));
    } catch {
      // Ignore malformed lines
    }
  }

  if (steps.length === 0) return null;

  // Find index of the last user turn
  let lastUserIdx = -1;
  for (let i = steps.length - 1; i >= 0; i--) {
    const s = steps[i];
    if (s.source === "USER_EXPLICIT" || s.type === "USER_INPUT") {
      lastUserIdx = i;
      break;
    }
  }

  if (lastUserIdx === -1) {
    lastUserIdx = 0;
  }

  const queryStep = steps[lastUserIdx];
  const query = cleanUserQuery(queryStep.content || "");

  const readSet = new Set<string>();
  const editSet = new Set<string>();
  let modelResponseText = "";
  let accumulatedThinking = "";

  for (let i = lastUserIdx; i < steps.length; i++) {
    const step = steps[i];

    // Collect tool calls
    if (step.tool_calls && Array.isArray(step.tool_calls)) {
      for (const call of step.tool_calls) {
        const name = (call.name || "").toLowerCase();
        const args = (call.args || {}) as Record<string, string>;

        if (name === "view_file" || name === "read_file" || name === "read_file_content") {
          const rawPath = args.AbsolutePath || args.TargetFile || args.path || args.file;
          if (rawPath) {
            const norm = normalizePath(rawPath, workspaceRoot);
            if (norm) readSet.add(norm);
          }
        } else if (
          name === "replace_file_content" ||
          name === "write_to_file" ||
          name === "edit_file"
        ) {
          const rawPath = args.TargetFile || args.AbsolutePath || args.path || args.file;
          if (rawPath) {
            const norm = normalizePath(rawPath, workspaceRoot);
            if (norm) editSet.add(norm);
          }
        }
      }
    }

    // Accumulate model thinking & response text
    if (step.source === "MODEL" || step.type === "PLANNER_RESPONSE") {
      if (step.thinking) {
        accumulatedThinking += "\n" + step.thinking;
      }
      if (step.content) {
        modelResponseText += "\n" + step.content;
      }
    }
  }

  const citedFiles = extractCitedFiles(modelResponseText, workspaceRoot);
  const thinkingTokens = extractThinkingTokens(accumulatedThinking);

  return {
    query,
    thinkingTokens,
    readFiles: Array.from(readSet),
    editedFiles: Array.from(editSet),
    citedFiles,
  };
}
