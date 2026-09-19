/**
 * Hybrid CJK 2-Gram + English Word Tokenizer and Similarity Engine
 * Optimized for zero-dependency, ultra-fast (<0.1ms) in-memory comparison.
 */

export function hybridTokenize(text: string): Set<string> {
  const normalized = text.toLowerCase().replace(/[\r\n\t]+/g, " ").trim();
  const tokens = new Set<string>();

  // 1. Extract English / alphanumeric words (length >= 2)
  const words = normalized.match(/[a-z0-9_]+/g) || [];
  for (const w of words) {
    if (w.length >= 2) {
      tokens.add(w);
    }
  }

  // 2. Extract CJK characters and generate 2-character sliding window (bi-grams)
  // Matching CJK Unified Ideographs, CJK Compatibility Ideographs, Hiragana, Katakana, Hangul
  const cjkOnly = normalized.replace(/[a-z0-9_\-\.\/\s\p{P}\p{S}]/gu, "");
  for (let i = 0; i < cjkOnly.length - 1; i++) {
    tokens.add(cjkOnly.slice(i, i + 2));
  }

  // If text is very short single CJK character, keep single character
  if (cjkOnly.length === 1 && tokens.size === 0) {
    tokens.add(cjkOnly);
  }

  return tokens;
}

export function calculateSimilarity(textA: string, textB: string): number {
  const cleanA = textA.trim();
  const cleanB = textB.trim();

  if (cleanA === cleanB) return 1.0;
  if (!cleanA || !cleanB) return 0.0;

  const setA = hybridTokenize(cleanA);
  const setB = hybridTokenize(cleanB);

  if (setA.size === 0 || setB.size === 0) return 0.0;

  let intersection = 0;
  for (const token of setA) {
    if (setB.has(token)) {
      intersection++;
    }
  }

  if (intersection === 0) return 0.0;

  // Overlap Coefficient (Szymkiewicz-Simpson) calculates containment: |A ∩ B| / min(|A|, |B|)
  const overlap = intersection / Math.min(setA.size, setB.size);
  // Jaccard similarity: |A ∩ B| / |A ∪ B|
  const jaccard = intersection / (setA.size + setB.size - intersection);

  // Weighted composite score: 70% Overlap + 30% Jaccard
  return 0.7 * overlap + 0.3 * jaccard;
}

export const DEDUPLICATION_SIMILARITY_THRESHOLD = 0.80;
