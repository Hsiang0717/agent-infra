import * as fs from "node:fs";
import * as path from "node:path";
import { RecommendationItem, RecommendationResult } from "./types.js";
import { loadEpisodes } from "./store.js";
import { getGitModifiedFiles, getGitTrackedFiles, getGitRecentFiles } from "./git.js";
import { loadConfig, ContextFlowConfig } from "./config.js";
import { BM25, Document } from "./bm25.js";
import { SemanticVectorEngine, VectorDocument } from "./vector.js";

export function tokenize(text: string): string[] {
  const tokens = new Set<string>();

  // 1. Extract path/file tokens intact: e.g. "src/parser.ts" or "context-flow.cjs"
  const pathRegex = /[a-zA-Z0-9_\-\./]+\.[a-zA-Z0-9_-]+/g;
  let match: RegExpExecArray | null;
  while ((match = pathRegex.exec(text)) !== null) {
    const p = match[0].toLowerCase().replace(/\\/g, "/");
    tokens.add(p);
  }

  // 2. Extract alphanumeric words and split camelCase / snake_case
  const words = text
    .toLowerCase()
    .replace(/[^\w\s\.-]/g, " ")
    .split(/\s+/)
    .filter((w) => w.length > 1);

  for (const w of words) {
    tokens.add(w);
    // Split sub-words if contains dots or dashes
    const sub = w.split(/[\.-]/).filter((s) => s.length > 1);
    for (const s of sub) tokens.add(s);
  }

  // 3. Extract CJK unigrams and bigrams
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

export function isConceptualQuery(query: string): boolean {
  const trimmed = query.trim().toLowerCase();
  if (trimmed.length <= 2) return true;

  const conceptualPatterns = [
    /^(hi|hello|hey|你好|您好|哈囉)$/i,
    /^(你是誰|自我介紹|你是什(麼|么))/i,
    /^(什麼是|什么是|解釋|解释|什麼意思|科普)/i,
    /^(how are you|who are you|what is|explain)/i,
    /^(好|ok|可以|沒問題|知道了)$/i,
  ];

  return conceptualPatterns.some((pattern) => pattern.test(trimmed));
}

export function recommendFiles(
  query: string,
  workspaceRoot?: string,
  customConfig?: ContextFlowConfig | null,
  currentConversationId?: string
): RecommendationResult {
  const config = customConfig !== undefined ? customConfig : loadConfig(workspaceRoot);

  if (!config || !config.recommend.enabled || isConceptualQuery(query)) {
    return { items: [], mode: "NONE" };
  }

  const queryTokens = tokenize(query);
  if (queryTokens.length === 0) {
    return { items: [], mode: "NONE" };
  }

  // Dual-Priors Bayesian Scoring Containers:
  // P(f) = Static Structural Prior (Git modified, recency, entrypoint, tree overlap)
  // P(q|f) = Dynamic Semantic Likelihood (Direct path, BM25, SVD-FastText)
  const candidatePriors: Record<string, { probs: number[]; reasons: string[] }> = {};
  const candidateSemantics: Record<string, { probs: number[]; reasons: string[] }> = {};

  const addPrior = (file: string, probability: number, reason: string) => {
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

  const addSemantic = (file: string, probability: number, reason: string) => {
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

  // --- CHANNEL 1: Dynamic Semantic Likelihood P(q|f) ---

  // 1. Direct path/filename mentioned in query (High Confidence: 0.85)
  for (const token of queryTokens) {
    if (token.includes("/") || token.includes(".")) {
      addSemantic(token, 0.85, "direct_path");
    }
  }

  // 2. Historical similarity using BM25 & Semantic Vector Engine (PPMI-SVD + FastText + Turn-based Step Decay)
  const episodes = loadEpisodes(workspaceRoot, 100);
  const totalEps = episodes.length;

  if (episodes.length > 0) {
    // A. BM25 keyword scoring
    const docs: Document[] = episodes.map((ep) => ({
      id: ep.id,
      tokens: Array.from(new Set([...tokenize(ep.query), ...(ep.thinkingTokens || [])])),
    }));

    const bm25 = new BM25(docs);
    const bm25Scores = bm25.score(queryTokens);

    for (const [epId, score] of bm25Scores.entries()) {
      if (score > 0.05) {
        const epIndex = episodes.findIndex((e) => e.id === epId);
        const ep = epIndex >= 0 ? episodes[epIndex] : undefined;
        if (ep) {
          // Logical Turn/Step distance from newest episode (0 = newest)
          const stepDist = Math.max(0, totalEps - 1 - epIndex);
          const stepDecay = Math.max(0.4, Math.pow(0.96, stepDist));

          // Active session focus: 1.25x boost if within current conversation
          const isCurrentSession = Boolean(currentConversationId && ep.conversationId === currentConversationId);
          const sessionMultiplier = isCurrentSession ? 1.25 : 1.0;
          const decay = Math.min(1.0, stepDecay * sessionMultiplier);

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

    // B. Semantic Vector Engine (PPMI-SVD + FastText Subwords + Cosine Similarity)
    const vecDocs: VectorDocument[] = episodes.map((ep) => ({
      id: ep.id,
      tokens: Array.from(new Set([...tokenize(ep.query), ...(ep.thinkingTokens || [])])),
      files: Array.from(new Set([...ep.editedFiles, ...ep.citedFiles, ...ep.readFiles])),
    }));

    const vectorEngine = new SemanticVectorEngine(vecDocs);
    const vectorResult = vectorEngine.score(queryTokens);

    // Direct Context/File cosine similarity
    for (const [file, sim] of vectorResult.fileScores.entries()) {
      if (sim > 0.3) {
        addSemantic(file, Math.min(0.6, Number(sim.toFixed(2))), "semantic_cosine");
      }
    }

    // Cross-lingual Episode cosine similarity
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

  // --- CHANNEL 2: Static Structural Prior P(f) ---

  // 1. Project Tree Path Match (Token overlap with repo files)
  const trackedFiles = getGitTrackedFiles(workspaceRoot);
  for (const file of trackedFiles) {
    const fileTokens = tokenize(file);
    const overlap = fileTokens.filter((t) => queryTokens.includes(t) && t.length > 2 && !t.includes("/"));
    if (overlap.length >= 2) {
      addPrior(file, Math.min(0.5, 0.25 * overlap.length), "tree_path_match");
    }

    // Entrypoint baseline prior (e.g., index.ts, main.ts, app.ts)
    const baseName = path.basename(file).toLowerCase();
    if (/^(index|main|app|server|cli)\.(ts|js|rs|go|py)$/.test(baseName)) {
      addPrior(file, 0.15, "entrypoint_prior");
    }
  }

  // 2. Git modified files (Active Working Tree: 0.40)
  const gitFiles = getGitModifiedFiles(workspaceRoot);
  for (const f of gitFiles) {
    addPrior(f, 0.4, "git_modified");
  }

  // 3. Git Recent Commits (Recency gravity: 0.15 ~ 0.40)
  const recentFiles = getGitRecentFiles(workspaceRoot, 15);
  for (const [f, score] of recentFiles.entries()) {
    addPrior(f, score, "git_recent_active");
  }

  // --- CHANNEL 3: Dirichlet Shrinkage & Bayesian Posterior Fusion ---
  // Lambda(N) = N / (N + mu), where mu = 4
  const mu = 4;
  const lambda = totalEps / (totalEps + mu);

  // w_prior: starts at 1.0 (cold start), smoothly settles to 0.6 (warm start)
  const wPrior = (0.6 * totalEps + mu) / (totalEps + mu);
  // w_semantic: starts at 0.5 (cold start), smoothly rises to 1.0 (warm start)
  const wSemantic = (totalEps + 0.5 * mu) / (totalEps + mu);

  const allCandidateKeys = new Set([...Object.keys(candidatePriors), ...Object.keys(candidateSemantics)]);
  const maxItems = config.recommend.maxItems ?? 3;
  const threshold = config.recommend.threshold ?? 0.35;

  const items: RecommendationItem[] = Array.from(allCandidateKeys)
    .map((filePath) => {
      const priorData = candidatePriors[filePath];
      const semanticData = candidateSemantics[filePath];

      const pPrior = priorData
        ? 1.0 - priorData.probs.reduce((acc, p) => acc * (1.0 - p), 1.0)
        : 0;
      const pSemantic = semanticData
        ? 1.0 - semanticData.probs.reduce((acc, p) => acc * (1.0 - p), 1.0)
        : 0;

      // Penalize test/declaration files in prior if not explicitly requested
      let penalty = 1.0;
      if (/\.(test|spec)\.[a-zA-Z0-9]+$/.test(filePath) || filePath.endsWith(".d.ts")) {
        penalty = 0.6;
      }

      const effectivePrior = pPrior * penalty;

      // Dual-Priors Bayesian Noisy-OR Fusion with Dirichlet Shrinkage
      const uncertPrior = Math.pow(1.0 - effectivePrior, wPrior);
      const uncertSemantic = Math.pow(1.0 - pSemantic, wSemantic);
      const posteriorScore = 1.0 - uncertPrior * uncertSemantic;

      const reasons = Array.from(
        new Set([...(priorData?.reasons || []), ...(semanticData?.reasons || [])])
      );

      return {
        path: filePath,
        score: Number(Math.min(0.98, posteriorScore).toFixed(2)),
        reasons,
      };
    })
    .filter((item) => {
      if (item.score < threshold) return false;
      // File existence check: filter out deleted/ghost files
      if (workspaceRoot) {
        const fullPath = path.resolve(workspaceRoot, item.path);
        if (!fs.existsSync(fullPath)) return false;
      }
      return true;
    })
    .sort((a, b) => b.score - a.score)
    .slice(0, maxItems);

  return items.length > 0 ? { items, mode: "SUGGESTION_HINT" } : { items: [], mode: "NONE" };
}
