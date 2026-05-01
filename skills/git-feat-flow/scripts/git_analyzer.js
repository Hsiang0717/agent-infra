const { execSync } = require('child_process');
const fs = require('fs');

function parseConfig() {
  const configPath = '.git-feat-flow.md';
  if (!fs.existsSync(configPath)) {
    return null;
  }

  const content = fs.readFileSync(configPath, 'utf8');
  const match = content.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  
  if (!match) return null;

  // Simple YAML-like parser for the frontmatter
  const yamlLines = match[1].split('\n');
  const config = { mappings: [], global_rules: [] };
  let currentSection = null;

  yamlLines.forEach(line => {
    const trimmed = line.trim();
    if (trimmed.startsWith('mappings:')) currentSection = 'mappings';
    else if (trimmed.startsWith('global_rules:')) currentSection = 'global_rules';
    else if (trimmed.startsWith('- path:')) {
      const path = trimmed.replace('- path:', '').trim().replace(/"/g, '');
      config.mappings.push({ path });
    } else if (trimmed.startsWith('scope:')) {
      if (config.mappings.length > 0) config.mappings[config.mappings.length - 1].scope = trimmed.replace('scope:', '').trim().replace(/"/g, '');
    } else if (trimmed.startsWith('default_type:')) {
      if (config.mappings.length > 0) config.mappings[config.mappings.length - 1].default_type = trimmed.replace('default_type:', '').trim().replace(/"/g, '');
    }
    // Simplification: only handling mappings for this demonstration
  });

  return config;
}

function getGitStatus(config) {
  try {
    const status = execSync('git status --porcelain', { encoding: 'utf8' });
    const lines = status.split('\n').filter(line => line.trim() !== '');
    
    const groups = {};

    lines.forEach(line => {
      const filePath = line.substring(3);
      let assigned = false;

      if (config && config.mappings) {
        for (const mapping of config.mappings) {
          if (filePath.startsWith(mapping.path)) {
            const scope = mapping.scope || mapping.path.replace('/', '');
            if (!groups[scope]) groups[scope] = [];
            groups[scope].push(filePath);
            assigned = true;
            break;
          }
        }
      }

      if (!assigned) {
        const topDir = filePath.split('/')[0];
        const scope = topDir.includes('.') ? 'root' : topDir;
        if (!groups[scope]) groups[scope] = [];
        groups[scope].push(filePath);
      }
    });

    return groups;
  } catch (error) {
    console.error('Error running git status:', error.message);
    process.exit(1);
  }
}

const config = parseConfig();
const statusGroups = getGitStatus(config);
console.log(JSON.stringify(statusGroups, null, 2));
