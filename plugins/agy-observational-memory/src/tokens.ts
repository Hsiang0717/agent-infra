/**
 * Token estimation utilities.
 * Approximates token counts considering both English and CJK characters.
 */
export function estimateStringTokens(text: string): number {
  if (!text) return 0;
  // CJK characters generally consume ~1 token each, whereas ASCII words average ~4 chars per token.
  let cjkCount = 0;
  for (let i = 0; i < text.length; i++) {
    const code = text.charCodeAt(i);
    if (
      (code >= 0x4e00 && code <= 0x9fff) ||
      (code >= 0x3400 && code <= 0x4dbf) ||
      (code >= 0x20000 && code <= 0x2a6df) ||
      (code >= 0x3040 && code <= 0x30ff) ||
      (code >= 0xac00 && code <= 0xd7af)
    ) {
      cjkCount++;
    }
  }
  const nonCjkLen = text.length - cjkCount;
  return cjkCount + Math.ceil(nonCjkLen / 4);
}

export function observationLineTokenCount(observation: {
  id: string;
  timestamp: string;
  relevance: string;
  content: string;
}): number {
  return estimateStringTokens(
    `[${observation.id}] ${observation.timestamp} [${observation.relevance}] ${observation.content}`
  );
}

export function reflectionLineTokenCount(reflection: {
  id: string;
  content: string;
}): number {
  return estimateStringTokens(`[${reflection.id}] ${reflection.content}`);
}
