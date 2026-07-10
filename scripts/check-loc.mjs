import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../src", import.meta.url));
let lines = 0;

function walk(dir) {
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry);
    const stat = statSync(path);
    if (stat.isDirectory()) {
      walk(path);
      continue;
    }
    if (!/\.rs$/.test(entry)) continue;
    const content = readFileSync(path, "utf8");
    lines += content.split(/\r?\n/).filter((line) => line.trim().length > 0).length;
  }
}

walk(root);

if (lines < 5000 || lines > 6000) {
  console.error(`src LOC out of expected range: ${lines}`);
  process.exit(1);
}

console.log(`src LOC: ${lines}`);
