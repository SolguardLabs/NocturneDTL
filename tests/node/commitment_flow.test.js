import test from "node:test";
import assert from "node:assert/strict";
import { runScenario } from "../helpers/nocturne.js";

test("accepts a valid commitment spend and withdrawal", () => {
  const out = runScenario([
    { op: "open_window", window: 2, anchor: "settlement-2" },
    {
      op: "deposit",
      note: "alice_note",
      owner: "alice",
      account: "alice-main",
      asset: "DTL",
      amount: 250,
      position: "alice-book-1",
      window: 1,
      blinding: "alice-blind-1",
    },
    { op: "spend", note: "alice_note", recipient: "alice-main", memo: "invoice settlement" },
    {
      op: "withdraw",
      account: "alice-main",
      asset: "DTL",
      amount: 250,
      destination: "dtl:alice:vault",
      window: 2,
    },
    { op: "reconcile" },
  ]);

  assert.match(out.notes.alice_note.commitment, /^cm_/);
  assert.equal(out.receipts.length, 3);
  assert.equal(out.reconciliations[0].balanced, true);
  assert.equal(out.snapshot.nullifiers, 1);
});
