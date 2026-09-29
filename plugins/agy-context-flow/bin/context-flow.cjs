#!/usr/bin/env node
"use strict";
var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
  // If the importer is in node compatibility mode or this is not an ESM
  // file that has been converted to a CommonJS file using a Babel-
  // compatible transform (i.e. "__esModule" has not been set), then set
  // "default" to the CommonJS "module.exports" for node compatibility.
  isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target,
  mod
));

// src/cli.ts
var fs5 = __toESM(require("node:fs"));
var path5 = __toESM(require("node:path"));
var import_node_crypto = require("node:crypto");

// src/parser.ts
var fs = __toESM(require("node:fs"));
var path = __toESM(require("node:path"));
function cleanUserQuery(rawContent) {
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
function normalizePath(filePath, workspaceRoot) {
  let cleaned = filePath.replace(/^file:\/\/\/?/i, "").replace(/[`'"]/g, "").trim();
  if (path.isAbsolute(cleaned) || /^[a-zA-Z]:/.test(cleaned)) {
    const rel = path.relative(workspaceRoot, cleaned);
    cleaned = rel;
  }
  return cleaned.replace(/\\/g, "/").replace(/^\.\//, "").trim();
}
function extractCitedFiles(text, workspaceRoot) {
  const cited = /* @__PURE__ */ new Set();
  const fileUriRegex = /file:\/\/\/([^\s\)"'#]+)/g;
  let match;
  while ((match = fileUriRegex.exec(text)) !== null) {
    const norm2 = normalizePath(decodeURIComponent(match[1]), workspaceRoot);
    if (norm2 && !norm2.startsWith("..")) {
      cited.add(norm2);
    }
  }
  const mdLinkRegex = /\[[^\]]+\]\(([^:\)\s]+\.[a-zA-Z0-9_-]+)(?:#[^\)]*)?\)/g;
  while ((match = mdLinkRegex.exec(text)) !== null) {
    const norm2 = normalizePath(match[1], workspaceRoot);
    if (norm2 && !norm2.startsWith("..") && !norm2.startsWith("http")) {
      cited.add(norm2);
    }
  }
  const backtickRegex = /`([a-zA-Z0-9_\-\.\/]+\.[a-zA-Z0-9_-]+)`/g;
  while ((match = backtickRegex.exec(text)) !== null) {
    const candidate = match[1];
    if (candidate.includes("/") || candidate.includes("\\")) {
      const norm2 = normalizePath(candidate, workspaceRoot);
      if (norm2 && !norm2.startsWith("..")) {
        cited.add(norm2);
      }
    }
  }
  return Array.from(cited);
}
var STOP_WORDS = /* @__PURE__ */ new Set([
  "the",
  "this",
  "that",
  "with",
  "from",
  "have",
  "will",
  "shall",
  "should",
  "could",
  "would",
  "about",
  "after",
  "before",
  "their",
  "there",
  "what",
  "which",
  "when",
  "where",
  "user",
  "want",
  "need",
  "also",
  "then",
  "just",
  "into",
  "some",
  "only",
  "first",
  "next",
  "last",
  "more",
  "most",
  "been",
  "being",
  "does",
  "done",
  "make",
  "made",
  "like",
  "well",
  "take",
  "took"
]);
function extractThinkingTokens(thinking) {
  if (!thinking) return [];
  const tokens = /* @__PURE__ */ new Set();
  const pathRegex = /[a-zA-Z0-9_\-\./]+\.[a-zA-Z0-9_-]+/g;
  let match;
  while ((match = pathRegex.exec(thinking)) !== null) {
    tokens.add(match[0].toLowerCase().replace(/\\/g, "/"));
  }
  const words = thinking.replace(/[^\w\s\.-]/g, " ").split(/\s+/).filter((w) => w.length >= 3);
  for (const rawWord of words) {
    const subWords = rawWord.replace(/([a-z])([A-Z])/g, "$1 $2").replace(/[_\.-]+/g, " ").toLowerCase().split(/\s+/).filter((w) => w.length >= 3);
    for (const sw of subWords) {
      if (!STOP_WORDS.has(sw) && !/^\d+$/.test(sw)) {
        tokens.add(sw);
      }
    }
  }
  return Array.from(tokens).slice(0, 50);
}
function parseLastTurn(transcriptPath, workspaceRoot) {
  if (!fs.existsSync(transcriptPath)) return null;
  const content = fs.readFileSync(transcriptPath, "utf8");
  const lines = content.split("\n").filter((l) => l.trim().length > 0);
  const steps = [];
  for (const line of lines) {
    try {
      steps.push(JSON.parse(line));
    } catch {
    }
  }
  if (steps.length === 0) return null;
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
  const readSet = /* @__PURE__ */ new Set();
  const editSet = /* @__PURE__ */ new Set();
  let modelResponseText = "";
  let accumulatedThinking = "";
  for (let i = lastUserIdx; i < steps.length; i++) {
    const step = steps[i];
    if (step.tool_calls && Array.isArray(step.tool_calls)) {
      for (const call of step.tool_calls) {
        const name = (call.name || "").toLowerCase();
        const args = call.args || {};
        if (name === "view_file" || name === "read_file" || name === "read_file_content") {
          const rawPath = args.AbsolutePath || args.TargetFile || args.path || args.file;
          if (rawPath) {
            const norm2 = normalizePath(rawPath, workspaceRoot);
            if (norm2) readSet.add(norm2);
          }
        } else if (name === "replace_file_content" || name === "write_to_file" || name === "edit_file") {
          const rawPath = args.TargetFile || args.AbsolutePath || args.path || args.file;
          if (rawPath) {
            const norm2 = normalizePath(rawPath, workspaceRoot);
            if (norm2) editSet.add(norm2);
          }
        }
      }
    }
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
    citedFiles
  };
}

// src/git.ts
var import_node_child_process = require("node:child_process");
function getGitModifiedFiles(workspaceRoot) {
  if (!workspaceRoot) return [];
  try {
    const output = (0, import_node_child_process.execSync)("git status --porcelain", {
      cwd: workspaceRoot,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
      timeout: 1500
    });
    const files = [];
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
function getGitTrackedFiles(workspaceRoot, limit = 500) {
  if (!workspaceRoot) return [];
  try {
    const output = (0, import_node_child_process.execSync)("git ls-files", {
      cwd: workspaceRoot,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
      timeout: 1500
    });
    const files = [];
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
function getGitRecentFiles(workspaceRoot, commitCount = 15) {
  const result = /* @__PURE__ */ new Map();
  if (!workspaceRoot) return result;
  try {
    const output = (0, import_node_child_process.execSync)(`git log --name-only -n ${commitCount} --pretty=format:`, {
      cwd: workspaceRoot,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
      timeout: 1500
    });
    const lines = output.split("\n").map((l) => l.trim()).filter(Boolean);
    const counts = /* @__PURE__ */ new Map();
    for (const line of lines) {
      const normalized = line.replace(/\\/g, "/");
      counts.set(normalized, (counts.get(normalized) || 0) + 1);
    }
    const maxCount = Math.max(1, ...Array.from(counts.values()));
    for (const [file, cnt] of counts.entries()) {
      const score = Math.min(0.4, Number((0.15 + 0.25 * (cnt / maxCount)).toFixed(2)));
      result.set(file, score);
    }
    return result;
  } catch {
    return result;
  }
}

// src/store.ts
var fs3 = __toESM(require("node:fs"));
var path3 = __toESM(require("node:path"));

// src/config.ts
var fs2 = __toESM(require("node:fs"));
var path2 = __toESM(require("node:path"));
var os = __toESM(require("node:os"));
var DEFAULT_CONFIDENCE_TIERS = {
  high: 0.75,
  medium: 0.5,
  low: 0.35
};
var DEFAULT_CONTINUITY_CONFIG = {
  enabled: true,
  maxSessions: 5
};
var DEFAULT_CONFIG = {
  record: {
    enabled: true
  },
  recommend: {
    enabled: true,
    threshold: 0.35,
    maxItems: 3,
    confidenceFormat: "categorical",
    confidenceTiers: { ...DEFAULT_CONFIDENCE_TIERS }
  },
  continuity: { ...DEFAULT_CONTINUITY_CONFIG }
};
function formatConfidence(score, format = "categorical", customTiers) {
  if (format === "hidden") return null;
  if (format === "numeric") return score.toFixed(2);
  const tiers = {
    high: customTiers?.high ?? DEFAULT_CONFIDENCE_TIERS.high,
    medium: customTiers?.medium ?? DEFAULT_CONFIDENCE_TIERS.medium,
    low: customTiers?.low ?? DEFAULT_CONFIDENCE_TIERS.low
  };
  if (score >= tiers.high) return "HIGH";
  if (score >= tiers.medium) return "MEDIUM";
  if (score >= tiers.low) return "LOW";
  return "LOW";
}
function findWorkspaceRoot(startDir = process.cwd()) {
  let current = path2.resolve(startDir);
  const home = path2.resolve(os.homedir());
  while (true) {
    if (current.toLowerCase() !== home.toLowerCase()) {
      if (path2.basename(current) === ".agents") {
        return path2.dirname(current);
      }
      if (fs2.existsSync(path2.join(current, ".git")) || fs2.existsSync(path2.join(current, ".agents"))) {
        return current;
      }
    }
    const parent = path2.dirname(current);
    if (parent === current) break;
    current = parent;
  }
  return void 0;
}
function getConfigPath(workspaceRoot) {
  const ws = workspaceRoot ? path2.resolve(workspaceRoot) : findWorkspaceRoot();
  if (!ws || !fs2.existsSync(ws)) return null;
  return path2.join(ws, ".agents", "context-flow", "config.json");
}
function isPluginEnabled(workspaceRoot) {
  const configPath = getConfigPath(workspaceRoot);
  return configPath !== null && fs2.existsSync(configPath);
}
function loadConfig(workspaceRoot) {
  const configPath = getConfigPath(workspaceRoot);
  if (!configPath || !fs2.existsSync(configPath)) {
    return null;
  }
  try {
    const raw = JSON.parse(fs2.readFileSync(configPath, "utf8"));
    return {
      record: { ...DEFAULT_CONFIG.record, ...raw.record || {} },
      recommend: {
        ...DEFAULT_CONFIG.recommend,
        ...raw.recommend || {},
        confidenceTiers: {
          ...DEFAULT_CONFIG.recommend.confidenceTiers,
          ...raw.recommend?.confidenceTiers || {}
        }
      },
      continuity: {
        ...DEFAULT_CONFIG.continuity,
        ...raw.continuity || {}
      }
    };
  } catch {
    return { ...DEFAULT_CONFIG };
  }
}
function saveConfig(config, workspaceRoot) {
  const ws = workspaceRoot ? path2.resolve(workspaceRoot) : findWorkspaceRoot();
  if (!ws || !fs2.existsSync(ws)) return false;
  const localDir = path2.join(ws, ".agents", "context-flow");
  if (!fs2.existsSync(localDir)) {
    fs2.mkdirSync(localDir, { recursive: true });
  }
  const targetFile = path2.join(localDir, "config.json");
  fs2.writeFileSync(targetFile, JSON.stringify(config, null, 2), "utf8");
  return true;
}
function initWorkspace(workspaceRoot, customScriptRelPath) {
  const ws = workspaceRoot ? path2.resolve(workspaceRoot) : findWorkspaceRoot() || process.cwd();
  const configAlreadyExists = isPluginEnabled(ws);
  if (!configAlreadyExists) {
    saveConfig(DEFAULT_CONFIG, ws);
  } else {
    const existing = loadConfig(ws);
    if (existing) {
      saveConfig(existing, ws);
    }
  }
  const configPath = getConfigPath(ws) || path2.join(ws, ".agents", "context-flow", "config.json");
  const agentsDir = path2.join(ws, ".agents");
  if (!fs2.existsSync(agentsDir)) {
    fs2.mkdirSync(agentsDir, { recursive: true });
  }
  const hooksPath = path2.join(agentsDir, "hooks.json");
  let existingHooks = {};
  if (fs2.existsSync(hooksPath)) {
    try {
      const content = fs2.readFileSync(hooksPath, "utf8").trim();
      if (content) {
        existingHooks = JSON.parse(content);
      }
    } catch {
      existingHooks = {};
    }
  }
  if (existingHooks["agy-file-recommand-hooks"]) {
    delete existingHooks["agy-file-recommand-hooks"];
  }
  const scriptPath = customScriptRelPath || "plugins/agy-context-flow/bin/context-flow.cjs";
  existingHooks["agy-context-flow-hooks"] = {
    PreInvocation: [
      {
        type: "command",
        command: `node ${scriptPath} hook pre-invocation`,
        timeout: 5
      }
    ],
    PostInvocation: [
      {
        type: "command",
        command: `node ${scriptPath} hook post-invocation`,
        timeout: 10
      }
    ],
    Stop: [
      {
        type: "command",
        command: `node ${scriptPath} hook stop`,
        timeout: 10
      }
    ]
  };
  fs2.writeFileSync(hooksPath, JSON.stringify(existingHooks, null, 2), "utf8");
  const snippetsDir = path2.join(ws, ".agents", "context-flow");
  if (!fs2.existsSync(snippetsDir)) {
    fs2.mkdirSync(snippetsDir, { recursive: true });
  }
  const snippetsPath = path2.join(snippetsDir, "snippets.md");
  let snippetsCreated = false;
  if (!fs2.existsSync(snippetsPath)) {
    const defaultSnippets = `#op \u4F60\u7684\u770B\u6CD5\u662F?
#rg \u9B6F\u68D2\u6027\u548C\u6CDB\u7528\u6027\uFF0C\u4F60\u7684\u770B\u6CD5\u662F?
`;
    fs2.writeFileSync(snippetsPath, defaultSnippets, "utf8");
    snippetsCreated = true;
  }
  return {
    configCreated: !configAlreadyExists,
    hooksUpdated: true,
    snippetsCreated,
    hooksPath,
    configPath,
    snippetsPath
  };
}

// src/store.ts
function getStoreDir(workspaceRoot) {
  const ws = workspaceRoot ? path3.resolve(workspaceRoot) : findWorkspaceRoot();
  if (!ws || !fs3.existsSync(ws)) return null;
  const dir = path3.join(ws, ".agents", "context-flow");
  if (!fs3.existsSync(dir)) {
    try {
      fs3.mkdirSync(dir, { recursive: true });
    } catch {
      return null;
    }
  }
  return dir;
}
function getStorePath(workspaceRoot) {
  const dir = getStoreDir(workspaceRoot);
  if (!dir) return null;
  return path3.join(dir, "episodes.jsonl");
}
function getLastSessionPath(workspaceRoot) {
  const dir = getStoreDir(workspaceRoot);
  if (!dir) return null;
  return path3.join(dir, "last_session.json");
}
function getSnippetsPath(workspaceRoot) {
  const dir = getStoreDir(workspaceRoot);
  if (!dir) return null;
  return path3.join(dir, "snippets.md");
}
function loadSnippets(workspaceRoot) {
  const snippets = /* @__PURE__ */ new Map();
  const p = getSnippetsPath(workspaceRoot);
  if (!p || !fs3.existsSync(p)) return snippets;
  try {
    const raw = fs3.readFileSync(p, "utf8");
    const lines = raw.split(/\r?\n/);
    let currentTag = null;
    let currentLines = [];
    const flush = () => {
      if (currentTag && currentLines.length > 0) {
        snippets.set(currentTag.toLowerCase(), currentLines.join("\n").trim());
      }
    };
    for (const line of lines) {
      const match = line.match(/^#+\s*([a-zA-Z0-9_\u4e00-\u9fa5\-]+)(?:\s+(.*))?$/);
      if (match) {
        flush();
        currentTag = match[1];
        currentLines = match[2] ? [match[2]] : [];
      } else if (currentTag) {
        currentLines.push(line);
      }
    }
    flush();
  } catch {
  }
  return snippets;
}
function saveSnippets(workspaceRoot, snippets) {
  const p = getSnippetsPath(workspaceRoot);
  if (!p) return;
  try {
    if (snippets.size === 0) {
      if (fs3.existsSync(p)) fs3.unlinkSync(p);
      return;
    }
    const lines = [];
    for (const [tag, content] of snippets.entries()) {
      if (content.includes("\n")) {
        lines.push(`#${tag}
${content}
`);
      } else {
        lines.push(`#${tag} ${content}`);
      }
    }
    fs3.writeFileSync(p, lines.join("\n").trim() + "\n", "utf8");
  } catch {
  }
}
function getCustomMessagePath(workspaceRoot) {
  const dir = getStoreDir(workspaceRoot);
  if (!dir) return null;
  return path3.join(dir, "custom_message.txt");
}
function saveCustomMessage(workspaceRoot, message) {
  const p = getCustomMessagePath(workspaceRoot);
  if (!p) return;
  try {
    if (message.trim().length === 0) {
      if (fs3.existsSync(p)) fs3.unlinkSync(p);
    } else {
      fs3.writeFileSync(p, message.trim(), "utf8");
    }
  } catch {
  }
}
function loadRecentSessions(workspaceRoot) {
  const lastSessionPath = getLastSessionPath(workspaceRoot);
  if (!lastSessionPath || !fs3.existsSync(lastSessionPath)) return [];
  try {
    const raw = JSON.parse(fs3.readFileSync(lastSessionPath, "utf8"));
    if (Array.isArray(raw)) {
      return raw.filter((item) => Boolean(item && item.conversationId));
    }
    if (raw && typeof raw === "object" && raw.conversationId) {
      return [raw];
    }
    return [];
  } catch {
    return [];
  }
}
function saveSessionSnapshot(workspaceRoot, snapshot, maxSessions = 5) {
  const lastSessionPath = getLastSessionPath(workspaceRoot);
  if (!lastSessionPath) return;
  const currentList = loadRecentSessions(workspaceRoot);
  const filtered = currentList.filter((s) => s.conversationId !== snapshot.conversationId);
  const updated = [snapshot, ...filtered].slice(0, Math.max(1, maxSessions));
  try {
    fs3.writeFileSync(lastSessionPath, JSON.stringify(updated, null, 2), "utf8");
  } catch {
  }
}
function saveEpisode(workspaceRoot, episode) {
  const storePath = getStorePath(workspaceRoot);
  if (!storePath) return;
  const line = JSON.stringify(episode) + "\n";
  try {
    fs3.appendFileSync(storePath, line, "utf8");
  } catch {
  }
}
function loadEpisodes(workspaceRoot, limit = 100) {
  const storePath = getStorePath(workspaceRoot);
  if (!storePath || !fs3.existsSync(storePath)) return [];
  try {
    const content = fs3.readFileSync(storePath, "utf8");
    const lines = content.split("\n").filter((l) => l.trim().length > 0);
    const episodes = [];
    for (let i = lines.length - 1; i >= 0; i--) {
      try {
        const ep = JSON.parse(lines[i]);
        episodes.push(ep);
        if (episodes.length >= limit) break;
      } catch {
      }
    }
    return episodes.reverse();
  } catch {
    return [];
  }
}
function getStats(workspaceRoot) {
  const episodes = loadEpisodes(workspaceRoot, 1e3);
  const readCount = {};
  const editCount = {};
  const citeCount = {};
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
  const toSortedArray = (map) => Object.entries(map).map(([file, count]) => ({ file, count })).sort((a, b) => b.count - a.count).slice(0, 10);
  return {
    totalEpisodes: episodes.length,
    topReadFiles: toSortedArray(readCount),
    topEditedFiles: toSortedArray(editCount),
    topCitedFiles: toSortedArray(citeCount),
    avgReadFilesPerTurn: episodes.length > 0 ? Number((totalRead / episodes.length).toFixed(2)) : 0,
    avgEditedFilesPerTurn: episodes.length > 0 ? Number((totalEdit / episodes.length).toFixed(2)) : 0
  };
}

