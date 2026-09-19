import * as fs from "node:fs";
import * as path from "node:path";
import { RecommendationItem, RecommendationResult } from "./types.js";
import { loadEpisodes } from "./store.js";
import { getGitModifiedFiles, getGitTrackedFiles } from "./git.js";
import { loadConfig, RecommandConfig } from "./config.js";
import { BM25, Document } from "./bm25.js";
import { SemanticVectorEngine, VectorDocument } from "./vector.js";

export function tokenize(text: string): string[] {
  const tokens = new Set<string>();

  // 1. Extract path/file tokens intact: e.g. "src/parser.ts" or "recommand.cjs"
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
  customConfig?: RecommandConfig | null,
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

  // Record probability factors and reasons for Noisy-OR fusion: S = 1 - Prod(1 - p_i)
  const candidateSignals: Record<string, { probs: number[]; reasons: string[] }> = {};

  const addSignal = (file: string, probability: number, reason: string) => {
    if (!file || probability <= 0) return;
    const normalized = file.replace(/\\/g, "/").replace(/^\.\//, "").trim();
    if (!candidateSignals[normalized]) {
      candidateSignals[normalized] = { probs: [], reasons: [] };
    }
    const clampedProb = Math.max(0.01, Math.min(0.95, probability));
    candidateSignals[normalized].probs.push(clampedProb);
    if (!candidateSignals[normalized].reasons.includes(reason)) {
      candidateSignals[normalized].reasons.push(reason);
    }
  };

  // 1. Direct path/filename mentioned in query (High Confidence: 0.8)
  for (const token of queryTokens) {
    if (token.includes("/") || token.includes(".")) {
      addSignal(token, 0.8, "direct_path");
    }
  }

  // 2. Historical similarity using BM25 & Semantic Vector Engine (PPMI-SVD + Cosine + Turn-based Step Decay)
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

          const confidence = Math.min(0.6, Number((Math.max(score * 0.15, 0.35) * decay).toFixed(2)));
          for (const f of ep.editedFiles) {
            addSignal(f, confidence, isCurrentSession ? "active_session_edit" : "historical_edit");
          }
          for (const f of ep.citedFiles) {
            addSignal(f, confidence * 0.7, isCurrentSession ? "active_session_cite" : "historical_cite");
          }
        }
      }
    }

    // B. Semantic Vector Engine (PPMI-SVD + Cosine Similarity)
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
        addSignal(file, Math.min(0.55, Number(sim.toFixed(2))), "semantic_cosine");
      }
    }

    // Cross-lingual Episode cosine similarity
    for (const [epId, sim] of vectorResult.episodeScores.entries()) {
      if (sim > 0.35) {
        const ep = episodes.find((e) => e.id === epId);
        if (ep) {
          for (const f of ep.editedFiles) {
            addSignal(f, Math.min(0.5, Number((sim * 0.8).toFixed(2))), "cross_lingual_semantic");
          }
        }
      }
    }
  }

  // 3. Project Tree Cold-Start Matching (for unseen files or sparse episodes)
  const trackedFiles = getGitTrackedFiles(workspaceRoot);
  for (const file of trackedFiles) {
    const fileTokens = tokenize(file);
    const overlap = fileTokens.filter((t) => queryTokens.includes(t) && t.length > 2 && !t.includes("/"));
    if (overlap.length >= 2) {
      addSignal(file, Math.min(0.5, 0.25 * overlap.length), "tree_path_match");
    }
  }

  // 4. Git modified files (Base Confidence: 0.3)
  const gitFiles = getGitModifiedFiles(workspaceRoot);
  for (const f of gitFiles) {
    addSignal(f, 0.3, "git_modified");
  }

  const maxItems = config.recommend.maxItems ?? 3;
  const threshold = config.recommend.threshold ?? 0.35;

  const items: RecommendationItem[] = Object.entries(candidateSignals)
    .map(([filePath, data]) => {
      // Noisy-OR probabilistic fusion: S = 1 - Prod(1 - p_i)
      const complementProduct = data.probs.reduce((acc, p) => acc * (1 - p), 1.0);
      const noisyOrScore = 1.0 - complementProduct;
      return {
        path: filePath,
        score: Number(Math.min(0.98, noisyOrScore).toFixed(2)),
        reasons: data.reasons,
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
