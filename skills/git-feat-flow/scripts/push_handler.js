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
    execSync(`git push -u origin ${currentBranch}`, { stdio: 'inherit' });

    const prLink = generatePRLink(remoteUrl, currentBranch);
    
    console.log('\n' + '='.repeat(60));
    console.log('🚀 PUSH SUCCESSFUL');
    console.log(`🔗 PR/MR Link: ${prLink}`);
    console.log('='.repeat(60));
    console.log('\n[GATEKEEPER PROTOCOL ACTIVATED]');
    console.log('DO NOT perform any local merges or deletions.');
    console.log('Please wait for the Merge Request to be reviewed and merged on the remote platform.');
    console.log('='.repeat(60) + '\n');

  } catch (error) {
    console.error(`Push failed: ${error.message}`);
    process.exit(1);
  }
}

pushAndGuard();
