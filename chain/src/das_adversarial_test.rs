//! INT-004: DAS adversarial tests — withholding attacks, partial responses,
//! selective availability, and byzantine provider behaviors.

#[cfg(test)]
mod tests {
    use crate::das::*;
    use crate::types::*;
    use sha2::{Digest, Sha256};

    fn make_chunks(n: usize, size: usize) -> Vec<Vec<u8>> {
        (0..n).map(|i| vec![(i & 0xff) as u8; size]).collect()
    }

    fn test_randomness(seed: u8) -> Hash {
        let mut h = Sha256::new();
        h.update([seed]);
        h.finalize().into()
    }

    // ─── Withholding attacks ─────────────────────────────────────────────

    #[test]
    fn test_total_withholding_penalizes_after_deadline() {
        // Provider commits data then never responds to any challenge
        let mut engine = DasEngine::new();
        let original = make_chunks(8, 64);
        let (blob_id, root, all_chunks, _) = prepare_blob(&original);

        engine.submit_commitment(blob_id, Address::test(1), root, all_chunks.len()).unwrap();
        let randomness = test_randomness(0);
        let challenge = engine.generate_challenge(blob_id, &randomness).unwrap();

        // Provider withholds — advance past deadline
        engine.set_epoch(challenge.deadline + 1);
        engine.process_expired();

        assert_eq!(engine.get_commitment(&blob_id).unwrap().status, DasStatus::Failed);
        assert_eq!(engine.get_penalty(&Address::test(1)), DAS_PENALTY);
    }

    #[test]
    fn test_selective_withholding_partial_rounds() {
        // Provider responds to round 0 but withholds on round 1
        let mut engine = DasEngine::new();
        let original = make_chunks(8, 64);
        let (blob_id, root, all_chunks, layers) = prepare_blob(&original);

        engine.submit_commitment(blob_id, Address::test(1), root, all_chunks.len()).unwrap();

        // Round 0: respond honestly
        let r0 = test_randomness(0);
        let ch0 = engine.generate_challenge(blob_id, &r0).unwrap();
        let proofs0 = build_chunk_proofs(&ch0.indices, &all_chunks, &layers);
        engine.respond_to_challenge(blob_id, ch0.round, &proofs0).unwrap();
        assert_eq!(engine.get_commitment(&blob_id).unwrap().rounds_completed, 1);

        // Round 1: withhold
        let r1 = test_randomness(1);
        let ch1 = engine.generate_challenge(blob_id, &r1).unwrap();
        engine.set_epoch(ch1.deadline + 1);
        engine.process_expired();

        assert_eq!(engine.get_commitment(&blob_id).unwrap().status, DasStatus::Failed);
        assert_eq!(engine.get_penalty(&Address::test(1)), DAS_PENALTY);
    }

    // ─── Invalid proof attacks ───────────────────────────────────────────

    #[test]
    fn test_corrupted_chunk_data_rejected() {
        // Provider returns chunk with flipped bits
        let mut engine = DasEngine::new();
        let original = make_chunks(8, 64);
        let (blob_id, root, all_chunks, layers) = prepare_blob(&original);

        engine.submit_commitment(blob_id, Address::test(1), root, all_chunks.len()).unwrap();
        let randomness = test_randomness(0);
        let challenge = engine.generate_challenge(blob_id, &randomness).unwrap();

        let mut proofs = build_chunk_proofs(&challenge.indices, &all_chunks, &layers);
        // Corrupt every chunk — adversary flips all bits
        for p in proofs.iter_mut() {
            p.data = p.data.iter().map(|b| !b).collect();
        }
        let err = engine.respond_to_challenge(blob_id, challenge.round, &proofs).unwrap_err();
        assert_eq!(err, "invalid merkle proof");
    }

