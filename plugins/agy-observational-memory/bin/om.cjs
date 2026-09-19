#!/usr/bin/env node
"use strict";
var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
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
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// src/cli.ts
var cli_exports = {};
__export(cli_exports, {
  runCli: () => runCli
});
module.exports = __toCommonJS(cli_exports);
var path3 = __toESM(require("node:path"));
var fs3 = __toESM(require("node:fs"));

// src/config.ts
var fs = __toESM(require("node:fs"));
var path = __toESM(require("node:path"));
var os = __toESM(require("node:os"));
var import_node_crypto = require("node:crypto");
function findWorkspaceRoot(startDir = process.cwd()) {
  let current = path.resolve(startDir);
  let home = path.resolve(os.homedir());
  try {
    home = fs.realpathSync.native(home).toLowerCase();
  } catch {
  }
  while (true) {
    let resolvedCurrent = current.toLowerCase();
    try {
      resolvedCurrent = fs.realpathSync.native(current).toLowerCase();
    } catch {
    }
    if (resolvedCurrent !== home) {
      if (fs.existsSync(path.join(current, ".git")) || fs.existsSync(path.join(current, ".agents"))) {
        return current;
      }
    }
    const parent = path.dirname(current);
    if (parent === current) break;
    current = parent;
  }
  return void 0;
}
function computeWorkspaceHash(workspaceRoot) {
  if (!workspaceRoot) return "global";
  const normalized = path.resolve(workspaceRoot).toLowerCase().replace(/\\/g, "/");
  return (0, import_node_crypto.createHash)("sha256").update(normalized).digest("hex").slice(0, 12);
}
function resolveStoragePaths(conversationId, startDir = process.cwd()) {
  const workspaceRoot = findWorkspaceRoot(startDir);
  const workspaceHash = computeWorkspaceHash(workspaceRoot);
  const baseCacheDir = path.join(os.homedir(), ".gemini", "observational-memory", workspaceHash);
  const sessionStoreDir = path.join(baseCacheDir, conversationId);
  const activeLedgerPath = path.join(baseCacheDir, "active_ledger.json");
  const activeProjectionPath = path.join(baseCacheDir, "active_projection.md");
  let projectReflectionsPath;
  if (workspaceRoot) {
    projectReflectionsPath = path.join(workspaceRoot, ".agents", "memory", "reflections.json");
  } else {
    projectReflectionsPath = path.join(baseCacheDir, "reflections.json");
  }
  return {
    projectReflectionsPath,
    sessionStoreDir,
    activeLedgerPath,
    activeProjectionPath,
    workspaceRoot,
    workspaceHash
  };
}

// src/storage.ts
var fs2 = __toESM(require("node:fs"));
var path2 = __toESM(require("node:path"));

// src/tokens.ts
function estimateStringTokens(text) {
  if (!text) return 0;
  let cjkCount = 0;
  for (let i = 0; i < text.length; i++) {
    const code = text.charCodeAt(i);
    if (code >= 19968 && code <= 40959 || code >= 13312 && code <= 19903 || code >= 131072 && code <= 173791 || code >= 12352 && code <= 12543 || code >= 44032 && code <= 55215) {
      cjkCount++;
    }
  }
  const nonCjkLen = text.length - cjkCount;
  return cjkCount + Math.ceil(nonCjkLen / 4);
}

// src/ids.ts
var import_node_crypto2 = require("node:crypto");
function hashId(content) {
  return (0, import_node_crypto2.createHash)("sha256").update(content).digest("hex").slice(0, 12);
}