// src/recommender.ts
var fs4 = __toESM(require("node:fs"));
var path4 = __toESM(require("node:path"));

// src/bm25.ts
var BM25 = class {
  k1;
  b;
  docCount = 0;
  avgDocLength = 0;
  docLengths = /* @__PURE__ */ new Map();
  termDocFreqs = /* @__PURE__ */ new Map();
  invertedIndex = /* @__PURE__ */ new Map();
  constructor(documents, k1 = 1.5, b = 0.75) {
    this.k1 = k1;
    this.b = b;
    this.buildIndex(documents);
  }
  buildIndex(documents) {
    this.docCount = documents.length;
    if (this.docCount === 0) return;
    let totalLength = 0;
    for (const doc of documents) {
      const len = doc.tokens.length;
      this.docLengths.set(doc.id, len);
      totalLength += len;
      const termFreqsInDoc = /* @__PURE__ */ new Map();
      for (const token of doc.tokens) {
        termFreqsInDoc.set(token, (termFreqsInDoc.get(token) || 0) + 1);
      }
      for (const [term, freq] of termFreqsInDoc.entries()) {
        this.termDocFreqs.set(term, (this.termDocFreqs.get(term) || 0) + 1);
        if (!this.invertedIndex.has(term)) {
          this.invertedIndex.set(term, /* @__PURE__ */ new Map());
        }
        this.invertedIndex.get(term).set(doc.id, freq);
      }
    }
    this.avgDocLength = totalLength / this.docCount;
  }
  score(queryTokens) {
    const scores = /* @__PURE__ */ new Map();
    if (this.docCount === 0 || queryTokens.length === 0) return scores;
    for (const token of queryTokens) {
      const docFreq = this.termDocFreqs.get(token) || 0;
      if (docFreq === 0) continue;
      const idf = Math.log(1 + (this.docCount - docFreq + 0.5) / (docFreq + 0.5));
      const postings = this.invertedIndex.get(token);
      if (!postings) continue;
      for (const [docId, tf] of postings.entries()) {
        const docLen = this.docLengths.get(docId) || this.avgDocLength;
        const numerator = tf * (this.k1 + 1);
        const denominator = tf + this.k1 * (1 - this.b + this.b * (docLen / this.avgDocLength));
        const tokenScore = idf * (numerator / denominator);
        scores.set(docId, (scores.get(docId) || 0) + tokenScore);
      }
    }
    return scores;
  }
};

