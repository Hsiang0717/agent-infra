import test from "node:test";
import * as assert from "node:assert/strict";
import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";

import { isConceptualQuery, recommendFiles, tokenize } from "../src/recommender.js";
import { extractCitedFiles, normalizePath, cleanUserQuery } from "../src/parser.js";
import { saveEpisode, loadEpisodes, getStats } from "../src/store.js";
import { loadConfig, saveConfig, DEFAULT_CONFIG } from "../src/config.js";
import { BM25, Document } from "../src/bm25.js";
import { Episode } from "../src/types.js";

test("isConceptualQuery filters conceptual and greeting questions", () => {
  assert.equal(isConceptualQuery("你好"), true);
  assert.equal(isConceptualQuery("什麼是 CSRF？"), true);
  assert.equal(isConceptualQuery("hi"), true);
  assert.equal(isConceptualQuery("好"), true);
  assert.equal(isConceptualQuery("修改 src/main.ts 的登入逾時時間"), false);
  assert.equal(isConceptualQuery("fix bug in discount calculator"), false);
});

test("tokenize extracts intact paths and words", () => {
  const tokens = tokenize("請幫我修改 plugins/agy-file-recommand/src/parser.ts 的正規");
  assert.ok(tokens.includes("plugins/agy-file-recommand/src/parser.ts"));
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





