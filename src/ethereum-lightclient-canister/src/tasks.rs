use ic_canister_log::log;
use crate::consensus::consensus::Inner;
use helios_common::consensus_spec::MainnetConsensusSpec;
use crate::ic_log::WARNING;
use crate::state::{mutate_state, read_state};

pub fn lightclient_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new("SCAN_EVM_TASK_NAME".to_string()) {
            Some(guard) => guard,
            None => return,
        };
        let store = read_state(|s|s.store.clone());
        let mut inner = Inner::<MainnetConsensusSpec>::new(store);
        match inner.advance().await {
            Ok(_) => {
                inner.store().await;
                mutate_state(|s|s.store = inner.store);
            }
            Err(err) => {
                log!(WARNING, "advance error: {}", err);
            }
        }
    });
}