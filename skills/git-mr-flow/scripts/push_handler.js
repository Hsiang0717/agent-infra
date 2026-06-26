const { execSync } = require('child_process');

function getRemoteUrl() {
  try {
    return execSync('git remote get-url origin', { encoding: 'utf8' }).trim();
  } catch (e) {
    return '';
  }
}

function generatePRLink(remoteUrl, branch) {
  if (remoteUrl.includes('github.com')) {
    const base = remoteUrl.replace('.git', '').replace('git@github.com:', 'https://github.com/');
    return `${base}/pull/new/${branch}`;
  } else if (remoteUrl.includes('gitlab.com')) {
    const base = remoteUrl.replace('.git', '');
    return `${base}/-/merge_requests/new?merge_request%5Bsource_branch%5D=${branch}`;
  }
  return 'N/A';
}

function pushAndGuard() {
  try {
    const currentBranch = execSync('git rev-parse --abbrev-ref HEAD', { encoding: 'utf8' }).trim();
    const remoteUrl = getRemoteUrl();

    if (currentBranch === 'main' || currentBranch === 'master') {
      console.error('Error: Cannot push directly from main/master in collaborative mode.');
      process.exit(1);
    }

    console.log(`Pushing branch: ${currentBranch} to origin...`);
    
    try {
      execSync(`git push -u origin ${currentBranch}`, { stdio: 'inherit' });
    } catch (pushError) {
      console.error('\n' + '!'.repeat(60));
      console.error('❌ PUSH FAILED');
      console.error('Possible causes:');
      console.error('1. You do not have write permissions to this remote repository.');
      console.error('2. Your SSH keys or Access Tokens (HTTPS) are not correctly configured.');
      console.error('3. The remote repository URL is invalid.');
      console.error('Please verify your Git setup and try pushing manually.');
      console.error('!'.repeat(60) + '\n');
      throw pushError;
    }

    const mrLink = generatePRLink(remoteUrl, currentBranch);
    
    console.log('\n' + '='.repeat(60));
    console.log('🚀 PUSH SUCCESSFUL');
    console.log(`🔗 MR Link: ${mrLink}`);
    console.log('='.repeat(60));
    console.log('\n[GATEKEEPER PROTOCOL ACTIVATED]');
    console.log('DO NOT perform any local merges or deletions.');
    console.log('Please wait for the Merge Request (MR) to be reviewed and merged on the remote platform.');
    console.log('='.repeat(60) + '\n');

  } catch (error) {
    console.error(`Execution failed: ${error.message}`);
    process.exit(1);
  }
}

pushAndGuard();
