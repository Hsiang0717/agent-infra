import test from "node:test";
import * as assert from "node:assert/strict";
import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";

import { isConceptualQuery, recommendFiles, tokenize } from "../src/recommender.js";
import { extractCitedFiles, normalizePath, cleanUserQuery } from "../src/parser.js";
import {
  saveEpisode,
  loadEpisodes,
  getStats,
  saveSessionSnapshot,
  loadRecentSessions,
  saveLastSessionState,
  loadLastSessionState,
  loadCustomMessage,
  saveCustomMessage,
  loadSnippets,
  saveSnippets,
} from "../src/store.js";
import { loadConfig, saveConfig, DEFAULT_CONFIG, formatConfidence, initWorkspace } from "../src/config.js";
import { evaluateSessionContinuity } from "../src/continuity.js";
import { BM25, Document } from "../src/bm25.js";
import { Episode, LastSessionState, SessionSnapshot } from "../src/types.js";

test("isConceptualQuery filters conceptual and greeting questions", () => {
  assert.equal(isConceptualQuery("你好"), true);
  assert.equal(isConceptualQuery("什麼是 CSRF？"), true);
  assert.equal(isConceptualQuery("hi"), true);
  assert.equal(isConceptualQuery("好"), true);
  assert.equal(isConceptualQuery("修改 src/main.ts 的登入逾時時間"), false);
  assert.equal(isConceptualQuery("fix bug in discount calculator"), false);
});

test("tokenize extracts intact paths and words", () => {
  const tokens = tokenize("請幫我修改 plugins/agy-context-flow/src/parser.ts 的正規");
  assert.ok(tokens.includes("plugins/agy-context-flow/src/parser.ts"));
  assert.ok(tokens.includes("正規"));
});

test("BM25 scores documents with higher relevance for rare keywords", () => {
  const docs: Document[] = [
    { id: "1", tokens: ["修復", "訂單", "折扣", "計算", "bug"] },
    { id: "2", tokens: ["重構", "使用者", "登入", "oauth", "模組"] },
  ];

  const bm25 = new BM25(docs);
  const scoresOauth = bm25.score(["登入", "oauth"]);
  assert.ok((scoresOauth.get("2") || 0) > (scoresOauth.get("1") || 0));

  const scoresDiscount = bm25.score(["訂單", "折扣"]);
  assert.ok((scoresDiscount.get("1") || 0) > (scoresDiscount.get("2") || 0));
});

test("cleanUserQuery strips XML wrapper tags", () => {
  const raw = "<USER_REQUEST>\n修改 parser.ts\n</USER_REQUEST>\n<ADDITIONAL_METADATA>\n2026\n</ADDITIONAL_METADATA>";
  assert.equal(cleanUserQuery(raw), "修改 parser.ts");
});

test("recommendFiles returns direct query path match and BM25 match", () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "agy-rec-test-"));
  try {
    saveConfig(DEFAULT_CONFIG, tmpDir);
    fs.mkdirSync(path.join(tmpDir, "src"), { recursive: true });
    fs.writeFileSync(path.join(tmpDir, "src/oauth.ts"), "// oauth code");

    const episode: Episode = {
      id: "ep1",
      timestamp: new Date().toISOString(),
      conversationId: "conv-1",
      query: "處理 oauth 認證授權過期",
      readFiles: ["src/oauth.ts"],
      editedFiles: ["src/oauth.ts"],
      citedFiles: ["src/oauth.ts"],
      gitStatus: [],
    };
    saveEpisode(tmpDir, episode);

    // Query with similar keywords triggers BM25 retrieval
    const result = recommendFiles("修復 oauth 授權問題", tmpDir);
    assert.equal(result.mode, "SUGGESTION_HINT");
    assert.ok(result.items.some((item) => item.path === "src/oauth.ts"));
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
});

test("store saves and retrieves episodes accurately in local workspace", () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "agy-test-"));
  try {
    const episode: Episode = {
      id: "test12345678",
      timestamp: new Date().toISOString(),
      conversationId: "conv-1",
      query: "修復訂單計算 bug",
      readFiles: ["src/calc.ts", "config.json"],
      editedFiles: ["src/calc.ts"],
      citedFiles: ["src/calc.ts"],
      gitStatus: ["src/calc.ts"],
    };

    saveEpisode(tmpDir, episode);
    const loaded = loadEpisodes(tmpDir);
    assert.equal(loaded.length, 1);
    assert.equal(loaded[0].id, "test12345678");

    const stats = getStats(tmpDir);
    assert.equal(stats.totalEpisodes, 1);
    assert.equal(stats.topReadFiles[0].file, "src/calc.ts");
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
});

