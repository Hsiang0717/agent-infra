/**
 * Scanner: 負責識別符號定義及其結束範圍
 */

function findBlockEnd(lines, startIdx, type) {
  if (type === 'braces') {
    let count = 0;
    let foundFirst = false;
    for (let i = startIdx; i < lines.length; i++) {
      const line = lines[i];
      if (line.includes('{')) { count += (line.match(/{/g) || []).length; foundFirst = true; }
      if (line.includes('}')) { count -= (line.match(/}/g) || []).length; }
      if (foundFirst && count === 0) return i + 1;
    }
  } else if (type === 'indent') {
    const startIndent = lines[startIdx].match(/^\s*/)[0].length;
    for (let i = startIdx + 1; i < lines.length; i++) {
      if (lines[i].trim() === '') continue;
      const currentIndent = lines[i].match(/^\s*/)[0].length;
      if (currentIndent <= startIndent) return i;
    }
  }
  return lines.length;
}

function getSymbolRanges(lines, langConfig) {
  const results = [];
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    for (const p of langConfig.patterns) {
      const match = line.match(p.regex);
      if (match) {
        if (p.type === 'var' && !line.includes('=>') && !line.includes('function')) continue;
        const start = i + 1;
        const end = findBlockEnd(lines, i, langConfig.blockType);
        results.push({ name: match[1], start, end });
        break;
      }
    }
  }
  return results;
}

module.exports = { getSymbolRanges };
