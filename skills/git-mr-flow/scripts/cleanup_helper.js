const { execSync } = require('child_process');

function getDefaultBranch() {
  try {
    // 1. Try to read local tracking info for origin HEAD (very fast, no network)
    const localRemoteHead = execSync('git symbolic-ref refs/remotes/origin/HEAD', { encoding: 'utf8', stdio: 'pipe' }).trim();
    const match = localRemoteHead.match(/origin\/(.*)/);
    if (match) return match[1].trim();
  } catch (e) {
    // Fallback if local remote HEAD tracking is not set
  }

  try {
    // 2. Try to get default branch from remote, fallback to main
    const remoteInfo = execSync('git remote show origin', { encoding: 'utf8', stdio: 'pipe' });
    const match = remoteInfo.match(/HEAD branch: (.*)/);
    return match ? match[1].trim() : 'main';
  } catch (e) {
    return 'main';
  }
}

function cleanup() {
  try {
    // Check if workspace is dirty before switching branches to avoid conflicts or loss of work
    const workspaceStatus = execSync('git status --porcelain', { encoding: 'utf8' }).trim();
    if (workspaceStatus !== '') {
      console.error('\n❌ Error: Working directory is dirty. Please stash or commit changes before cleanup.\n');
      process.exit(1);
    }

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
      console.warn(`Please ensure the MR is approved and merged on the remote platform first.\n`);
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
