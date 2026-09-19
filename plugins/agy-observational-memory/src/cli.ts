import * as path from "node:path";
import * as fs from "node:fs";
import { findWorkspaceRoot, resolveStoragePaths } from "./config.js";
import { StorageManager } from "./storage.js";
import type { Observation, Reflection, Relevance, ProjectBaseline, FocusState } from "./types.js";

function printUsage() {
  console.log(`
Observational Memory CLI 2.0 (Antigravity Skill-First)

Usage:
  om status [workspace] [--json]                 Display memory stats, counts, and active storage
  om view [workspace] [--session <id>]           View active folded memory markdown projection
  om focus "<target>" [--next "<action>"]        Set or update current focus target & next step
  om focus --clear                               Clear current focus target & next step
  om checkpoint "<milestone>" --next "<action>"  Atomic settlement: record + update focus [+ resolves]
  om record "<text>" [-r <level>] [--resolves <id1,id2>] Record an atomic observation & prune superseded
  om pin "<text>" [--replace <id>]               Pin a durable reflection (auto-deduplicates & merges)
  om unpin <id1> [id2 ...]                       Remove durable project reflection(s)
  om recall <id>                                 Deterministically recall details for a 12-char ID
  om drop <id1> [id2 ...] [--session <id>]       Prune specific observation IDs from active memory
  om hook pre-invocation                         PreInvocation lifecycle hook context injector
  om clear [workspace] [--session <id>]          Clear session observations (keeps reflections)
  om clear [workspace] --all                     Clear all memory including project reflections

Relevance levels for record / checkpoint:
  low, medium, high, critical (default for record: medium, default for checkpoint: high)

Examples:
  om status
  om view
  om focus "Implementing AST validation" --next "Run unit test suite"
  om checkpoint "AST Validator v2 implemented" --next "Write property tests" --resolves d4e5f6a1b2c3
  om record "Selected PostgreSQL over MySQL for JSONB support" -r high
  om record "Refactored parser to AST" --resolves 23109baf1fbe,023ff2c2ab32
  om pin "Project uses Bun runtime and Vitest for testing"
  om pin "Project uses Bun runtime and Vitest with coverage" --replace a1b2c3d4e5f6
  om unpin a1b2c3d4e5f6
  om recall d4e5f6a1b2c3
  om drop d4e5f6a1b2c3 e5f6a1b2c3d4
  om clear
  om clear --all
`);
}

function resolveSessionId(wsBase: string, explicitSession?: string): string {
  if (explicitSession) return explicitSession;
  if (process.env.AGY_CONVERSATION_ID) return process.env.AGY_CONVERSATION_ID;

  if (fs.existsSync(wsBase)) {
    try {
      const entries = fs.readdirSync(wsBase, { withFileTypes: true });
      const sessionDirs = entries
        .filter((e) => e.isDirectory())
        .map((e) => {
          const fullPath = path.join(wsBase, e.name);
          return { name: e.name, mtime: fs.statSync(fullPath).mtimeMs };
        })
        .sort((a, b) => b.mtime - a.mtime);

      if (sessionDirs.length > 0) {
        return sessionDirs[0].name;
      }
    } catch {}
  }

  return "active-session";
}

async function readStdin(timeoutMs: number = 2000): Promise<string> {
  if (process.stdin.isTTY) return "";
  return new Promise((resolve) => {
    const chunks: Buffer[] = [];
    const timer = setTimeout(() => {
      resolve(Buffer.concat(chunks).toString("utf-8"));
    }, timeoutMs);

    process.stdin.on("data", (chunk) => {
      chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
    });
    process.stdin.on("end", () => {
      clearTimeout(timer);
      resolve(Buffer.concat(chunks).toString("utf-8"));
    });
    process.stdin.on("error", () => {
      clearTimeout(timer);
      resolve("");
    });
  });
}

