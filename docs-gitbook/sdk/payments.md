---
description: Top up, escrow, withdraw.
---

# Payments

Prova bills in **USDC on Base**. The `prova.payments` service handles the on-chain payment rail.

## Balance

```ts
const balance = await prova.payments.balance()
// → { available: bigint, escrowed: bigint, slashed: bigint }
```

## Top up

```ts
await prova.payments.deposit({ amount: 100_000000n })   // 100 USDC
```

Approves USDC and transfers it to the escrow contract. Subsequent uploads draw from the escrow.

## Withdraw

```ts
await prova.payments.withdraw({ amount: 50_000000n })
```

Pulls available (non-escrowed) USDC back to your wallet.

## Per-deal payment

When you call `prova.storage.upload(...)`, the SDK automatically reserves the deal's full cost from your escrow. If your escrow is too low, the upload throws a typed `InsufficientFundsError` that includes how much you need to top up.

```ts
try {
  await prova.storage.upload(bytes, { termDays: 365 })
} catch (e) {
  if (e instanceof InsufficientFundsError) {
    await prova.payments.deposit({ amount: e.shortfall })
    // retry
  }
}
```

## Streaming payment release

A deal's escrow is released to the prover **per successful proof**. If a proof is missed, that block's payment stays in escrow. If the deal is cancelled or the prover is slashed, the unreleased escrow is refunded to you.

You can read the release schedule directly:

```ts
const released = await prova.payments.dealRelease(dealId)
// → { paidOut: bigint, remaining: bigint, lastReleasedAt: number }
```

## Gas

The SDK estimates and includes gas headroom on every transaction. Override with:

```ts
await prova.payments.deposit({ amount: 100_000000n, gasOverride: { gasLimit: 200_000n } })
```

## See also

* [`StorageMarketplace.sol`](https://github.com/prova-network/prova/blob/main/contracts/src/StorageMarketplace.sol) — the on-chain escrow logic
