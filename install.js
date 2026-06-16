const fs = require('fs');
const path = require('path');
const https = require('https');
const os = require('os');

const pkg = require('./package.json');
const VERSION = `v${pkg.version}`;

const MODEL_URL = 'https://huggingface.co/unsloth/gemma-3-270m-it-GGUF/resolve/main/gemma-3-270m-it-UD-Q8_K_XL.gguf';
const MODEL_DIR = path.join(os.homedir(), '.vimtelligence', 'models');
const MODEL_PATH = path.join(MODEL_DIR, 'gemma-3-270m-it-UD-Q8_K_XL.gguf');

const BIN_URL = `https://github.com/turtle170/Vimtelligence/releases/download/${VERSION}/vimtelligence.exe`;
const GUI_BIN_URL = `https://github.com/turtle170/Vimtelligence/releases/download/${VERSION}/vimtelligence-gui.exe`;
const BIN_DIR = path.join(os.homedir(), '.vimtelligence', 'bin');
const BIN_PATH = path.join(BIN_DIR, 'vimtelligence.exe');
const GUI_BIN_PATH = path.join(BIN_DIR, 'vimtelligence-gui.exe');

function downloadFile(url, dest) {
// ... unchanged ...
}

async function main() {
  if (!fs.existsSync(MODEL_DIR)) {
    fs.mkdirSync(MODEL_DIR, { recursive: true });
  }
  
  if (!fs.existsSync(BIN_DIR)) {
    fs.mkdirSync(BIN_DIR, { recursive: true });
  }

  if (!fs.existsSync(MODEL_PATH)) {
    console.log(`Model not found locally. Downloading from Hugging Face...`);
    try {
      await downloadFile(MODEL_URL, MODEL_PATH);
    } catch (err) {
      console.error(`Error downloading model: ${err.message}`);
    }
  } else {
    console.log(`Model already exists at ${MODEL_PATH}. Skipping download.`);
  }

  // Only download binary on windows for now since it's the only one built in CI
  if (os.platform() === 'win32') {
    console.log(`Downloading Vimtelligence ${VERSION} Executables from GitHub Releases...`);
    try {
      await downloadFile(BIN_URL, BIN_PATH);
      await downloadFile(GUI_BIN_URL, GUI_BIN_PATH);
    } catch (err) {
      console.error(`Error downloading binary: ${err.message}`);
    }
  } else {
      console.log(`Please build from source on non-Windows platforms. Binary not provided via NPM yet.`);
  }
}

main();
