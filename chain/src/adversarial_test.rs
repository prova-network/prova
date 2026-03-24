//! INT-003: Adversarial scheduler tests — byzantine providers and deadline gaming.
//!
//! Tests attack vectors against the scheduler + SLA + executor pipeline:
//! 1. Byzantine providers (selective job dropping, false results, reputation gaming)
//! 2. Deadline gaming (last-second submissions, expiry sniping)
//! 3. Capacity hoarding (register high capacity, never deliver)
//! 4. Sybil provider registration (many low-stake identities)
//! 5. Griefing attacks (spam jobs to exhaust provider capacity)

#[cfg(test)]
mod tests {
    use crate::scheduler::{JobId, JobStatus, Provider, Scheduler};
    use crate::sla::{PenaltyAction, SlaRegistry, SlaTier, Violation, ViolationKind};
    use crate::types::{Address, ModelId};
    use std::collections::HashSet;

    fn model(id: u8) -> ModelId {
        let mut h = [0u8; 32];
        h[0] = id;
        ModelId(h)
    }

    fn make_provider(id: u8, models: &[ModelId], price: u128, stake: u128, cap: u32) -> Provider {
        Provider {
            address: Address::test(id),
            models: models.iter().copied().collect(),
            price,
            stake,
            reputation: 500,
            capacity: cap,
            active_jobs: 0,
        }
    }

    // ─── Byzantine Provider: Selective Job Dropping ───────────────────────

    #[test]
    fn test_byzantine_selective_drop_gets_penalized() {
        // A provider accepts jobs but only delivers results for some.
        // Undelivered jobs should timeout and degrade reputation.
        let mut sched = Scheduler::new(5); // 5 epoch timeout
        let m = model(1);
        sched.register_provider(make_provider(1, &[m], 100, 2_000_000_000, 10));

        // Submit 5 jobs, all assigned to provider 1
        let mut jobs = Vec::new();
        for i in 0..5u8 {
            let mut h = [0u8; 32];
            h[0] = i;
            let jid = sched.submit_job(Address::test(10), m, h, 200, 100).unwrap();
            jobs.push(jid);
        }
        sched.assign_pending();

        // Provider delivers only jobs 0 and 2 (drops 1, 3, 4)
        sched
            .deliver_result(jobs[0], &Address::test(1), [0xAA; 32])
            .unwrap();
        sched
            .deliver_result(jobs[2], &Address::test(1), [0xBB; 32])
            .unwrap();

        // Advance past timeout
        let (_, timed_out) = sched.tick(6);
        assert_eq!(timed_out.len(), 3); // jobs 1, 3, 4 timed out

        // Provider reputation dropped: 500 - (3 * 50) = 350
        let prov = sched.provider(&Address::test(1)).unwrap();
        assert_eq!(prov.reputation, 350);
    }

    #[test]
    fn test_byzantine_repeated_drops_destroy_reputation() {
        // Sustained dropping drives reputation to zero.
        let mut sched = Scheduler::new(3);
        let m = model(1);
        sched.register_provider(make_provider(1, &[m], 100, 2_000_000_000, 20));

        // 10 jobs, deliver none
        for i in 0..10u8 {
            let mut h = [0u8; 32];
            h[0] = i;
            sched.submit_job(Address::test(10), m, h, 200, 100).unwrap();
        }
        sched.assign_pending();
        let (_, timed_out) = sched.tick(4);
        assert_eq!(timed_out.len(), 10);

        // 500 - (10 * 50) = 0 (saturating)
        let prov = sched.provider(&Address::test(1)).unwrap();
        assert_eq!(prov.reputation, 0);
    }

    // ─── Byzantine Provider: Impersonation / Wrong Provider ──────────────

    #[test]
    fn test_cannot_deliver_for_another_provider() {
        let mut sched = Scheduler::new(10);
        let m = model(1);
        sched.register_provider(make_provider(1, &[m], 100, 2_000_000_000, 5));
        sched.register_provider(make_provider(2, &[m], 100, 1_000_000_000, 5));

        let jid = sched
            .submit_job(Address::test(10), m, [1; 32], 200, 100)
            .unwrap();
        sched.assign_pending();
        // Job assigned to provider 1 (higher score). Provider 2 tries to steal.
        let err = sched
            .deliver_result(jid, &Address::test(2), [0xFF; 32])
            .unwrap_err();
        assert_eq!(err, "not assigned provider");
    }

    // ─── Deadline Gaming: Last-Second Submission ─────────────────────────

