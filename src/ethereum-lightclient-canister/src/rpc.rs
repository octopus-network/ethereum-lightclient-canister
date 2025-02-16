use helios_common::http::get;
use helios_consensus_core::consensus_spec::MainnetConsensusSpec;
use helios_consensus_core::types::FinalityUpdate;
use crate::state::read_state;

async fn get_finality_update() -> eyre::Result<FinalityUpdate<MainnetConsensusSpec>> {
    let rpc = read_state(|s|s.consensus_rpc.clone());
    let req = format!("{}/eth/v1/beacon/light_client/finality_update", rpc);
    let res: FinalityUpdateResponse<S> = get(&req)
        .await
        .map_err(|e| RpcError::new("finality_update", e))?;

    Ok(res.data)
}

/*
pub async fn advance() -> eyre::Result<()> {
    let finality_update = self.rpc.get_finality_update().await?;
    self.verify_finality_update(&finality_update)?;
    self.apply_finality_update(&finality_update);

    if self.store.next_sync_committee.is_none() {
        debug!(target: "helios::consensus", "checking for sync committee update");
        let current_period = calc_sync_period::<S>(self.store.finalized_header.beacon().slot);
        let mut updates = self.rpc.get_updates(current_period, 1).await?;

        if updates.len() == 1 {
            let update = updates.get_mut(0).unwrap();
            let res = self.verify_update(update);

            if res.is_ok() {
                info!(target: "helios::consensus", "updating sync committee");
                self.apply_update(update);
            }
        }
    }

    Ok(())
}*/