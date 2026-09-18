import * as path from "node:path";
import * as fs from "node:fs";
import { findWorkspaceRoot, resolveStoragePaths } from "./config.js";
import { StorageManager } from "./storage.js";
import type { Observation, Reflection, Relevance, ProjectBaseline } from "./types.js";

function printUsage() {
  console.log(`
Observational Memory CLI (Antigravity Skill-First)

Usage:
  om status [workspace] [--json]                 Display memory stats, counts, and active storage
  om view [workspace] [--session <id>]           View active folded memory markdown projection
  om record "<text>" [-r <level>] [--session <id>] Instantly record an atomic observation
  om pin "<text>" [--session <id>]               Instantly pin a durable project-level reflection
  om recall <id>                                 Deterministically recall details for a 12-char ID
  om drop <id1> [id2 ...] [--session <id>]       Prune specific observation IDs from active memory
  om hook pre-invocation                         PreInvocation lifecycle hook context injector
  om clear [workspace] [--session <id>]          Clear session observations (keeps reflections)
  om clear [workspace] --all                     Clear all memory including project reflections

Relevance levels for record:
  low, medium, high, critical (default: medium)

Examples:
  om status
  om view
  om record "Selected PostgreSQL over MySQL for JSONB support" -r high
  om pin "Project uses Bun runtime and Vitest for testing"
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
  const explicitRelevance = (getFlagValue(["--relevance", "-r"]) || "medium") as Relevance;
  const isJson = args.includes("--json");
  const isAll = args.includes("--all");

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
        // Intermediate tool execution turns (invocationNum > 0) skip injection to save tokens and eliminate latency.
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

    if (fs.existsSync(storage.ledgerPath)) {
      try {
        const ledger = JSON.parse(fs.readFileSync(storage.ledgerPath, "utf-8"));
        activeObsCount = ledger.activeObservations?.length || 0;
        totalRecorded = ledger.allObservations?.length || 0;
        reflectionsCount = ledger.reflections?.length || 0;
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
            workspaceRoot: paths.workspaceRoot || null,
            workspaceHash: paths.workspaceHash,
            conversationId,
            projectBaselinePath: paths.projectReflectionsPath || null,
            activeLedgerPath: paths.activeLedgerPath,
            sessionStoreDir: paths.sessionStoreDir,
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

    console.log("=== Observational Memory Status ===");
    console.log(`Architecture:               Skill-First (Zero-Config)`);
    console.log(`Workspace Root:             ${paths.workspaceRoot || "(none - global mode)"}`);
    console.log(`Active Session ID:          ${conversationId}`);
    console.log(`Project Baseline Path:      ${paths.projectReflectionsPath || "(none)"}`);
    console.log(`Active Workspace Ledger:    ${paths.activeLedgerPath}`);
    console.log(`Active Observations:        ${activeObsCount} items`);
    console.log(`Total Recorded Observations:${totalRecorded} items`);
    console.log(`Session Reflections:        ${reflectionsCount} items`);
    console.log(`Project Baseline Facts:     ${baselineCount} items`);
    console.log("===================================");
    return;
  }

  if (command === "view") {
    const projection = storage.readProjection();
    if (!projection) {
      console.log(`No active observations or reflections recorded yet for session [${conversationId}].`);
      console.log(`Use 'om record "<text>"' or 'om pin "<text>"' to add memories.`);
      return;
    }
    console.log(`=== Active Folded Memory Projection [${conversationId}] ===\n`);
    console.log(projection);
    console.log("\n==========================================================");
    return;
  }

  if (command === "record") {
    const text = nonFlagArgs[1];
    if (!text) {
      console.error("Error: Missing text for record. Usage: om record \"<text>\" [-r <level>]");
      process.exit(1);
    }

    const obs = storage.recordObservation(conversationId, text, explicitRelevance);
    console.log(`[OK] Recorded observation [${obs.id}] (${obs.relevance}): "${obs.content}"`);
    return;
  }

  if (command === "pin") {
    const text = nonFlagArgs[1];
    if (!text) {
      console.error("Error: Missing text for pin. Usage: om pin \"<text>\"");
      process.exit(1);
    }

    const ref = storage.pinReflection(conversationId, text);
    console.log(`[OK] Pinned reflection [${ref.id}]: "${ref.content}"`);
    if (paths.projectReflectionsPath) {
      console.log(`Synced to: ${paths.projectReflectionsPath}`);
    }
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