// src/similarity.ts
function hybridTokenize(text) {
  const normalized = text.toLowerCase().replace(/[\r\n\t]+/g, " ").trim();
  const tokens = /* @__PURE__ */ new Set();
  const words = normalized.match(/[a-z0-9_]+/g) || [];
  for (const w of words) {
    if (w.length >= 2) {
      tokens.add(w);
    }
  }
  const cjkOnly = normalized.replace(/[a-z0-9_\-\.\/\s\p{P}\p{S}]/gu, "");
  for (let i = 0; i < cjkOnly.length - 1; i++) {
    tokens.add(cjkOnly.slice(i, i + 2));
  }
  if (cjkOnly.length === 1 && tokens.size === 0) {
    tokens.add(cjkOnly);
  }
  return tokens;
}
function calculateSimilarity(textA, textB) {
  const cleanA = textA.trim();
  const cleanB = textB.trim();
  if (cleanA === cleanB) return 1;
  if (!cleanA || !cleanB) return 0;
  const setA = hybridTokenize(cleanA);
  const setB = hybridTokenize(cleanB);
  if (setA.size === 0 || setB.size === 0) return 0;
  let intersection = 0;
  for (const token of setA) {
    if (setB.has(token)) {
      intersection++;
    }
  }
  if (intersection === 0) return 0;
  const overlap = intersection / Math.min(setA.size, setB.size);
  const jaccard = intersection / (setA.size + setB.size - intersection);
  return 0.7 * overlap + 0.3 * jaccard;
}
var DEDUPLICATION_SIMILARITY_THRESHOLD = 0.8;

// src/storage.ts
var CONTEXT_USAGE_INSTRUCTIONS = `These are condensed memories from earlier in this session.

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
function renderSummary(reflections, observations, focus) {
  const hasReflections = reflections.length > 0;
  const hasObservations = observations.length > 0;
  const hasFocus = !!(focus && (focus.goal || focus.nextAction));
  if (!hasReflections && !hasObservations && !hasFocus) return "";
  const parts = [CONTEXT_USAGE_INSTRUCTIONS];
  if (hasFocus && focus) {
    const focusLines = [];
    if (focus.goal) focusLines.push(`- Goal: ${focus.goal}`);
    if (focus.nextAction) focusLines.push(`- Next Action: ${focus.nextAction}`);
    parts.push(`## Current Focus & Next Steps
${focusLines.join("\n")}`);
  }
  if (hasReflections) {
    const reflectionLines = reflections.map((r) => `[${r.id}] ${r.content}`).join("\n");
    parts.push(`## Reflections
${reflectionLines}`);
  }
  if (hasObservations) {
    const observationLines = observations.map((o) => `[${o.id}] ${o.timestamp} [${o.relevance}] ${o.content}`).join("\n");
    parts.push(`## Observations
