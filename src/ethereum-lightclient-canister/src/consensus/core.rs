use std::cmp;

use crate::config::Forks;
use crate::consensus::errors::ConsensusError;
use eyre::Result;
use ic_canister_log::log;
use ssz_types::BitVector;
use tree_hash::fixed_bytes::B256;
use tree_hash::TreeHash;

use crate::consensus::proof::{
    is_current_committee_proof_valid, is_execution_payload_proof_valid, is_finality_proof_valid,
    is_next_committee_proof_valid,
};
use helios_common::consensus_spec::{ConsensusSpec, MainnetConsensusSpec};
use helios_common::rpc_types::bls::{PublicKey, Signature};
use helios_common::rpc_types::bootstrap::Bootstrap;
use helios_common::rpc_types::finality_update::FinalityUpdate;
use helios_common::rpc_types::lightclient_header::{BeaconBlockHeader, LightClientHeader};
use helios_common::rpc_types::lightclient_store::{GenericUpdate, LightClientStore};
use helios_common::rpc_types::update::Update;

use crate::consensus::utils::{
    calculate_fork_version, compute_committee_sign_root, compute_fork_data_root,
    get_participating_keys,
};
use crate::ic_log::{INFO, WARNING};

pub fn verify_bootstrap<S: ConsensusSpec>(
    bootstrap: &Bootstrap,
    checkpoint: B256,
    forks: &Forks,
) -> Result<()> {
    if !is_valid_header::<S>(&bootstrap.header, forks) {
        return Err(ConsensusError::InvalidExecutionPayloadProof.into());
    }

    let committee_valid = is_current_committee_proof_valid::<S>(
        &bootstrap.header.beacon,
        &bootstrap.current_sync_committee,
        &bootstrap.current_sync_committee_branch,
        bootstrap.header.beacon.slot / S::slots_per_epoch(),
        forks,
    );

    let header_hash = bootstrap.header.beacon.tree_hash_root();
    let header_valid = header_hash == checkpoint;

    if !header_valid {
        return Err(ConsensusError::InvalidHeaderHash(
            hex::encode(checkpoint.0.as_slice()),
            hex::encode(header_hash.0.as_slice()),
        )
        .into());
    }

    if !committee_valid {
        return Err(ConsensusError::InvalidCurrentSyncCommitteeProof.into());
    }

    Ok(())
}

pub fn verify_update<S: ConsensusSpec>(
    update: &Update,
    expected_current_slot: u64,
    store: &LightClientStore,
    genesis_root: B256,
    forks: &Forks,
) -> Result<()> {
    let update = GenericUpdate::from(update);
    verify_generic_update::<S>(&update, expected_current_slot, store, genesis_root, forks)
}

pub fn verify_finality_update<S: ConsensusSpec>(
    update: &FinalityUpdate,
    expected_current_slot: u64,
    store: &LightClientStore,
    genesis_root: B256,
    forks: &Forks,
) -> Result<()> {
    let update = GenericUpdate::from(update);
    verify_generic_update::<S>(&update, expected_current_slot, store, genesis_root, forks)
}

pub fn apply_bootstrap<S: ConsensusSpec>(store: &mut LightClientStore, bootstrap: &Bootstrap) {
    *store = LightClientStore {
        finalized_header: bootstrap.header.clone(),
        current_sync_committee: bootstrap.current_sync_committee.clone(),
        next_sync_committee: None,
        optimistic_header: bootstrap.header.clone(),
        previous_max_active_participants: 0,
        current_max_active_participants: 0,
        best_valid_update: None,
    };
}

pub fn apply_update<S: ConsensusSpec>(
    store: &mut LightClientStore,
    update: &Update,
) -> Option<B256> {
    let update = GenericUpdate::from(update);
    apply_generic_update::<S>(store, &update)
}

pub fn apply_finality_update<S: ConsensusSpec>(
    store: &mut LightClientStore,
    update: &FinalityUpdate,
) -> Option<B256> {
    let update = GenericUpdate::from(update);
    apply_generic_update::<S>(store, &update)
}

