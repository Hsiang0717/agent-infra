import * as fs from "node:fs";
import * as path from "node:path";
import type { Observation, Reflection, SessionLedger, ProjectBaseline, Relevance, StoragePaths, FocusState } from "./types.js";
import { estimateStringTokens } from "./tokens.js";
import { hashId } from "./ids.js";
import { calculateSimilarity, DEDUPLICATION_SIMILARITY_THRESHOLD } from "./similarity.js";

const CONTEXT_USAGE_INSTRUCTIONS = `These are condensed memories from earlier in this session.

- Current Focus: active task target and immediate next action.
- Reflections: stable, long-lived facts about the user, project, decisions, and constraints.
- Observations: timestamped events from the conversation history, in chronological order. Observation lines include ids in brackets.

Treat these as past records. When entries conflict, the most recent observation reflects the latest known state. Work that prior observations describe as completed should not be redone unless the user explicitly asks to revisit it.

When exact source context is needed for precision or traceability, use 'om recall <id>' with the relevant observation or reflection id.

Autonomous Memory Protocol & Trigger Rules:
- On Milestone / Verification Pass: Run 'om checkpoint "<milestone>" --next "<next_action>"' or 'om record "<summary>" -r high [--resolves <ids>]'.
- On Invariant / Constraint Finalized: Run 'om pin "<durable_rule>"' (automatically deduplicated/merged) or 'om pin "<rule>" --replace <id>'.
- On Rule Deprecated: Run 'om unpin <id1> [id2 ...]'.
- On Obsolete / Superseded Tasks: Pass '--resolves <id1,id2>' or run 'om drop <id1> <id2> ...'.
- On Session Handoff / Clear: Update 'om focus "<goal>" --next "<action>"' to guarantee seamless cold-start continuity.`;