${observationLines}`);
  }
  return parts.join("\n\n");
}
function atomicWriteFileSync(targetPath, content) {
  const dir = path2.dirname(targetPath);
  if (!fs2.existsSync(dir)) fs2.mkdirSync(dir, { recursive: true });
  const tmpPath = `${targetPath}.tmp.${Date.now()}.${Math.random().toString(36).substring(2, 8)}`;
  try {
    fs2.writeFileSync(tmpPath, content, "utf-8");
    fs2.renameSync(tmpPath, targetPath);
  } catch {
    fs2.writeFileSync(targetPath, content, "utf-8");
    if (fs2.existsSync(tmpPath)) {
      try {
        fs2.unlinkSync(tmpPath);
      } catch {
      }
    }
  }
}
var StorageManager = class _StorageManager {
  static MAX_ACTIVE_OBSERVATIONS = 20;
  storeDir;
  projectBaselinePath;
  workspaceRoot;
  activeLedgerPath;
  activeProjectionPath;
  constructor(storeDirOrPaths, projectBaselinePathOrIsWorkspace, workspaceRoot) {
    if (typeof storeDirOrPaths === "object") {
      this.storeDir = storeDirOrPaths.sessionStoreDir;
      this.projectBaselinePath = storeDirOrPaths.projectReflectionsPath;
      this.workspaceRoot = storeDirOrPaths.workspaceRoot;
      this.activeLedgerPath = storeDirOrPaths.activeLedgerPath || path2.join(storeDirOrPaths.sessionStoreDir, "ledger.json");
      this.activeProjectionPath = storeDirOrPaths.activeProjectionPath || path2.join(storeDirOrPaths.sessionStoreDir, "folded_projection.md");
    } else {
      this.storeDir = storeDirOrPaths;
      if (typeof projectBaselinePathOrIsWorkspace === "string") {
        this.projectBaselinePath = projectBaselinePathOrIsWorkspace;
      } else if (projectBaselinePathOrIsWorkspace === true && workspaceRoot) {
        this.projectBaselinePath = path2.join(workspaceRoot, ".agents", "memory", "reflections.json");
      }
      this.workspaceRoot = workspaceRoot;
      this.activeLedgerPath = path2.join(this.storeDir, "ledger.json");
      this.activeProjectionPath = path2.join(this.storeDir, "folded_projection.md");
    }
  }
  get ledgerPath() {
    return this.activeLedgerPath;
  }
  get projectionPath() {
    return this.activeProjectionPath;
  }
  loadLedger(conversationId) {
    if (fs2.existsSync(this.ledgerPath)) {
      try {
        const raw = fs2.readFileSync(this.ledgerPath, "utf-8");
        const parsed = JSON.parse(raw);
        if (parsed && typeof parsed === "object") {
          if (typeof parsed.version !== "number") parsed.version = 1;
          return parsed;
        }
      } catch (e) {
      }
    }
    let initialReflections = [];
    if (this.projectBaselinePath && fs2.existsSync(this.projectBaselinePath)) {
      try {
        const baselineRaw = fs2.readFileSync(this.projectBaselinePath, "utf-8");
        const baseline = JSON.parse(baselineRaw);
        initialReflections = baseline.reflections || [];
      } catch {
      }
    }
    const now = (/* @__PURE__ */ new Date()).toISOString();
    const ledger = {
      version: 0,
      conversationId,
      workspacePath: this.workspaceRoot,
      createdAt: now,
      updatedAt: now,
      activeObservations: [],
      allObservations: [],
      reflections: initialReflections,
      droppedObservationIds: []
    };
    this.saveLedger(ledger);
    return ledger;
  }
  trimActiveObservations(observations) {
    if (observations.length <= _StorageManager.MAX_ACTIVE_OBSERVATIONS) {
      return observations;
    }
    const criticalOrHigh = [];
    const other = [];
    for (const o of observations) {
      if (o.relevance === "critical" || o.relevance === "high") {
        criticalOrHigh.push(o);
      } else {
        other.push(o);
      }
    }
    const slotsForOther = Math.max(0, _StorageManager.MAX_ACTIVE_OBSERVATIONS - criticalOrHigh.length);
    const keptOther = other.slice(-slotsForOther);
    const combined = [...criticalOrHigh, ...keptOther];
    combined.sort((a, b) => a.timestamp.localeCompare(b.timestamp));
    return combined.slice(-_StorageManager.MAX_ACTIVE_OBSERVATIONS);
  }
  mergeLedgers(base, incoming) {
    const droppedSet = /* @__PURE__ */ new Set([
      ...base.droppedObservationIds || [],
      ...incoming.droppedObservationIds || []
    ]);
    const activeMap = /* @__PURE__ */ new Map();
    for (const obs of base.activeObservations || []) {
      if (!droppedSet.has(obs.id)) activeMap.set(obs.id, obs);
    }
    for (const obs of incoming.activeObservations || []) {
      if (!droppedSet.has(obs.id)) activeMap.set(obs.id, obs);
    }
    const allMap = /* @__PURE__ */ new Map();
    for (const obs of base.allObservations || []) {
      allMap.set(obs.id, obs);
    }
    for (const obs of incoming.allObservations || []) {
      allMap.set(obs.id, obs);
    }
    const refMap = /* @__PURE__ */ new Map();
    for (const ref of base.reflections || []) {
      refMap.set(ref.id, ref);
    }
    for (const ref of incoming.reflections || []) {
      refMap.set(ref.id, ref);
    }
    let mergedFocus = void 0;
    if (base.focus && incoming.focus) {
      mergedFocus = (base.focus.updatedAt || "") >= (incoming.focus.updatedAt || "") ? base.focus : incoming.focus;
    } else {
      mergedFocus = incoming.focus || base.focus;
    }
    const merged = {
      version: Math.max(base.version || 1, incoming.version || 1),
      conversationId: incoming.conversationId || base.conversationId,
      workspacePath: incoming.workspacePath || base.workspacePath,
      createdAt: base.createdAt || incoming.createdAt,
      updatedAt: (/* @__PURE__ */ new Date()).toISOString(),
      focus: mergedFocus,
      activeObservations: Array.from(activeMap.values()),
      allObservations: Array.from(allMap.values()),
      reflections: Array.from(refMap.values()),
      droppedObservationIds: Array.from(droppedSet)
    };
    return merged;
  }
  saveLedger(ledger, maxRetries = 3) {
    let currentLedger = ledger;
    currentLedger.activeObservations = this.trimActiveObservations(currentLedger.activeObservations);
    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      let diskLedger = null;
      if (fs2.existsSync(this.ledgerPath)) {
        try {
          const raw = fs2.readFileSync(this.ledgerPath, "utf-8");
          diskLedger = JSON.parse(raw);
        } catch {
          diskLedger = null;
        }
      }
      if (!diskLedger || typeof diskLedger.version !== "number" || diskLedger.version === currentLedger.version) {
        currentLedger.version = (currentLedger.version || 0) + 1;
        currentLedger.updatedAt = (/* @__PURE__ */ new Date()).toISOString();
        atomicWriteFileSync(this.ledgerPath, JSON.stringify(currentLedger, null, 2));
        const summary = renderSummary(currentLedger.reflections, currentLedger.activeObservations, currentLedger.focus);
        atomicWriteFileSync(this.projectionPath, summary);
        this.syncProjectBaseline(currentLedger);
        return;
      }
      currentLedger = this.mergeLedgers(diskLedger, currentLedger);
      currentLedger.activeObservations = this.trimActiveObservations(currentLedger.activeObservations);
      if (attempt === maxRetries) {
        currentLedger.version = (diskLedger.version || 0) + 1;
        currentLedger.updatedAt = (/* @__PURE__ */ new Date()).toISOString();
        atomicWriteFileSync(this.ledgerPath, JSON.stringify(currentLedger, null, 2));
        const summary = renderSummary(currentLedger.reflections, currentLedger.activeObservations, currentLedger.focus);
        atomicWriteFileSync(this.projectionPath, summary);
        this.syncProjectBaseline(currentLedger);
        return;
      }
      const jitterMs = 10 + Math.floor(Math.random() * 20);
      try {
        Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, jitterMs);
      } catch {
      }
    }
  }
  syncProjectBaseline(ledger) {
    if (this.projectBaselinePath && ledger.reflections.length > 0) {
      let mergedReflections = [...ledger.reflections];
      if (fs2.existsSync(this.projectBaselinePath)) {
        try {
          const raw = fs2.readFileSync(this.projectBaselinePath, "utf-8");
          const existing = JSON.parse(raw);
          if (Array.isArray(existing.reflections)) {
            const existingMap = /* @__PURE__ */ new Map();
            for (const r of existing.reflections) {
              if (r && r.id) existingMap.set(r.id, r);
            }
            for (const r of ledger.reflections) {
              existingMap.set(r.id, r);
            }
            mergedReflections = Array.from(existingMap.values());
          }
        } catch {
        }
      }
      const baseline = {
        workspacePath: this.workspaceRoot || "",
        updatedAt: ledger.updatedAt,
        reflections: mergedReflections
      };
      atomicWriteFileSync(this.projectBaselinePath, JSON.stringify(baseline, null, 2));
    }
  }
  setFocus(conversationId, goal, nextAction = "") {
    const ledger = this.loadLedger(conversationId);
    const focus = {
      goal,
      nextAction,
      updatedAt: (/* @__PURE__ */ new Date()).toISOString()
    };
    ledger.focus = focus;
    this.saveLedger(ledger);
    return focus;
  }
  clearFocus(conversationId) {
    const ledger = this.loadLedger(conversationId);
    delete ledger.focus;
    this.saveLedger(ledger);
  }
  recordObservation(conversationId, content, relevance = "medium", sourceStepIndices = [], resolvesIds = []) {
    const ledger = this.loadLedger(conversationId);
    const now = /* @__PURE__ */ new Date();
    const pad = (n) => n.toString().padStart(2, "0");
    const timestamp = `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())} ${pad(now.getHours())}:${pad(now.getMinutes())}`;
    const id = hashId(`${content}-${Date.now()}-${Math.random()}`);
    const tokenCount = estimateStringTokens(content) + 10;
    const obs = {
      id,
      content,
      timestamp,
      relevance,
      sourceStepIndices,
      tokenCount,
      conversationId
    };
    const droppedIds = [];
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
  pinReflection(conversationId, content, supportingObservationIdsOrOptions) {
    const ledger = this.loadLedger(conversationId);
    const tokenCount = estimateStringTokens(content) + 5;
    let replaceId;
    let supportingObservationIds = [];
    let autoDeduplicate = true;
    if (Array.isArray(supportingObservationIdsOrOptions)) {
      supportingObservationIds = supportingObservationIdsOrOptions;
    } else if (supportingObservationIdsOrOptions && typeof supportingObservationIdsOrOptions === "object") {
      replaceId = supportingObservationIdsOrOptions.replaceId;
      supportingObservationIds = supportingObservationIdsOrOptions.supportingObservationIds || [];
      autoDeduplicate = supportingObservationIdsOrOptions.autoDeduplicate !== false;
    }
    if (replaceId) {
      const existingIdx = ledger.reflections.findIndex((r) => r.id === replaceId);
      if (existingIdx !== -1) {
        const oldRef = ledger.reflections[existingIdx];
        const mergedSupp = Array.from(/* @__PURE__ */ new Set([...oldRef.supportingObservationIds || [], ...supportingObservationIds]));
        const updatedRef = {
          id: replaceId,
          content,
          supportingObservationIds: mergedSupp,
          tokenCount
        };
        ledger.reflections[existingIdx] = updatedRef;
        this.saveLedger(ledger);
        return Object.assign(updatedRef, {
          action: "replaced",
          replacedId: replaceId
        });
      }
    }
    if (autoDeduplicate && ledger.reflections.length > 0) {
      let maxScore = 0;
      let bestMatch = null;
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
        const mergedSupp = Array.from(/* @__PURE__ */ new Set([...bestMatch.supportingObservationIds || [], ...supportingObservationIds]));
        const updatedRef = {
          id: bestMatch.id,
          content,
          supportingObservationIds: mergedSupp,
          tokenCount
        };
        ledger.reflections[bestMatchIdx] = updatedRef;
        this.saveLedger(ledger);
        return Object.assign(updatedRef, {
          action: "merged",
          replacedId: bestMatch.id,
          similarityScore: maxScore
        });
      }
    }
    const id = hashId(`pin-${content}-${Date.now()}`);
    const ref = {
      id,
      content,
      supportingObservationIds,
      tokenCount
    };
    ledger.reflections.push(ref);
    this.saveLedger(ledger);
    return Object.assign(ref, { action: "created" });
  }
  unpinReflections(conversationId, ids) {
    const ledger = this.loadLedger(conversationId);
    const idSet = new Set(ids);
    const unpinned = [];
    ledger.reflections = ledger.reflections.filter((r) => {
      if (idSet.has(r.id)) {
        unpinned.push(r.id);
        return false;
      }
      return true;
    });
    this.saveLedger(ledger);
    if (this.projectBaselinePath && fs2.existsSync(this.projectBaselinePath)) {
      try {
        const raw = fs2.readFileSync(this.projectBaselinePath, "utf-8");
        const baseline = JSON.parse(raw);
        if (Array.isArray(baseline.reflections)) {
          baseline.reflections = baseline.reflections.filter((r) => !idSet.has(r.id));
          baseline.updatedAt = (/* @__PURE__ */ new Date()).toISOString();
          atomicWriteFileSync(this.projectBaselinePath, JSON.stringify(baseline, null, 2));
        }
      } catch {
      }
    }
    return {
      unpinned,
      remaining: ledger.reflections.length
    };
  }
  checkpoint(conversationId, milestone, nextAction, options = {}) {
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
      droppedIds: obs.droppedIds || []
    };
  }
  dropObservations(conversationId, ids) {
    const ledger = this.loadLedger(conversationId);
    const idSet = new Set(ids);
    const dropped = [];
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
      dropped
    };
  }
  readProjection() {
    if (fs2.existsSync(this.projectionPath)) {
      return fs2.readFileSync(this.projectionPath, "utf-8").trim();
    }
    return "";
  }
  getOrInitProjection(conversationId) {
    if (fs2.existsSync(this.projectionPath)) {
      return fs2.readFileSync(this.projectionPath, "utf-8").trim();
    }
    const ledger = this.loadLedger(conversationId);
    if (ledger.reflections.length > 0 || ledger.activeObservations.length > 0 || ledger.focus && (ledger.focus.goal || ledger.focus.nextAction)) {
      if (fs2.existsSync(this.projectionPath)) {
        return fs2.readFileSync(this.projectionPath, "utf-8").trim();
      }
    }
    return "";
  }
  findItemById(id) {
    if (fs2.existsSync(this.ledgerPath)) {
      try {
        const ledger = JSON.parse(fs2.readFileSync(this.ledgerPath, "utf-8"));
        const obs = ledger.allObservations.find((o) => o.id === id);
        if (obs) return { type: "observation", item: obs };
        const ref = ledger.reflections.find((r) => r.id === id);
        if (ref) return { type: "reflection", item: ref };
      } catch {
      }
    }
    if (this.projectBaselinePath && fs2.existsSync(this.projectBaselinePath)) {
      try {
        const baseline = JSON.parse(fs2.readFileSync(this.projectBaselinePath, "utf-8"));
        if (Array.isArray(baseline.reflections)) {
          const ref = baseline.reflections.find((r) => r.id === id);
          if (ref) return { type: "reflection", item: ref };
        }
      } catch {
      }
    }
    return void 0;
  }
};

// src/cli.ts
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
function resolveSessionId(wsBase, explicitSession) {
  if (explicitSession) return explicitSession;
  if (process.env.AGY_CONVERSATION_ID) return process.env.AGY_CONVERSATION_ID;
  if (fs3.existsSync(wsBase)) {
    try {
      const entries = fs3.readdirSync(wsBase, { withFileTypes: true });
      const sessionDirs = entries.filter((e) => e.isDirectory()).map((e) => {
        const fullPath = path3.join(wsBase, e.name);
        return { name: e.name, mtime: fs3.statSync(fullPath).mtimeMs };
      }).sort((a, b) => b.mtime - a.mtime);
      if (sessionDirs.length > 0) {
        return sessionDirs[0].name;
      }
    } catch {
    }
  }
  return "active-session";
}
async function readStdin(timeoutMs = 2e3) {
  if (process.stdin.isTTY) return "";
  return new Promise((resolve2) => {
    const chunks = [];
    const timer = setTimeout(() => {
      resolve2(Buffer.concat(chunks).toString("utf-8"));
    }, timeoutMs);
    process.stdin.on("data", (chunk) => {
      chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
    });
    process.stdin.on("end", () => {
      clearTimeout(timer);
      resolve2(Buffer.concat(chunks).toString("utf-8"));
    });
    process.stdin.on("error", () => {
      clearTimeout(timer);
      resolve2("");
    });
  });
}
async function runCli() {
  const args = process.argv.slice(2);
  const command = args[0] || "status";
  if (command === "help" || command === "--help" || command === "-h") {
    printUsage();
    return;
  }
  const getFlagValue = (flagNames) => {
    for (const name of flagNames) {
      const idx = args.indexOf(name);
      if (idx !== -1 && idx + 1 < args.length) return args[idx + 1];
    }
    return void 0;
  };
  const explicitSession = getFlagValue(["--session", "-s"]);
  const explicitRelevance = getFlagValue(["--relevance", "-r"]);
  const nextActionFlag = getFlagValue(["--next", "-n"]);
  const replaceIdFlag = getFlagValue(["--replace"]);
  const resolvesFlag = getFlagValue(["--resolves"]);
  const isJson = args.includes("--json");
  const isAll = args.includes("--all");
  const isClearFlag = args.includes("--clear");
  const parsedResolvesIds = resolvesFlag ? resolvesFlag.split(",").map((s) => s.trim()).filter(Boolean) : [];
  const nonFlagArgs = args.filter((a) => !a.startsWith("-"));
  let explicitWorkspace;
  if (["status", "view", "clear"].includes(command) && nonFlagArgs.length > 1) {
    explicitWorkspace = nonFlagArgs[1];
  }
  if (command === "hook") {
    const hookSubcommand = nonFlagArgs[1] || "pre-invocation";
    if (hookSubcommand === "pre-invocation") {
      try {
        const rawStdin = await readStdin();
        let hookInput = {};
        if (rawStdin && rawStdin.trim()) {
          try {
            const cleaned = rawStdin.replace(/^\uFEFF/, "").trim();
            hookInput = JSON.parse(cleaned);
          } catch {
          }
        }
        if (typeof hookInput.invocationNum === "number" && hookInput.invocationNum > 0) {
          console.log(JSON.stringify({ injectSteps: [] }));
          return;
        }
        const wsRoot = Array.isArray(hookInput.workspacePaths) && hookInput.workspacePaths[0] ? hookInput.workspacePaths[0] : findWorkspaceRoot(process.cwd());
        const convId = hookInput.conversationId || "active-session";
        const hookPaths = resolveStoragePaths(convId, wsRoot);
        const hookStorage = new StorageManager(hookPaths);
        const projection = hookStorage.getOrInitProjection(convId);
        if (projection && projection.trim().length > 0) {
          const ephemeralMessage = `<observational_memory>
