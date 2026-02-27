#!/usr/bin/env node
import { resolve } from "path";
import { startNeurolink } from "../src/server.js";

function printNeurolinkBanner() {
  const lines = [
    "                  _   _  _____ _   _ ____   ___  _     ___ _   _ _  __",
    "                 | \\ | || ____| | | |  _ \\ / _ \\| |   |_ _| \\ | | |/ /",
    "                 |  \\| ||  _| | | | | |_) | | | | |    | ||  \\| | ' / ",
    "                 | |\\  || |___| |_| |  _ <| |_| | |___ | || |\\  | . \\ ",
    "                 |_| \\_||_____|\\___/|_| \\_\\\\___/|_____|___|_| \\_|_|\\_\\",
  ];
  const shades = [97, 37, 96, 37, 97];
  console.log("");
  lines.forEach((line, idx) => {
    console.log(`\x1b[1;${shades[idx]}m${line}\x1b[0m`);
  });
  console.log("\x1b[1;97m                     NEUROLINK · Express Runtime\x1b[0m");
  console.log("");
}

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
printNeurolinkBanner();
startNeurolink({
  port: args.port,
  storage: resolve(args.storage),
}).catch((err) => {
  console.error("Failed to start neurolink:", err);
  process.exit(1);
});