// src/vector.ts
function dot(a, b) {
  let sum = 0;
  for (let i = 0; i < a.length; i++) {
    sum += a[i] * b[i];
  }
  return sum;
}
function norm(a) {
  const d = dot(a, a);
  return d > 0 ? Math.sqrt(d) : 0;
}
function cosineSimilarity(a, b) {
  const normA = norm(a);
  const normB = norm(b);
  if (normA === 0 || normB === 0) return 0;
  const score = dot(a, b) / (normA * normB);
  return Math.max(0, Math.min(1, score));
}
function truncatedSVD(matrix, k, iterations = 25) {
  const m = matrix.length;
  if (m === 0) return { U: [], S: [], V: [] };
  const n = matrix[0].length;
  if (n === 0) return { U: [], S: [], V: [] };
  const targetK = Math.min(k, m, n);
  if (targetK === 0) return { U: [], S: [], V: [] };
  const U = Array.from({ length: m }, () => new Array(targetK).fill(0));
  const V = Array.from({ length: n }, () => new Array(targetK).fill(0));
  const S = new Array(targetK).fill(0);
  const A = matrix.map((row) => [...row]);
  for (let comp = 0; comp < targetK; comp++) {
    let v = new Array(n).fill(0).map((_, i) => comp === 0 ? 1 : Math.cos((i + 1) * (comp + 1)));
    let vNorm = norm(v);
    if (vNorm === 0) v[0] = 1;
    else v = v.map((x) => x / (vNorm || 1));
    let u = new Array(m).fill(0);
    for (let iter = 0; iter < iterations; iter++) {
      for (let i = 0; i < m; i++) {
        let sum = 0;
        for (let j = 0; j < n; j++) {
          sum += A[i][j] * v[j];
        }
        u[i] = sum;
      }
      for (let prev = 0; prev < comp; prev++) {
        let proj = 0;
        for (let i = 0; i < m; i++) proj += u[i] * U[i][prev];
        for (let i = 0; i < m; i++) u[i] -= proj * U[i][prev];
      }
      const uNorm = norm(u);
      if (uNorm > 1e-12) {
        u = u.map((x) => x / uNorm);
      } else {
        break;
      }
      for (let j = 0; j < n; j++) {
        let sum = 0;
        for (let i = 0; i < m; i++) {
          sum += A[i][j] * u[i];
        }
        v[j] = sum;
      }
      for (let prev = 0; prev < comp; prev++) {
        let proj = 0;
        for (let j = 0; j < n; j++) proj += v[j] * V[j][prev];
        for (let j = 0; j < n; j++) v[j] -= proj * V[j][prev];
      }
      const nextVNorm = norm(v);
      if (nextVNorm > 1e-12) {
        v = v.map((x) => x / nextVNorm);
      } else {
        break;
      }
    }
    let sigma = 0;
    for (let i = 0; i < m; i++) {
      for (let j = 0; j < n; j++) {
        sigma += u[i] * A[i][j] * v[j];
      }
    }
    if (sigma < 0) {
      sigma = -sigma;
      u = u.map((x) => -x);
    }
    S[comp] = sigma;
    for (let i = 0; i < m; i++) U[i][comp] = u[i];
    for (let j = 0; j < n; j++) V[j][comp] = v[j];
    for (let i = 0; i < m; i++) {
      for (let j = 0; j < n; j++) {
        A[i][j] -= sigma * u[i] * v[j];
      }
    }
  }
  return { U, S, V };
}
function extractSubwords(word, minN = 3, maxN = 4) {
  if (word.length < minN || !/^[a-zA-Z0-9_]+$/.test(word)) return [];
  const subwords = [];
  const wrapped = `<${word.toLowerCase()}>`;
  const len = wrapped.length;
  for (let n = minN; n <= Math.min(maxN, len); n++) {
    for (let i = 0; i <= len - n; i++) {
      subwords.push(wrapped.slice(i, i + n));
    }
  }
  return subwords;
}
var SemanticVectorEngine = class {
  vocab = /* @__PURE__ */ new Map();
  // word -> index
  vocabList = [];
  contextList = [];
  // file/entity -> index
  contextMap = /* @__PURE__ */ new Map();
  wordEmbeddings = /* @__PURE__ */ new Map();
  contextEmbeddings = /* @__PURE__ */ new Map();
  episodeEmbeddings = /* @__PURE__ */ new Map();
  dim;
  constructor(documents, dim = 24) {
    this.dim = dim;
    this.buildIndex(documents);
  }
  buildIndex(docs) {
    if (docs.length === 0) return;
    const wordCounts = /* @__PURE__ */ new Map();
    const contextCounts = /* @__PURE__ */ new Map();
    const cooccur = /* @__PURE__ */ new Map();
    let totalCooccur = 0;
    const recordToken = (token, uniqueFiles) => {
      wordCounts.set(token, (wordCounts.get(token) || 0) + 1);
      for (const file of uniqueFiles) {
        contextCounts.set(file, (contextCounts.get(file) || 0) + 1);
        if (!cooccur.has(token)) cooccur.set(token, /* @__PURE__ */ new Map());
        const map = cooccur.get(token);
        map.set(file, (map.get(file) || 0) + 1);
        totalCooccur++;
      }
    };
    for (const doc of docs) {
      const uniqueTokens = Array.from(new Set(doc.tokens));
      const uniqueFiles = Array.from(new Set(doc.files));
      for (const token of uniqueTokens) {
        recordToken(token, uniqueFiles);
        if (token.length >= 4) {
          const subwords = extractSubwords(token);
          for (const sw of subwords) {
            recordToken(sw, uniqueFiles);
          }
        }
      }
    }
    this.vocabList = Array.from(wordCounts.keys());
    this.vocabList.forEach((w, i) => this.vocab.set(w, i));
    this.contextList = Array.from(contextCounts.keys());
    this.contextList.forEach((c, i) => this.contextMap.set(c, i));
    const m = this.vocabList.length;
    const n = this.contextList.length;
    if (m === 0 || n === 0 || totalCooccur === 0) return;
    const ppmiMatrix = Array.from({ length: m }, () => new Array(n).fill(0));
    for (let i = 0; i < m; i++) {
      const token = this.vocabList[i];
      const cw = wordCounts.get(token) || 0;
      const fileMap = cooccur.get(token);
      if (!fileMap) continue;
      for (let j = 0; j < n; j++) {
        const file = this.contextList[j];
        const cf = contextCounts.get(file) || 0;
        const c_wf = fileMap.get(file) || 0;
        if (c_wf > 0) {
          const ratio = c_wf * totalCooccur / (cw * cf);
          const pmi = Math.log(1 + ratio);
          ppmiMatrix[i][j] = Math.max(0, pmi);
        }
      }
    }
    const k = Math.min(this.dim, m, n);
    this.dim = k;
    const { U, S, V } = truncatedSVD(ppmiMatrix, k);
    for (let i = 0; i < m; i++) {
      const vec = [];
      for (let c = 0; c < k; c++) {
        vec.push(U[i][c] * Math.sqrt(S[c] || 0));
      }
      this.wordEmbeddings.set(this.vocabList[i], vec);
    }
    for (let j = 0; j < n; j++) {
      const vec = [];
      for (let c = 0; c < k; c++) {
        vec.push(V[j][c] * Math.sqrt(S[c] || 0));
      }
      this.contextEmbeddings.set(this.contextList[j], vec);
    }
    for (const doc of docs) {
      const pooled = this.embedTokens(doc.tokens);
      if (norm(pooled) > 0) {
        this.episodeEmbeddings.set(doc.id, pooled);
      }
    }
  }
  embedTokens(tokens) {
    const k = this.dim;
    if (k === 0) return [];
    const pooled = new Array(k).fill(0);
    let count = 0;
    for (const token of tokens) {
      let vec = this.wordEmbeddings.get(token);
      if (!vec && token.length >= 4) {
        const subwords = extractSubwords(token);
        const subVec = new Array(k).fill(0);
        let subCount = 0;
        for (const sw of subwords) {
          const swEmbedding = this.wordEmbeddings.get(sw);
          if (swEmbedding) {
            for (let i = 0; i < k; i++) subVec[i] += swEmbedding[i];
            subCount++;
          }
        }
        if (subCount > 0) {
          vec = subVec.map((x) => x / subCount);
        }
      }
      if (vec) {
        for (let i = 0; i < vec.length; i++) {
          pooled[i] += vec[i];
        }
        count++;
      }
    }
    if (count === 0) return [];
    return pooled.map((x) => x / count);
  }
  score(queryTokens) {
    const fileScores = /* @__PURE__ */ new Map();
    const episodeScores = /* @__PURE__ */ new Map();
    const qVec = this.embedTokens(queryTokens);
    if (qVec.length === 0 || norm(qVec) === 0) {
      return { fileScores, episodeScores };
    }
    for (const [file, fVec] of this.contextEmbeddings.entries()) {
      const sim = cosineSimilarity(qVec, fVec);
      if (sim > 0.1) {
        fileScores.set(file, sim);
      }
    }
    for (const [epId, epVec] of this.episodeEmbeddings.entries()) {
      const sim = cosineSimilarity(qVec, epVec);
      if (sim > 0.1) {
        episodeScores.set(epId, sim);
      }
    }
    return { fileScores, episodeScores };
  }
};

