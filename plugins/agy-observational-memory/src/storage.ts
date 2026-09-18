import * as fs from "node:fs";
import * as path from "node:path";
import type { Observation, Reflection, SessionLedger, ProjectBaseline, Relevance, StoragePaths } from "./types.js";
import { estimateStringTokens } from "./tokens.js";
import { hashId } from "./ids.js";

const CONTEXT_USAGE_INSTRUCTIONS = `These are condensed memories from earlier in this session.

- Reflections: stable, long-lived facts about the user, project, decisions, and constraints.
- Observations: timestamped events from the conversation history, in chronological order. Observation lines include ids in brackets.

Treat these as past records. When entries conflict, the most recent observation reflects the latest known state. Work that prior observations describe as completed should not be redone unless the user explicitly asks to revisit it.

When exact source context is needed for precision or traceability, use 'om recall <id>' with the relevant observation or reflection id.

Autonomous Memory Protocol:
- Record: When a milestone or verification is completed, record via 'om record "<summary>" -r <level>'.
- Prune: When prior observations are superseded, resolved, or refuted, proactively drop clutter via 'om drop <id1> <id2> ...'.
- Guardrail: Never drop failure lessons (synthesize them to reflections first); never drop steps of ongoing, unverified tasks.
- Synthesize: When multiple observations converge into a stable pattern or invariant, promote to durable reflection via 'om pin "<rule>"'.`;

export function renderSummary(reflections: Reflection[], observations: Observation[]): string {
  if (reflections.length === 0 && observations.length === 0) return "";

  const parts: string[] = [CONTEXT_USAGE_INSTRUCTIONS];
  if (reflections.length > 0) {
    const reflectionLines = reflections
      .map((r) => `[${r.id}] ${r.content}`)
      .join("\n");
    parts.push(`## Reflections\n${reflectionLines}`);
  }
  if (observations.length > 0) {
    const observationLines = observations
      .map((o) => `[${o.id}] ${o.timestamp} [${o.relevance}] ${o.content}`)
      .join("\n");
    parts.push(`## Observations\n${observationLines}`);
  }
  return parts.join("\n\n");
}

function atomicWriteFileSync(targetPath: string, content: string): void {
  const dir = path.dirname(targetPath);
  if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
  const tmpPath = `${targetPath}.tmp.${Date.now()}.${Math.random().toString(36).substring(2, 8)}`;
  try {
    fs.writeFileSync(tmpPath, content, "utf-8");
    fs.renameSync(tmpPath, targetPath);
  } catch {
    fs.writeFileSync(targetPath, content, "utf-8");
    if (fs.existsSync(tmpPath)) {
      try {
        fs.unlinkSync(tmpPath);
      } catch {}
    }
  }
}

export class StorageManager {
  static readonly MAX_ACTIVE_OBSERVATIONS = 20;

  readonly storeDir: string;
  readonly projectBaselinePath?: string;
  readonly workspaceRoot?: string;
  readonly activeLedgerPath: string;
  readonly activeProjectionPath: string;

  constructor(
    storeDirOrPaths: string | StoragePaths,
    projectBaselinePathOrIsWorkspace?: string | boolean,
    workspaceRoot?: string
  ) {
    if (typeof storeDirOrPaths === "object") {
      this.storeDir = storeDirOrPaths.sessionStoreDir;
      this.projectBaselinePath = storeDirOrPaths.projectReflectionsPath;
      this.workspaceRoot = storeDirOrPaths.workspaceRoot;
      this.activeLedgerPath = storeDirOrPaths.activeLedgerPath || path.join(storeDirOrPaths.sessionStoreDir, "ledger.json");
      this.activeProjectionPath = storeDirOrPaths.activeProjectionPath || path.join(storeDirOrPaths.sessionStoreDir, "folded_projection.md");
    } else {
      this.storeDir = storeDirOrPaths;
      if (typeof projectBaselinePathOrIsWorkspace === "string") {
        this.projectBaselinePath = projectBaselinePathOrIsWorkspace;
      } else if (projectBaselinePathOrIsWorkspace === true && workspaceRoot) {
        this.projectBaselinePath = path.join(workspaceRoot, ".agents", "memory", "reflections.json");
      }
      this.workspaceRoot = workspaceRoot;
      this.activeLedgerPath = path.join(this.storeDir, "ledger.json");
      this.activeProjectionPath = path.join(this.storeDir, "folded_projection.md");
    }
  }

  get ledgerPath(): string {
    return this.activeLedgerPath;
  }

  get projectionPath(): string {
    return this.activeProjectionPath;
  }