${projection}
</observational_memory>`;
          console.log(
            JSON.stringify({
              injectSteps: [
                {
                  ephemeralMessage
                }
              ]
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
  const workspaceRoot = findWorkspaceRoot(explicitWorkspace || process.cwd());
  const initialPaths = resolveStoragePaths("temp", workspaceRoot);
  const baseCacheDir = path3.dirname(initialPaths.sessionStoreDir);
  const conversationId = resolveSessionId(baseCacheDir, explicitSession);
  const paths = resolveStoragePaths(conversationId, workspaceRoot);
  const storage = new StorageManager(paths);
  if (command === "status") {
    let activeObsCount = 0;
    let totalRecorded = 0;
    let reflectionsCount = 0;
    let currentFocus;
    if (fs3.existsSync(storage.ledgerPath)) {
      try {
        const ledger = JSON.parse(fs3.readFileSync(storage.ledgerPath, "utf-8"));
        activeObsCount = ledger.activeObservations?.length || 0;
        totalRecorded = ledger.allObservations?.length || 0;
        reflectionsCount = ledger.reflections?.length || 0;
        currentFocus = ledger.focus;
      } catch {
      }
    }
    let baselineCount = 0;
    if (paths.projectReflectionsPath && fs3.existsSync(paths.projectReflectionsPath)) {
      try {
        const baseline = JSON.parse(fs3.readFileSync(paths.projectReflectionsPath, "utf-8"));
        baselineCount = baseline.reflections?.length || 0;
      } catch {
      }
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
            projectBaselineReflectionsCount: baselineCount
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
    console.log(`=== Active Folded Memory Projection [${conversationId}] ===
