#!/usr/bin/env node

const { spawnSync } = require('child_process');
const path = require('path');
const os = require('os');
const fs = require('fs');

const binaryName = os.platform() === 'win32' ? 'vimtelligence-gui.exe' : 'vimtelligence-gui';
const globalBinPath = path.join(__dirname, '..', binaryName);
const localBinPath = path.join(os.homedir(), '.vimtelligence', 'bin', binaryName);

let exePath = '';

if (fs.existsSync(localBinPath)) {
    exePath = localBinPath;
} else if (fs.existsSync(globalBinPath)) {
    exePath = globalBinPath;
} else {
    console.error("Vimtelligence executable not found. Did the postinstall script run successfully?");
    process.exit(1);
}

const result = spawnSync(exePath, process.argv.slice(2), { stdio: 'inherit' });

if (result.error) {
    console.error(`Failed to start Vimtelligence: ${result.error.message}`);
    process.exit(1);
}

process.exit(result.status);