pub fn apply_generic_update<S: ConsensusSpec>(
    store: &mut LightClientStore,
    update: &GenericUpdate,
) -> Option<B256> {
    let committee_bits = get_bits::<S>(&update.sync_aggregate.sync_committee_bits);
    // update best valid update
    if store.best_valid_update.is_none()
        || is_better_update::<S>(update, &store.best_valid_update.as_ref().unwrap())
    {
        store.best_valid_update = Some(update.clone());
    }

    store.current_max_active_participants =
        u64::max(store.current_max_active_participants, committee_bits);

    let should_update_optimistic = committee_bits > safety_threshold::<S>(store)
        && update.attested_header.beacon.slot > store.optimistic_header.beacon.slot;

    if should_update_optimistic {
        store.optimistic_header = update.attested_header.clone();
    }

    let update_attested_period = calc_sync_period::<S>(update.attested_header.beacon.slot);

    let update_finalized_slot = update
        .finalized_header
        .as_ref()
        .map(|h| h.beacon.slot)
        .unwrap_or(0);

    let update_finalized_period = calc_sync_period::<S>(update_finalized_slot);

    let update_has_finalized_next_committee = store.next_sync_committee.is_none()
        && has_sync_update::<S>(update)
        && has_finality_update::<S>(update)
        && update_finalized_period == update_attested_period;

    let should_apply_update = {
        let has_majority = committee_bits * 3 >= S::sync_commitee_size() * 2;
        if !has_majority {
            log!(WARNING, "skipping block with low vote count");
        }

        let update_is_newer = update_finalized_slot > store.finalized_header.beacon.slot;
        let good_update = update_is_newer || update_has_finalized_next_committee;

        has_majority && good_update
    };

    if should_apply_update {
        let checkpoint = apply_update_no_quorum_check::<S>(store, update);
        store.best_valid_update = None;
        checkpoint
    } else {
        None
    }
}

fn apply_update_no_quorum_check<S: ConsensusSpec>(
    store: &mut LightClientStore,
    update: &GenericUpdate,
) -> Option<B256> {
    let store_period = calc_sync_period::<S>(store.finalized_header.beacon.slot);
    let update_finalized_slot = update
        .finalized_header
        .as_ref()
        .map(|h| h.beacon.slot)
        .unwrap_or(0);
    let update_finalized_period = calc_sync_period::<S>(update_finalized_slot);

    if store.next_sync_committee.is_none() {
        if update_finalized_period != store_period {
            return None;
        }
        store
            .next_sync_committee
            .clone_from(&update.next_sync_committee);
    } else if update_finalized_period == store_period + 1 {
        log!(INFO, "sync committee updated");
        store.current_sync_committee = store.next_sync_committee.clone().unwrap();
        store
            .next_sync_committee
            .clone_from(&update.next_sync_committee);
        store.previous_max_active_participants = store.current_max_active_participants;
        store.current_max_active_participants = 0;
    }

    if update_finalized_slot > store.finalized_header.beacon.slot {
        store.finalized_header = update.finalized_header.clone().unwrap();

        if store.finalized_header.beacon.slot > store.optimistic_header.beacon.slot {
            store.optimistic_header = store.finalized_header.clone();
        }

        if store.finalized_header.beacon.slot % S::slots_per_epoch() == 0 {
            let checkpoint = store.finalized_header.beacon.tree_hash_root();
            return Some(checkpoint);
        }
    }

    None
}

