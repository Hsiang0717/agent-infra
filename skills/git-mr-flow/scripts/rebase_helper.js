const { execSync } = require('child_process');

function checkPreRebase() {
  try {
    const branch = execSync('git rev-parse --abbrev-ref HEAD', { encoding: 'utf8' }).trim();
    if (branch === 'main' || branch === 'master') {
      console.error(JSON.stringify({ error: 'Cannot rebase while on main branch.' }));
      process.exit(1);
    }

    const status = execSync('git status --porcelain', { encoding: 'utf8' }).trim();
    if (status !== '') {
      console.error(JSON.stringify({ error: 'Working directory is dirty. Please stash or commit changes.' }));
      process.exit(1);
    }

    console.log(JSON.stringify({ status: 'ready' }));
  } catch (error) {
    console.error(JSON.stringify({ error: error.message }));
    process.exit(1);
  }
}

checkPreRebase();
