export interface Document {
  id: string;
  tokens: string[];
}

export class BM25 {
  private k1: number;
  private b: number;
  private docCount: number = 0;
  private avgDocLength: number = 0;
  private docLengths: Map<string, number> = new Map();
  private termDocFreqs: Map<string, number> = new Map();
  private invertedIndex: Map<string, Map<string, number>> = new Map();

  constructor(documents: Document[], k1: number = 1.5, b: number = 0.75) {
    this.k1 = k1;
    this.b = b;
    this.buildIndex(documents);
  }

  private buildIndex(documents: Document[]): void {
    this.docCount = documents.length;
    if (this.docCount === 0) return;

    let totalLength = 0;

    for (const doc of documents) {
      const len = doc.tokens.length;
      this.docLengths.set(doc.id, len);
      totalLength += len;

      const termFreqsInDoc = new Map<string, number>();
      for (const token of doc.tokens) {
        termFreqsInDoc.set(token, (termFreqsInDoc.get(token) || 0) + 1);
      }

      for (const [term, freq] of termFreqsInDoc.entries()) {
        this.termDocFreqs.set(term, (this.termDocFreqs.get(term) || 0) + 1);

        if (!this.invertedIndex.has(term)) {
          this.invertedIndex.set(term, new Map());
        }
        this.invertedIndex.get(term)!.set(doc.id, freq);
      }
    }

    this.avgDocLength = totalLength / this.docCount;
  }

  public score(queryTokens: string[]): Map<string, number> {
    const scores = new Map<string, number>();
    if (this.docCount === 0 || queryTokens.length === 0) return scores;

    for (const token of queryTokens) {
      const docFreq = this.termDocFreqs.get(token) || 0;
      if (docFreq === 0) continue;

      // Robertson-Spärck Jones IDF
      const idf = Math.log(1 + (this.docCount - docFreq + 0.5) / (docFreq + 0.5));
      const postings = this.invertedIndex.get(token);
      if (!postings) continue;

      for (const [docId, tf] of postings.entries()) {
        const docLen = this.docLengths.get(docId) || this.avgDocLength;
        const numerator = tf * (this.k1 + 1);
        const denominator = tf + this.k1 * (1 - this.b + this.b * (docLen / this.avgDocLength));
        const tokenScore = idf * (numerator / denominator);

        scores.set(docId, (scores.get(docId) || 0) + tokenScore);
      }
    }

    return scores;
  }
}