    #[test]
    fn test_swapped_chunk_indices_rejected() {
        // Provider returns valid chunks but swaps their indices
        let mut engine = DasEngine::new();
        let original = make_chunks(8, 64);
        let (blob_id, root, all_chunks, layers) = prepare_blob(&original);

        engine.submit_commitment(blob_id, Address::test(1), root, all_chunks.len()).unwrap();
        let randomness = test_randomness(0);
        let challenge = engine.generate_challenge(blob_id, &randomness).unwrap();

        let mut proofs = build_chunk_proofs(&challenge.indices, &all_chunks, &layers);
        if proofs.len() >= 2 {
            // Swap data between first two proofs (keep indices the same)
            let d0 = proofs[0].data.clone();
            proofs[0].data = proofs[1].data.clone();
            proofs[1].data = d0;
        }
        let err = engine.respond_to_challenge(blob_id, challenge.round, &proofs).unwrap_err();
        assert_eq!(err, "invalid merkle proof");
    }

    #[test]
    fn test_wrong_proof_path_rejected() {
        // Provider returns correct chunk data but wrong Merkle proof path
        let mut engine = DasEngine::new();
        let original = make_chunks(8, 64);
        let (blob_id, root, all_chunks, layers) = prepare_blob(&original);

        engine.submit_commitment(blob_id, Address::test(1), root, all_chunks.len()).unwrap();
        let randomness = test_randomness(0);
        let challenge = engine.generate_challenge(blob_id, &randomness).unwrap();

        let mut proofs = build_chunk_proofs(&challenge.indices, &all_chunks, &layers);
        // Swap Merkle proof paths between first two proofs
        if proofs.len() >= 2 {
            let p0 = proofs[0].proof.clone();
            proofs[0].proof = proofs[1].proof.clone();
            proofs[1].proof = p0;
        }
        let err = engine.respond_to_challenge(blob_id, challenge.round, &proofs).unwrap_err();
        assert_eq!(err, "invalid merkle proof");
    }

    #[test]
    fn test_fabricated_data_root_rejected() {
        // Provider commits a fabricated root that doesn't match any real data
        let mut engine = DasEngine::new();
        let original = make_chunks(8, 64);
        let (blob_id, _, all_chunks, layers) = prepare_blob(&original);
        let fake_root: Hash = [0xAB; 32];

        engine.submit_commitment(blob_id, Address::test(1), fake_root, all_chunks.len()).unwrap();
        let randomness = test_randomness(0);
        let challenge = engine.generate_challenge(blob_id, &randomness).unwrap();

        // Proofs are valid against real root but not against fake_root
        let proofs = build_chunk_proofs(&challenge.indices, &all_chunks, &layers);
        let err = engine.respond_to_challenge(blob_id, challenge.round, &proofs).unwrap_err();
        assert_eq!(err, "invalid merkle proof");
    }

    // ─── Replay and resubmission attacks ─────────────────────────────────

    #[test]
    fn test_replayed_proof_for_wrong_round_rejected() {
        // Provider tries to replay round 0 proofs for round 1
        let mut engine = DasEngine::new();
        let original = make_chunks(8, 64);
        let (blob_id, root, all_chunks, layers) = prepare_blob(&original);

        engine.submit_commitment(blob_id, Address::test(1), root, all_chunks.len()).unwrap();

        // Complete round 0
        let r0 = test_randomness(0);
        let ch0 = engine.generate_challenge(blob_id, &r0).unwrap();
        let proofs0 = build_chunk_proofs(&ch0.indices, &all_chunks, &layers);
        engine.respond_to_challenge(blob_id, ch0.round, &proofs0).unwrap();

        // Generate round 1 challenge
        let r1 = test_randomness(1);
        let ch1 = engine.generate_challenge(blob_id, &r1).unwrap();

        // Try replaying round 0 proofs (wrong indices for round 1 challenge)
        let result = engine.respond_to_challenge(blob_id, ch1.round, &proofs0);
        // Should fail — different challenge indices
        assert!(result.is_err());
    }

    #[test]
    fn test_double_response_to_same_challenge_rejected() {
        // Provider tries to respond twice to the same challenge
        let mut engine = DasEngine::new();
        let original = make_chunks(8, 64);
        let (blob_id, root, all_chunks, layers) = prepare_blob(&original);

        engine.submit_commitment(blob_id, Address::test(1), root, all_chunks.len()).unwrap();
        let randomness = test_randomness(0);
        let challenge = engine.generate_challenge(blob_id, &randomness).unwrap();
        let proofs = build_chunk_proofs(&challenge.indices, &all_chunks, &layers);

        // First response succeeds
        engine.respond_to_challenge(blob_id, challenge.round, &proofs).unwrap();
        // Second response should fail (challenge already responded)
        let err = engine.respond_to_challenge(blob_id, challenge.round, &proofs).unwrap_err();
        assert_eq!(err, "challenge not found");
    }