// src/recommender.ts
function tokenize(text) {
  const tokens = /* @__PURE__ */ new Set();
  const pathRegex = /[a-zA-Z0-9_\-\./]+\.[a-zA-Z0-9_-]+/g;
  let match;
  while ((match = pathRegex.exec(text)) !== null) {
    const p = match[0].toLowerCase().replace(/\\/g, "/");
    tokens.add(p);
  }
  const words = text.toLowerCase().replace(/[^\w\s\.-]/g, " ").split(/\s+/).filter((w) => w.length > 1);
  for (const w of words) {
    tokens.add(w);
    const sub = w.split(/[\.-]/).filter((s) => s.length > 1);
    for (const s of sub) tokens.add(s);
  }
  const cjkChars = text.match(/[\u4e00-\u9fa5]/g);
  if (cjkChars) {
    for (let i = 0; i < cjkChars.length; i++) {
      tokens.add(cjkChars[i]);
      if (i + 1 < cjkChars.length) {
        tokens.add(cjkChars[i] + cjkChars[i + 1]);
      }
    }
  }
  return Array.from(tokens);
}
function isConceptualQuery(query) {
  const trimmed = query.trim().toLowerCase();
  if (trimmed.length <= 2) return true;
  const conceptualPatterns = [
    /^(hi|hello|hey|你好|您好|哈囉)$/i,
    /^(你是誰|自我介紹|你是什(麼|么))/i,
    /^(什麼是|什么是|解釋|解释|什麼意思|科普)/i,
    /^(how are you|who are you|what is|explain)/i,
    /^(好|ok|可以|沒問題|知道了)$/i
  ];
  return conceptualPatterns.some((pattern) => pattern.test(trimmed));
}
function recommendFiles(query, workspaceRoot, customConfig, currentConversationId) {
  const config = customConfig !== void 0 ? customConfig : loadConfig(workspaceRoot);
  if (!config || !config.recommend.enabled || isConceptualQuery(query)) {
    return { items: [], mode: "NONE" };
  }
  const queryTokens = tokenize(query);
  if (queryTokens.length === 0) {
    return { items: [], mode: "NONE" };
  }
  const candidatePriors = {};
  const candidateSemantics = {};
  const addPrior = (file, probability, reason) => {
    if (!file || probability <= 0) return;
    const normalized = file.replace(/\\/g, "/").replace(/^\.\//, "").trim();
    if (!candidatePriors[normalized]) {
      candidatePriors[normalized] = { probs: [], reasons: [] };
    }
    const clamped = Math.max(0.01, Math.min(0.95, probability));
    candidatePriors[normalized].probs.push(clamped);
    if (!candidatePriors[normalized].reasons.includes(reason)) {
      candidatePriors[normalized].reasons.push(reason);
    }
  };
  const addSemantic = (file, probability, reason) => {
    if (!file || probability <= 0) return;
    const normalized = file.replace(/\\/g, "/").replace(/^\.\//, "").trim();
    if (!candidateSemantics[normalized]) {
      candidateSemantics[normalized] = { probs: [], reasons: [] };
    }
    const clamped = Math.max(0.01, Math.min(0.95, probability));
    candidateSemantics[normalized].probs.push(clamped);
    if (!candidateSemantics[normalized].reasons.includes(reason)) {
      candidateSemantics[normalized].reasons.push(reason);
    }
  };
  for (const token of queryTokens) {
    if (token.includes("/") || token.includes(".")) {
      addSemantic(token, 0.85, "direct_path");
    }
  }
  const episodes = loadEpisodes(workspaceRoot, 100);
  const totalEps = episodes.length;
  if (episodes.length > 0) {
    const docs = episodes.map((ep) => ({
      id: ep.id,
      tokens: Array.from(/* @__PURE__ */ new Set([...tokenize(ep.query), ...ep.thinkingTokens || []]))
    }));
    const bm25 = new BM25(docs);
    const bm25Scores = bm25.score(queryTokens);
    for (const [epId, score] of bm25Scores.entries()) {
      if (score > 0.05) {
        const epIndex = episodes.findIndex((e) => e.id === epId);
        const ep = epIndex >= 0 ? episodes[epIndex] : void 0;
        if (ep) {
          const stepDist = Math.max(0, totalEps - 1 - epIndex);
          const stepDecay = Math.max(0.4, Math.pow(0.96, stepDist));
          const isCurrentSession = Boolean(currentConversationId && ep.conversationId === currentConversationId);
          const sessionMultiplier = isCurrentSession ? 1.25 : 1;
          const decay = Math.min(1, stepDecay * sessionMultiplier);
          const confidence = Math.min(0.65, Number((Math.max(score * 0.15, 0.35) * decay).toFixed(2)));
          for (const f of ep.editedFiles) {
            addSemantic(f, confidence, isCurrentSession ? "active_session_edit" : "historical_edit");
          }
          for (const f of ep.citedFiles) {
            addSemantic(f, confidence * 0.7, isCurrentSession ? "active_session_cite" : "historical_cite");
          }
        }
      }
    }
    const vecDocs = episodes.map((ep) => ({
      id: ep.id,
      tokens: Array.from(/* @__PURE__ */ new Set([...tokenize(ep.query), ...ep.thinkingTokens || []])),
      files: Array.from(/* @__PURE__ */ new Set([...ep.editedFiles, ...ep.citedFiles, ...ep.readFiles]))
    }));
    const vectorEngine = new SemanticVectorEngine(vecDocs);
    const vectorResult = vectorEngine.score(queryTokens);
    for (const [file, sim] of vectorResult.fileScores.entries()) {
      if (sim > 0.3) {
        addSemantic(file, Math.min(0.6, Number(sim.toFixed(2))), "semantic_cosine");
      }
    }
    for (const [epId, sim] of vectorResult.episodeScores.entries()) {
      if (sim > 0.35) {
        const ep = episodes.find((e) => e.id === epId);
        if (ep) {
          for (const f of ep.editedFiles) {
            addSemantic(f, Math.min(0.55, Number((sim * 0.8).toFixed(2))), "cross_lingual_semantic");
          }
        }
      }
    }
  }
  const trackedFiles = getGitTrackedFiles(workspaceRoot);
  for (const file of trackedFiles) {
    const fileTokens = tokenize(file);
    const overlap = fileTokens.filter((t) => queryTokens.includes(t) && t.length > 2 && !t.includes("/"));
    if (overlap.length >= 2) {
      addPrior(file, Math.min(0.5, 0.25 * overlap.length), "tree_path_match");
    }
    const baseName = path4.basename(file).toLowerCase();
    if (/^(index|main|app|server|cli)\.(ts|js|rs|go|py)$/.test(baseName)) {
      addPrior(file, 0.15, "entrypoint_prior");
    }
  }
  const gitFiles = getGitModifiedFiles(workspaceRoot);
  for (const f of gitFiles) {
    addPrior(f, 0.4, "git_modified");
  }
  const recentFiles = getGitRecentFiles(workspaceRoot, 15);
  for (const [f, score] of recentFiles.entries()) {
    addPrior(f, score, "git_recent_active");
  }
  const mu = 4;
  const lambda = totalEps / (totalEps + mu);
  const wPrior = (0.6 * totalEps + mu) / (totalEps + mu);
  const wSemantic = (totalEps + 0.5 * mu) / (totalEps + mu);
  const allCandidateKeys = /* @__PURE__ */ new Set([...Object.keys(candidatePriors), ...Object.keys(candidateSemantics)]);
  const maxItems = config.recommend.maxItems ?? 3;
  const threshold = config.recommend.threshold ?? 0.35;
  const items = Array.from(allCandidateKeys).map((filePath) => {
    const priorData = candidatePriors[filePath];
    const semanticData = candidateSemantics[filePath];
    const pPrior = priorData ? 1 - priorData.probs.reduce((acc, p) => acc * (1 - p), 1) : 0;
    const pSemantic = semanticData ? 1 - semanticData.probs.reduce((acc, p) => acc * (1 - p), 1) : 0;
    let penalty = 1;
    if (/\.(test|spec)\.[a-zA-Z0-9]+$/.test(filePath) || filePath.endsWith(".d.ts")) {
      penalty = 0.6;
    }
    const effectivePrior = pPrior * penalty;
    const uncertPrior = Math.pow(1 - effectivePrior, wPrior);
    const uncertSemantic = Math.pow(1 - pSemantic, wSemantic);
    const posteriorScore = 1 - uncertPrior * uncertSemantic;
    const reasons = Array.from(
      /* @__PURE__ */ new Set([...priorData?.reasons || [], ...semanticData?.reasons || []])
    );
    return {
      path: filePath,
      score: Number(Math.min(0.98, posteriorScore).toFixed(2)),
      reasons
    };
  }).filter((item) => {
    if (item.score < threshold) return false;
    if (workspaceRoot) {
      const fullPath = path4.resolve(workspaceRoot, item.path);
      if (!fs4.existsSync(fullPath)) return false;
    }
    return true;
  }).sort((a, b) => b.score - a.score).slice(0, maxItems);
  return items.length > 0 ? { items, mode: "SUGGESTION_HINT" } : { items: [], mode: "NONE" };
}

// src/continuity.ts
function evaluateSessionContinuity(currentConversationId, workspaceRoot, config, overrideRecentSessions) {
  if (!config || !config.continuity || !config.continuity.enabled) {
    return { isContinuity: false };
  }
  let recentList;
  if (overrideRecentSessions !== void 0) {
    if (Array.isArray(overrideRecentSessions)) {
      recentList = overrideRecentSessions;
    } else if (overrideRecentSessions && overrideRecentSessions.conversationId) {
      recentList = [overrideRecentSessions];
    } else {
      recentList = [];
    }
  } else {
    recentList = loadRecentSessions(workspaceRoot);
  }
  if (recentList.length === 0) {
    return { isContinuity: false };
  }
  const topSession = recentList[0];
  if (!currentConversationId || topSession.conversationId === currentConversationId) {
    return { isContinuity: false };
  }
  const handoverSessions = recentList.filter((s) => s.conversationId !== currentConversationId);
  if (handoverSessions.length === 0) {
    return { isContinuity: false };
  }
  return {
    isContinuity: true,
    recentSessions: handoverSessions,
    previousId: handoverSessions[0].conversationId,
    lastQuery: handoverSessions[0].lastQuery,
    updatedAt: handoverSessions[0].updatedAt
  };
}

// src/cli.ts
async function readStdin(timeoutMs = 1500) {
  if (process.stdin.isTTY) return "";
  return new Promise((resolve5) => {
    const chunks = [];
    const timer = setTimeout(() => {
      resolve5(Buffer.concat(chunks).toString("utf8"));
    }, timeoutMs);
    process.stdin.on("data", (chunk) => {
      chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
    });
    process.stdin.on("end", () => {
      clearTimeout(timer);
      resolve5(Buffer.concat(chunks).toString("utf8"));
    });
    process.stdin.on("error", () => {
      clearTimeout(timer);
      resolve5("");
    });
  });
}
function getWorkspaceRoot(input) {
  if (input.workspacePaths && input.workspacePaths.length > 0 && fs5.existsSync(input.workspacePaths[0])) {
    return input.workspacePaths[0];
  }
  if (fs5.existsSync(process.cwd())) {
    return process.cwd();
  }
  return void 0;
}
async function handlePreInvocation() {
  try {
    const rawInput = await readStdin();
    if (!rawInput || !rawInput.trim()) {
      console.log(JSON.stringify({ injectSteps: [] }));
      return;
    }
    let input;
    try {
      const cleaned = rawInput.replace(/^\uFEFF/, "").trim();
      input = JSON.parse(cleaned);
    } catch {
      console.log(JSON.stringify({ injectSteps: [] }));
      return;
    }
    const workspaceRoot = getWorkspaceRoot(input);
    const config = loadConfig(workspaceRoot);
    if (!config) {
      console.log(JSON.stringify({ injectSteps: [] }));
      return;
    }
    const transcriptPath = input.transcriptPath;
    if (!transcriptPath || !fs5.existsSync(transcriptPath)) {
      console.log(JSON.stringify({ injectSteps: [] }));
      return;
    }
    const parsed = parseLastTurn(transcriptPath, workspaceRoot || process.cwd());
    if (!parsed || !parsed.query) {
      console.log(JSON.stringify({ injectSteps: [] }));
      return;
    }
    const injectMessages = [];
    if (config.continuity?.enabled) {
      const continuityResult = evaluateSessionContinuity(
        input.conversationId,
        workspaceRoot,
        config
      );
      if (continuityResult.isContinuity && continuityResult.recentSessions && continuityResult.recentSessions.length > 0) {
        const sessionLines = continuityResult.recentSessions.map((s) => {
          const queryAttr = s.lastQuery ? ` last_query="${s.lastQuery.replace(/"/g, "&quot;")}"` : "";
          const updatedAttr = s.updatedAt ? ` updated_at="${s.updatedAt}"` : "";
          return `  <session id="${s.conversationId}"${queryAttr}${updatedAttr} />`;
        });
        const continuityXml = `<recent_sessions>
${sessionLines.join("\n")}
</recent_sessions>`;
        injectMessages.push(continuityXml);
      }
    }
    const snippets = loadSnippets(workspaceRoot);
    if (snippets.size > 0 && parsed.query) {
      const matches = Array.from(parsed.query.matchAll(/#([a-zA-Z0-9_\u4e00-\u9fa5\-]+)/g));
      const matchedTags = Array.from(new Set(matches.map((m) => m[1].toLowerCase())));
      const triggeredItems = [];
      for (const tag of matchedTags) {
        if (snippets.has(tag)) {
          const content = snippets.get(tag);
          triggeredItems.push(`  <item tag="#${tag}">${content}</item>`);
        }
      }
      if (triggeredItems.length > 0) {
        const reqXml = `<additional_requirements>
${triggeredItems.join("\n")}
</additional_requirements>`;
        injectMessages.push(reqXml);
      }
    }
    if (config.recommend.enabled) {
      const recommendation = recommendFiles(parsed.query, workspaceRoot, config, input.conversationId);
      if (recommendation.mode === "SUGGESTION_HINT" && recommendation.items.length > 0) {
        const format = config.recommend.confidenceFormat ?? "categorical";
        const xmlItems = recommendation.items.map((item) => {
          const conf = formatConfidence(item.score, format, config.recommend.confidenceTiers);
          const confAttr = conf !== null ? ` confidence="${conf}"` : "";
          return `  <file path="${item.path}"${confAttr} reason="${item.reasons.join(",")}" />`;
        });
        const recMessage = `<context_recommendations source="context-flow">
${xmlItems.join("\n")}
</context_recommendations>`;
        injectMessages.push(recMessage);
      }
    }
    if (injectMessages.length > 0) {
      console.log(
        JSON.stringify({
          injectSteps: [
            {
              ephemeralMessage: injectMessages.join("\n\n")
            }
          ]
        })
      );
      return;
    }
    console.log(JSON.stringify({ injectSteps: [] }));
  } catch {
    console.log(JSON.stringify({ injectSteps: [] }));
  }
}
async function handlePostInvocation() {
  try {
    const rawInput = await readStdin();
    if (!rawInput || !rawInput.trim()) {
      console.log(JSON.stringify({}));
      return;
    }
    let input;
    try {
      const cleaned = rawInput.replace(/^\uFEFF/, "").trim();
      input = JSON.parse(cleaned);
    } catch {
      console.log(JSON.stringify({}));
      return;
    }
    const workspaceRoot = getWorkspaceRoot(input);
    const config = loadConfig(workspaceRoot);
    if (!config || !config.record.enabled) {
      console.log(JSON.stringify({}));
      return;
    }
    const transcriptPath = input.transcriptPath;
    if (!transcriptPath || !fs5.existsSync(transcriptPath)) {
      console.log(JSON.stringify({}));
      return;
    }
    const parsed = parseLastTurn(transcriptPath, workspaceRoot || process.cwd());
    if (parsed && parsed.query) {
      const nowIso = (/* @__PURE__ */ new Date()).toISOString();
      const currentConvId = input.conversationId || "unknown";
      const gitFiles = getGitModifiedFiles(workspaceRoot);
      const hasAction = parsed.readFiles.length > 0 || parsed.editedFiles.length > 0 || parsed.citedFiles.length > 0;
      if (hasAction) {
        const episode = {
          id: (0, import_node_crypto.randomUUID)().replace(/-/g, "").slice(0, 12),
          timestamp: nowIso,
          conversationId: currentConvId,
          query: parsed.query,
          thinkingTokens: parsed.thinkingTokens,
          readFiles: parsed.readFiles,
          editedFiles: parsed.editedFiles,
          citedFiles: parsed.citedFiles,
          gitStatus: gitFiles
        };
        saveEpisode(workspaceRoot, episode);
      }
      const maxSessions = config.continuity?.maxSessions ?? 5;
      saveSessionSnapshot(
        workspaceRoot,
        {
          conversationId: currentConvId,
          updatedAt: nowIso,
          lastQuery: parsed.query
        },
        maxSessions
      );
    }
    console.log(JSON.stringify({}));
  } catch {
    console.log(JSON.stringify({}));
  }
}
function handleInit() {
  const workspaceRoot = findWorkspaceRoot() || process.cwd();
  let binRelPath = "plugins/agy-context-flow/bin/context-flow.cjs";
  if (process.argv[1]) {
    const executedScript = path5.resolve(process.argv[1]);
    const relToWs = path5.relative(workspaceRoot, executedScript).replace(/\\/g, "/");
    if (!relToWs.startsWith("..") && !path5.isAbsolute(relToWs)) {
      binRelPath = relToWs;
    }
  }
  const result = initWorkspace(workspaceRoot, binRelPath);
  if (result.configCreated) {
    console.log("\u2705 Created configuration: .agents/context-flow/config.json");
  } else {
    console.log("\u2139\uFE0F  Configuration already present: .agents/context-flow/config.json");
  }
  if (result.hooksUpdated) {
    console.log("\u2705 Registered lifecycle hooks in: .agents/hooks.json");
  }
  if (result.snippetsCreated) {
    console.log("\u2705 Created initial snippets: .agents/context-flow/snippets.md (#op, #rg)");
  } else {
    console.log("\u2139\uFE0F  Snippets file already present: .agents/context-flow/snippets.md");
  }
  console.log("\u{1F680} agy-context-flow initialized and active!");
}
function handleStats() {
  const workspaceRoot = process.cwd();
  const config = loadConfig(workspaceRoot);
  if (!config) {
    console.log("\u26A0\uFE0F  Plugin is not enabled in this project. Run `node plugins/agy-context-flow/bin/context-flow.cjs init` to enable.");
    return;
  }
  const stats = getStats(workspaceRoot);
  console.log("\n\u{1F4CA} === agy-context-flow Statistics ===");
  console.log(`Total Episodes Recorded: ${stats.totalEpisodes}`);
  console.log(`Avg Read Files / Turn  : ${stats.avgReadFilesPerTurn}`);
  console.log(`Avg Edited Files / Turn: ${stats.avgEditedFilesPerTurn}`);
  console.log("\n\u{1F525} Top Read Files:");
  if (stats.topReadFiles.length === 0) console.log("  (None)");
  else stats.topReadFiles.forEach((f, i) => console.log(`  ${i + 1}. [${f.count}x] ${f.file}`));
  console.log("\n\u270F\uFE0F  Top Edited Files:");
  if (stats.topEditedFiles.length === 0) console.log("  (None)");
  else stats.topEditedFiles.forEach((f, i) => console.log(`  ${i + 1}. [${f.count}x] ${f.file}`));
  console.log("\n\u{1F4AC} Top Cited Files in AI Responses:");
  if (stats.topCitedFiles.length === 0) console.log("  (None)");
  else stats.topCitedFiles.forEach((f, i) => console.log(`  ${i + 1}. [${f.count}x] ${f.file}`));
  console.log("");
}
function handleList(limit = 10) {
  const workspaceRoot = process.cwd();
  const config = loadConfig(workspaceRoot);
  if (!config) {
    console.log("\u26A0\uFE0F  Plugin is not enabled in this project. Run `node plugins/agy-context-flow/bin/context-flow.cjs init` to enable.");
    return;
  }
  const episodes = loadEpisodes(workspaceRoot, limit);
  console.log(`
\u{1F4DC} === Last ${episodes.length} Episodes ===`);
  if (episodes.length === 0) {
    console.log("  No episodes recorded yet.");
    return;
  }
  for (const ep of episodes) {
    console.log(`
[${ep.id}] ${ep.timestamp} (Conv: ${ep.conversationId.slice(0, 8)})`);
    console.log(`  Query : ${ep.query.slice(0, 80)}`);
    console.log(`  Read  : ${ep.readFiles.length > 0 ? ep.readFiles.join(", ") : "(None)"}`);
    console.log(`  Edit  : ${ep.editedFiles.length > 0 ? ep.editedFiles.join(", ") : "(None)"}`);
    console.log(`  Cited : ${ep.citedFiles.length > 0 ? ep.citedFiles.join(", ") : "(None)"}`);
    console.log(`  Git   : ${ep.gitStatus.length > 0 ? ep.gitStatus.join(", ") : "(Clean)"}`);
  }
  console.log("");
}
function handleRecommend(query) {
  const workspaceRoot = process.cwd();
  const config = loadConfig(workspaceRoot);
  if (!config) {
    console.log("\u26A0\uFE0F  Plugin is not enabled in this project. Run `node plugins/agy-context-flow/bin/context-flow.cjs init` to enable.");
    return;
  }
  console.log(`
\u{1F50D} Evaluating recommendation for query: "${query}"...`);
  const result = recommendFiles(query, workspaceRoot, config);
  if (result.mode === "NONE" || result.items.length === 0) {
    console.log("  Result: No file recommendations (gating closed or below threshold).");
  } else {
    console.log(`  Result (${result.items.length} suggestions):`);
    result.items.forEach((item, idx) => {
      const conf = formatConfidence(
        item.score,
        config.recommend.confidenceFormat ?? "categorical",
        config.recommend.confidenceTiers
      );
      const confLabel = conf !== null ? ` [${conf}]` : "";
      console.log(`  ${idx + 1}. ${item.path}${confLabel} (Score: ${(item.score * 100).toFixed(0)}%)`);
      item.reasons.forEach((r) => console.log(`     - ${r}`));
    });
  }
  console.log("");
}
function handleSessions() {
  const workspaceRoot = process.cwd();
  const list = loadRecentSessions(workspaceRoot);
  if (list.length === 0) {
    console.log("\n\u2139\uFE0F  No recorded sessions in .agents/context-flow/last_session.json\n");
    return;
  }
  console.log(`
\u{1F4CB} === Recent Sessions Buffer (${list.length}) ===`);
  list.forEach((s, idx) => {
    console.log(`  ${idx + 1}. [${s.conversationId}] (${s.updatedAt})`);
    console.log(`     Last Query: "${s.lastQuery}"`);
  });
  console.log("");
}
function handleCustomMessage(args) {
  const workspaceRoot = process.cwd();
  const snippets = loadSnippets(workspaceRoot);
  if (args.length === 0) {
    if (snippets.size === 0) {
      console.log("\n\u2139\uFE0F  No snippets configured in .agents/context-flow/snippets.md");
      console.log("Usage: context-flow snippet set <tag> <content>");
      console.log('Example: context-flow snippet set op "\u4F60\u7684\u770B\u6CD5\u662F?"\n');
      return;
    }
    console.log(`
\u{1F4DD} === Active Snippets (${snippets.size}) [.agents/context-flow/snippets.md] ===`);
    for (const [tag, content] of snippets.entries()) {
      console.log(`  #${tag} => "${content}"`);
    }
    console.log("\n(Triggered dynamically when your prompt includes #<tag>)\n");
    return;
  }
  if (args[0] === "clear") {
    saveSnippets(workspaceRoot, /* @__PURE__ */ new Map());
    saveCustomMessage(workspaceRoot, "");
    console.log("\u2705 Cleared all snippets (.agents/context-flow/snippets.md cleared).");
    return;
  }
  if (args[0] === "set" && args.length >= 3) {
    const rawTag2 = args[1].replace(/^#+/, "").toLowerCase();
    const content = args.slice(2).join(" ");
    snippets.set(rawTag2, content);
    saveSnippets(workspaceRoot, snippets);
    console.log(`\u2705 Saved snippet #${rawTag2} to .agents/context-flow/snippets.md`);
    return;
  }
  if (args[0] === "delete" && args.length >= 2) {
    const rawTag2 = args[1].replace(/^#+/, "").toLowerCase();
    if (snippets.delete(rawTag2)) {
      saveSnippets(workspaceRoot, snippets);
      console.log(`\u2705 Deleted snippet #${rawTag2}`);
    } else {
      console.log(`\u26A0\uFE0F Snippet #${rawTag2} not found.`);
    }
    return;
  }
  const rawTag = "msg";
  const messageText = args.join(" ");
  snippets.set(rawTag, messageText);
  saveSnippets(workspaceRoot, snippets);
  console.log(`\u2705 Saved snippet #${rawTag}: "${messageText}" to .agents/context-flow/snippets.md`);
}
function handleConfig(args) {
  const workspaceRoot = process.cwd();
  const config = loadConfig(workspaceRoot);
  if (args.length === 0) {
    if (!config) {
      console.log("\n\u26A0\uFE0F  Status: Disabled in this project (No .agents/context-flow/config.json).");
      console.log("Run `node plugins/agy-context-flow/bin/context-flow.cjs init` to enable.\n");
      return;
    }
    console.log("\n\u2699\uFE0F === Current agy-context-flow Configuration ===");
    console.log(JSON.stringify(config, null, 2));
    console.log("");
    return;
  }
  const action = args[0];
  if (action === "set") {
    const key = args[1];
    const value = args[2];
    if (!key || value === void 0) {
      console.log(
        "Usage: context-flow config set <record.enabled|recommend.enabled|recommend.threshold|recommend.maxItems|recommend.confidenceFormat|recommend.confidenceTiers.high|recommend.confidenceTiers.medium|recommend.confidenceTiers.low|continuity.enabled|continuity.maxSessions> <value>"
      );
      return;
    }
    const currentConfig = config || { ...DEFAULT_CONFIG };
    if (!currentConfig.recommend.confidenceTiers) {
      currentConfig.recommend.confidenceTiers = { high: 0.75, medium: 0.5, low: 0.35 };
    }
    if (!currentConfig.continuity) {
      currentConfig.continuity = { enabled: true, maxSessions: 5 };
    }
    if (key === "record.enabled") {
      currentConfig.record.enabled = value === "true" || value === "1";
    } else if (key === "recommend.enabled") {
      currentConfig.recommend.enabled = value === "true" || value === "1";
    } else if (key === "recommend.threshold") {
      currentConfig.recommend.threshold = parseFloat(value);
    } else if (key === "recommend.maxItems") {
      currentConfig.recommend.maxItems = parseInt(value, 10);
    } else if (key === "recommend.confidenceFormat") {
      if (value !== "categorical" && value !== "numeric" && value !== "hidden") {
        console.log("Invalid format. Must be 'categorical', 'numeric', or 'hidden'.");
        return;
      }
      currentConfig.recommend.confidenceFormat = value;
    } else if (key === "recommend.confidenceTiers.high") {
      currentConfig.recommend.confidenceTiers.high = parseFloat(value);
    } else if (key === "recommend.confidenceTiers.medium") {
      currentConfig.recommend.confidenceTiers.medium = parseFloat(value);
    } else if (key === "recommend.confidenceTiers.low") {
      currentConfig.recommend.confidenceTiers.low = parseFloat(value);
    } else if (key === "continuity.enabled") {
      currentConfig.continuity.enabled = value === "true" || value === "1";
    } else if (key === "continuity.maxSessions") {
      currentConfig.continuity.maxSessions = parseInt(value, 10);
    } else {
      console.log(`Unknown config key: ${key}`);
      return;
    }
    saveConfig(currentConfig, workspaceRoot);
    console.log(`\u2705 Config updated in .agents/context-flow/config.json: ${key} = ${value}`);
  }
}
async function main() {
  const args = process.argv.slice(2);
  const command = args[0];
  const subCommand = args[1];
  if (command === "hook") {
    if (subCommand === "pre-invocation") {
      await handlePreInvocation();
    } else if (subCommand === "post-invocation" || subCommand === "stop") {
      await handlePostInvocation();
    } else {
      console.error(`Unknown hook command: ${subCommand}`);
      process.exit(1);
    }
  } else if (command === "init") {
    handleInit();
  } else if (command === "stats") {
    handleStats();
  } else if (command === "list") {
    const limit = args[1] ? parseInt(args[1], 10) : 10;
    handleList(limit);
  } else if (command === "sessions") {
    handleSessions();
  } else if (command === "message" || command === "custom-message" || command === "snippet" || command === "snippets") {
    handleCustomMessage(args.slice(1));
  } else if (command === "recommend") {
    const query = args.slice(1).join(" ");
    if (!query) {
      console.error("Please provide a query: context-flow recommend '<query>'");
      process.exit(1);
    }
    handleRecommend(query);
  } else if (command === "config") {
    handleConfig(args.slice(1));
  } else {
    console.log("Usage: context-flow [init | config [set <k> <v>] | snippet [set <tag> <text>|delete <tag>|clear] | message [<text>|clear] | sessions | hook <pre-invocation|post-invocation> | stats | list | recommend <query>]");
  }
}
main().catch((err) => {
  console.error("CLI error:", err);
  process.exit(1);
});
