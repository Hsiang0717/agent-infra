const { execSync } = require('child_process');

function getDefaultBranch() {
  try {
    // Try to get default branch from remote, fallback to main
    const remoteInfo = execSync('git remote show origin', { encoding: 'utf8' });
    const match = remoteInfo.match(/HEAD branch: (.*)/);
    return match ? match[1] : 'main';
  } catch (e) {
    return 'main';
  }
}

function cleanup() {
  try {
    const currentBranch = execSync('git rev-parse --abbrev-ref HEAD', { encoding: 'utf8' }).trim();
    const mainBranch = getDefaultBranch();

    if (currentBranch === mainBranch || currentBranch === 'master') {
      console.log(`Already on ${currentBranch}. Syncing from origin...`);
      execSync(`git pull origin ${currentBranch}`, { stdio: 'inherit' });
      execSync('git fetch --prune', { stdio: 'inherit' });
      return;
    }

    // Collaborative check: Is it merged on remote?
    console.log(`Verifying merge status for ${currentBranch}...`);
    execSync('git fetch origin', { stdio: 'pipe' });
    const mergedBranches = execSync(`git branch -r --merged origin/${mainBranch}`, { encoding: 'utf8' });
    
    if (!mergedBranches.includes(`origin/${currentBranch}`)) {
      console.warn(`\n⚠️  WARNING: Branch '${currentBranch}' does not appear to be merged into 'origin/${mainBranch}'.`);
      console.warn(`Please ensure the PR/MR is approved and merged on the remote platform first.\n`);
      // We don't exit(1) because the user might have merged it and we just need to pull, 
      // but we will proceed with caution.
    }

    console.log(`Switching to ${mainBranch}...`);
    execSync(`git checkout ${mainBranch}`, { stdio: 'inherit' });
    execSync(`git pull origin ${mainBranch}`, { stdio: 'inherit' });
    execSync('git fetch --prune', { stdio: 'inherit' });

    try {
      // Use -d to ensure it only deletes if merged locally
      execSync(`git branch -d ${currentBranch}`, { stdio: 'inherit' });
      console.log(`\n✅ Local branch ${currentBranch} deleted safely.`);
    } catch (e) {
      console.log(`\nℹ️  Branch ${currentBranch} was not deleted (it may contain unmerged commits).`);
      console.log(`Use 'git branch -D ${currentBranch}' if you want to force delete it.`);
    }

    console.log('✨ Workspace synchronized.');
  } catch (error) {
    console.error(`Cleanup failed: ${error.message}`);
    process.exit(1);
  }
}

cleanup();