    #[test]
    fn test_resubmit_blob_after_failure_rejected() {
        // Provider fails DAS, then tries to resubmit the same blob
        let mut engine = DasEngine::new();
        let original = make_chunks(8, 64);
        let (blob_id, root, all_chunks, _) = prepare_blob(&original);

        engine.submit_commitment(blob_id, Address::test(1), root, all_chunks.len()).unwrap();
        let randomness = test_randomness(0);
        let challenge = engine.generate_challenge(blob_id, &randomness).unwrap();
        engine.set_epoch(challenge.deadline + 1);
        engine.process_expired();

        assert_eq!(engine.get_commitment(&blob_id).unwrap().status, DasStatus::Failed);

        // Try resubmitting
        let err = engine.submit_commitment(blob_id, Address::test(1), root, all_chunks.len()).unwrap_err();
        assert_eq!(err, "blob already committed");
    }

    // ─── Multi-provider adversarial scenarios ────────────────────────────

    #[test]
    fn test_multiple_providers_independent_penalties() {
        // Two providers fail — each penalized independently
        let mut engine = DasEngine::new();
        let orig1 = make_chunks(4, 32);
        let orig2 = make_chunks(4, 64);
        let (id1, root1, chunks1, _) = prepare_blob(&orig1);
        let (id2, root2, chunks2, _) = prepare_blob(&orig2);

        engine.submit_commitment(id1, Address::test(1), root1, chunks1.len()).unwrap();
        engine.submit_commitment(id2, Address::test(2), root2, chunks2.len()).unwrap();

        // Both get challenged
        let r1 = test_randomness(0);
        let r2 = test_randomness(1);
        let ch1 = engine.generate_challenge(id1, &r1).unwrap();
        let ch2 = engine.generate_challenge(id2, &r2).unwrap();

        // Both withhold
        let max_deadline = ch1.deadline.max(ch2.deadline);
        engine.set_epoch(max_deadline + 1);
        engine.process_expired();

        assert_eq!(engine.get_penalty(&Address::test(1)), DAS_PENALTY);
        assert_eq!(engine.get_penalty(&Address::test(2)), DAS_PENALTY);
    }

    #[test]
    fn test_same_provider_multiple_failures_accumulate_penalty() {
        // Same provider fails twice — penalty stacks
        let mut engine = DasEngine::new();
        let orig1 = make_chunks(4, 32);
        let orig2 = make_chunks(4, 48);
        let (id1, root1, chunks1, _) = prepare_blob(&orig1);
        let (id2, root2, chunks2, _) = prepare_blob(&orig2);

        engine.submit_commitment(id1, Address::test(1), root1, chunks1.len()).unwrap();
        let r1 = test_randomness(0);
        let ch1 = engine.generate_challenge(id1, &r1).unwrap();
        engine.set_epoch(ch1.deadline + 1);
        engine.process_expired();

        assert_eq!(engine.get_penalty(&Address::test(1)), DAS_PENALTY);

        // Second blob, same provider
        engine.submit_commitment(id2, Address::test(1), root2, chunks2.len()).unwrap();
        let r2 = test_randomness(10);
        let ch2 = engine.generate_challenge(id2, &r2).unwrap();
        engine.set_epoch(ch2.deadline + 1);
        engine.process_expired();

        assert_eq!(engine.get_penalty(&Address::test(1)), DAS_PENALTY * 2);
    }

    // ─── Edge cases ──────────────────────────────────────────────────────

    #[test]
    fn test_empty_proofs_rejected() {
        let mut engine = DasEngine::new();
        let original = make_chunks(8, 64);
        let (blob_id, root, all_chunks, _) = prepare_blob(&original);

        engine.submit_commitment(blob_id, Address::test(1), root, all_chunks.len()).unwrap();
        let randomness = test_randomness(0);
        let _challenge = engine.generate_challenge(blob_id, &randomness).unwrap();

        let err = engine.respond_to_challenge(blob_id, 0, &[]).unwrap_err();
        assert_eq!(err, "wrong number of proofs");
    }

