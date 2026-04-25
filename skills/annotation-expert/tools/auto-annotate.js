const fs = require('fs');
const path = require('path');
const CONFIG = require('./lib/config');
const { getSymbolRanges } = require('./lib/scanner');
const { findRefs } = require('./lib/searcher');

/**
 * 跨語言自動增強註解引擎 (Modular Version)
 * 僅負責補全 @referenced_by，不干涉或生成其他標籤
 */

function augmentFile(filePath) {
  const ext = path.extname(filePath);
  const lang = CONFIG[ext];
  if (!lang) return console.log(`[Skip] Unsupported extension: ${ext}`);

  const content = fs.readFileSync(filePath, 'utf-8');
  let lines = content.split('\n');
  
  const ranges = getSymbolRanges(lines, lang);
  
  // 反向遍歷處理以保持行號一致性
  for (let i = ranges.length - 1; i >= 0; i--) {
    const symbol = ranges[i];
    const refs = findRefs(symbol.name, filePath);
    const refTag = `${lang.comment.line} @referenced_by ${refs.length > 0 ? refs.join(', ') : 'standalone'}`;
    
    // 尋找現有的 JSDoc 區塊
    let docStart = -1;
    let docEnd = symbol.start - 2;
    
    if (lines[docEnd] && (lines[docEnd].includes('*/') || (lang.comment.end !== '*/' && lines[docEnd].includes(lang.comment.end)))) {
      for (let j = docEnd; j >= 0; j--) {
        if (lines[j].includes(lang.comment.start)) { docStart = j; break; }
      }
    }

    if (docStart !== -1) {
      // 僅更新 @referenced_by 標籤，保留現有其他所有內容 (包括手動加入的標籤)
      let newDoc = lines.slice(docStart, docEnd + 1).filter(l => !l.includes('@referenced_by'));
      newDoc.splice(newDoc.length - 1, 0, refTag);
      lines.splice(docStart, (docEnd - docStart) + 1, ...newDoc);
    } else {
      // 建立新註解 (僅包含 @name 與 @referenced_by)
      lines.splice(symbol.start - 1, 0, lang.comment.start, `${lang.comment.line} @name ${symbol.name}`, refTag, lang.comment.end);
    }
  }

  fs.writeFileSync(filePath, lines.join('\n'));
  console.log(`[Success] Augmented ${filePath}`);
}

const target = process.argv[2];
if (target) augmentFile(path.resolve(target));