  loadLedger(conversationId: string): SessionLedger {
    if (fs.existsSync(this.ledgerPath)) {
      try {
        const raw = fs.readFileSync(this.ledgerPath, "utf-8");
        const parsed = JSON.parse(raw);
        if (parsed && typeof parsed === "object") {
          if (typeof parsed.version !== "number") parsed.version = 1;
          return parsed;
        }
      } catch (e) {
        // Fallback to fresh ledger
      }
    }

    // New Ledger initialization
    let initialReflections: Reflection[] = [];
    if (this.projectBaselinePath && fs.existsSync(this.projectBaselinePath)) {
      try {
        const baselineRaw = fs.readFileSync(this.projectBaselinePath, "utf-8");
        const baseline: ProjectBaseline = JSON.parse(baselineRaw);
        initialReflections = baseline.reflections || [];
      } catch {
        // Ignore
      }
    }

    const now = new Date().toISOString();
    const ledger: SessionLedger = {
      version: 0,
      conversationId,
      workspacePath: this.workspaceRoot,
      createdAt: now,
      updatedAt: now,
      activeObservations: [],
      allObservations: [],
      reflections: initialReflections,
      droppedObservationIds: [],
    };

    this.saveLedger(ledger);
    return ledger;
  }

  trimActiveObservations(observations: Observation[]): Observation[] {
    if (observations.length <= StorageManager.MAX_ACTIVE_OBSERVATIONS) {
      return observations;
    }
    const criticalOrHigh: Observation[] = [];
    const other: Observation[] = [];
    for (const o of observations) {
      if (o.relevance === "critical" || o.relevance === "high") {
        criticalOrHigh.push(o);
      } else {
        other.push(o);
      }
    }

    const slotsForOther = Math.max(0, StorageManager.MAX_ACTIVE_OBSERVATIONS - criticalOrHigh.length);
    const keptOther = other.slice(-slotsForOther);
    const combined = [...criticalOrHigh, ...keptOther];
    combined.sort((a, b) => a.timestamp.localeCompare(b.timestamp));
    return combined.slice(-StorageManager.MAX_ACTIVE_OBSERVATIONS);
  }

  mergeLedgers(base: SessionLedger, incoming: SessionLedger): SessionLedger {
    const droppedSet = new Set([
      ...(base.droppedObservationIds || []),
      ...(incoming.droppedObservationIds || []),
    ]);

    const activeMap = new Map<string, Observation>();
    for (const obs of base.activeObservations || []) {
      if (!droppedSet.has(obs.id)) activeMap.set(obs.id, obs);
    }
    for (const obs of incoming.activeObservations || []) {
      if (!droppedSet.has(obs.id)) activeMap.set(obs.id, obs);
    }

    const allMap = new Map<string, Observation>();
    for (const obs of base.allObservations || []) {
      allMap.set(obs.id, obs);
    }
    for (const obs of incoming.allObservations || []) {
      allMap.set(obs.id, obs);
    }

    const refMap = new Map<string, Reflection>();
    for (const ref of base.reflections || []) {
      refMap.set(ref.id, ref);
    }
    for (const ref of incoming.reflections || []) {
      refMap.set(ref.id, ref);
    }

    const merged: SessionLedger = {
      version: Math.max(base.version || 1, incoming.version || 1),
      conversationId: incoming.conversationId || base.conversationId,
      workspacePath: incoming.workspacePath || base.workspacePath,
      createdAt: base.createdAt || incoming.createdAt,
      updatedAt: new Date().toISOString(),
      activeObservations: Array.from(activeMap.values()),
      allObservations: Array.from(allMap.values()),
      reflections: Array.from(refMap.values()),
      droppedObservationIds: Array.from(droppedSet),
    };

    return merged;
  }

