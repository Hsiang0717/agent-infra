module.exports = {
  '.ts': { 
    patterns: [
      { regex: /(?:async\s+)?function\s+([a-zA-Z0-9_$]+)/, type: 'func' },
      { regex: /(?:const|let|var)\s+([a-zA-Z0-9_$]+)\s*=\s*(?:async\s*)?\(?/, type: 'var' },
      { regex: /class\s+([a-zA-Z0-9_$]+)/, type: 'class' }
    ],
    comment: { start: '/**', line: ' *', end: ' */' },
    blockType: 'braces'
  },
  '.js': { 
    patterns: [
      { regex: /(?:async\s+)?function\s+([a-zA-Z0-9_$]+)/, type: 'func' },
      { regex: /class\s+([a-zA-Z0-9_$]+)/, type: 'class' }
    ],
    comment: { start: '/**', line: ' *', end: ' */' },
    blockType: 'braces'
  },
  '.py': {
    patterns: [
      { regex: /^def\s+([a-zA-Z0-9_$]+)\s*\(/, type: 'func' },
      { regex: /^class\s+([a-zA-Z0-9_$]+)\s*(?:\(|:)/, type: 'class' }
    ],
    comment: { start: '###', line: '###', end: '###' },
    blockType: 'indent'
  },
  '.go': {
    patterns: [
      { regex: /func\s+(?:\([^*)]+\*\?[^)]+\)\s+)?([a-zA-Z0-9_$]+)\s*\(/, type: 'func' },
      { regex: /type\s+([a-zA-Z0-9_$]+)\s+struct/, type: 'class' }
    ],
    comment: { start: '//', line: '//', end: '//' },
    blockType: 'braces'
  }
};
