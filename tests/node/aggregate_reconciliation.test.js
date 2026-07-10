import test from "node:test";
import assert from "node:assert/strict";
import { runScenario } from "../helpers/nocturne.js";

test("reconciles aggregate vault state across multiple accounts", () => {
  const out = runScenario([
    { op: "open_window", window: 2, anchor: "batch-2" },
    {
      op: "deposit",
      note: "eve_0",
      owner: "eve",
      account: "eve-main",
      asset: "DTL",
      amount: 120,
      position: "eve-p1",
      window: 1,
      blinding: "eve-b0",
    },
    {
      op: "deposit",
      note: "frank_0",
      owner: "frank",
      account: "frank-main",
      asset: "DTL",
      amount: 230,
      position: "frank-p1",
      window: 1,
      blinding: "frank-b0",
    },
    { op: "spend", note: "eve_0", recipient: "eve-main", memo: "net settlement" },
    { op: "spend", note: "frank_0", recipient: "frank-main", memo: "net settlement" },
    {
      op: "withdraw",
      account: "eve-main",
      asset: "DTL",
      amount: 70,
      destination: "dtl:eve",
      window: 2,
    },
    {
      op: "withdraw",
      account: "frank-main",
      asset: "DTL",
      amount: 130,
      destination: "dtl:frank",
      window: 2,
    },
    { op: "reconcile" },
  ]);

  const asset = out.reconciliations[0].assets.find((item) => item.asset === "DTL");
  assert.equal(asset.deposits, 350);
  assert.equal(asset.withdrawals, 200);
  assert.equal(asset.reserves, 150);
  assert.equal(asset.account_balances, 150);
  assert.equal(out.reconciliations[0].balanced, true);
});
