const { execSync } = require('child_process');

function cleanup() {
  try {
    const currentBranch = execSync('git rev-parse --abbrev-ref HEAD', { encoding: 'utf8' }).trim();
    
    if (currentBranch === 'main' || currentBranch === 'master') {
      console.log('You are already on the main branch. Checking for remote updates and pruning...');
      execSync('git pull origin main', { stdio: 'inherit' });
      execSync('git fetch --prune', { stdio: 'inherit' });
      return;
    }

    console.log(`Mission Complete! Starting cleanup for branch: ${currentBranch}`);

    // 1. Switch to main
    console.log('Switching to main branch...');
    execSync('git checkout main', { stdio: 'inherit' });

    // 2. Pull latest main
    console.log('Syncing with remote main...');
    execSync('git pull origin main', { stdio: 'inherit' });

    // 3. Prune remote
    console.log('Pruning remote tracking branches...');
    execSync('git fetch --prune', { stdio: 'inherit' });

    // 4. Delete the feature branch safely
    try {
      console.log(`Deleting local branch: ${currentBranch}...`);
      execSync(`git branch -d ${currentBranch}`, { stdio: 'inherit' });
      console.log('✅ Branch deleted successfully.');
    } catch (e) {
      console.log(`⚠️  Branch ${currentBranch} could not be deleted with -d (perhaps not fully merged).`);
      console.log('If you are SURE you want to delete it, use git branch -D [name].');
    }

    console.log('\n✨ Workspace is now clean and synced with main.');
  } catch (error) {
    console.error('Cleanup failed:', error.message);
    process.exit(1);
  }
}

cleanup();
