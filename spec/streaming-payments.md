# SPEC-006: Streaming Payments Specification

**Status:** Draft  
**Author:** Capri  
**Created:** 2026-03-04

## 1. Overview

Prova uses streaming payment channels for both inference and storage services. Payers lock funds in a channel; providers earn per-inference or per-epoch. Settlement happens on-chain with a dispute window.

This design is inspired by Filecoin Pay but simplified for Prova's dual compute+storage model.

## 2. Channel Lifecycle

```
OPEN → ACTIVE → SETTLING → CLOSED
                    ↑
                  (dispute)
```

### 2.1 Open
Payer locks funds with parameters:
- `provider`: recipient address
- `locked`: total amount locked
- `rate`: payment per inference (or per epoch for storage)

### 2.2 Active
During active state:
- Inferences trigger `pay_inference()` calls
- Each payment deducts `rate` from locked balance
- 0.5% network fee is extracted per payment
- Either party can top up the channel

### 2.3 Settling
Either party can initiate close:
- 480-epoch settlement window opens (~4 hours)
- No new payments during settlement
- Counterparty can dispute during window

### 2.4 Closed
After settlement window:
- Provider receives accumulated payments
- Payer receives refund of remaining balance
- Channel archived on-chain

## 3. Fee Structure

| Fee | Rate | Destination |
|-----|------|-------------|
| Network fee | 0.5% per payment | Protocol treasury |
| Gas | Variable | Block producers |

### 3.1 Network Fee Collection
```
gross_payment = channel.rate
network_fee = gross_payment × 0.005
provider_receives = gross_payment - network_fee
```

Network fees accumulate in the protocol treasury and can be:
- Burned (deflationary)
- Distributed to stakers (dividend)
- Governed by token holders

## 4. Payment Types

### 4.1 Per-Inference
For compute services. Each successful inference triggers one payment.
```
rate = price_per_inference (varies by model complexity)
```

### 4.2 Per-Epoch (Storage)
For PDP storage services. Payments stream continuously.
```
rate = price_per_GB_per_epoch
payment_per_epoch = rate × stored_GB
```

### 4.3 Hybrid
Channels can support both: storage streaming + inference bursts.

## 5. Dispute Resolution

If a provider claims payment for an inference that's disputed via QBP:
1. Payment is held in escrow during dispute
2. If provider wins: payment released to provider
3. If provider loses: payment refunded to payer + slash from stake

## 6. Multi-Channel Batching

For high-throughput providers, multiple small payments can be batched:
```
batch_settle(channel_id, count=100, epoch)
// Settles 100 accumulated inference payments in one tx
```

This reduces gas costs for frequent small payments.

## 7. Security Properties

- **Griefing resistance:** Minimum channel lock prevents dust attacks
- **Atomicity:** Payment and inference commit are linked — no payment without commit
- **Timeout protection:** Stale channels can be force-closed after inactivity
- **Front-running resistance:** Payments are deterministic (rate × count)

## 8. Parameters

| Parameter | Value | Governance |
|-----------|-------|------------|
| Network fee | 0.5% (50 bps) | Yes |
| Settlement window | 480 epochs (~4h) | Yes |
| Minimum lock | 10,000 tokens | Yes |
| Max channels per address | 100 | Yes |
| Inactivity timeout | 40,320 epochs (~14 days) | Yes |

## 9. References

- [Filecoin Pay specification](https://github.com/FilOzone/filecoin-pay)
- [SPEC-001: QBP Protocol](./qbp-protocol.md)
- [SPEC-004: PDP Integration](./pdp-integration.md)
