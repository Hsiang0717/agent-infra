const { execSync } = require('child_process');

function cleanup() {
  try {
    const currentBranch = execSync('git rev-parse --abbrev-ref HEAD', { encoding: 'utf8' }).trim();
    const mainBranch = 'main';

    if (currentBranch === mainBranch || currentBranch === 'master') {
      console.log(`Already on ${currentBranch}. Syncing...`);
      execSync(`git pull origin ${currentBranch}`, { stdio: 'pipe' });
      execSync('git fetch --prune', { stdio: 'pipe' });
      return;
    }

    console.log(`Starting cleanup for: ${currentBranch}`);

    execSync(`git checkout ${mainBranch}`, { stdio: 'pipe' });
    execSync(`git pull origin ${mainBranch}`, { stdio: 'pipe' });
    execSync('git fetch --prune', { stdio: 'pipe' });

    try {
      execSync(`git branch -d ${currentBranch}`, { stdio: 'pipe' });
      console.log(`Branch ${currentBranch} deleted.`);
    } catch (e) {
      console.log(`Branch ${currentBranch} preserved (not merged locally).`);
    }

    console.log('Workspace cleaned.');
  } catch (error) {
    console.error(JSON.stringify({ error: error.message }));
    process.exit(1);
  }
}

cleanup();
