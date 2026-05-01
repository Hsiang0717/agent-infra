const { execSync } = require('child_process');

function getGitStatus() {
  try {
    const status = execSync('git status --porcelain', { encoding: 'utf8' });
    const lines = status.split('\n').filter(line => line.trim() !== '');
    
    const groups = {
      agents: [],
      extensions: [],
      skills: [],
      docs: [],
      chore: [],
      other: []
    };

    lines.forEach(line => {
      const filePath = line.substring(3);
      if (filePath.startsWith('agents/')) groups.agents.push(filePath);
      else if (filePath.startsWith('extensions/')) groups.extensions.push(filePath);
      else if (filePath.startsWith('skills/')) groups.skills.push(filePath);
      else if (filePath.startsWith('docs/')) groups.docs.push(filePath);
      else if (filePath.includes('.gitignore') || filePath.includes('package.json') || filePath.includes('pnpm-lock.yaml')) groups.chore.push(filePath);
      else groups.other.push(filePath);
    });

    return groups;
  } catch (error) {
    console.error('Error running git status:', error.message);
    process.exit(1);
  }
}

const statusGroups = getGitStatus();
console.log(JSON.stringify(statusGroups, null, 2));
