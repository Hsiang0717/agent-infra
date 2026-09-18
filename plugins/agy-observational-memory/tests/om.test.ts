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


