import test from "node:test";
import assert from "node:assert/strict";
import { runScenario } from "../helpers/nocturne.js";

test("supports settlement after note rollover into a later window", () => {
  const out = runScenario([
    { op: "open_window", window: 2, anchor: "rollover-window" },
    {
      op: "deposit",
      note: "origin",
      owner: "dana",
      account: "dana-main",
      asset: "DTL",
      amount: 180,
      position: "dana-p1",
      window: 1,
      blinding: "dana-b0",
    },
    { op: "roll", source: "origin", note: "rolled", window: 2, blinding: "dana-b1" },
    { op: "spend", note: "rolled", recipient: "dana-main", memo: "cycle close" },
    {
      op: "withdraw",
      account: "dana-main",
      asset: "DTL",
      amount: 180,
      destination: "dtl:dana",
      window: 2,
    },
    { op: "reconcile" },
  ]);

  assert.equal(out.notes.rolled.window, 2);
  assert.equal(out.reconciliations[0].balanced, true);
  assert.equal(out.snapshot.commitments, 2);
});
