const fs = require('fs');
const path = require('path');
const https = require('https');

const MODEL_URL = 'https://huggingface.co/unsloth/gemma-3-270m-it-GGUF/resolve/main/gemma-3-270m-it-UD-Q8_K_XL.gguf';
const MODEL_DIR = path.join(__dirname, 'models');
const MODEL_PATH = path.join(MODEL_DIR, 'gemma-3-270m-it-UD-Q8_K_XL.gguf');

function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    https.get(url, (response) => {
      if (response.statusCode === 301 || response.statusCode === 302) {
        return downloadFile(response.headers.location, dest).then(resolve).catch(reject);
      }
      
      if (response.statusCode !== 200) {
        return reject(new Error(`Failed to download ${url}: ${response.statusCode}`));
      }

      const totalSize = parseInt(response.headers['content-length'], 10);
      let downloaded = 0;

      response.on('data', (chunk) => {
        downloaded += chunk.length;
        const percent = ((downloaded / totalSize) * 100).toFixed(2);
        process.stdout.write(`Downloading model... ${percent}%\r`);
      });

      response.pipe(file);
      file.on('finish', () => {
        file.close();
        console.log(`\nDownloaded ${dest}`);
        resolve();
      });
    }).on('error', (err) => {
      fs.unlink(dest, () => {});
      reject(err);
    });
  });
}

async function main() {
  if (!fs.existsSync(MODEL_DIR)) {
    fs.mkdirSync(MODEL_DIR, { recursive: true });
  }

  if (!fs.existsSync(MODEL_PATH)) {
    console.log(`Model not found locally. Downloading from Hugging Face...`);
    try {
      await downloadFile(MODEL_URL, MODEL_PATH);
    } catch (err) {
      console.error(`Error downloading model: ${err.message}`);
      process.exit(1);
    }
  } else {
    console.log(`Model already exists at ${MODEL_PATH}. Skipping download.`);
  }
}

main();