    #[test]
    fn test_extra_proofs_rejected() {
        let mut engine = DasEngine::new();
        let original = make_chunks(8, 64);
        let (blob_id, root, all_chunks, layers) = prepare_blob(&original);

        engine.submit_commitment(blob_id, Address::test(1), root, all_chunks.len()).unwrap();
        let randomness = test_randomness(0);
        let challenge = engine.generate_challenge(blob_id, &randomness).unwrap();

        let mut proofs = build_chunk_proofs(&challenge.indices, &all_chunks, &layers);
        // Add an extra proof
        proofs.push(proofs[0].clone());
        let err = engine.respond_to_challenge(blob_id, challenge.round, &proofs).unwrap_err();
        assert_eq!(err, "wrong number of proofs");
    }

    #[test]
    fn test_challenge_on_nonexistent_blob() {
        let mut engine = DasEngine::new();
        let fake_id = BlobId([0xFF; 32]);
        let randomness = test_randomness(0);
        let err = engine.generate_challenge(fake_id, &randomness).unwrap_err();
        assert_eq!(err, "blob not found");
    }

    #[test]
    fn test_response_to_nonexistent_blob() {
        let mut engine = DasEngine::new();
        let fake_id = BlobId([0xFF; 32]);
        let err = engine.respond_to_challenge(fake_id, 0, &[]).unwrap_err();
        assert_eq!(err, "blob not found");
    }

    #[test]
    fn test_challenge_on_confirmed_blob_rejected() {
        let mut engine = DasEngine::new();
        let original = make_chunks(8, 64);
        let (blob_id, root, all_chunks, layers) = prepare_blob(&original);

        engine.submit_commitment(blob_id, Address::test(1), root, all_chunks.len()).unwrap();

        // Confirm it
        for seed in 0..REQUIRED_ROUNDS as u8 {
            let r = test_randomness(seed);
            let ch = engine.generate_challenge(blob_id, &r).unwrap();
            let proofs = build_chunk_proofs(&ch.indices, &all_chunks, &layers);
            engine.respond_to_challenge(blob_id, ch.round, &proofs).unwrap();
        }

        // Try another challenge after confirmation
        let r = test_randomness(99);
        let err = engine.generate_challenge(blob_id, &r).unwrap_err();
        assert_eq!(err, "blob not in pending state");
    }

    #[test]
    fn test_invalid_chunk_count_zero() {
        let mut engine = DasEngine::new();
        let fake_id = BlobId([0x01; 32]);
        let err = engine.submit_commitment(fake_id, Address::test(1), [0; 32], 0).unwrap_err();
        assert_eq!(err, "invalid chunk count");
    }

    #[test]
    fn test_invalid_chunk_count_too_large() {
        let mut engine = DasEngine::new();
        let fake_id = BlobId([0x02; 32]);
        let err = engine.submit_commitment(fake_id, Address::test(1), [0; 32], TOTAL_CHUNKS * 4 + 1).unwrap_err();
        assert_eq!(err, "invalid chunk count");
    }

    #[test]
    fn test_honest_provider_survives_all_rounds() {
        // Honest provider with large blob survives complete challenge cycle
        let mut engine = DasEngine::new();
        let original = make_chunks(ORIGINAL_CHUNKS, 128);
        let (blob_id, root, all_chunks, layers) = prepare_blob(&original);
        assert_eq!(all_chunks.len(), TOTAL_CHUNKS); // 64 original + 64 parity

        engine.submit_commitment(blob_id, Address::test(1), root, all_chunks.len()).unwrap();

        for seed in 0..REQUIRED_ROUNDS as u8 {
            let r = test_randomness(seed);
            let ch = engine.generate_challenge(blob_id, &r).unwrap();
            assert_eq!(ch.indices.len(), SAMPLES_PER_ROUND);
            let proofs = build_chunk_proofs(&ch.indices, &all_chunks, &layers);
            engine.respond_to_challenge(blob_id, ch.round, &proofs).unwrap();
        }

        assert_eq!(engine.get_commitment(&blob_id).unwrap().status, DasStatus::Confirmed);
        assert_eq!(engine.get_penalty(&Address::test(1)), 0);
    }
}
