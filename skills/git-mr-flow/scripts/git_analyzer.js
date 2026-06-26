const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

function parseConfig() {
  const configPath = '.git-mr-flow.md';
  if (!fs.existsSync(configPath)) return null;

  try {
    const content = fs.readFileSync(configPath, 'utf8');
    const match = content.match(/^---\r?\n([\s\S]*?)\r?\n---/);
    if (!match) return null;

    const config = { mappings: [], global_rules: [] };
    const lines = match[1].split(/\r?\n/);
    let currentSection = null;

    lines.forEach(line => {
      // Remove comments and trim whitespace
      const cleanLine = line.split('#')[0].trim();
      if (!cleanLine) return;

      if (cleanLine.startsWith('mappings:')) {
        currentSection = 'mappings';
        return;
      }
      if (cleanLine.startsWith('global_rules:')) {
        currentSection = 'global_rules';
        return;
      }
      if (cleanLine.startsWith('workflow_mode:')) {
        config.workflow_mode = cleanLine.replace('workflow_mode:', '').trim().replace(/['"]/g, '');
        return;
      }

      // Parse list items
      if (currentSection === 'mappings') {
        if (cleanLine.startsWith('-')) {
          const rest = cleanLine.substring(1).trim();
          const item = {};
          if (rest.startsWith('path:')) {
            item.path = rest.replace('path:', '').trim().replace(/['"]/g, '');
          }
          config.mappings.push(item);
        } else {
          const parts = cleanLine.split(':');
          if (parts.length >= 2) {
            const key = parts[0].trim();
            const val = parts.slice(1).join(':').trim().replace(/['"]/g, '');
            if (config.mappings.length > 0) {
              config.mappings[config.mappings.length - 1][key] = val;
            }
          }
        }
      } else if (currentSection === 'global_rules') {
        if (cleanLine.startsWith('-')) {
          const rest = cleanLine.substring(1).trim();
          const item = {};
          if (rest.startsWith('pattern:')) {
            item.pattern = rest.replace('pattern:', '').trim().replace(/['"]/g, '');
          }
          config.global_rules.push(item);
        } else {
          const parts = cleanLine.split(':');
          if (parts.length >= 2) {
            const key = parts[0].trim();
            const val = parts.slice(1).join(':').trim().replace(/['"]/g, '');
            if (config.global_rules.length > 0) {
              config.global_rules[config.global_rules.length - 1][key] = val;
            }
          }
        }
      }
    });

    return config;
  } catch (e) {
    return null;
  }
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