export function renderSummary(reflections: Reflection[], observations: Observation[], focus?: FocusState): string {
  const hasReflections = reflections.length > 0;
  const hasObservations = observations.length > 0;
  const hasFocus = !!(focus && (focus.goal || focus.nextAction));

  if (!hasReflections && !hasObservations && !hasFocus) return "";

  const parts: string[] = [CONTEXT_USAGE_INSTRUCTIONS];

  if (hasFocus && focus) {
    const focusLines: string[] = [];
    if (focus.goal) focusLines.push(`- Goal: ${focus.goal}`);
    if (focus.nextAction) focusLines.push(`- Next Action: ${focus.nextAction}`);
    parts.push(`## Current Focus & Next Steps\n${focusLines.join("\n")}`);
  }

  if (hasReflections) {
    const reflectionLines = reflections
      .map((r) => `[${r.id}] ${r.content}`)
      .join("\n");
    parts.push(`## Reflections\n${reflectionLines}`);
  }

  if (hasObservations) {
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

export interface PinReflectionResult extends Reflection {
  action: "created" | "replaced" | "merged";
  replacedId?: string;
  similarityScore?: number;
}

export interface RecordObservationResult extends Observation {
  droppedIds?: string[];
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

    let mergedFocus: FocusState | undefined = undefined;
    if (base.focus && incoming.focus) {
      mergedFocus = (base.focus.updatedAt || "") >= (incoming.focus.updatedAt || "") ? base.focus : incoming.focus;
    } else {
      mergedFocus = incoming.focus || base.focus;
    }

    const merged: SessionLedger = {
      version: Math.max(base.version || 1, incoming.version || 1),
      conversationId: incoming.conversationId || base.conversationId,
      workspacePath: incoming.workspacePath || base.workspacePath,
      createdAt: base.createdAt || incoming.createdAt,
      updatedAt: new Date().toISOString(),
      focus: mergedFocus,
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

        const summary = renderSummary(currentLedger.reflections, currentLedger.activeObservations, currentLedger.focus);
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

        const summary = renderSummary(currentLedger.reflections, currentLedger.activeObservations, currentLedger.focus);
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

  setFocus(conversationId: string, goal: string, nextAction: string = ""): FocusState {
    const ledger = this.loadLedger(conversationId);
    const focus: FocusState = {
      goal,
      nextAction,
      updatedAt: new Date().toISOString(),
    };
    ledger.focus = focus;
    this.saveLedger(ledger);
    return focus;
  }

  clearFocus(conversationId: string): void {
    const ledger = this.loadLedger(conversationId);
    delete ledger.focus;
    this.saveLedger(ledger);
  }

  recordObservation(
    conversationId: string,
    content: string,
    relevance: Relevance = "medium",
    sourceStepIndices: number[] = [],
    resolvesIds: string[] = []
  ): RecordObservationResult {
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

    const droppedIds: string[] = [];
    if (resolvesIds.length > 0) {
      const resolvesSet = new Set(resolvesIds);
      ledger.activeObservations = ledger.activeObservations.filter((o) => {
        if (resolvesSet.has(o.id)) {
          droppedIds.push(o.id);
          return false;
        }
        return true;
      });
      ledger.droppedObservationIds.push(...droppedIds);
    }

    ledger.activeObservations.push(obs);
    ledger.allObservations.push(obs);
    this.saveLedger(ledger);

    return Object.assign(obs, { droppedIds });
  }

  pinReflection(
    conversationId: string,
    content: string,
    supportingObservationIdsOrOptions?: string[] | {
      replaceId?: string;
      supportingObservationIds?: string[];
      autoDeduplicate?: boolean;
    }
  ): PinReflectionResult {
    const ledger = this.loadLedger(conversationId);
    const tokenCount = estimateStringTokens(content) + 5;

    let replaceId: string | undefined;
    let supportingObservationIds: string[] = [];
    let autoDeduplicate: boolean = true;

    if (Array.isArray(supportingObservationIdsOrOptions)) {
      supportingObservationIds = supportingObservationIdsOrOptions;
    } else if (supportingObservationIdsOrOptions && typeof supportingObservationIdsOrOptions === "object") {
      replaceId = supportingObservationIdsOrOptions.replaceId;
      supportingObservationIds = supportingObservationIdsOrOptions.supportingObservationIds || [];
      autoDeduplicate = supportingObservationIdsOrOptions.autoDeduplicate !== false;
    }

    // 1. Explicit replacement via replaceId
    if (replaceId) {
      const existingIdx = ledger.reflections.findIndex((r) => r.id === replaceId);
      if (existingIdx !== -1) {
        const oldRef = ledger.reflections[existingIdx];
        const mergedSupp = Array.from(new Set([...(oldRef.supportingObservationIds || []), ...supportingObservationIds]));
        const updatedRef: Reflection = {
          id: replaceId,
          content,
          supportingObservationIds: mergedSupp,
          tokenCount,
        };
        ledger.reflections[existingIdx] = updatedRef;
        this.saveLedger(ledger);
        return Object.assign(updatedRef, {
          action: "replaced" as const,
          replacedId: replaceId,
        });
      }
    }

    // 2. Automatic Deduplication / Merging using CJK N-Gram + Word Similarity
    if (autoDeduplicate && ledger.reflections.length > 0) {
      let maxScore = 0;
      let bestMatch: Reflection | null = null;
      let bestMatchIdx = -1;

      for (let i = 0; i < ledger.reflections.length; i++) {
        const existing = ledger.reflections[i];
        const score = calculateSimilarity(content, existing.content);
        if (score > maxScore) {
          maxScore = score;
          bestMatch = existing;
          bestMatchIdx = i;
        }
      }

      if (bestMatch && maxScore >= DEDUPLICATION_SIMILARITY_THRESHOLD) {
        const mergedSupp = Array.from(new Set([...(bestMatch.supportingObservationIds || []), ...supportingObservationIds]));
        const updatedRef: Reflection = {
          id: bestMatch.id,
          content,
          supportingObservationIds: mergedSupp,
          tokenCount,
        };
        ledger.reflections[bestMatchIdx] = updatedRef;
        this.saveLedger(ledger);
        return Object.assign(updatedRef, {
          action: "merged" as const,
          replacedId: bestMatch.id,
          similarityScore: maxScore,
        });
      }
    }

    // 3. New reflection creation
    const id = hashId(`pin-${content}-${Date.now()}`);
    const ref: Reflection = {
      id,
      content,
      supportingObservationIds,
      tokenCount,
    };

    ledger.reflections.push(ref);
    this.saveLedger(ledger);

    return Object.assign(ref, { action: "created" as const });
  }

  unpinReflections(conversationId: string, ids: string[]): { unpinned: string[]; remaining: number } {
    const ledger = this.loadLedger(conversationId);
    const idSet = new Set(ids);
    const unpinned: string[] = [];

    ledger.reflections = ledger.reflections.filter((r) => {
      if (idSet.has(r.id)) {
        unpinned.push(r.id);
        return false;
      }
      return true;
    });

    this.saveLedger(ledger);

    // Also remove from project baseline file if present
    if (this.projectBaselinePath && fs.existsSync(this.projectBaselinePath)) {
      try {
        const raw = fs.readFileSync(this.projectBaselinePath, "utf-8");
        const baseline: ProjectBaseline = JSON.parse(raw);
        if (Array.isArray(baseline.reflections)) {
          baseline.reflections = baseline.reflections.filter((r) => !idSet.has(r.id));
          baseline.updatedAt = new Date().toISOString();
          atomicWriteFileSync(this.projectBaselinePath, JSON.stringify(baseline, null, 2));
        }
      } catch {}
    }

    return {
      unpinned,
      remaining: ledger.reflections.length,
    };
  }

  checkpoint(
    conversationId: string,
    milestone: string,
    nextAction: string,
    options: {
      relevance?: Relevance;
      resolvesIds?: string[];
      sourceStepIndices?: number[];
    } = {}
  ): { observation: Observation; focus: FocusState; droppedIds: string[] } {
    const { relevance = "high", resolvesIds = [], sourceStepIndices = [] } = options;
    const obs = this.recordObservation(
      conversationId,
      milestone,
      relevance,
      sourceStepIndices,
      resolvesIds
    );
    const focus = this.setFocus(conversationId, milestone, nextAction);
    return {
      observation: obs,
      focus,
      droppedIds: obs.droppedIds || [],
    };
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
    if (ledger.reflections.length > 0 || ledger.activeObservations.length > 0 || (ledger.focus && (ledger.focus.goal || ledger.focus.nextAction))) {
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
