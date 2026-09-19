import test from "node:test";
import * as assert from "node:assert";
import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";

import { hashId } from "../src/ids.js";
import { estimateStringTokens } from "../src/tokens.js";
import { StorageManager, renderSummary } from "../src/storage.js";
import { findWorkspaceRoot, computeWorkspaceHash, resolveStoragePaths } from "../src/config.js";
import type { Observation, Reflection } from "../src/types.js";

test("hashId generates 12-character hex ID", () => {
  const id1 = hashId("test string 1");
  const id2 = hashId("test string 2");
  assert.strictEqual(id1.length, 12);
  assert.strictEqual(id2.length, 12);
  assert.match(id1, /^[0-9a-f]{12}$/);
  assert.notStrictEqual(id1, id2);
});

test("estimateStringTokens handles English and CJK", () => {
  const eng = "This is a simple test sentence.";
  const cjk = "這是一個測試句子。";
  assert.ok(estimateStringTokens(eng) > 0);
  assert.ok(estimateStringTokens(cjk) > 0);
});

test("StorageManager and renderSummary produce valid projections", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "om-test-"));
  const projectDir = path.join(tempDir, "workspace");
  const sessionDir = path.join(projectDir, ".gemini", "memory", "test-conv");
  fs.mkdirSync(sessionDir, { recursive: true });

  try {
    const storage = new StorageManager(sessionDir, true, projectDir);
    const ledger = storage.loadLedger("test-conv");

    assert.strictEqual(ledger.conversationId, "test-conv");
    assert.strictEqual(ledger.activeObservations.length, 0);
    assert.strictEqual(ledger.reflections.length, 0);

    const obs: Observation = {
      id: hashId("obs-1"),
      content: "User decided to migrate from REST to GraphQL",
      timestamp: "2026-09-16 14:00",
      relevance: "high",
      sourceStepIndices: [1, 2],
      tokenCount: 15,
    };

    const ref: Reflection = {
      id: hashId("ref-1"),
      content: "Project uses Next.js 15 with Supabase authentication",
      supportingObservationIds: [obs.id],
      tokenCount: 10,
    };

    ledger.activeObservations.push(obs);
    ledger.allObservations.push(obs);
    ledger.reflections.push(ref);

    storage.saveLedger(ledger);

    // Verify projection
    const projection = storage.readProjection();
    assert.ok(projection.includes("## Reflections"));
    assert.ok(projection.includes(ref.id));
    assert.ok(projection.includes("Project uses Next.js 15"));
    assert.ok(projection.includes("## Observations"));
    assert.ok(projection.includes(obs.id));
    assert.ok(projection.includes("User decided to migrate from REST to GraphQL"));

    // Verify lookup by id
    const foundObs = storage.findItemById(obs.id);
    assert.ok(foundObs);
    assert.strictEqual(foundObs.type, "observation");
    assert.strictEqual((foundObs.item as Observation).content, obs.content);

    const foundRef = storage.findItemById(ref.id);
    assert.ok(foundRef);
    assert.strictEqual(foundRef.type, "reflection");
    assert.strictEqual((foundRef.item as Reflection).content, ref.content);
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

test("Multi-session baseline reflections atomic merge prevents clobbering", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "om-merge-"));
  const projectDir = path.join(tempDir, "workspace");
  const session1Dir = path.join(projectDir, ".gemini", "memory", "session-1");
  const session2Dir = path.join(projectDir, ".gemini", "memory", "session-2");

  try {
    const storage1 = new StorageManager(session1Dir, true, projectDir);
    const ledger1 = storage1.loadLedger("session-1");

    const ref1: Reflection = {
      id: hashId("ref-session-1"),
      content: "Session 1 decided: use Tailwind CSS v4",
      supportingObservationIds: [],
      tokenCount: 10,
    };
    ledger1.reflections.push(ref1);
    storage1.saveLedger(ledger1);

    // Session 2 in separate instance
    const storage2 = new StorageManager(session2Dir, true, projectDir);
    const ledger2 = storage2.loadLedger("session-2");

    const ref2: Reflection = {
      id: hashId("ref-session-2"),
      content: "Session 2 decided: use SQLite database",
      supportingObservationIds: [],
      tokenCount: 10,
    };
    ledger2.reflections.push(ref2);
    storage2.saveLedger(ledger2);

    // Verify baseline file has BOTH reflections merged
    const baselineFile = storage1.projectBaselinePath!;
    assert.ok(fs.existsSync(baselineFile));
    const baselineData = JSON.parse(fs.readFileSync(baselineFile, "utf-8"));
    assert.strictEqual(baselineData.reflections.length, 2);
    const ids = baselineData.reflections.map((r: Reflection) => r.id);
    assert.ok(ids.includes(ref1.id));
    assert.ok(ids.includes(ref2.id));
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

test("resolveStoragePaths implements clean separation between project baseline and ephemeral session cache", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "om-paths-test-"));
  const workspaceDir = path.join(tempDir, "my-app");
  fs.mkdirSync(path.join(workspaceDir, ".agents"), { recursive: true });

  try {
    const paths = resolveStoragePaths("session-123", workspaceDir);
    assert.strictEqual(paths.workspaceRoot, workspaceDir);
    assert.strictEqual(paths.projectReflectionsPath, path.join(workspaceDir, ".agents", "memory", "reflections.json"));
    assert.ok(paths.sessionStoreDir.includes(path.join(".gemini", "observational-memory")));
    assert.ok(paths.sessionStoreDir.endsWith("session-123"));
    assert.notStrictEqual(paths.workspaceHash, "global");

    // Outside workspace test
    const globalPaths = resolveStoragePaths("session-456", tempDir);
    assert.strictEqual(
      globalPaths.projectReflectionsPath,
      path.join(os.homedir(), ".gemini", "observational-memory", "global", "reflections.json")
    );
    assert.strictEqual(globalPaths.workspaceHash, "global");
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

test("StorageManager.recordObservation and pinReflection update ledger, projection, and baseline", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "om-skill-test-"));
  const projectDir = path.join(tempDir, "workspace");
  const sessionDir = path.join(projectDir, ".gemini", "memory", "skill-conv");
  fs.mkdirSync(sessionDir, { recursive: true });

  try {
    const storage = new StorageManager(sessionDir, true, projectDir);

    // Record observation
    const obs = storage.recordObservation("skill-conv", "Chose Vitest for fast unit testing", "high", [10, 11]);
    assert.strictEqual(obs.content, "Chose Vitest for fast unit testing");
    assert.strictEqual(obs.relevance, "high");
    assert.strictEqual(obs.id.length, 12);

    // Verify ledger has it
    const ledger = storage.loadLedger("skill-conv");
    assert.strictEqual(ledger.activeObservations.length, 1);
    assert.strictEqual(ledger.allObservations.length, 1);
    assert.strictEqual(ledger.activeObservations[0].id, obs.id);

    // Verify projection has it
    let projection = storage.readProjection();
    assert.ok(projection.includes(obs.id));
    assert.ok(projection.includes("Chose Vitest for fast unit testing"));

    // Pin reflection
    const ref = storage.pinReflection("skill-conv", "Repository mandates TypeScript strict mode");
    assert.strictEqual(ref.content, "Repository mandates TypeScript strict mode");
    assert.strictEqual(ref.id.length, 12);

    // Verify ledger and projection have reflection
    const updatedLedger = storage.loadLedger("skill-conv");
    assert.strictEqual(updatedLedger.reflections.length, 1);
    assert.strictEqual(updatedLedger.reflections[0].id, ref.id);

    projection = storage.readProjection();
    assert.ok(projection.includes("## Reflections"));
    assert.ok(projection.includes(ref.id));
    assert.ok(projection.includes("Repository mandates TypeScript strict mode"));

    // Verify baseline file exists and has reflection
    const baselineFile = storage.projectBaselinePath!;
    assert.ok(fs.existsSync(baselineFile));
    const baselineContent = JSON.parse(fs.readFileSync(baselineFile, "utf-8"));
    assert.ok(baselineContent.reflections.some((r: any) => r.id === ref.id));

    // Verify recall lookup by ID
    const foundObs = storage.findItemById(obs.id);
    assert.ok(foundObs);
    assert.strictEqual(foundObs.type, "observation");
    assert.strictEqual((foundObs.item as Observation).content, obs.content);

    const foundRef = storage.findItemById(ref.id);
    assert.ok(foundRef);
    assert.strictEqual(foundRef.type, "reflection");
    assert.strictEqual((foundRef.item as Reflection).content, ref.content);
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

test("StorageManager.dropObservations prunes active observations and updates projection", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "om-drop-test-"));
  const sessionDir = path.join(tempDir, "session");
  fs.mkdirSync(sessionDir, { recursive: true });

  try {
    const storage = new StorageManager(sessionDir, false);
    const obs1 = storage.recordObservation("test-conv", "Observation 1 to keep");
    const obs2 = storage.recordObservation("test-conv", "Observation 2 to prune");

    let ledger = storage.loadLedger("test-conv");
    assert.strictEqual(ledger.activeObservations.length, 2);

    // Drop obs2
    const dropResult = storage.dropObservations("test-conv", [obs2.id]);
    assert.strictEqual(dropResult.dropped.length, 1);
    assert.strictEqual(dropResult.dropped[0], obs2.id);
    assert.strictEqual(dropResult.remaining, 1);

    // Verify ledger
    ledger = storage.loadLedger("test-conv");
    assert.strictEqual(ledger.activeObservations.length, 1);
    assert.strictEqual(ledger.activeObservations[0].id, obs1.id);
    assert.strictEqual(ledger.droppedObservationIds.length, 1);
    assert.strictEqual(ledger.droppedObservationIds[0], obs2.id);

    // Verify projection no longer has obs2
    const projection = storage.readProjection();
    assert.ok(projection.includes(obs1.id));
    assert.ok(!projection.includes(obs2.id));
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

test("StorageManager.getOrInitProjection loads baseline reflections for new sessions", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "om-hook-test-"));
  const projectDir = path.join(tempDir, "workspace");
  const baselineDir = path.join(projectDir, ".agents", "memory");
  fs.mkdirSync(baselineDir, { recursive: true });

  const baselineRef: Reflection = {
    id: hashId("baseline-ref"),
    content: "Project uses Antigravity Lifecycle Hooks",
    supportingObservationIds: [],
    tokenCount: 8,
  };
  fs.writeFileSync(
    path.join(baselineDir, "reflections.json"),
    JSON.stringify({ workspacePath: projectDir, updatedAt: new Date().toISOString(), reflections: [baselineRef] })
  );

  try {
    const paths = resolveStoragePaths("new-session-conv", projectDir);
    const storage = new StorageManager(paths);

    // Initial projection should load baseline reflections even before any record command
    const projection = storage.getOrInitProjection("new-session-conv");
    assert.ok(projection.includes("## Reflections"));
    assert.ok(projection.includes(baselineRef.id));
    assert.ok(projection.includes("Project uses Antigravity Lifecycle Hooks"));
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

test("Workspace-level active ledger maintains observation continuity across sessions (/clear)", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "om-cross-session-"));
  const projectDir = path.join(tempDir, "workspace");
  fs.mkdirSync(path.join(projectDir, ".git"), { recursive: true });

  try {
    const pathsSession1 = resolveStoragePaths("session-alpha", projectDir);
    const storageSession1 = new StorageManager(pathsSession1);

    // Session 1 records an observation
    const obs = storageSession1.recordObservation("session-alpha", "Completed Step 1: database migration", "high");
    assert.strictEqual(obs.conversationId, "session-alpha");

    // Session 2 starts (simulating user executing /clear and starting new conversation)
    const pathsSession2 = resolveStoragePaths("session-beta", projectDir);
    const storageSession2 = new StorageManager(pathsSession2);

    // Verify Session 2 immediately gets projection containing Session 1's observation
    const projectionSession2 = storageSession2.getOrInitProjection("session-beta");
    assert.ok(projectionSession2.includes("## Observations"));
    assert.ok(projectionSession2.includes(obs.id));
    assert.ok(projectionSession2.includes("Completed Step 1: database migration"));

    // Session 2 can recall it by ID
    const found = storageSession2.findItemById(obs.id);
    assert.ok(found);
    assert.strictEqual(found.type, "observation");
    assert.strictEqual((found.item as Observation).conversationId, "session-alpha");
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

test("Optimistic locking resolves concurrent modifications without lost updates", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "om-opt-lock-"));
  const projectDir = path.join(tempDir, "workspace");
  fs.mkdirSync(path.join(projectDir, ".git"), { recursive: true });

  try {
    const pathsA = resolveStoragePaths("client-A", projectDir);
    const storageA = new StorageManager(pathsA);
    const pathsB = resolveStoragePaths("client-B", projectDir);
    const storageB = new StorageManager(pathsB);

    // Client A and B both load the ledger at version 1
    const ledgerA = storageA.loadLedger("client-A");
    const ledgerB = storageB.loadLedger("client-B");
    assert.strictEqual(ledgerA.version, 1);
    assert.strictEqual(ledgerB.version, 1);

    // Client A adds an observation and saves (version increments to 2)
    const obsA: Observation = {
      id: hashId("obs-A"),
      content: "Observation from client A",
      timestamp: "2026-09-18 08:30",
      relevance: "high",
      sourceStepIndices: [],
      tokenCount: 10,
    };
    ledgerA.activeObservations.push(obsA);
    ledgerA.allObservations.push(obsA);
    storageA.saveLedger(ledgerA);

    // Client B adds an observation to its stale in-memory ledger (still at version 1) and saves
    const obsB: Observation = {
      id: hashId("obs-B"),
      content: "Observation from client B",
      timestamp: "2026-09-18 08:31",
      relevance: "high",
      sourceStepIndices: [],
      tokenCount: 10,
    };
    ledgerB.activeObservations.push(obsB);
    ledgerB.allObservations.push(obsB);
    // saveLedger detects conflict, merges obsA and obsB, and increments version
    storageB.saveLedger(ledgerB);

    // Verify both observations exist and version is incremented
    const finalLedger = storageA.loadLedger("client-A");
    assert.strictEqual(finalLedger.version, 3);
    assert.strictEqual(finalLedger.activeObservations.length, 2);
    const ids = finalLedger.activeObservations.map((o) => o.id);
    assert.ok(ids.includes(obsA.id));
    assert.ok(ids.includes(obsB.id));
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

test("Sliding window cap trims low/medium observations when exceeding limit", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "om-sliding-window-"));
  const projectDir = path.join(tempDir, "workspace");
  fs.mkdirSync(path.join(projectDir, ".git"), { recursive: true });

  try {
    const paths = resolveStoragePaths("sliding-conv", projectDir);
    const storage = new StorageManager(paths);

    // Record 25 observations (exceeding MAX_ACTIVE_OBSERVATIONS = 20)
    for (let i = 1; i <= 25; i++) {
      const relevance = i <= 5 ? "high" : "medium";
      storage.recordObservation("sliding-conv", `Obs ${i}`, relevance);
    }

    const ledger = storage.loadLedger("sliding-conv");
    assert.strictEqual(ledger.activeObservations.length, 20);
    assert.strictEqual(ledger.allObservations.length, 25);

    // High relevance items from 1-5 must be preserved
    const contents = ledger.activeObservations.map((o) => o.content);
    assert.ok(contents.includes("Obs 1"));
    assert.ok(contents.includes("Obs 5"));
    assert.ok(contents.includes("Obs 25"));
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

import { calculateSimilarity, hybridTokenize, DEDUPLICATION_SIMILARITY_THRESHOLD } from "../src/similarity.js";

test("hybridTokenize and calculateSimilarity accurately detect CJK and English similarity", () => {
  const text1 = "目前所有討論嚴格限定在 exp/ 目錄，優先評估魯棒性與泛用性。";
  const text2 = "目前所有討論嚴格限定在 exp/ 目錄，優先以魯棒性與泛用性為最高標準。";
  const text3 = "專案全面改用 PostgreSQL 與 Drizzle ORM 管理資料庫。";

  const tokens1 = hybridTokenize(text1);
  const tokens2 = hybridTokenize(text2);
  assert.ok(tokens1.has("exp"));
  assert.ok(tokens1.has("目錄"));
  assert.ok(tokens1.has("魯棒"));

  const sim12 = calculateSimilarity(text1, text2);
  assert.ok(sim12 >= DEDUPLICATION_SIMILARITY_THRESHOLD, `Similarity ${sim12} should be >= ${DEDUPLICATION_SIMILARITY_THRESHOLD}`);

  const sim13 = calculateSimilarity(text1, text3);
  assert.ok(sim13 < 0.3, `Similarity ${sim13} should be low`);
});

test("StorageManager.setFocus, clearFocus, and renderSummary project Current Focus at top", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "om-focus-test-"));
  const sessionDir = path.join(tempDir, "session");
  fs.mkdirSync(sessionDir, { recursive: true });

  try {
    const storage = new StorageManager(sessionDir, false);
    
    // Set focus
    const focus = storage.setFocus("test-focus", "Implement AST Validation", "Run unit tests");
    assert.strictEqual(focus.goal, "Implement AST Validation");
    assert.strictEqual(focus.nextAction, "Run unit tests");

    // Projection should include Current Focus at top
    let projection = storage.readProjection();
    assert.ok(projection.includes("## Current Focus & Next Steps"));
    assert.ok(projection.includes("- Goal: Implement AST Validation"));
    assert.ok(projection.includes("- Next Action: Run unit tests"));

    // Clear focus
    storage.clearFocus("test-focus");
    projection = storage.readProjection();
    assert.ok(!projection.includes("## Current Focus & Next Steps"));
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

test("StorageManager.checkpoint atomically settles milestone, updates focus, and resolves prior IDs", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "om-checkpoint-test-"));
  const sessionDir = path.join(tempDir, "session");
  fs.mkdirSync(sessionDir, { recursive: true });

  try {
    const storage = new StorageManager(sessionDir, false);
    
    // Step 1: Record initial preliminary draft observation
    const draftObs = storage.recordObservation("test-cp", "Drafted initial AST schema", "medium");
    assert.strictEqual(storage.loadLedger("test-cp").activeObservations.length, 1);

    // Step 2: Checkpoint settles milestone and resolves draftObs
    const cpResult = storage.checkpoint(
      "test-cp",
      "AST Validation schema finalized and verified with tests",
      "Implement AST Transformer",
      {
        relevance: "high",
        resolvesIds: [draftObs.id],
      }
    );

    assert.strictEqual(cpResult.observation.content, "AST Validation schema finalized and verified with tests");
    assert.strictEqual(cpResult.focus.goal, "AST Validation schema finalized and verified with tests");
    assert.strictEqual(cpResult.focus.nextAction, "Implement AST Transformer");
    assert.strictEqual(cpResult.droppedIds.length, 1);
    assert.strictEqual(cpResult.droppedIds[0], draftObs.id);

    // Verify projection has milestone, focus, and does NOT have draftObs
    const projection = storage.readProjection();
    assert.ok(projection.includes("## Current Focus & Next Steps"));
    assert.ok(projection.includes("- Next Action: Implement AST Transformer"));
    assert.ok(projection.includes(cpResult.observation.id));
    assert.ok(!projection.includes(draftObs.id));

    // Verify active observations in ledger
    const ledger = storage.loadLedger("test-cp");
    assert.strictEqual(ledger.activeObservations.length, 1);
    assert.strictEqual(ledger.activeObservations[0].id, cpResult.observation.id);
    assert.ok(ledger.droppedObservationIds.includes(draftObs.id));
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

test("StorageManager.pinReflection auto-deduplicates similar reflections and supports --replace", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "om-pin-dedup-"));
  const sessionDir = path.join(tempDir, "session");
  fs.mkdirSync(sessionDir, { recursive: true });

  try {
    const storage = new StorageManager(sessionDir, false);

    // Pin 1st reflection
    const ref1 = storage.pinReflection(
      "test-dedup",
      "目前所有討論嚴格限定在 exp/ 目錄，優先評估魯棒性與泛用性。"
    );
    assert.strictEqual(ref1.action, "created");
    assert.strictEqual(storage.loadLedger("test-dedup").reflections.length, 1);

    // Pin 2nd similar reflection (should auto-merge/replace instead of creating duplicate)
    const ref2 = storage.pinReflection(
      "test-dedup",
      "目前所有討論嚴格限定在 exp/ 目錄，優先以魯棒性與泛用性為最高標準。"
    );
    assert.strictEqual(ref2.action, "merged");
    assert.strictEqual(ref2.id, ref1.id); // Same ID preserved
    assert.strictEqual(ref2.content, "目前所有討論嚴格限定在 exp/ 目錄，優先以魯棒性與泛用性為最高標準。");

    const ledger = storage.loadLedger("test-dedup");
    assert.strictEqual(ledger.reflections.length, 1, "Should only have 1 reflection after auto-dedup");

    // Explicit replace by ID
    const ref3 = storage.pinReflection("test-dedup", "專案限定使用 Vitest 與 TypeScript", {
      replaceId: ref1.id,
    });
    assert.strictEqual(ref3.action, "replaced");
    assert.strictEqual(ref3.id, ref1.id);
    assert.strictEqual(storage.loadLedger("test-dedup").reflections.length, 1);
    assert.strictEqual(storage.loadLedger("test-dedup").reflections[0].content, "專案限定使用 Vitest 與 TypeScript");
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

test("StorageManager.unpinReflections removes reflections from session and baseline", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "om-unpin-test-"));
  const projectDir = path.join(tempDir, "workspace");
  const sessionDir = path.join(projectDir, ".gemini", "memory", "unpin-conv");
  fs.mkdirSync(sessionDir, { recursive: true });

  try {
    const storage = new StorageManager(sessionDir, true, projectDir);

    const ref = storage.pinReflection("unpin-conv", "Temporary architectural constraint to remove");
    assert.strictEqual(storage.loadLedger("unpin-conv").reflections.length, 1);

    const unpinRes = storage.unpinReflections("unpin-conv", [ref.id]);
    assert.strictEqual(unpinRes.unpinned.length, 1);
    assert.strictEqual(unpinRes.unpinned[0], ref.id);
    assert.strictEqual(unpinRes.remaining, 0);

    const updatedLedger = storage.loadLedger("unpin-conv");
    assert.strictEqual(updatedLedger.reflections.length, 0);

    // Baseline reflections file must also have 0 reflections
    const baselineFile = storage.projectBaselinePath!;
    assert.ok(fs.existsSync(baselineFile));
    const baselineData = JSON.parse(fs.readFileSync(baselineFile, "utf-8"));
    assert.strictEqual(baselineData.reflections.length, 0);
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

import { execFileSync } from "node:child_process";

test("CLI execution of om focus, checkpoint, record --resolves, pin --replace, and unpin", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "om-cli-2.0-"));
  const projectDir = path.join(tempDir, "workspace");
  fs.mkdirSync(path.join(projectDir, ".git"), { recursive: true });
  const omBin = path.resolve(__dirname, "..", "bin", "om.cjs");

  try {
    const env = { ...process.env, AGY_CONVERSATION_ID: "cli-conv-2" };

    // 1. om focus
    const focusOut = execFileSync("node", [omBin, "focus", "Build parser engine", "--next", "Run AST tests"], {
      cwd: projectDir,
      env,
      encoding: "utf-8",
    });
    assert.ok(focusOut.includes("[OK] Updated Current Focus"));
    assert.ok(focusOut.includes("Build parser engine"));

    // 2. om record preliminary observation
    const recOut = execFileSync("node", [omBin, "record", "Parser draft 1 completed"], {
      cwd: projectDir,
      env,
      encoding: "utf-8",
    });
    const draftIdMatch = recOut.match(/\[([0-9a-f]{12})\]/);
    assert.ok(draftIdMatch);
    const draftId = draftIdMatch[1];

    // 3. om checkpoint with --resolves
    const cpOut = execFileSync("node", [omBin, "checkpoint", "Parser v2 verified", "--next", "Implement optimizer", "--resolves", draftId], {
      cwd: projectDir,
      env,
      encoding: "utf-8",
    });
    assert.ok(cpOut.includes("[OK] Checkpoint settled"));
    assert.ok(cpOut.includes("Auto-resolved & dropped 1 superseded observation"));

    // 4. om pin with auto-dedup
    const pin1Out = execFileSync("node", [omBin, "pin", "Repo requires strict typescript and esbuild"], {
      cwd: projectDir,
      env,
      encoding: "utf-8",
    });
    assert.ok(pin1Out.includes("[OK] Pinned new reflection"));
    const pin1IdMatch = pin1Out.match(/\[([0-9a-f]{12})\]/);
    assert.ok(pin1IdMatch);
    const pin1Id = pin1IdMatch[1];

    const pin2Out = execFileSync("node", [omBin, "pin", "Repo requires strict typescript and esbuild bundler"], {
      cwd: projectDir,
      env,
      encoding: "utf-8",
    });
    assert.ok(pin2Out.includes("Deduplicated & updated existing reflection"));

    // 5. om unpin
    const unpinOut = execFileSync("node", [omBin, "unpin", pin1Id], {
      cwd: projectDir,
      env,
      encoding: "utf-8",
    });
    assert.ok(unpinOut.includes("[OK] Unpinned 1 reflection"));
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});




