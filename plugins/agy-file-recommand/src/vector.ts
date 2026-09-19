/**
 * Pure Mathematical Vector Embedding & PPMI-SVD Engine for Cross-Lingual Semantic Retrieval
 * No external neural network / heavy dependencies required. Runs in < 15ms.
 */

export interface VectorDocument {
  id: string;
  tokens: string[];
  files: string[];
}

export interface SemanticVectorResult {
  fileScores: Map<string, number>;
  episodeScores: Map<string, number>;
}

export function dot(a: number[], b: number[]): number {
  let sum = 0;
  for (let i = 0; i < a.length; i++) {
    sum += a[i] * b[i];
  }
  return sum;
}

export function norm(a: number[]): number {
  const d = dot(a, a);
  return d > 0 ? Math.sqrt(d) : 0;
}

export function cosineSimilarity(a: number[], b: number[]): number {
  const normA = norm(a);
  const normB = norm(b);
  if (normA === 0 || normB === 0) return 0;
  const score = dot(a, b) / (normA * normB);
  return Math.max(0, Math.min(1, score));
}

/**
 * Truncated SVD using Power Iteration with Gram-Schmidt Orthogonalization
 * A: m x n matrix
 * Returns: U (m x k), S (k), V (n x k)
 */
export function truncatedSVD(
  matrix: number[][],
  k: number,
  iterations = 25
): { U: number[][]; S: number[]; V: number[][] } {
  const m = matrix.length;
  if (m === 0) return { U: [], S: [], V: [] };
  const n = matrix[0].length;
  if (n === 0) return { U: [], S: [], V: [] };

  const targetK = Math.min(k, m, n);
  if (targetK === 0) return { U: [], S: [], V: [] };

  const U: number[][] = Array.from({ length: m }, () => new Array(targetK).fill(0));
  const V: number[][] = Array.from({ length: n }, () => new Array(targetK).fill(0));
  const S: number[] = new Array(targetK).fill(0);

  // Work on deflated matrix copy
  const A: number[][] = matrix.map((row) => [...row]);

  for (let comp = 0; comp < targetK; comp++) {
    // Deterministic non-zero initialization
    let v: number[] = new Array(n).fill(0).map((_, i) => (comp === 0 ? 1.0 : Math.cos((i + 1) * (comp + 1))));
    let vNorm = norm(v);
    if (vNorm === 0) v[0] = 1;
    else v = v.map((x) => x / (vNorm || 1));

    let u: number[] = new Array(m).fill(0);

    for (let iter = 0; iter < iterations; iter++) {
      // u = A * v
      for (let i = 0; i < m; i++) {
        let sum = 0;
        for (let j = 0; j < n; j++) {
          sum += A[i][j] * v[j];
        }
        u[i] = sum;
      }

      // Orthogonalize u against prior U columns (Gram-Schmidt)
      for (let prev = 0; prev < comp; prev++) {
        let proj = 0;
        for (let i = 0; i < m; i++) proj += u[i] * U[i][prev];
        for (let i = 0; i < m; i++) u[i] -= proj * U[i][prev];
      }

      const uNorm = norm(u);
      if (uNorm > 1e-12) {
        u = u.map((x) => x / uNorm);
      } else {
        break;
      }

      // v = A^T * u
      for (let j = 0; j < n; j++) {
        let sum = 0;
        for (let i = 0; i < m; i++) {
          sum += A[i][j] * u[i];
        }
        v[j] = sum;
      }

      // Orthogonalize v against prior V columns (Gram-Schmidt)
      for (let prev = 0; prev < comp; prev++) {
        let proj = 0;
        for (let j = 0; j < n; j++) proj += v[j] * V[j][prev];
        for (let j = 0; j < n; j++) v[j] -= proj * V[j][prev];
      }

      const nextVNorm = norm(v);
      if (nextVNorm > 1e-12) {
        v = v.map((x) => x / nextVNorm);
      } else {
        break;
      }
    }

    // Singular value sigma = u^T * A * v
    let sigma = 0;
    for (let i = 0; i < m; i++) {
      for (let j = 0; j < n; j++) {
        sigma += u[i] * A[i][j] * v[j];
      }
    }

    // If sigma is negative, flip u so singular value is non-negative
    if (sigma < 0) {
      sigma = -sigma;
      u = u.map((x) => -x);
    }

    S[comp] = sigma;
    for (let i = 0; i < m; i++) U[i][comp] = u[i];
    for (let j = 0; j < n; j++) V[j][comp] = v[j];

    // Deflate A: A = A - sigma * u * v^T
    for (let i = 0; i < m; i++) {
      for (let j = 0; j < n; j++) {
        A[i][j] -= sigma * u[i] * v[j];
      }
    }
  }

  return { U, S, V };
}

