#!/usr/bin/env node
"use strict";

const { spawnSync } = require("child_process");
const os = require("os");
const { resolveBin } = require("../index");

const exe = os.platform() === "win32" ? "vidra.exe" : "vidra";
const bin = resolveBin(exe);

if (!bin) {
  console.error(
    "vidra: native binary not found for this platform.\n" +
      "Reinstall to ensure optionalDependencies are installed: npm install @sansavision/vidra\n",
  );
  process.exit(1);
}

const r = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });
if (r.error) {
  console.error("vidra: failed to launch:", r.error.message);
  process.exit(1);
}
process.exit(r.status ?? 0);
