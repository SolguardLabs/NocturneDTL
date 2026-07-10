import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
export const root = resolve(here, "../..");

export function runScenario(operations) {
  const result = spawnSync("cargo", ["run", "--quiet", "--", "--json"], {
    cwd: root,
    input: JSON.stringify({ operations }),
    encoding: "utf8",
    windowsHide: true,
  });
  if (result.status !== 0) {
    throw new Error(result.stderr || result.stdout || `scenario failed with ${result.status}`);
  }
  return JSON.parse(result.stdout);
}

export function runScenarioFailure(operations) {
  const result = spawnSync("cargo", ["run", "--quiet", "--", "--json"], {
    cwd: root,
    input: JSON.stringify({ operations }),
    encoding: "utf8",
    windowsHide: true,
  });
  if (result.status === 0) {
    throw new Error(`scenario unexpectedly passed: ${result.stdout}`);
  }
  return result.stderr.trim();
}
