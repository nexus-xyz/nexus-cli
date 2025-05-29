#!/usr/bin/env node

const {execSync} = require('child_process');

const installScriptUrl = 'https://cli.nexus.xyz/';
const curlCommand = `curl -fsSL ${installScriptUrl} | sh`;

try {
    console.log('Installing Nexus CLI...');
    execSync(curlCommand, {stdio: 'inherit'});
    console.log('Nexus CLI installation complete!');
} catch (err) {
    console.error('Installation failed:', err);
    process.exit(1);
}