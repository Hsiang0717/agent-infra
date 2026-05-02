const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

function parseConfig() {
  const configPath = '.git-feat-flow.md';
  if (!fs.existsSync(configPath)) return null;

  const content = fs.readFileSync(configPath, 'utf8');
  const match = content.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  if (!match) return null;

  const config = { mappings: [], global_rules: [] };
  const lines = match[1].split('\n');
  let currentSection = null;

  lines.forEach(line => {
    const trimmed = line.trim();
    if (trimmed.startsWith('mappings:')) {
      currentSection = 'mappings';
    } else if (trimmed.startsWith('global_rules:')) {
      currentSection = 'global_rules';
    } else if (trimmed.startsWith('- path:')) {
      const p = trimmed.replace('- path:', '').trim().replace(/['"]/g, '');
      config.mappings.push({ path: p });
    } else if (trimmed.startsWith('scope:') && currentSection === 'mappings') {
      if (config.mappings.length > 0) {
        config.mappings[config.mappings.length - 1].scope = trimmed.replace('scope:', '').trim().replace(/['"]/g, '');
      }
    } else if (trimmed.startsWith('pattern:') && currentSection === 'global_rules') {
      const p = trimmed.replace('pattern:', '').trim().replace(/['"]/g, '');
      config.global_rules.push({ pattern: p });
    } else if (trimmed.startsWith('type:') && currentSection === 'global_rules') {
      if (config.global_rules.length > 0) {
        config.global_rules[config.global_rules.length - 1].type = trimmed.replace('type:', '').trim().replace(/['"]/g, '');
      }
    }
  });

  return config;
}

function getGitStatus(config) {
  try {
    const status = execSync('git status --porcelain', { encoding: 'utf8' });
    const lines = status.split('\n').filter(line => line.trim() !== '');
    const groups = {};

    lines.forEach(line => {
      const filePath = line.substring(3).replace(/\\/g, '/');
      let assigned = false;

      // 1. Check global rules
      if (config && config.global_rules) {
        for (const rule of config.global_rules) {
          if (filePath.endsWith(rule.pattern) || filePath === rule.pattern) {
            const scope = 'root';
            if (!groups[scope]) groups[scope] = [];
            groups[scope].push({ file: filePath, type: rule.type });
            assigned = true;
            break;
          }
        }
      }

      // 2. Check mappings
      if (!assigned && config && config.mappings) {
        for (const mapping of config.mappings) {
          if (filePath.startsWith(mapping.path)) {
            const scope = mapping.scope || mapping.path.split('/')[0];
            if (!groups[scope]) groups[scope] = [];
            groups[scope].push({ file: filePath });
            assigned = true;
            break;
          }
        }
      }

      // 3. Fallback
      if (!assigned) {
        const topDir = filePath.split('/')[0];
        const scope = topDir.includes('.') ? 'root' : topDir;
        if (!groups[scope]) groups[scope] = [];
        groups[scope].push({ file: filePath });
      }
    });

    return groups;
  } catch (error) {
    console.error(JSON.stringify({ error: error.message }));
    process.exit(1);
  }
}

const config = parseConfig();
const statusGroups = getGitStatus(config);
console.log(JSON.stringify(statusGroups, null, 2));
