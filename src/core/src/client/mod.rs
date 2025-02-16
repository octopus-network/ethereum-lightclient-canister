use std::sync::Arc;

use eyre::Result;
use tracing::{info, warn};

use crate::client::node::Node;
#[cfg(not(target_arch = "wasm32"))]
use crate::client::rpc::Rpc;
use crate::consensus::Consensus;
use crate::fork_schedule::ForkSchedule;
use crate::network_spec::NetworkSpec;

pub mod node;

pub struct Client<N: NetworkSpec, C: Consensus<N::BlockResponse>> {
    node: Arc<Node<N, C>>,
    #[cfg(not(target_arch = "wasm32"))]
    rpc: Option<Rpc<N, C>>,
}

impl<N: NetworkSpec, C: Consensus<N::BlockResponse>> Client<N, C> {
    pub fn new(
        execution_rpc: &str,
        consensus: C,
        fork_schedule: ForkSchedule,
        #[cfg(not(target_arch = "wasm32"))] rpc_address: Option<SocketAddr>,
    ) -> Result<Self> {
        let node = Node::new(execution_rpc, consensus, fork_schedule)?;
        let node = Arc::new(node);

        #[cfg(not(target_arch = "wasm32"))]
        let mut rpc: Option<Rpc<N, C>> = None;

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(rpc_address) = rpc_address {
            rpc = Some(Rpc::new(node.clone(), rpc_address));
        }

        Ok(Client {
            node,
            #[cfg(not(target_arch = "wasm32"))]
            rpc,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(rpc) = &mut self.rpc {
            rpc.start().await?;
        }

        Ok(())
    }

    pub async fn shutdown(&self) {
        info!(target: "helios::client","shutting down");
        if let Err(err) = self.node.consensus.shutdown() {
            warn!(target: "helios::client", error = %err, "graceful shutdown failed");
        }
    }

}
