import test from "node:test";
import assert from "node:assert/strict";
import { runScenario, runScenarioFailure } from "../helpers/nocturne.js";

const policy = { yield_curve: { window_rate_bps: 1_000 } };

test("a funded two-window valuation settles against the recorded reserve", () => {
  const out = runScenario(
    [
      { op: "open_window", window: 3, anchor: "close-3" },
      {
        op: "deposit",
        note: "origin",
        owner: "dana",
        account: "dana-main",
        asset: "DTL",
        amount: 1_000,
        position: "dana-p1",
        window: 1,
        blinding: "dana-0",
      },
      { op: "fund_yield", operator: "operator.local", asset: "DTL", amount: 210, window: 3 },
      { op: "roll", source: "origin", note: "valued", window: 3, blinding: "dana-3" },
      { op: "spend", note: "valued", recipient: "dana-main" },
      {
        op: "withdraw",
        account: "dana-main",
        asset: "DTL",
        amount: 1_210,
        destination: "dtl:dana",
        window: 3,
      },
      { op: "reconcile" },
    ],
    policy,
  );
  assert.equal(out.notes.valued.amount, 1_210);
  assert.equal(out.reconciliations[0].balanced, true);
  assert.equal(out.reconciliations[0].assets.find((asset) => asset.asset === "DTL").reserves, 0);
  assert.equal(out.receipts.filter((receipt) => receipt.type === "treasury_funding").length, 1);
});

test("rotation retires the previous commitment", () => {
  const err = runScenarioFailure([
    { op: "open_window", window: 2, anchor: "next" },
    {
      op: "deposit", note: "origin", owner: "dana", account: "dana-main", asset: "DTL",
      amount: 100, position: "dana-p1", window: 1, blinding: "dana-0",
    },
    { op: "roll", source: "origin", note: "successor", window: 2, blinding: "dana-1" },
    { op: "spend", note: "origin", recipient: "dana-main" },
  ]);
  assert.match(err, /commitment already consumed/);
});

test("a position identifier cannot be reopened by a second deposit", () => {
  const err = runScenarioFailure([
    { op: "deposit", note: "first", owner: "dana", account: "dana-main", asset: "DTL", amount: 100, position: "dana-p1", window: 1, blinding: "dana-0" },
    { op: "deposit", note: "second", owner: "dana", account: "dana-main", asset: "DTL", amount: 100, position: "dana-p1", window: 1, blinding: "dana-1" },
  ]);
  assert.match(err, /position already opened/);
});

test("treasury funding requires the configured operator", () => {
  const err = runScenarioFailure([
    { op: "fund_yield", operator: "unregistered", asset: "DTL", amount: 100, window: 1 },
  ]);
  assert.match(err, /unauthorized treasury operator/);
});

test("a rotation must advance within the permitted window gap", () => {
  const err = runScenarioFailure([
    { op: "open_window", window: 10, anchor: "later" },
    { op: "deposit", note: "origin", owner: "dana", account: "dana-main", asset: "DTL", amount: 100, position: "dana-p1", window: 1, blinding: "dana-0" },
    { op: "roll", source: "origin", note: "later", window: 10, blinding: "dana-10" },
  ]);
  assert.match(err, /invalid window transition/);
});
