const { execSync } = require('child_process');

function checkPreRebase() {
  try {
    // 1. Check branch
    const branch = execSync('git rev-parse --abbrev-ref HEAD', { encoding: 'utf8' }).trim();
    if (branch === 'main' || branch === 'master') {
      console.log('Error: You are on the main branch. Please switch to a feature branch.');
      process.exit(1);
    }

    // 2. Check dirty status
    const status = execSync('git status --porcelain', { encoding: 'utf8' }).trim();
    if (status !== '') {
      console.log('Error: Your working directory is dirty. Please stash or commit your changes first.');
      process.exit(1);
    }

    console.log('Pre-rebase check passed.');
  } catch (error) {
    console.error('Git error:', error.message);
    process.exit(1);
  }
}

checkPreRebase();