`);
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
      console.error('Error: Missing target for focus. Usage: om focus "<target>" [--next "<action>"] or om focus --clear');
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
      console.error('Error: Missing milestone summary for checkpoint. Usage: om checkpoint "<milestone>" --next "<action>" [--resolves <id1,id2>]');
      process.exit(1);
    }
    const nextAction = nextActionFlag || "";
    const relevance = explicitRelevance || "high";
    const result = storage.checkpoint(conversationId, milestone, nextAction, {
      relevance,
      resolvesIds: parsedResolvesIds
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
      console.error('Error: Missing text for record. Usage: om record "<text>" [-r <level>] [--resolves <id1,id2>]');
      process.exit(1);
    }
    const relevance = explicitRelevance || "medium";
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
      console.error('Error: Missing text for pin. Usage: om pin "<text>" [--replace <id>]');
      process.exit(1);
    }
    const result = storage.pinReflection(conversationId, text, {
      replaceId: replaceIdFlag,
      autoDeduplicate: true
    });
    if (result.action === "replaced") {
      console.log(`[OK] Explicitly replaced reflection [${result.id}] with new content: "${result.content}"`);
    } else if (result.action === "merged") {
      console.log(`[OK] Deduplicated & updated existing reflection [${result.id}] (similarity: ${(result.similarityScore * 100).toFixed(1)}%): "${result.content}"`);
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
      const obs = result.item;
      console.log(`=== Observation Record [${obs.id}] ===`);
      console.log(`Timestamp:   ${obs.timestamp}`);
      console.log(`Relevance:   ${obs.relevance}`);
      if (obs.conversationId) console.log(`Session:     ${obs.conversationId}`);
      console.log(`Tokens:      ~${obs.tokenCount}`);
      console.log(`Sources:     ${obs.sourceStepIndices?.length ? obs.sourceStepIndices.join(", ") : "(manual)"}`);
      console.log(`Content:     ${obs.content}`);
      console.log("========================================");
    } else {
      const ref = result.item;
      console.log(`=== Reflection Record [${ref.id}] ===`);
      console.log(`Tokens:      ~${ref.tokenCount}`);
      console.log(
        `Supporting:  ${ref.supportingObservationIds?.length ? ref.supportingObservationIds.map((s) => `[${s}]`).join(", ") : "(none)"}`
      );
      console.log(`Content:     ${ref.content}`);
      console.log("========================================");
    }
    return;
  }
  if (command === "clear") {
    if (isAll) {
      if (paths.projectReflectionsPath && fs3.existsSync(paths.projectReflectionsPath)) {
        fs3.rmSync(paths.projectReflectionsPath, { force: true });
        console.log(`Cleared project reflections: ${paths.projectReflectionsPath}`);
      }
      if (fs3.existsSync(baseCacheDir)) {
        fs3.rmSync(baseCacheDir, { recursive: true, force: true });
        console.log(`Cleared session cache: ${baseCacheDir}`);
      }
    } else if (explicitSession) {
      const sessionDir = path3.join(baseCacheDir, explicitSession);
      if (fs3.existsSync(sessionDir)) {
        fs3.rmSync(sessionDir, { recursive: true, force: true });
        console.log(`Cleared memory for session [${explicitSession}]`);
      } else {
        console.log(`Session directory not found: ${sessionDir}`);
      }
    } else {
      if (fs3.existsSync(baseCacheDir)) {
        fs3.rmSync(baseCacheDir, { recursive: true, force: true });
        console.log(
          `Cleared session observation cache: ${baseCacheDir}
Preserved project reflections at: ${paths.projectReflectionsPath || "(none)"}`
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
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  runCli
});