  saveLedger(ledger: SessionLedger, maxRetries: number = 3): void {
    let currentLedger = ledger;
    currentLedger.activeObservations = this.trimActiveObservations(currentLedger.activeObservations);

    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      let diskLedger: SessionLedger | null = null;
      if (fs.existsSync(this.ledgerPath)) {
        try {
          const raw = fs.readFileSync(this.ledgerPath, "utf-8");
          diskLedger = JSON.parse(raw);
        } catch {
          diskLedger = null;
        }
      }

      // If disk does not exist or version matches, safe to commit
      if (!diskLedger || typeof diskLedger.version !== "number" || diskLedger.version === currentLedger.version) {
        currentLedger.version = (currentLedger.version || 0) + 1;
        currentLedger.updatedAt = new Date().toISOString();

        atomicWriteFileSync(this.ledgerPath, JSON.stringify(currentLedger, null, 2));

        const summary = renderSummary(currentLedger.reflections, currentLedger.activeObservations);
        atomicWriteFileSync(this.projectionPath, summary);

        this.syncProjectBaseline(currentLedger);
        return;
      }

      // Optimistic lock conflict detected: diskLedger.version !== currentLedger.version
      currentLedger = this.mergeLedgers(diskLedger, currentLedger);
      currentLedger.activeObservations = this.trimActiveObservations(currentLedger.activeObservations);

      if (attempt === maxRetries) {
        currentLedger.version = (diskLedger.version || 0) + 1;
        currentLedger.updatedAt = new Date().toISOString();
        atomicWriteFileSync(this.ledgerPath, JSON.stringify(currentLedger, null, 2));

        const summary = renderSummary(currentLedger.reflections, currentLedger.activeObservations);
        atomicWriteFileSync(this.projectionPath, summary);

        this.syncProjectBaseline(currentLedger);
        return;
      }

      const jitterMs = 10 + Math.floor(Math.random() * 20);
      try {
        Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, jitterMs);
      } catch {}
    }
  }

  private syncProjectBaseline(ledger: SessionLedger): void {
    if (this.projectBaselinePath && ledger.reflections.length > 0) {
      let mergedReflections: Reflection[] = [...ledger.reflections];
      if (fs.existsSync(this.projectBaselinePath)) {
        try {
          const raw = fs.readFileSync(this.projectBaselinePath, "utf-8");
          const existing: ProjectBaseline = JSON.parse(raw);
          if (Array.isArray(existing.reflections)) {
            const existingMap = new Map<string, Reflection>();
            for (const r of existing.reflections) {
              if (r && r.id) existingMap.set(r.id, r);
            }
            for (const r of ledger.reflections) {
              existingMap.set(r.id, r);
            }
            mergedReflections = Array.from(existingMap.values());
          }
        } catch {
          // fallback to ledger.reflections
        }
      }

      const baseline: ProjectBaseline = {
        workspacePath: this.workspaceRoot || "",
        updatedAt: ledger.updatedAt,
        reflections: mergedReflections,
      };

      atomicWriteFileSync(this.projectBaselinePath, JSON.stringify(baseline, null, 2));
    }
  }

  recordObservation(
    conversationId: string,
    content: string,
    relevance: Relevance = "medium",
    sourceStepIndices: number[] = []
  ): Observation {
    const ledger = this.loadLedger(conversationId);
    const now = new Date();
    const pad = (n: number) => n.toString().padStart(2, "0");
    const timestamp = `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())} ${pad(now.getHours())}:${pad(now.getMinutes())}`;
    
    const id = hashId(`${content}-${Date.now()}-${Math.random()}`);
    const tokenCount = estimateStringTokens(content) + 10;

    const obs: Observation = {
      id,
      content,
      timestamp,
      relevance,
      sourceStepIndices,
      tokenCount,
      conversationId,
    };

    ledger.activeObservations.push(obs);
    ledger.allObservations.push(obs);
    this.saveLedger(ledger);

    return obs;
  }

  pinReflection(
    conversationId: string,
    content: string,
    supportingObservationIds: string[] = []
  ): Reflection {
    const ledger = this.loadLedger(conversationId);
    const id = hashId(`pin-${content}-${Date.now()}`);
    const tokenCount = estimateStringTokens(content) + 5;

    const ref: Reflection = {
      id,
      content,
      supportingObservationIds,
      tokenCount,
    };

    ledger.reflections.push(ref);
    this.saveLedger(ledger);

    return ref;
  }

  dropObservations(
    conversationId: string,
    ids: string[]
  ): { remaining: number; dropped: string[] } {
    const ledger = this.loadLedger(conversationId);
    const idSet = new Set(ids);
    const dropped: string[] = [];

    ledger.activeObservations = ledger.activeObservations.filter((o) => {
      if (idSet.has(o.id)) {
        dropped.push(o.id);
        return false;
      }
      return true;
    });

    ledger.droppedObservationIds.push(...dropped);
    this.saveLedger(ledger);

    return {
      remaining: ledger.activeObservations.length,
      dropped,
    };
  }

  readProjection(): string {
    if (fs.existsSync(this.projectionPath)) {
      return fs.readFileSync(this.projectionPath, "utf-8").trim();
    }
    return "";
  }

  getOrInitProjection(conversationId: string): string {
    if (fs.existsSync(this.projectionPath)) {
      return fs.readFileSync(this.projectionPath, "utf-8").trim();
    }
    const ledger = this.loadLedger(conversationId);
    if (ledger.reflections.length > 0 || ledger.activeObservations.length > 0) {
      if (fs.existsSync(this.projectionPath)) {
        return fs.readFileSync(this.projectionPath, "utf-8").trim();
      }
    }
    return "";
  }

  findItemById(id: string): { type: "observation" | "reflection"; item: Observation | Reflection } | undefined {
    if (fs.existsSync(this.ledgerPath)) {
      try {
        const ledger: SessionLedger = JSON.parse(fs.readFileSync(this.ledgerPath, "utf-8"));
        const obs = ledger.allObservations.find((o) => o.id === id);
        if (obs) return { type: "observation", item: obs };

        const ref = ledger.reflections.find((r) => r.id === id);
        if (ref) return { type: "reflection", item: ref };
      } catch {
        // Ignore
      }
    }

    if (this.projectBaselinePath && fs.existsSync(this.projectBaselinePath)) {
      try {
        const baseline: ProjectBaseline = JSON.parse(fs.readFileSync(this.projectBaselinePath, "utf-8"));
        if (Array.isArray(baseline.reflections)) {
          const ref = baseline.reflections.find((r) => r.id === id);
          if (ref) return { type: "reflection", item: ref };
        }
      } catch {
        // Ignore
      }
    }

    return undefined;
  }
}
