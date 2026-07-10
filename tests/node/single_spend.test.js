import test from "node:test";
import assert from "node:assert/strict";
import { runScenarioFailure } from "../helpers/nocturne.js";

test("rejects replay of the same commitment", () => {
  const err = runScenarioFailure([
    {
      op: "deposit",
      note: "n0",
      owner: "bob",
      account: "bob-main",
      asset: "DTL",
      amount: 100,
      position: "bob-p1",
      window: 1,
      blinding: "bob-b0",
    },
    { op: "spend", note: "n0", recipient: "bob-main", memo: "first clearing" },
    { op: "spend", note: "n0", recipient: "bob-main", memo: "replay" },
  ]);

  assert.match(err, /commitment already consumed|nullifier already spent/);
});