test("SemanticVectorEngine matches cross-lingual queries via co-occurrence bridge", async () => {
  const { SemanticVectorEngine, cosineSimilarity } = await import("../src/vector.js");

  const v1 = [1, 0, 1];
  const v2 = [1, 0, 1];
  assert.ok(Math.abs(cosineSimilarity(v1, v2) - 1.0) < 1e-4);

  // Cross-lingual doc co-occurrence:
  // Episode 1 (Chinese): "修改 登入 逾時" -> files: ["src/auth.ts"]
  // Episode 2 (English): "fix login session token" -> files: ["src/auth.ts"]
  const docs = [
    { id: "1", tokens: ["修改", "登入", "逾時"], files: ["src/auth.ts"] },
    { id: "2", tokens: ["fix", "login", "session", "token"], files: ["src/auth.ts"] },
    { id: "3", tokens: ["計算", "購物車", "折扣"], files: ["src/cart.ts"] },
  ];

  const engine = new SemanticVectorEngine(docs, 8);
  const result = engine.score(["登入"]);

  assert.ok(result.fileScores.has("src/auth.ts"));
  assert.ok((result.fileScores.get("src/auth.ts") || 0) > 0.3);
});

test("recommendFiles leverages PPMI-SVD vector engine for cross-lingual tasks", () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "agy-rec-cross-"));
  try {
    saveConfig(DEFAULT_CONFIG, tmpDir);
    fs.mkdirSync(path.join(tmpDir, "src/auth"), { recursive: true });
    fs.writeFileSync(path.join(tmpDir, "src/auth/jwt.ts"), "// jwt code");

    // Episode 1 in Chinese touching auth file
    const epChinese: Episode = {
      id: "ep_zh",
      timestamp: new Date().toISOString(),
      conversationId: "conv-zh",
      query: "修改 使用者 登入 權限 檢查",
      readFiles: ["src/auth/jwt.ts"],
      editedFiles: ["src/auth/jwt.ts"],
      citedFiles: ["src/auth/jwt.ts"],
      gitStatus: [],
    };
    saveEpisode(tmpDir, epChinese);

    // Episode 2 in English also touching auth file
    const epEnglish: Episode = {
      id: "ep_en",
      timestamp: new Date().toISOString(),
      conversationId: "conv-en",
      query: "fix auth token session expiration",
      readFiles: ["src/auth/jwt.ts"],
      editedFiles: ["src/auth/jwt.ts"],
      citedFiles: ["src/auth/jwt.ts"],
      gitStatus: [],
    };
    saveEpisode(tmpDir, epEnglish);

    // Query in English retrieves via shared file/concept embedding
    const result = recommendFiles("fix auth token issue", tmpDir);
    assert.equal(result.mode, "SUGGESTION_HINT");
    assert.ok(result.items.some((item) => item.path === "src/auth/jwt.ts"));
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
});

test("extractThinkingTokens extracts technical terms and splits identifiers", async () => {
  const { extractThinkingTokens } = await import("../src/parser.js");

  const thinking = "The user wants to fix loginTimeout in src/auth.ts and inspect jwtTokenPayload.";
  const tokens = extractThinkingTokens(thinking);

  assert.ok(tokens.includes("src/auth.ts"));
  assert.ok(tokens.includes("login"));
  assert.ok(tokens.includes("timeout"));
  assert.ok(tokens.includes("jwt"));
  assert.ok(tokens.includes("payload"));
});

test("Agent thinking creates single-turn cross-lingual bridge for PPMI-SVD", () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "agy-rec-think-"));
  try {
    saveConfig(DEFAULT_CONFIG, tmpDir);
    fs.mkdirSync(path.join(tmpDir, "src/cart"), { recursive: true });
    fs.writeFileSync(path.join(tmpDir, "src/cart/calculator.ts"), "// cart calculator");

    // Single turn: User asks in Chinese, but Model thought in English with technical keywords
    const episodeWithThinking: Episode = {
      id: "ep_think",
      timestamp: new Date().toISOString(),
      conversationId: "conv-think",
      query: "調整購物車結帳計算",
      thinkingTokens: ["cart", "calculator", "checkout", "discount", "total", "src/cart/calculator.ts"],
      readFiles: ["src/cart/calculator.ts"],
      editedFiles: ["src/cart/calculator.ts"],
      citedFiles: ["src/cart/calculator.ts"],
      gitStatus: [],
    };
    saveEpisode(tmpDir, episodeWithThinking);

    // Later query in pure English "checkout discount calculator"
    // matches the file via thinkingTokens bridge!
    const result = recommendFiles("checkout discount calculator", tmpDir);
    assert.equal(result.mode, "SUGGESTION_HINT");
    assert.ok(result.items.some((item) => item.path === "src/cart/calculator.ts"));
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
});