// implements checks from validate_light_client_update and process_light_client_update in the
// specification
pub fn verify_generic_update<S: ConsensusSpec>(
    update: &GenericUpdate,
    expected_current_slot: u64,
    store: &LightClientStore,
    genesis_root: B256,
    forks: &Forks,
) -> Result<()> {
    let bits = get_bits::<MainnetConsensusSpec>(&update.sync_aggregate.sync_committee_bits);
    if bits == 0 {
        return Err(ConsensusError::InsufficientParticipation.into());
    }

    if !is_valid_header::<S>(&update.attested_header, forks) {
        return Err(ConsensusError::InvalidExecutionPayloadProof.into());
    }

    let update_finalized_slot = update
        .finalized_header
        .clone()
        .map(|v| v.beacon.slot)
        .unwrap_or_default();

    let valid_time: bool = expected_current_slot >= update.signature_slot
        && update.signature_slot > update.attested_header.beacon.slot
        && update.attested_header.beacon.slot >= update_finalized_slot;

    if !valid_time {
        return Err(ConsensusError::InvalidTimestamp.into());
    }

    let store_period = calc_sync_period::<S>(store.finalized_header.beacon.slot);
    let update_sig_period = calc_sync_period::<S>(update.signature_slot);
    let valid_period = if store.next_sync_committee.is_some() {
        update_sig_period == store_period || update_sig_period == store_period + 1
    } else {
        update_sig_period == store_period
    };

    if !valid_period {
        return Err(ConsensusError::InvalidPeriod.into());
    }

    let update_attested_period = calc_sync_period::<S>(update.attested_header.beacon.slot);
    let update_has_next_committee = store.next_sync_committee.is_none()
        && update.next_sync_committee.is_some()
        && update_attested_period == store_period;

    if update.attested_header.beacon.slot <= store.finalized_header.beacon.slot
        && !update_has_next_committee
    {
        return Err(ConsensusError::NotRelevant.into());
    }

    if let Some(finalized_header) = &update.finalized_header {
        if let Some(finality_branch) = &update.finality_branch {
            if !is_valid_header::<S>(finalized_header, forks) {
                return Err(ConsensusError::InvalidExecutionPayloadProof.into());
            }

            let is_valid = is_finality_proof_valid(
                &update.attested_header.beacon,
                &finalized_header.beacon,
                &finality_branch,
                update.attested_header.beacon.slot / S::slots_per_epoch(),
                forks,
            );

            if !is_valid {
                return Err(ConsensusError::InvalidFinalityProof.into());
            }
        } else {
            return Err(ConsensusError::InvalidFinalityProof.into());
        }
    }

    if let Some(next_sync_committee) = &update.next_sync_committee {
        if let Some(next_sync_committee_branch) = &update.next_sync_committee_branch {
            let is_valid = is_next_committee_proof_valid::<S>(
                &update.attested_header.beacon,
                &next_sync_committee,
                &next_sync_committee_branch,
                update.attested_header.beacon.slot / S::slots_per_epoch(),
                forks,
            );

            if !is_valid {
                return Err(ConsensusError::InvalidNextSyncCommitteeProof.into());
            }
        } else {
            return Err(ConsensusError::InvalidNextSyncCommitteeProof.into());
        }
    }

    let sync_committee = if update_sig_period == store_period {
        &store.current_sync_committee
    } else {
        store.next_sync_committee.as_ref().unwrap()
    };

    let pks =
        get_participating_keys::<S>(sync_committee, &update.sync_aggregate.sync_committee_bits)?;

    let fork_version = calculate_fork_version::<S>(forks, update.signature_slot.saturating_sub(1));
    let fork_data_root = compute_fork_data_root(fork_version, genesis_root);
    let is_valid_sig = verify_sync_committee_signture(
        &pks,
        &update.attested_header.beacon,
        &update.sync_aggregate.sync_committee_signature,
        fork_data_root,
    );

    if !is_valid_sig {
        return Err(ConsensusError::InvalidSignature.into());
    }

    Ok(())
}

pub fn expected_current_slot(time_now: u64, genesis_time: u64) -> u64 {
    let since_genesis = time_now / 1000000000 - genesis_time;

    since_genesis / 12
}

pub fn calc_sync_period<S: ConsensusSpec>(slot: u64) -> u64 {
    let epoch = slot / S::slots_per_epoch();
    epoch / S::epochs_per_sync_commitee_period()
}

pub fn get_bits<S: ConsensusSpec>(
    bitfield: &BitVector<<MainnetConsensusSpec as ConsensusSpec>::SyncCommitteeSize>,
) -> u64 {
    bitfield.iter().filter(|v| *v).count() as u64
}

