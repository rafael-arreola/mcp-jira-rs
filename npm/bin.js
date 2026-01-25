#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const binName = process.platform === 'win32' ? 'jira-rs.exe' : 'jira-rs';
const binPath = path.join(__dirname, 'bin', binName);

if (!fs.existsSync(binPath)) {
  console.error(`Binary not found at ${binPath}`);
  console.error('Please try reinstalling the package.');
  process.exit(1);
}

const child = spawn(binPath, process.argv.slice(2), {
  stdio: 'inherit',
  env: process.env
});

child.on('close', (code) => {
  process.exit(code);
});

child.on('error', (err) => {
  console.error(err);
  process.exit(1);
});