export class SemanticVectorEngine {
  private vocab: Map<string, number> = new Map(); // word -> index
  private vocabList: string[] = [];
  private contextList: string[] = []; // file/entity -> index
  private contextMap: Map<string, number> = new Map();
  private wordEmbeddings: Map<string, number[]> = new Map();
  private contextEmbeddings: Map<string, number[]> = new Map();
  private episodeEmbeddings: Map<string, number[]> = new Map();
  private dim: number;

  constructor(documents: VectorDocument[], dim = 24) {
    this.dim = dim;
    this.buildIndex(documents);
  }

  private buildIndex(docs: VectorDocument[]): void {
    if (docs.length === 0) return;

    // 1. Build vocabulary and context indices
    const wordCounts = new Map<string, number>();
    const contextCounts = new Map<string, number>();
    const cooccur = new Map<string, Map<string, number>>();

    let totalCooccur = 0;

    for (const doc of docs) {
      const uniqueTokens = Array.from(new Set(doc.tokens));
      const uniqueFiles = Array.from(new Set(doc.files));

      for (const token of uniqueTokens) {
        wordCounts.set(token, (wordCounts.get(token) || 0) + 1);

        for (const file of uniqueFiles) {
          contextCounts.set(file, (contextCounts.get(file) || 0) + 1);

          if (!cooccur.has(token)) cooccur.set(token, new Map());
          const map = cooccur.get(token)!;
          map.set(file, (map.get(file) || 0) + 1);
          totalCooccur++;
        }
      }
    }

    this.vocabList = Array.from(wordCounts.keys());
    this.vocabList.forEach((w, i) => this.vocab.set(w, i));

    this.contextList = Array.from(contextCounts.keys());
    this.contextList.forEach((c, i) => this.contextMap.set(c, i));

    const m = this.vocabList.length;
    const n = this.contextList.length;

    if (m === 0 || n === 0 || totalCooccur === 0) return;

    // 2. Compute PPMI Matrix
    const ppmiMatrix: number[][] = Array.from({ length: m }, () => new Array(n).fill(0));

    for (let i = 0; i < m; i++) {
      const token = this.vocabList[i];
      const cw = wordCounts.get(token) || 0;
      const fileMap = cooccur.get(token);
      if (!fileMap) continue;

      for (let j = 0; j < n; j++) {
        const file = this.contextList[j];
        const cf = contextCounts.get(file) || 0;
        const c_wf = fileMap.get(file) || 0;

        if (c_wf > 0) {
          const ratio = (c_wf * totalCooccur) / (cw * cf);
          const pmi = Math.log(1.0 + ratio);
          ppmiMatrix[i][j] = Math.max(0, pmi);
        }
      }
    }

    // 3. Truncated SVD Decomposition
    const k = Math.min(this.dim, m, n);
    this.dim = k;
    const { U, S, V } = truncatedSVD(ppmiMatrix, k);

    // 4. Construct embeddings: WordVector = U * sqrt(S), ContextVector = V * sqrt(S)
    for (let i = 0; i < m; i++) {
      const vec: number[] = [];
      for (let c = 0; c < k; c++) {
        vec.push(U[i][c] * Math.sqrt(S[c] || 0));
      }
      this.wordEmbeddings.set(this.vocabList[i], vec);
    }

    for (let j = 0; j < n; j++) {
      const vec: number[] = [];
      for (let c = 0; c < k; c++) {
        vec.push(V[j][c] * Math.sqrt(S[c] || 0));
      }
      this.contextEmbeddings.set(this.contextList[j], vec);
    }

    // 5. Build Episode Vectors by average pooling tokens
    for (const doc of docs) {
      const pooled = this.embedTokens(doc.tokens);
      if (norm(pooled) > 0) {
        this.episodeEmbeddings.set(doc.id, pooled);
      }
    }
  }

  public embedTokens(tokens: string[]): number[] {
    const k = this.dim;
    if (k === 0) return [];

    const pooled = new Array(k).fill(0);
    let count = 0;

    for (const token of tokens) {
      const vec = this.wordEmbeddings.get(token);
      if (vec) {
        for (let i = 0; i < vec.length; i++) {
          pooled[i] += vec[i];
        }
        count++;
      }
    }

    if (count === 0) return [];
    return pooled.map((x) => x / count);
  }

  public score(queryTokens: string[]): SemanticVectorResult {
    const fileScores = new Map<string, number>();
    const episodeScores = new Map<string, number>();

    const qVec = this.embedTokens(queryTokens);
    if (qVec.length === 0 || norm(qVec) === 0) {
      return { fileScores, episodeScores };
    }

    // Cosine similarity against files / contexts
    for (const [file, fVec] of this.contextEmbeddings.entries()) {
      const sim = cosineSimilarity(qVec, fVec);
      if (sim > 0.1) {
        fileScores.set(file, sim);
      }
    }

    // Cosine similarity against historical episodes
    for (const [epId, epVec] of this.episodeEmbeddings.entries()) {
      const sim = cosineSimilarity(qVec, epVec);
      if (sim > 0.1) {
        episodeScores.set(epId, sim);
      }
    }

    return { fileScores, episodeScores };
  }
}
