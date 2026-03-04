//! Job scheduler — matches inference requests to provider nodes.
//!
//! Inference requests specify a model ID, max price, and deadline.
//! The scheduler maintains a pool of available providers and assigns
//! jobs based on: (1) model availability, (2) stake weight, (3) price,
//! (4) reputation score. Supports cancellation and timeout eviction.

use crate::types::{Address, Epoch, Hash, ModelId, StakeAmount};
use std::collections::{BTreeMap, HashMap, HashSet};

/// Unique job identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JobId(pub u64);

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "job-{}", self.0)
    }
}

/// An inference job request.
#[derive(Debug, Clone)]
pub struct JobRequest {
    pub id: JobId,
    pub requester: Address,
    pub model_id: ModelId,
    /// Maximum price (in smallest token unit) the requester will pay.
    pub max_price: u128,
    /// Input hash — identifies the specific inference input.
    pub input_hash: Hash,
    /// Deadline epoch — job expires if unassigned by this epoch.
    pub deadline: Epoch,
    /// Epoch when the job was submitted.
    pub submitted_at: Epoch,
}

/// Job lifecycle status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    /// Waiting in queue for assignment.
    Pending,
    /// Assigned to a provider, awaiting result.
    Assigned { provider: Address, assigned_at: Epoch },
    /// Provider delivered a result commit.
    Completed { provider: Address, commit_hash: Hash },
    /// Expired past deadline without assignment.
    Expired,
    /// Cancelled by requester before assignment.
    Cancelled,
}

/// A registered inference provider.
#[derive(Debug, Clone)]
pub struct Provider {
    pub address: Address,
    /// Models this provider can serve.
    pub models: HashSet<ModelId>,
    /// Price per inference (provider's ask).
    pub price: u128,
    /// Current stake — used for weighted selection.
    pub stake: StakeAmount,
    /// Reputation score 0–1000 (starts at 500).
    pub reputation: u32,
    /// How many concurrent jobs this provider accepts.
    pub capacity: u32,
    /// Currently assigned job count.
    pub active_jobs: u32,
}

impl Provider {
    pub fn available(&self) -> bool {
        self.active_jobs < self.capacity
    }

    /// Composite score for ranking: stake_weight * reputation.
    pub fn score(&self) -> u128 {
        (self.stake / 1_000_000).saturating_mul(self.reputation as u128)
    }
}

/// The job scheduler.
pub struct Scheduler {
    /// All registered providers.
    providers: HashMap<Address, Provider>,
    /// All jobs by ID.
    jobs: HashMap<JobId, (JobRequest, JobStatus)>,
    /// Pending job queue ordered by deadline (earliest first).
    pending_queue: BTreeMap<(Epoch, u64), JobId>,
    /// Next job ID counter.
    next_job_id: u64,
    /// Current epoch.
    current_epoch: Epoch,
    /// Result delivery timeout (epochs after assignment).
    pub result_timeout: Epoch,
}

impl Scheduler {
    pub fn new(result_timeout: Epoch) -> Self {
        Self {
            providers: HashMap::new(),
            jobs: HashMap::new(),
            pending_queue: BTreeMap::new(),
            next_job_id: 1,
            current_epoch: 0,
            result_timeout,
        }
    }

    /// Register or update a provider.
    pub fn register_provider(&mut self, provider: Provider) {
        self.providers.insert(provider.address, provider);
    }

