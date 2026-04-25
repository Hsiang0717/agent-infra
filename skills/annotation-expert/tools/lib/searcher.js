const { execSync } = require('child_process');
const path = require('path');

/**
 * Searcher: 負責全專案搜尋符號引用 (過濾雜訊版)
 */

function findRefs(symbolName, filePath) {
  const isWin = process.platform === 'win32';
  try {
    const cmd = isWin 
      ? `findstr /s /n /i /c:"${symbolName}" *.*`
      : `grep -r "\\b${symbolName}\\b" . -n --exclude-dir={node_modules,.git,annotation-expert}`;

    const output = execSync(cmd, { encoding: 'utf-8', maxBuffer: 1024 * 1024 });
    return output.split('\n')
      .filter(l => l.trim())
      .map(l => {
        const parts = l.split(':');
        if (parts.length < 2) return null;
        const f = path.relative(process.cwd(), parts[0]);
        const lineNum = parts[1];
        return `${f}:L${lineNum}`;
      })
      .filter(ref => {
        if (!ref) return false;
        const normalizedRef = ref.replace(/\\/g, '/');
        // 過濾掉當前檔案、node_modules、以及技能工具目錄
        return !normalizedRef.includes(path.basename(filePath)) && 
               !normalizedRef.includes('node_modules') &&
               !normalizedRef.includes('annotation-expert');
      })
      .slice(0, 3);
  } catch (e) { return []; }
}

module.exports = { findRefs };