export async function runCli(): Promise<void> {
  const args = process.argv.slice(2);
  const command = args[0] || "status";

  if (command === "help" || command === "--help" || command === "-h") {
    printUsage();
    return;
  }

  // Parse common flags
  const getFlagValue = (flagNames: string[]): string | undefined => {
    for (const name of flagNames) {
      const idx = args.indexOf(name);
      if (idx !== -1 && idx + 1 < args.length) return args[idx + 1];
    }
    return undefined;
  };

  const explicitSession = getFlagValue(["--session", "-s"]);
  const explicitRelevance = getFlagValue(["--relevance", "-r"]) as Relevance | undefined;
  const nextActionFlag = getFlagValue(["--next", "-n"]);
  const replaceIdFlag = getFlagValue(["--replace"]);
  const resolvesFlag = getFlagValue(["--resolves"]);
  const isJson = args.includes("--json");
  const isAll = args.includes("--all");
  const isClearFlag = args.includes("--clear");

  const parsedResolvesIds: string[] = resolvesFlag
    ? resolvesFlag.split(",").map((s) => s.trim()).filter(Boolean)
    : [];

  // Positional workspace path if passed
  const nonFlagArgs = args.filter((a) => !a.startsWith("-"));
  let explicitWorkspace: string | undefined;
  if (["status", "view", "clear"].includes(command) && nonFlagArgs.length > 1) {
    explicitWorkspace = nonFlagArgs[1];
  }

  // Hook handler for Antigravity lifecycle
  if (command === "hook") {
    const hookSubcommand = nonFlagArgs[1] || "pre-invocation";
    if (hookSubcommand === "pre-invocation") {
      try {
        const rawStdin = await readStdin();
        let hookInput: {
          conversationId?: string;
          workspacePaths?: string[];
          invocationNum?: number;
        } = {};
        if (rawStdin && rawStdin.trim()) {
          try {
            const cleaned = rawStdin.replace(/^\uFEFF/, "").trim();
            hookInput = JSON.parse(cleaned);
          } catch {}
        }

        // Optimization: Only inject on User Turn (invocationNum === 0 or undefined).
        if (typeof hookInput.invocationNum === "number" && hookInput.invocationNum > 0) {
          console.log(JSON.stringify({ injectSteps: [] }));
          return;
        }

        const wsRoot =
          Array.isArray(hookInput.workspacePaths) && hookInput.workspacePaths[0]
            ? hookInput.workspacePaths[0]
            : findWorkspaceRoot(process.cwd());

        const convId = hookInput.conversationId || "active-session";
        const hookPaths = resolveStoragePaths(convId, wsRoot);
        const hookStorage = new StorageManager(hookPaths);

        const projection = hookStorage.getOrInitProjection(convId);
        if (projection && projection.trim().length > 0) {
          const ephemeralMessage = `<observational_memory>\n${projection}\n</observational_memory>`;
          console.log(
            JSON.stringify({
              injectSteps: [
                {
                  ephemeralMessage,
                },
              ],
            })
          );
        } else {
          console.log(JSON.stringify({ injectSteps: [] }));
        }
      } catch {
        console.log(JSON.stringify({ injectSteps: [] }));
      }
      return;
    }
  }

  // Resolve zero-config paths
  const workspaceRoot = findWorkspaceRoot(explicitWorkspace || process.cwd());
  const initialPaths = resolveStoragePaths("temp", workspaceRoot);
  const baseCacheDir = path.dirname(initialPaths.sessionStoreDir);
  const conversationId = resolveSessionId(baseCacheDir, explicitSession);
  const paths = resolveStoragePaths(conversationId, workspaceRoot);

  const storage = new StorageManager(paths);

  if (command === "status") {
    let activeObsCount = 0;
    let totalRecorded = 0;
    let reflectionsCount = 0;
    let currentFocus: FocusState | undefined;

    if (fs.existsSync(storage.ledgerPath)) {
      try {
        const ledger = JSON.parse(fs.readFileSync(storage.ledgerPath, "utf-8"));
        activeObsCount = ledger.activeObservations?.length || 0;
        totalRecorded = ledger.allObservations?.length || 0;
        reflectionsCount = ledger.reflections?.length || 0;
        currentFocus = ledger.focus;
      } catch {}
    }

    let baselineCount = 0;
    if (paths.projectReflectionsPath && fs.existsSync(paths.projectReflectionsPath)) {
      try {
        const baseline: ProjectBaseline = JSON.parse(fs.readFileSync(paths.projectReflectionsPath, "utf-8"));
        baselineCount = baseline.reflections?.length || 0;
      } catch {}
    }

    if (isJson) {
      console.log(
        JSON.stringify(
          {
            architecture: "skill-first",
            version: "2.0.0",
            workspaceRoot: paths.workspaceRoot || null,
            workspaceHash: paths.workspaceHash,
            conversationId,
            projectBaselinePath: paths.projectReflectionsPath || null,
            activeLedgerPath: paths.activeLedgerPath,
            sessionStoreDir: paths.sessionStoreDir,
            focus: currentFocus || null,
            activeObservationsCount: activeObsCount,
            totalRecordedObservations: totalRecorded,
            sessionReflectionsCount: reflectionsCount,
            projectBaselineReflectionsCount: baselineCount,
          },
          null,
          2
        )
      );
      return;
    }

    console.log("=== Observational Memory Status (v2.0) ===");
    console.log(`Architecture:               Skill-First (Zero-Config)`);
    console.log(`Workspace Root:             ${paths.workspaceRoot || "(none - global mode)"}`);
    console.log(`Active Session ID:          ${conversationId}`);
    if (currentFocus) {
      console.log(`Current Target Focus:       ${currentFocus.goal}`);
      console.log(`Immediate Next Action:      ${currentFocus.nextAction || "(none specified)"}`);
    } else {
      console.log(`Current Focus:              (idle / not set)`);
    }
    console.log(`Project Baseline Path:      ${paths.projectReflectionsPath || "(none)"}`);
    console.log(`Active Workspace Ledger:    ${paths.activeLedgerPath}`);
    console.log(`Active Observations:        ${activeObsCount} items`);
    console.log(`Total Recorded Observations:${totalRecorded} items`);
    console.log(`Session Reflections:        ${reflectionsCount} items`);
    console.log(`Project Baseline Facts:     ${baselineCount} items`);
    console.log("==========================================");
    return;
  }

  if (command === "view") {
    const projection = storage.readProjection();
    if (!projection) {
      console.log(`No active observations or reflections recorded yet for session [${conversationId}].`);
      console.log(`Use 'om checkpoint', 'om record', 'om focus', or 'om pin' to add memories.`);
      return;
    }
    console.log(`=== Active Folded Memory Projection [${conversationId}] ===\n`);
    console.log(projection);
    console.log("\n==========================================================");
    return;
  }

  if (command === "focus") {
    if (isClearFlag) {
      storage.clearFocus(conversationId);
      console.log(`[OK] Cleared current focus for session [${conversationId}].`);
      return;
    }

    const targetGoal = nonFlagArgs[1];
    if (!targetGoal) {
      console.error("Error: Missing target for focus. Usage: om focus \"<target>\" [--next \"<action>\"] or om focus --clear");
      process.exit(1);
    }

    const nextAction = nextActionFlag || "";
    const focus = storage.setFocus(conversationId, targetGoal, nextAction);
    console.log(`[OK] Updated Current Focus:`);
    console.log(`  Target: ${focus.goal}`);
    if (focus.nextAction) {
      console.log(`  Next:   ${focus.nextAction}`);
    }
    return;
  }

  if (command === "checkpoint") {
    const milestone = nonFlagArgs[1];
    if (!milestone) {
      console.error("Error: Missing milestone summary for checkpoint. Usage: om checkpoint \"<milestone>\" --next \"<action>\" [--resolves <id1,id2>]");
      process.exit(1);
    }

    const nextAction = nextActionFlag || "";
    const relevance: Relevance = explicitRelevance || "high";

    const result = storage.checkpoint(conversationId, milestone, nextAction, {
      relevance,
      resolvesIds: parsedResolvesIds,
    });

    console.log(`[OK] Checkpoint settled [${result.observation.id}] (${result.observation.relevance}): "${result.observation.content}"`);
    if (result.droppedIds.length > 0) {
      console.log(`  Auto-resolved & dropped ${result.droppedIds.length} superseded observation(s): ${result.droppedIds.map((id) => `[${id}]`).join(", ")}`);
    }
    console.log(`  Active Focus updated -> Next Action: "${result.focus.nextAction || result.focus.goal}"`);
    return;
  }

  if (command === "record") {
    const text = nonFlagArgs[1];
    if (!text) {
      console.error("Error: Missing text for record. Usage: om record \"<text>\" [-r <level>] [--resolves <id1,id2>]");
      process.exit(1);
    }

    const relevance: Relevance = explicitRelevance || "medium";
    const obs = storage.recordObservation(conversationId, text, relevance, [], parsedResolvesIds);
    console.log(`[OK] Recorded observation [${obs.id}] (${obs.relevance}): "${obs.content}"`);
    if (obs.droppedIds && obs.droppedIds.length > 0) {
      console.log(`  Auto-resolved & dropped ${obs.droppedIds.length} superseded observation(s): ${obs.droppedIds.map((id) => `[${id}]`).join(", ")}`);
    }
    return;
  }

  if (command === "pin") {
    const text = nonFlagArgs[1];
    if (!text) {
      console.error("Error: Missing text for pin. Usage: om pin \"<text>\" [--replace <id>]");
      process.exit(1);
    }

    const result = storage.pinReflection(conversationId, text, {
      replaceId: replaceIdFlag,
      autoDeduplicate: true,
    });

    if (result.action === "replaced") {
      console.log(`[OK] Explicitly replaced reflection [${result.id}] with new content: "${result.content}"`);
    } else if (result.action === "merged") {
      console.log(`[OK] Deduplicated & updated existing reflection [${result.id}] (similarity: ${(result.similarityScore! * 100).toFixed(1)}%): "${result.content}"`);
    } else {
      console.log(`[OK] Pinned new reflection [${result.id}]: "${result.content}"`);
    }

    if (paths.projectReflectionsPath) {
      console.log(`Synced to: ${paths.projectReflectionsPath}`);
    }
    return;
  }

  if (command === "unpin") {
    const idsToUnpin = nonFlagArgs.slice(1);
    if (idsToUnpin.length === 0) {
      console.error("Error: Missing IDs to unpin. Usage: om unpin <id1> [id2 ...]");
      process.exit(1);
    }

    const result = storage.unpinReflections(conversationId, idsToUnpin);
    console.log(`[OK] Unpinned ${result.unpinned.length} reflection(s). ${result.remaining} reflection(s) remaining.`);
    return;
  }

  if (command === "drop") {
    const idsToDrop = nonFlagArgs.slice(1);
    if (idsToDrop.length === 0) {
      console.error("Error: Missing IDs to drop. Usage: om drop <id1> [id2 ...]");
      process.exit(1);
    }

    const result = storage.dropObservations(conversationId, idsToDrop);
    console.log(`[OK] Dropped ${result.dropped.length} observation(s). ${result.remaining} active observation(s) remaining.`);
    return;
  }

  if (command === "recall") {
    const id = nonFlagArgs[1];
    if (!id) {
      console.error("Error: Missing ID for recall. Usage: om recall <id>");
      process.exit(1);
    }

    const result = storage.findItemById(id);
    if (!result) {
      console.error(`Record [${id}] not found in active session or project baseline.`);
      process.exit(1);
    }

    if (result.type === "observation") {
      const obs = result.item as Observation;
      console.log(`=== Observation Record [${obs.id}] ===`);
      console.log(`Timestamp:   ${obs.timestamp}`);
      console.log(`Relevance:   ${obs.relevance}`);
      if (obs.conversationId) console.log(`Session:     ${obs.conversationId}`);
      console.log(`Tokens:      ~${obs.tokenCount}`);
      console.log(`Sources:     ${obs.sourceStepIndices?.length ? obs.sourceStepIndices.join(", ") : "(manual)"}`);
      console.log(`Content:     ${obs.content}`);
      console.log("========================================");
    } else {
      const ref = result.item as Reflection;
      console.log(`=== Reflection Record [${ref.id}] ===`);
      console.log(`Tokens:      ~${ref.tokenCount}`);
      console.log(
        `Supporting:  ${
          ref.supportingObservationIds?.length
            ? ref.supportingObservationIds.map((s) => `[${s}]`).join(", ")
            : "(none)"
        }`
      );
      console.log(`Content:     ${ref.content}`);
      console.log("========================================");
    }
    return;
  }

  if (command === "clear") {
    if (isAll) {
      if (paths.projectReflectionsPath && fs.existsSync(paths.projectReflectionsPath)) {
        fs.rmSync(paths.projectReflectionsPath, { force: true });
        console.log(`Cleared project reflections: ${paths.projectReflectionsPath}`);
      }
      if (fs.existsSync(baseCacheDir)) {
        fs.rmSync(baseCacheDir, { recursive: true, force: true });
        console.log(`Cleared session cache: ${baseCacheDir}`);
      }
    } else if (explicitSession) {
      const sessionDir = path.join(baseCacheDir, explicitSession);
      if (fs.existsSync(sessionDir)) {
        fs.rmSync(sessionDir, { recursive: true, force: true });
        console.log(`Cleared memory for session [${explicitSession}]`);
      } else {
        console.log(`Session directory not found: ${sessionDir}`);
      }
    } else {
      if (fs.existsSync(baseCacheDir)) {
        fs.rmSync(baseCacheDir, { recursive: true, force: true });
        console.log(
          `Cleared session observation cache: ${baseCacheDir}\nPreserved project reflections at: ${paths.projectReflectionsPath || "(none)"}`
        );
      } else {
        console.log("No session observations found to clear.");
      }
    }
    return;
  }

  printUsage();
}

runCli().catch((err) => {
  console.error("CLI Error:", err);
  process.exit(1);
});