    /// Remove a provider. Fails if they have active jobs.
    pub fn deregister_provider(&mut self, addr: &Address) -> Result<(), &'static str> {
        if let Some(p) = self.providers.get(addr) {
            if p.active_jobs > 0 {
                return Err("provider has active jobs");
            }
            self.providers.remove(addr);
            Ok(())
        } else {
            Err("provider not found")
        }
    }

    /// Submit a new inference job. Returns the assigned JobId.
    pub fn submit_job(
        &mut self,
        requester: Address,
        model_id: ModelId,
        input_hash: Hash,
        max_price: u128,
        deadline: Epoch,
    ) -> Result<JobId, &'static str> {
        if deadline <= self.current_epoch {
            return Err("deadline must be in the future");
        }
        let id = JobId(self.next_job_id);
        self.next_job_id += 1;
        let job = JobRequest {
            id,
            requester,
            model_id,
            max_price,
            input_hash,
            deadline,
            submitted_at: self.current_epoch,
        };
        self.jobs.insert(id, (job, JobStatus::Pending));
        self.pending_queue.insert((deadline, id.0), id);
        Ok(id)
    }

    /// Cancel a pending job. Only the requester can cancel.
    pub fn cancel_job(&mut self, id: JobId, caller: &Address) -> Result<(), &'static str> {
        let (req, status) = self.jobs.get_mut(&id).ok_or("job not found")?;
        if req.requester != *caller {
            return Err("only requester can cancel");
        }
        if *status != JobStatus::Pending {
            return Err("can only cancel pending jobs");
        }
        self.pending_queue.remove(&(req.deadline, id.0));
        *status = JobStatus::Cancelled;
        Ok(())
    }

    /// Try to assign pending jobs to available providers.
    /// Returns list of (JobId, provider Address) assignments made.
    pub fn assign_pending(&mut self) -> Vec<(JobId, Address)> {
        let mut assignments = Vec::new();
        let pending_ids: Vec<JobId> = self.pending_queue.values().copied().collect();

        for job_id in pending_ids {
            let (req, status) = match self.jobs.get(&job_id) {
                Some((r, s)) if *s == JobStatus::Pending => (r.clone(), s.clone()),
                _ => continue,
            };

            // Find best available provider for this model + price
            let best = self
                .providers
                .values()
                .filter(|p| {
                    p.available()
                        && p.models.contains(&req.model_id)
                        && p.price <= req.max_price
                })
                .max_by_key(|p| p.score());

            if let Some(provider) = best {
                let addr = provider.address;
                // Update provider
                self.providers.get_mut(&addr).unwrap().active_jobs += 1;
                // Update job
                let (_, status) = self.jobs.get_mut(&job_id).unwrap();
                *status = JobStatus::Assigned {
                    provider: addr,
                    assigned_at: self.current_epoch,
                };
                self.pending_queue.remove(&(req.deadline, job_id.0));
                assignments.push((job_id, addr));
            }
        }
        assignments
    }

    /// Provider delivers a result for an assigned job.
    pub fn deliver_result(
        &mut self,
        job_id: JobId,
        provider: &Address,
        commit_hash: Hash,
    ) -> Result<(), &'static str> {
        let (_, status) = self.jobs.get_mut(&job_id).ok_or("job not found")?;
        match status {
            JobStatus::Assigned {
                provider: assigned, ..
            } => {
                if assigned != provider {
                    return Err("not assigned provider");
                }
                let addr = *assigned;
                *status = JobStatus::Completed {
                    provider: addr,
                    commit_hash,
                };
                // Free provider capacity
                if let Some(p) = self.providers.get_mut(&addr) {
                    p.active_jobs = p.active_jobs.saturating_sub(1);
                }
                Ok(())
            }
            _ => Err("job not in assigned state"),
        }
    }

    /// Advance epoch. Expires overdue pending jobs, times out stale assignments.
    /// Returns (expired_jobs, timed_out_jobs).
    pub fn tick(&mut self, new_epoch: Epoch) -> (Vec<JobId>, Vec<JobId>) {
        self.current_epoch = new_epoch;
        let mut expired = Vec::new();
        let mut timed_out = Vec::new();

        // Expire pending jobs past deadline
        let overdue: Vec<(Epoch, u64)> = self
            .pending_queue
            .range(..=(new_epoch, u64::MAX))
            .map(|(k, _)| *k)
            .collect();
        for key in overdue {
            if let Some(job_id) = self.pending_queue.remove(&key) {
                if let Some((_, status)) = self.jobs.get_mut(&job_id) {
                    *status = JobStatus::Expired;
                    expired.push(job_id);
                }
            }
        }

        // Timeout assigned jobs past result_timeout
        let stale: Vec<(JobId, Address)> = self
            .jobs
            .iter()
            .filter_map(|(id, (_, status))| match status {
                JobStatus::Assigned {
                    provider,
                    assigned_at,
                } if new_epoch > assigned_at + self.result_timeout => {
                    Some((*id, *provider))
                }
                _ => None,
            })
            .collect();
        for (id, addr) in stale {
            if let Some((_, status)) = self.jobs.get_mut(&id) {
                *status = JobStatus::Expired;
                if let Some(p) = self.providers.get_mut(&addr) {
                    p.active_jobs = p.active_jobs.saturating_sub(1);
                    // Reputation penalty for timeout
                    p.reputation = p.reputation.saturating_sub(50);
                }
                timed_out.push(id);
            }
        }

        (expired, timed_out)
    }

    /// Boost provider reputation on successful completion.
    pub fn reward_provider(&mut self, addr: &Address, bonus: u32) {
        if let Some(p) = self.providers.get_mut(addr) {
            p.reputation = (p.reputation + bonus).min(1000);
        }
    }

    /// Get job status.
    pub fn job_status(&self, id: &JobId) -> Option<&JobStatus> {
        self.jobs.get(id).map(|(_, s)| s)
    }

    /// Get provider info.
    pub fn provider(&self, addr: &Address) -> Option<&Provider> {
        self.providers.get(addr)
    }

    /// Count of pending jobs.
    pub fn pending_count(&self) -> usize {
        self.pending_queue.len()
    }

    /// Count of all jobs.
    pub fn total_jobs(&self) -> usize {
        self.jobs.len()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_submit_and_assign() {
        let mut sched = Scheduler::new(10);
        let m = model(1);
        sched.register_provider(make_provider(1, &[m], 100, 1_000_000_000, 2));

        let jid = sched.submit_job(Address::test(10), m, [0; 32], 200, 50).unwrap();
        assert_eq!(sched.pending_count(), 1);

        let assignments = sched.assign_pending();
        assert_eq!(assignments.len(), 1);
        assert_eq!(assignments[0], (jid, Address::test(1)));
        assert_eq!(sched.pending_count(), 0);

        match sched.job_status(&jid).unwrap() {
            JobStatus::Assigned { provider, .. } => assert_eq!(*provider, Address::test(1)),
            _ => panic!("expected assigned"),
        }
    }

    #[test]
    fn test_price_filter() {
        let mut sched = Scheduler::new(10);
        let m = model(1);
        // Provider asks 300, but requester max is 200 → no match
        sched.register_provider(make_provider(1, &[m], 300, 1_000_000_000, 2));

        sched.submit_job(Address::test(10), m, [0; 32], 200, 50).unwrap();
        let assignments = sched.assign_pending();
        assert!(assignments.is_empty());
        assert_eq!(sched.pending_count(), 1);
    }

    #[test]
    fn test_model_filter() {
        let mut sched = Scheduler::new(10);
        let m1 = model(1);
        let m2 = model(2);
        // Provider only serves model 1
        sched.register_provider(make_provider(1, &[m1], 100, 1_000_000_000, 2));

        // Job requests model 2 → no match
        sched.submit_job(Address::test(10), m2, [0; 32], 200, 50).unwrap();
        let assignments = sched.assign_pending();
        assert!(assignments.is_empty());
    }

    #[test]
    fn test_capacity_exhaustion() {
        let mut sched = Scheduler::new(10);
        let m = model(1);
        sched.register_provider(make_provider(1, &[m], 100, 1_000_000_000, 1)); // capacity=1

        let j1 = sched.submit_job(Address::test(10), m, [0; 32], 200, 50).unwrap();
        let j2 = sched.submit_job(Address::test(11), m, [1; 32], 200, 50).unwrap();

        let a = sched.assign_pending();
        assert_eq!(a.len(), 1); // only 1 slot
        assert_eq!(a[0].0, j1);
        assert_eq!(sched.pending_count(), 1); // j2 still pending
    }

    #[test]
    fn test_best_provider_selection() {
        let mut sched = Scheduler::new(10);
        let m = model(1);
        // Provider A: lower stake
        let mut pa = make_provider(1, &[m], 100, 1_000_000_000, 5);
        pa.reputation = 800;
        sched.register_provider(pa);
        // Provider B: higher stake + reputation → better score
        let mut pb = make_provider(2, &[m], 100, 5_000_000_000, 5);
        pb.reputation = 900;
        sched.register_provider(pb);

        sched.submit_job(Address::test(10), m, [0; 32], 200, 50).unwrap();
        let a = sched.assign_pending();
        assert_eq!(a[0].1, Address::test(2)); // B wins
    }

    #[test]
    fn test_deliver_result() {
        let mut sched = Scheduler::new(10);
        let m = model(1);
        sched.register_provider(make_provider(1, &[m], 100, 1_000_000_000, 2));

        let jid = sched.submit_job(Address::test(10), m, [0; 32], 200, 50).unwrap();
        sched.assign_pending();

        let commit = [42u8; 32];
        sched.deliver_result(jid, &Address::test(1), commit).unwrap();

        match sched.job_status(&jid).unwrap() {
            JobStatus::Completed { commit_hash, .. } => assert_eq!(*commit_hash, commit),
            _ => panic!("expected completed"),
        }
        // Provider capacity freed
        assert_eq!(sched.provider(&Address::test(1)).unwrap().active_jobs, 0);
    }

    #[test]
    fn test_wrong_provider_delivery() {
        let mut sched = Scheduler::new(10);
        let m = model(1);
        sched.register_provider(make_provider(1, &[m], 100, 1_000_000_000, 2));

        let jid = sched.submit_job(Address::test(10), m, [0; 32], 200, 50).unwrap();
        sched.assign_pending();

        let err = sched.deliver_result(jid, &Address::test(99), [0; 32]).unwrap_err();
        assert_eq!(err, "not assigned provider");
    }

    #[test]
    fn test_cancel_job() {
        let mut sched = Scheduler::new(10);
        let m = model(1);
        let requester = Address::test(10);
        let jid = sched.submit_job(requester, m, [0; 32], 200, 50).unwrap();

        sched.cancel_job(jid, &requester).unwrap();
        assert_eq!(*sched.job_status(&jid).unwrap(), JobStatus::Cancelled);
        assert_eq!(sched.pending_count(), 0);
    }

    #[test]
    fn test_cancel_wrong_caller() {
        let mut sched = Scheduler::new(10);
        let m = model(1);
        let jid = sched.submit_job(Address::test(10), m, [0; 32], 200, 50).unwrap();

        let err = sched.cancel_job(jid, &Address::test(99)).unwrap_err();
        assert_eq!(err, "only requester can cancel");
    }

    #[test]
    fn test_expire_overdue_pending() {
        let mut sched = Scheduler::new(10);
        let m = model(1);
        // No providers registered → job stays pending
        let jid = sched.submit_job(Address::test(10), m, [0; 32], 200, 5).unwrap();

        let (expired, _) = sched.tick(6);
        assert_eq!(expired, vec![jid]);
        assert_eq!(*sched.job_status(&jid).unwrap(), JobStatus::Expired);
    }

    #[test]
    fn test_timeout_stale_assignment() {
        let mut sched = Scheduler::new(10); // 10 epoch timeout
        let m = model(1);
        sched.register_provider(make_provider(1, &[m], 100, 1_000_000_000, 2));

        let jid = sched.submit_job(Address::test(10), m, [0; 32], 200, 50).unwrap();
        sched.assign_pending(); // assigned at epoch 0

        let (_, timed_out) = sched.tick(11); // 11 > 0 + 10
        assert_eq!(timed_out, vec![jid]);
        assert_eq!(*sched.job_status(&jid).unwrap(), JobStatus::Expired);
        // Provider gets reputation penalty
        assert_eq!(sched.provider(&Address::test(1)).unwrap().reputation, 450);
    }

    #[test]
    fn test_deregister_provider() {
        let mut sched = Scheduler::new(10);
        let m = model(1);
        sched.register_provider(make_provider(1, &[m], 100, 1_000_000_000, 2));

        sched.deregister_provider(&Address::test(1)).unwrap();
        assert!(sched.provider(&Address::test(1)).is_none());
    }

    #[test]
    fn test_deregister_busy_provider_fails() {
        let mut sched = Scheduler::new(10);
        let m = model(1);
        sched.register_provider(make_provider(1, &[m], 100, 1_000_000_000, 2));
        sched.submit_job(Address::test(10), m, [0; 32], 200, 50).unwrap();
        sched.assign_pending();

        let err = sched.deregister_provider(&Address::test(1)).unwrap_err();
        assert_eq!(err, "provider has active jobs");
    }

    #[test]
    fn test_reputation_reward() {
        let mut sched = Scheduler::new(10);
        let m = model(1);
        sched.register_provider(make_provider(1, &[m], 100, 1_000_000_000, 2));

        sched.reward_provider(&Address::test(1), 200);
        assert_eq!(sched.provider(&Address::test(1)).unwrap().reputation, 700);

        // Capped at 1000
        sched.reward_provider(&Address::test(1), 500);
        assert_eq!(sched.provider(&Address::test(1)).unwrap().reputation, 1000);
    }

    #[test]
    fn test_deadline_past_rejected() {
        let mut sched = Scheduler::new(10);
        sched.tick(100);
        let err = sched.submit_job(Address::test(10), model(1), [0; 32], 200, 50).unwrap_err();
        assert_eq!(err, "deadline must be in the future");
    }

    #[test]
    fn test_multi_job_assignment_ordering() {
        let mut sched = Scheduler::new(10);
        let m = model(1);
        sched.register_provider(make_provider(1, &[m], 100, 1_000_000_000, 10));

        // Submit 3 jobs with different deadlines
        let j1 = sched.submit_job(Address::test(10), m, [1; 32], 200, 30).unwrap();
        let j2 = sched.submit_job(Address::test(11), m, [2; 32], 200, 10).unwrap();
        let j3 = sched.submit_job(Address::test(12), m, [3; 32], 200, 20).unwrap();

        let a = sched.assign_pending();
        assert_eq!(a.len(), 3);
        // Assigned in deadline order (earliest first): j2(10), j3(20), j1(30)
        assert_eq!(a[0].0, j2);
        assert_eq!(a[1].0, j3);
        assert_eq!(a[2].0, j1);
    }
}
