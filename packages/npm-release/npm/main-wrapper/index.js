"use strict";

// Optional: expose programmatic helpers (paths, spawn helpers, etc.)

const os = require("os");
const path = require("path");
const fs = require("fs");

function platformPackageName() {
  const p = os.platform();
  const a = os.arch();

  if (p === "darwin" && a === "arm64")
    return "@sansavision/vidra-darwin-arm64";
  if (p === "darwin" && a === "x64") return "@sansavision/vidra-darwin-x64";
  if (p === "linux" && a === "x64")
    return "@sansavision/vidra-linux-x64-musl";
  if (p === "linux" && a === "arm64")
    return "@sansavision/vidra-linux-arm64-musl";
  if (p === "win32" && a === "x64")
    return "@sansavision/vidra-win32-x64-msvc";

  return null;
}

function resolveBin(name) {
  const pkg = platformPackageName();
  if (!pkg) return null;

  let pkgJsonPath;
  try {
    pkgJsonPath = require.resolve(`${pkg}/package.json`);
  } catch {
    return null;
  }

  const root = path.dirname(pkgJsonPath);
  const p = path.join(root, "bin", name);
  try {
    fs.accessSync(p);
    return p;
  } catch {
    return null;
  }
}

module.exports = { platformPackageName, resolveBin };