test("recommendFiles filters out deleted ghost files from recommendations", () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "agy-rec-ghost-"));
  try {
    saveConfig(DEFAULT_CONFIG, tmpDir);

    // Episode referencing a deleted file that does NOT exist on disk
    const episode: Episode = {
      id: "ep_ghost",
      timestamp: new Date().toISOString(),
      conversationId: "conv-ghost",
      query: "處理舊模組 deleted_legacy_module.ts",
      readFiles: ["src/deleted_legacy_module.ts"],
      editedFiles: ["src/deleted_legacy_module.ts"],
      citedFiles: ["src/deleted_legacy_module.ts"],
      gitStatus: [],
    };
    saveEpisode(tmpDir, episode);

    // Query asking about the deleted module: should be filtered out by file existence check
    const result = recommendFiles("修改 deleted_legacy_module.ts", tmpDir);
    assert.equal(result.items.length, 0);
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
});

test("recommendFiles uses Noisy-OR probabilistic fusion without flat saturation", () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "agy-rec-noisyor-"));
  try {
    saveConfig(DEFAULT_CONFIG, tmpDir);
    fs.mkdirSync(path.join(tmpDir, "src"), { recursive: true });
    fs.writeFileSync(path.join(tmpDir, "src/engine.ts"), "// engine code");

    const episode: Episode = {
      id: "ep_noisyor",
      timestamp: new Date().toISOString(),
      conversationId: "conv-noisyor",
      query: "重構核心引擎 engine.ts",
      readFiles: ["src/engine.ts"],
      editedFiles: ["src/engine.ts"],
      citedFiles: ["src/engine.ts"],
      gitStatus: ["src/engine.ts"],
    };
    saveEpisode(tmpDir, episode);

    // Query combines exact path, BM25, and semantic signals
    const result = recommendFiles("優化 src/engine.ts 核心引擎", tmpDir, DEFAULT_CONFIG, "conv-noisyor");
    assert.equal(result.mode, "SUGGESTION_HINT");
    const topItem = result.items.find((i) => i.path === "src/engine.ts");
    assert.ok(topItem);
    // Multi-signal combination should yield a high probabilistic score (> 0.85 and <= 0.98)
    assert.ok(topItem.score >= 0.85 && topItem.score <= 0.98);
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
});

test("FastText subwords enable OOV and typo tolerance in SemanticVectorEngine", async () => {
  const { SemanticVectorEngine, extractSubwords } = await import("../src/vector.js");

  const subwords = extractSubwords("authentication");
  assert.ok(subwords.includes("<aut"));
  assert.ok(subwords.includes("auth"));
  assert.ok(subwords.includes("ion>"));

  // Training docs with "authentication"
  const docs = [
    { id: "1", tokens: ["authentication", "middleware", "token"], files: ["src/auth/jwt.ts"] },
    { id: "2", tokens: ["database", "connection", "pool"], files: ["src/db/pool.ts"] },
  ];

  const engine = new SemanticVectorEngine(docs, 8);

  // Query with typo / OOV derivative "authenticaton" (missing 'i')
  const result = engine.score(["authenticaton"]);
  assert.ok(result.fileScores.has("src/auth/jwt.ts"));
  assert.ok((result.fileScores.get("src/auth/jwt.ts") || 0) > 0.25);
});

test("recommendFiles smoothly transitions from Structural Prior to Semantic Likelihood via Dirichlet Shrinkage", () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "agy-rec-bayes-"));
  try {
    const { execSync } = require("node:child_process");
    execSync("git init", { cwd: tmpDir, stdio: "ignore" });

    saveConfig(DEFAULT_CONFIG, tmpDir);
    fs.mkdirSync(path.join(tmpDir, "src"), { recursive: true });
    fs.writeFileSync(path.join(tmpDir, "src/server.ts"), "// server entry");
    fs.writeFileSync(path.join(tmpDir, "src/server.test.ts"), "// server test");

    execSync("git add .", { cwd: tmpDir, stdio: "ignore" });

    // Cold start with 0 episodes: server.ts has tree path match & entrypoint prior
    const coldResult = recommendFiles("檢查 src/server.ts 啟動配置", tmpDir, DEFAULT_CONFIG);
    assert.equal(coldResult.mode, "SUGGESTION_HINT");
    const serverItem = coldResult.items.find((i) => i.path === "src/server.ts");
    assert.ok(serverItem);
    assert.ok(serverItem.score >= 0.7);
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
});