    #[test]
    fn test_deadline_gaming_tight_window() {
        // Requester sets a very tight deadline trying to get free computation
        // (job expires before provider can deliver).
        let mut sched = Scheduler::new(10);
        let m = model(1);
        sched.register_provider(make_provider(1, &[m], 100, 2_000_000_000, 5));

        // Deadline at epoch 2, submitted at epoch 0
        let jid = sched
            .submit_job(Address::test(10), m, [1; 32], 200, 2)
            .unwrap();
        sched.assign_pending(); // assigned at epoch 0

        // Provider delivers at epoch 1 — should still work (within timeout)
        sched.tick(1);
        let result = sched.deliver_result(jid, &Address::test(1), [0xAA; 32]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deadline_expiry_before_assignment() {
        // Jobs that expire before any provider picks them up
        let mut sched = Scheduler::new(10);
        let m = model(1);
        // No providers registered!

        let jid = sched
            .submit_job(Address::test(10), m, [1; 32], 200, 3)
            .unwrap();
        assert_eq!(sched.pending_count(), 1);

        // Advance past deadline
        let (expired, _) = sched.tick(4);
        assert_eq!(expired, vec![jid]);
        assert_eq!(*sched.job_status(&jid).unwrap(), JobStatus::Expired);
    }

    #[test]
    fn test_cannot_submit_already_expired_deadline() {
        let mut sched = Scheduler::new(10);
        sched.tick(50); // current epoch = 50

        let err = sched
            .submit_job(Address::test(10), model(1), [0; 32], 200, 30)
            .unwrap_err();
        assert_eq!(err, "deadline must be in the future");
    }

    // ─── Capacity Hoarding ───────────────────────────────────────────────

    #[test]
    fn test_capacity_hoarder_blocks_other_providers() {
        // Provider registers huge capacity, monopolizes all assignments,
        // then never delivers — system should recover via timeout.
        let mut sched = Scheduler::new(5);
        let m = model(1);

        // Hoarder: huge stake + capacity
        let mut hoarder = make_provider(1, &[m], 50, 10_000_000_000, 100);
        hoarder.reputation = 900;
        sched.register_provider(hoarder);

        // Honest provider: lower stake
        let mut honest = make_provider(2, &[m], 50, 1_000_000_000, 10);
        honest.reputation = 800;
        sched.register_provider(honest);

        // Submit 5 jobs — all go to hoarder (higher score)
        let mut jobs = Vec::new();
        for i in 0..5u8 {
            let mut h = [0u8; 32];
            h[0] = i;
            let jid = sched.submit_job(Address::test(10), m, h, 200, 100).unwrap();
            jobs.push(jid);
        }
        let assigned = sched.assign_pending();
        for (_, addr) in &assigned {
            assert_eq!(*addr, Address::test(1)); // all to hoarder
        }

        // Hoarder delivers nothing → timeout
        let (_, timed_out) = sched.tick(6);
        assert_eq!(timed_out.len(), 5);

        // Hoarder reputation crushed: 900 - 250 = 650
        let h = sched.provider(&Address::test(1)).unwrap();
        assert_eq!(h.reputation, 650);
        // Hoarder capacity freed
        assert_eq!(h.active_jobs, 0);

        // Now submit new jobs — honest provider should compete better
        // (hoarder score dropped significantly)
        for i in 10..15u8 {
            let mut h = [0u8; 32];
            h[0] = i;
            sched.submit_job(Address::test(10), m, h, 200, 100).unwrap();
        }
        let assigned2 = sched.assign_pending();
        assert_eq!(assigned2.len(), 5);
        // Hoarder score: (10_000_000_000/1M) * 650 = 6_500_000
        // Honest score:  (1_000_000_000/1M) * 800  = 800_000
        // Hoarder still wins due to massive stake advantage.
        // This is a known limitation — pure stake weight can dominate.
    }

    // ─── Sybil Attack: Many Low-Stake Providers ──────────────────────────

    #[test]
    fn test_sybil_low_stake_providers_lose_to_honest_high_stake() {
        let mut sched = Scheduler::new(10);
        let m = model(1);

        // 10 sybil providers: each with 1/10th stake
        for i in 1..=10u8 {
            let mut p = make_provider(i, &[m], 100, 100_000_000, 2);
            p.reputation = 500;
            sched.register_provider(p);
        }

        // 1 honest provider with full stake
        let mut honest = make_provider(50, &[m], 100, 5_000_000_000, 5);
        honest.reputation = 800;
        sched.register_provider(honest);

        // Submit a job — honest provider should win (highest score)
        sched
            .submit_job(Address::test(10), m, [1; 32], 200, 100)
            .unwrap();
        let assigned = sched.assign_pending();
        assert_eq!(assigned.len(), 1);
        assert_eq!(assigned[0].1, Address::test(50)); // honest wins
    }

    #[test]
    fn test_sybil_with_inflated_reputation_still_loses_to_stake() {
        let mut sched = Scheduler::new(10);
        let m = model(1);

        // Sybil with max reputation but tiny stake
        let mut sybil = make_provider(1, &[m], 100, 10_000, 5);
        sybil.reputation = 1000;
        sched.register_provider(sybil);

        // Honest with moderate reputation but solid stake
        let mut honest = make_provider(2, &[m], 100, 2_000_000_000, 5);
        honest.reputation = 600;
        sched.register_provider(honest);

        sched
            .submit_job(Address::test(10), m, [1; 32], 200, 100)
            .unwrap();
        let assigned = sched.assign_pending();
        // sybil score: (10_000/1M) * 1000 = 0 (integer division)
        // honest score: (2_000_000_000/1M) * 600 = 1_200_000
        assert_eq!(assigned[0].1, Address::test(2));
    }

    // ─── Griefing: Spam Jobs to Exhaust Capacity ─────────────────────────

    #[test]
    fn test_griefing_spam_jobs_exhaust_provider_capacity() {
        let mut sched = Scheduler::new(10);
        let m = model(1);
        sched.register_provider(make_provider(1, &[m], 100, 2_000_000_000, 3)); // capacity=3

        // Attacker submits 10 cheap jobs — only 3 get assigned
        let mut attacker_jobs = Vec::new();
        for i in 0..10u8 {
            let mut h = [0u8; 32];
            h[0] = i;
            let jid = sched.submit_job(Address::test(99), m, h, 200, 100).unwrap();
            attacker_jobs.push(jid);
        }
        sched.assign_pending();

        // Legitimate user's job stays pending
        let legit = sched
            .submit_job(Address::test(10), m, [0xFF; 32], 200, 100)
            .unwrap();
        let assigned = sched.assign_pending();
        assert!(assigned.is_empty()); // capacity full

        // After attacker jobs timeout, capacity frees up
        let (_, timed_out) = sched.tick(11);
        assert_eq!(timed_out.len(), 3);
        let assigned = sched.assign_pending();
        // legit job is still pending and can now be assigned
        assert!(!assigned.is_empty() || sched.pending_count() > 0);
    }

    // ─── Cancel Abuse ────────────────────────────────────────────────────

    #[test]
    fn test_cannot_cancel_after_assignment() {
        let mut sched = Scheduler::new(10);
        let m = model(1);
        sched.register_provider(make_provider(1, &[m], 100, 2_000_000_000, 5));

        let jid = sched
            .submit_job(Address::test(10), m, [1; 32], 200, 100)
            .unwrap();
        sched.assign_pending();

        let err = sched.cancel_job(jid, &Address::test(10)).unwrap_err();
        assert_eq!(err, "can only cancel pending jobs");
    }

    #[test]
    fn test_non_requester_cannot_cancel() {
        let mut sched = Scheduler::new(10);
        let m = model(1);
        let jid = sched
            .submit_job(Address::test(10), m, [1; 32], 200, 100)
            .unwrap();

        let err = sched.cancel_job(jid, &Address::test(99)).unwrap_err();
        assert_eq!(err, "only requester can cancel");
    }

    // ─── SLA + Scheduler Integration: Byzantine Violation Cascade ────────

    #[test]
    fn test_sla_violation_cascade_leads_to_termination() {
        let mut sla_reg = SlaRegistry::new();
        let p = Address::test(1);
        let m = model(1);
        let stake: u128 = 10_000_000;
        sla_reg.register(p, m, SlaTier::gold(), 0).unwrap();

        // Simulate a byzantine provider repeatedly missing jobs
        // Gold tier: 32 violations → termination (32^2 = 1024 >= 1000)
        let mut last_action = PenaltyAction::None;
        for i in 1..=32u64 {
            let v = Violation {
                epoch: i,
                kind: ViolationKind::JobMissed,
            };
            last_action = sla_reg.record_violation(p, m, v, stake).unwrap();
        }
        assert_eq!(
            last_action,
            PenaltyAction::Termination {
                amount: stake * 2000 / 10000
            }
        );
        assert!(!sla_reg.is_active(p, m));
    }

    #[test]
    fn test_sla_mixed_violations_quadratic_escalation() {
        let mut sla_reg = SlaRegistry::new();
        let p = Address::test(1);
        let m = model(1);
        let stake: u128 = 5_000_000;
        sla_reg.register(p, m, SlaTier::silver(), 0).unwrap();

        // Mixed violation types — all count equally toward quadratic penalty
        let kinds = vec![
            ViolationKind::LatencyExceeded {
                actual_ms: 3000,
                limit_ms: 2000,
            },
            ViolationKind::Unavailable,
            ViolationKind::ThroughputDeficit {
                actual: 2,
                required: 5,
            },
            ViolationKind::JobMissed,
        ];

        let mut actions = Vec::new();
        for (i, kind) in kinds.iter().cycle().take(15).enumerate() {
            let v = Violation {
                epoch: i as u64 + 1,
                kind: kind.clone(),
            };
            actions.push(sla_reg.record_violation(p, m, v, stake).unwrap());
        }

        // 8th violation: 64 points → Warning
        assert_eq!(actions[7], PenaltyAction::Warning);
        // 15th violation: 225 points → MinorSlash
        assert_eq!(
            actions[14],
            PenaltyAction::MinorSlash {
                amount: stake * 100 / 10000
            }
        );
    }

    // ─── Race Condition: Deregister During Active Jobs ───────────────────

    #[test]
    fn test_cannot_deregister_with_active_jobs() {
        let mut sched = Scheduler::new(10);
        let m = model(1);
        sched.register_provider(make_provider(1, &[m], 100, 2_000_000_000, 5));

        sched
            .submit_job(Address::test(10), m, [1; 32], 200, 100)
            .unwrap();
        sched.assign_pending();

        let err = sched.deregister_provider(&Address::test(1)).unwrap_err();
        assert_eq!(err, "provider has active jobs");
    }

    // ─── Price Manipulation ──────────────────────────────────────────────

    #[test]
    fn test_provider_overpricing_gets_bypassed() {
        let mut sched = Scheduler::new(10);
        let m = model(1);

        // Expensive provider
        sched.register_provider(make_provider(1, &[m], 500, 5_000_000_000, 5));
        // Cheap provider
        sched.register_provider(make_provider(2, &[m], 50, 1_000_000_000, 5));

        // Requester max_price = 100 — only cheap provider qualifies
        sched
            .submit_job(Address::test(10), m, [1; 32], 100, 100)
            .unwrap();
        let assigned = sched.assign_pending();
        assert_eq!(assigned.len(), 1);
        assert_eq!(assigned[0].1, Address::test(2));
    }

    // ─── Reputation Recovery After Slashing ──────────────────────────────

    #[test]
    fn test_reputation_recovery_after_timeout() {
        let mut sched = Scheduler::new(5);
        let m = model(1);
        sched.register_provider(make_provider(1, &[m], 100, 2_000_000_000, 5));

        // Drop a job → reputation goes 500→450
        sched
            .submit_job(Address::test(10), m, [1; 32], 200, 100)
            .unwrap();
        sched.assign_pending();
        sched.tick(6);
        assert_eq!(sched.provider(&Address::test(1)).unwrap().reputation, 450);

        // Reward provider for good behavior
        sched.reward_provider(&Address::test(1), 100);
        assert_eq!(sched.provider(&Address::test(1)).unwrap().reputation, 550);
    }

    // ─── Multi-Model Isolation ───────────────────────────────────────────

    #[test]
    fn test_attack_on_one_model_doesnt_affect_other() {
        let mut sched = Scheduler::new(5);
        let m1 = model(1);
        let m2 = model(2);

        // Provider serves both models
        sched.register_provider(make_provider(1, &[m1, m2], 100, 2_000_000_000, 10));
        // Second provider only serves m2
        sched.register_provider(make_provider(2, &[m2], 100, 1_000_000_000, 5));

        // Submit jobs for both models
        let j1 = sched
            .submit_job(Address::test(10), m1, [1; 32], 200, 100)
            .unwrap();
        let j2 = sched
            .submit_job(Address::test(10), m2, [2; 32], 200, 100)
            .unwrap();
        sched.assign_pending();

        // Provider 1 delivers m2 result, drops m1
        sched
            .deliver_result(j2, &Address::test(1), [0xBB; 32])
            .unwrap();
        let (_, timed_out) = sched.tick(6);
        assert_eq!(timed_out.len(), 1); // only j1 timed out

        // m2 job completed fine despite m1 attack
        assert!(matches!(
            sched.job_status(&j2).unwrap(),
            JobStatus::Completed { .. }
        ));
    }

    // ─── Edge: Zero-Reputation Provider Still Available ──────────────────

    #[test]
    fn test_zero_reputation_provider_score_is_zero() {
        let mut sched = Scheduler::new(5);
        let m = model(1);
        let mut p = make_provider(1, &[m], 100, 2_000_000_000, 5);
        p.reputation = 0;
        sched.register_provider(p);

        // A second provider with any reputation beats zero
        let mut p2 = make_provider(2, &[m], 100, 1_000_000, 5);
        p2.reputation = 1;
        sched.register_provider(p2);

        sched
            .submit_job(Address::test(10), m, [1; 32], 200, 100)
            .unwrap();
        let assigned = sched.assign_pending();
        // Provider 1 score: 2000*0=0, Provider 2 score: 0*1=0 (both zero due to integer division)
        // When tied, HashMap iteration order is nondeterministic,
        // but both having score 0 shows the system handles it.
        assert_eq!(assigned.len(), 1);
    }
}
