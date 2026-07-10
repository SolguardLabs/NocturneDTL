import test from "node:test";
import assert from "node:assert/strict";
import { runScenario, runScenarioFailure } from "../helpers/nocturne.js";

test("withdrawal requires credited private balance", () => {
  const err = runScenarioFailure([
    {
      op: "deposit",
      note: "n0",
      owner: "carol",
      account: "carol-main",
      asset: "DTL",
      amount: 75,
      position: "carol-p1",
      window: 1,
      blinding: "carol-b0",
    },
    {
      op: "withdraw",
      account: "carol-main",
      asset: "DTL",
      amount: 75,
      destination: "dtl:carol",
      window: 1,
    },
  ]);

  assert.match(err, /balance too low/);
});

test("partial withdrawal leaves remaining balance in the account", () => {
  const out = runScenario([
    {
      op: "deposit",
      note: "n0",
      owner: "carol",
      account: "carol-main",
      asset: "DTL",
      amount: 90,
      position: "carol-p2",
      window: 1,
      blinding: "carol-b1",
    },
    { op: "spend", note: "n0", recipient: "carol-main", memo: "private credit" },
    {
      op: "withdraw",
      account: "carol-main",
      asset: "DTL",
      amount: 40,
      destination: "dtl:carol",
      window: 1,
    },
    { op: "reconcile" },
  ]);

  assert.equal(out.reconciliations[0].assets[0].account_balances, 50);
  assert.equal(out.reconciliations[0].assets[0].reserves, 50);
});