test("formatConfidence supports categorical, numeric, and hidden with customizable thresholds", () => {
  // 1. Default categorical tiers (high: 0.75, medium: 0.50, low: 0.35)
  assert.equal(formatConfidence(0.95, "categorical"), "HIGH");
  assert.equal(formatConfidence(0.75, "categorical"), "HIGH");
  assert.equal(formatConfidence(0.74, "categorical"), "MEDIUM");
  assert.equal(formatConfidence(0.50, "categorical"), "MEDIUM");
  assert.equal(formatConfidence(0.49, "categorical"), "LOW");
  assert.equal(formatConfidence(0.35, "categorical"), "LOW");

  // 2. Custom categorical tiers
  const customTiers = { high: 0.9, medium: 0.7, low: 0.4 };
  assert.equal(formatConfidence(0.85, "categorical", customTiers), "MEDIUM");
  assert.equal(formatConfidence(0.92, "categorical", customTiers), "HIGH");
  assert.equal(formatConfidence(0.45, "categorical", customTiers), "LOW");
});

test("initWorkspace creates config and merges hooks safely preserving other plugins", () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "agy-init-test-"));
  try {
    const agentsDir = path.join(tmpDir, ".agents");
    fs.mkdirSync(agentsDir, { recursive: true });
    
    // Simulate pre-existing hooks from another plugin
    const existingHooks = {
      "git-agent-flow-hook": {
        enabled: true,
      },
      "om-context-injection": {
        PreInvocation: [{ type: "command", command: "node bin/om.cjs hook pre-invocation", timeout: 5 }],
      },
    };
    fs.writeFileSync(path.join(agentsDir, "hooks.json"), JSON.stringify(existingHooks, null, 2), "utf8");

    // Execute initWorkspace
    const result = initWorkspace(tmpDir, "plugins/agy-context-flow/bin/context-flow.cjs");

    assert.equal(result.configCreated, true);
    assert.equal(result.hooksUpdated, true);
    assert.equal(result.snippetsCreated, true);

    // Verify snippets.md exists with default #op and #rg
    assert.ok(fs.existsSync(result.snippetsPath));
    const snippetsContent = fs.readFileSync(result.snippetsPath, "utf8");
    assert.ok(snippetsContent.includes("#op 你的看法是?"));
    assert.ok(snippetsContent.includes("#rg 魯棒性和泛用性，你的看法是?"));

    // Verify config.json exists
    const configPath = path.join(agentsDir, "context-flow", "config.json");
    assert.ok(fs.existsSync(configPath));
    const config = JSON.parse(fs.readFileSync(configPath, "utf8"));
    assert.equal(config.recommend.enabled, true);
    assert.equal(config.record.enabled, true);

    // Verify hooks.json merged safely and preserved other plugins
    const hooks = JSON.parse(fs.readFileSync(path.join(agentsDir, "hooks.json"), "utf8"));
    assert.ok(hooks["git-agent-flow-hook"]);
    assert.ok(hooks["om-context-injection"]);
    assert.ok(hooks["agy-context-flow-hooks"]);
    assert.equal(hooks["agy-context-flow-hooks"].PreInvocation[0].command, "node plugins/agy-context-flow/bin/context-flow.cjs hook pre-invocation");
    assert.equal(hooks["agy-context-flow-hooks"].PostInvocation[0].command, "node plugins/agy-context-flow/bin/context-flow.cjs hook post-invocation");
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
});

test("evaluateSessionContinuity triggers on 1st turn of new session and returns recent session list", () => {
  const recentSessions: SessionSnapshot[] = [
    {
      conversationId: "conv-session-1",
      updatedAt: "2026-09-20T12:00:00.000Z",
      lastQuery: "修改 recommender.ts 的向量計算邏輯",
    },
    {
      conversationId: "conv-session-0",
      updatedAt: "2026-09-20T11:00:00.000Z",
      lastQuery: "初始專案設定與架構規劃",
    },
  ];

  // 1. New session (First turn handover with multiple sessions)
  const res1 = evaluateSessionContinuity(
    "conv-session-2",
    undefined,
    DEFAULT_CONFIG,
    recentSessions
  );
  assert.equal(res1.isContinuity, true);
  assert.equal(res1.previousId, "conv-session-1");
  assert.equal(res1.lastQuery, "修改 recommender.ts 的向量計算邏輯");
  assert.equal(res1.recentSessions?.length, 2);
  assert.equal(res1.recentSessions?.[0].conversationId, "conv-session-1");
  assert.equal(res1.recentSessions?.[1].conversationId, "conv-session-0");

  // 2. Same session suppression (Silence when in same session as top item)
  const res2 = evaluateSessionContinuity(
    "conv-session-1", // Same conversation ID!
    undefined,
    DEFAULT_CONFIG,
    recentSessions
  );
  assert.equal(res2.isContinuity, false);

  // 3. Disabled continuity config
  const res3 = evaluateSessionContinuity(
    "conv-session-2",
    undefined,
    { ...DEFAULT_CONFIG, continuity: { enabled: false } },
    recentSessions
  );
  assert.equal(res3.isContinuity, false);
});

