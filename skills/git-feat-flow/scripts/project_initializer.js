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

    // 3. Detect Remotes and collaborative environment
    let remotes = '';
    try {
      const { execSync } = require('child_process');
      remotes = execSync('git remote -v', { encoding: 'utf8' });
    } catch (e) {}
    
    metadata.remotes = remotes;
    metadata.isCollaborative = remotes.includes('github.com') || remotes.includes('gitlab.com') || remotes.includes('bitbucket.org');
    metadata.suggestedMode = metadata.isCollaborative ? 'collaborative' : 'local';

    console.log(JSON.stringify(metadata, null, 2));
  } catch (error) {
    console.error(JSON.stringify({ error: error.message }));
    process.exit(1);
  }
}

gatherProjectMetadata();
