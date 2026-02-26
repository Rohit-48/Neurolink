#!/usr/bin/env node
import { resolve } from "path";
import { startNeurolink } from "../src/server.js";

function parseArgs(argv) {
  const out = {
    port: 3000,
    storage: "./shared",
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if ((arg === "--port" || arg === "-p") && argv[i + 1]) {
      out.port = Number.parseInt(argv[i + 1], 10);
      i += 1;
    } else if ((arg === "--storage" || arg === "-s") && argv[i + 1]) {
      out.storage = argv[i + 1];
      i += 1;
    }
  }

  if (!Number.isFinite(out.port) || out.port <= 0) {
    out.port = 3000;
  }
  return out;
}

const args = parseArgs(process.argv.slice(2));
startNeurolink({
  port: args.port,
  storage: resolve(args.storage),
}).catch((err) => {
  console.error("Failed to start neurolink:", err);
  process.exit(1);
});
