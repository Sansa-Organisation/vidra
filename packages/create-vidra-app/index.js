#!/usr/bin/env node

// ─── create-vidra-app ───────────────────────────────────────────────
// Scaffolds a new Vidra project with SDK, Player, and Web Capture
// ready to go.
//
// Usage:
//   npx @sansavision/create-vidra-app my-video
//   cd my-video && npm install && npm run dev

import { mkdirSync, writeFileSync, readFileSync, readdirSync, statSync, copyFileSync } from "fs";
import { join, dirname, resolve, basename } from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const args = process.argv.slice(2);
const projectName = args[0];

if (!projectName) {
  console.log(`
  \x1b[1m\x1b[35m🎬 create-vidra-app\x1b[0m

  Scaffold a new Vidra video project.

  \x1b[1mUsage:\x1b[0m
    npx @sansavision/create-vidra-app \x1b[36m<project-name>\x1b[0m

  \x1b[1mExample:\x1b[0m
    npx @sansavision/create-vidra-app my-video
    cd my-video
    npm install
    npm run dev
  `);
  process.exit(1);
}

const targetDir = resolve(process.cwd(), projectName);
const templateDir = join(__dirname, "template");
const baseName = basename(targetDir);

console.log();
console.log(`  \x1b[1m\x1b[35m🎬 Creating Vidra project:\x1b[0m ${projectName}`);
console.log();

// Recursively copy template directory
function copyDir(src, dest) {
  mkdirSync(dest, { recursive: true });
  for (const entry of readdirSync(src)) {
    const srcPath = join(src, entry);
    const destPath = join(dest, entry);
    if (statSync(srcPath).isDirectory()) {
      copyDir(srcPath, destPath);
    } else {
      let content = readFileSync(srcPath, "utf-8");
      // Replace template variables
      content = content.replace(/\{\{PROJECT_NAME\}\}/g, baseName);
      writeFileSync(destPath, content);
    }
  }
}

copyDir(templateDir, targetDir);

console.log(`  \x1b[32m✓\x1b[0m Project scaffolded at \x1b[1m${targetDir}\x1b[0m`);
console.log();
console.log(`  \x1b[1mNext steps:\x1b[0m`);
console.log();
console.log(`    cd ${projectName}`);
console.log(`    npm install`);
console.log(`    npm run dev          \x1b[2m# Live browser preview\x1b[0m`);
console.log(`    npm run build:video  \x1b[2m# Generate project IR JSON\x1b[0m`);
console.log();
console.log(`  \x1b[2mTo render to MP4 (requires Vidra CLI):\x1b[0m`);
console.log(`    vidra render video.vidra -o output.mp4`);
console.log();
console.log(`  \x1b[35m📖 Docs:\x1b[0m https://github.com/Sansa-Organisation/vidra`);
console.log();
