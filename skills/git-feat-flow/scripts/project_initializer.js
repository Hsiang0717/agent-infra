const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

function gatherProjectMetadata() {
  const metadata = {
    structure: '',
    keyFiles: {},
    projectType: 'unknown'
  };

  try {
    // 1. Get directory structure (Top level + 1 deep)
    metadata.structure = execSync('dir /B /A:D', { encoding: 'utf8' })
      .split('\n')
      .filter(line => line.trim() !== '' && !line.includes('.git') && !line.includes('node_modules'))
      .join('\n');

    // 2. Identify key files
    const criticalFiles = ['package.json', 'go.mod', 'requirements.txt', 'Cargo.toml', 'GEMINI.md', 'README.md'];
    criticalFiles.forEach(file => {
      if (fs.existsSync(file)) {
        metadata.keyFiles[file] = true;
        if (file === 'package.json') metadata.projectType = 'Node.js';
        if (file === 'go.mod') metadata.projectType = 'Go';
        if (file === 'requirements.txt') metadata.projectType = 'Python';
        if (file === 'Cargo.toml') metadata.projectType = 'Rust';
      }
    });

    console.log(JSON.stringify(metadata, null, 2));
  } catch (error) {
    console.error('Error gathering metadata:', error.message);
    process.exit(1);
  }
}

gatherProjectMetadata();
