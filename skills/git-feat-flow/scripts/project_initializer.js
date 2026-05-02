const fs = require('fs');
const path = require('path');

function gatherProjectMetadata() {
  const metadata = {
    structure: [],
    keyFiles: [],
    projectType: 'unknown'
  };

  try {
    const rootDir = process.cwd();
    
    // 1. Get directory structure (Top level)
    const files = fs.readdirSync(rootDir);
    metadata.structure = files.filter(f => {
      try {
        const stats = fs.statSync(path.join(rootDir, f));
        return stats.isDirectory() && !f.startsWith('.') && f !== 'node_modules';
      } catch (e) { return false; }
    });

    // 2. Identify key files
    const criticalFiles = ['package.json', 'go.mod', 'requirements.txt', 'Cargo.toml', 'GEMINI.md', 'README.md', 'pyproject.toml'];
    metadata.keyFiles = files.filter(f => criticalFiles.includes(f));

    if (metadata.keyFiles.includes('package.json')) metadata.projectType = 'Node.js';
    else if (metadata.keyFiles.includes('go.mod')) metadata.projectType = 'Go';
    else if (metadata.keyFiles.includes('requirements.txt') || metadata.keyFiles.includes('pyproject.toml')) metadata.projectType = 'Python';
    else if (metadata.keyFiles.includes('Cargo.toml')) metadata.projectType = 'Rust';

    console.log(JSON.stringify(metadata, null, 2));
  } catch (error) {
    console.error(JSON.stringify({ error: error.message }));
    process.exit(1);
  }
}

gatherProjectMetadata();