fn is_better_update<S: ConsensusSpec>(
    new_update: &GenericUpdate,
    old_update: &GenericUpdate,
) -> bool {
    let max_active_participants = new_update.sync_aggregate.sync_committee_bits.len() as u64;
    let new_num_active_participants = get_bits::<S>(&new_update.sync_aggregate.sync_committee_bits);
    let old_num_active_participants = get_bits::<S>(&old_update.sync_aggregate.sync_committee_bits);
    let new_has_supermajority = new_num_active_participants * 3 >= max_active_participants * 2;
    let old_has_supermajority = old_num_active_participants * 3 >= max_active_participants * 2;

    if new_has_supermajority != old_has_supermajority {
        return new_has_supermajority;
    }

    if !new_has_supermajority && new_num_active_participants != old_num_active_participants {
        return new_num_active_participants > old_num_active_participants;
    }

    // compare presence of relevant sync committee
    let new_has_relevant_sync_committee = new_update.next_sync_committee_branch.is_some()
        && calc_sync_period::<S>(new_update.attested_header.beacon.slot)
            == calc_sync_period::<S>(new_update.signature_slot);
    let old_has_relevant_sync_committee = old_update.next_sync_committee_branch.is_some()
        && calc_sync_period::<S>(old_update.attested_header.beacon.slot)
            == calc_sync_period::<S>(old_update.signature_slot);
    if new_has_relevant_sync_committee != old_has_relevant_sync_committee {
        return new_has_relevant_sync_committee;
    }

    // compare indication of any finality
    let new_has_finality = new_update.finality_branch.is_some();
    let old_has_finality = old_update.finality_branch.is_some();
    if new_has_finality != old_has_finality {
        return new_has_finality;
    }

    // compare sync committee finality
    if new_has_finality {
        let new_has_sync_committee_finality =
            calc_sync_period::<S>(
                new_update
                    .finalized_header
                    .clone()
                    .unwrap_or_default()
                    .beacon
                    .slot,
            ) == calc_sync_period::<S>(new_update.attested_header.beacon.slot);
        let old_has_sync_committee_finality =
            calc_sync_period::<S>(
                old_update
                    .finalized_header
                    .clone()
                    .unwrap_or_default()
                    .beacon
                    .slot,
            ) == calc_sync_period::<S>(old_update.attested_header.beacon.slot);
        if new_has_sync_committee_finality != old_has_sync_committee_finality {
            return new_has_sync_committee_finality;
        }
    }

    // tiebreaker 1: Sync committee participation beyond supermajority
    if new_num_active_participants != old_num_active_participants {
        return new_num_active_participants > old_num_active_participants;
    }

    // tiebreaker 2: Prefer older data (fewer changes to best)
    if new_update.attested_header.beacon.slot != old_update.attested_header.beacon.slot {
        return new_update.attested_header.beacon.slot < old_update.attested_header.beacon.slot;
    }
    new_update.signature_slot < old_update.signature_slot
}

fn has_sync_update<S: ConsensusSpec>(update: &GenericUpdate) -> bool {
    update.next_sync_committee.is_some() && update.next_sync_committee_branch.is_some()
}

fn has_finality_update<S: ConsensusSpec>(update: &GenericUpdate) -> bool {
    update.finality_branch.is_some()
}

fn verify_sync_committee_signture(
    pks: &[PublicKey],
    attested_header: &BeaconBlockHeader,
    signature: &Signature,
    fork_data_root: B256,
) -> bool {
    let header_root = attested_header.tree_hash_root();
    let signing_root = compute_committee_sign_root(header_root, fork_data_root);
    let instruction_c1 = ic_cdk::api::instruction_counter();
    let r = signature.verify(signing_root.as_slice(), pks);
    let instruction_c2 = ic_cdk::api::instruction_counter();
    log!(INFO, "bls signature instractions {} {} {}", instruction_c1, instruction_c2, instruction_c2 - instruction_c1);
    r
}

fn safety_threshold<S: ConsensusSpec>(store: &LightClientStore) -> u64 {
    cmp::max(
        store.current_max_active_participants,
        store.previous_max_active_participants,
    ) / 2
}

fn is_valid_header<S: ConsensusSpec>(header: &LightClientHeader, forks: &Forks) -> bool {
    let epoch = header.beacon.slot / S::slots_per_epoch();

    if epoch < forks.capella.epoch {
        false
    } else {
        let execution = header.execution.clone();
        let execution_branch = header.execution_branch.clone();

        let valid_execution_type = epoch >= forks.deneb.epoch;

        let proof_valid =
            is_execution_payload_proof_valid(&header.beacon, &execution, &execution_branch);

        proof_valid && valid_execution_type
    }
}