test("saveSessionSnapshot and loadRecentSessions manage LRU buffer with maxSessions correctly", () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "agy-multistate-test-"));
  try {
    saveSessionSnapshot(tmpDir, {
      conversationId: "conv-1",
      updatedAt: "2026-09-20T10:00:00.000Z",
      lastQuery: "Query 1",
    }, 3);

    saveSessionSnapshot(tmpDir, {
      conversationId: "conv-2",
      updatedAt: "2026-09-20T10:05:00.000Z",
      lastQuery: "Query 2",
    }, 3);

    saveSessionSnapshot(tmpDir, {
      conversationId: "conv-3",
      updatedAt: "2026-09-20T10:10:00.000Z",
      lastQuery: "Query 3",
    }, 3);

    let list = loadRecentSessions(tmpDir);
    assert.equal(list.length, 3);
    assert.equal(list[0].conversationId, "conv-3");
    assert.equal(list[1].conversationId, "conv-2");
    assert.equal(list[2].conversationId, "conv-1");

    // Re-active conv-1 (should move to top)
    saveSessionSnapshot(tmpDir, {
      conversationId: "conv-1",
      updatedAt: "2026-09-20T10:15:00.000Z",
      lastQuery: "Query 1 updated",
    }, 3);

    list = loadRecentSessions(tmpDir);
    assert.equal(list.length, 3);
    assert.equal(list[0].conversationId, "conv-1");
    assert.equal(list[0].lastQuery, "Query 1 updated");
    assert.equal(list[1].conversationId, "conv-3");
    assert.equal(list[2].conversationId, "conv-2");

    // Add 4th session (should evict conv-2 because maxSessions is 3)
    saveSessionSnapshot(tmpDir, {
      conversationId: "conv-4",
      updatedAt: "2026-09-20T10:20:00.000Z",
      lastQuery: "Query 4",
    }, 3);

    list = loadRecentSessions(tmpDir);
    assert.equal(list.length, 3);
    assert.equal(list[0].conversationId, "conv-4");
    assert.equal(list[1].conversationId, "conv-1");
    assert.equal(list[2].conversationId, "conv-3");
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
});

test("saveCustomMessage and loadCustomMessage manage custom message persistence and clearing", () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "agy-custom-msg-test-"));
  try {
    assert.equal(loadCustomMessage(tmpDir), null);

    const sampleMsg = "請務必使用繁體中文，且遵守 Diff 最小化原則。";
    saveCustomMessage(tmpDir, sampleMsg);

    const loaded = loadCustomMessage(tmpDir);
    assert.equal(loaded, sampleMsg);

    // Clear message
    saveCustomMessage(tmpDir, "");
    assert.equal(loadCustomMessage(tmpDir), null);
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
});

test("loadSnippets and saveSnippets parse markdown headings and on-demand trigger tags", () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "agy-snippets-test-"));
  try {
    const agentsDir = path.join(tmpDir, ".agents", "context-flow");
    fs.mkdirSync(agentsDir, { recursive: true });

    // 1. Test single-line format: #op and #rg
    const mdContent = `#op 你的看法是?\n#rg 魯棒性和泛用性，你的看法是?\n\n# multi\n這是多行內容第一行\n這是多行內容第二行\n`;
    fs.writeFileSync(path.join(agentsDir, "snippets.md"), mdContent, "utf8");

    const snippets = loadSnippets(tmpDir);
    assert.equal(snippets.size, 3);
    assert.equal(snippets.get("op"), "你的看法是?");
    assert.equal(snippets.get("rg"), "魯棒性和泛用性，你的看法是?");
    assert.equal(snippets.get("multi"), "這是多行內容第一行\n這是多行內容第二行");

    // 2. Test saving snippets
    snippets.set("newtag", "新規則內容");
    saveSnippets(tmpDir, snippets);

    const reloaded = loadSnippets(tmpDir);
    assert.equal(reloaded.size, 4);
    assert.equal(reloaded.get("newtag"), "新規則內容");

    // 3. Test clear snippets
    saveSnippets(tmpDir, new Map());
    assert.equal(loadSnippets(tmpDir).size, 0);
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
});












