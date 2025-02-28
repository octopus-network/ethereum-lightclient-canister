dfx canister create ethereum_lightclient_canister
cargo build --release --target wasm32-unknown-unknown --package ethereum_lightclient_canister
candid-extractor target/wasm32-unknown-unknown/release/ethereum_lightclient_canister.wasm > src/ethereum-lightclient-canister/candid.did
dfx build ethereum_lightclient_canister
dfx deploy ethereum_lightclient_canister --argument '(record {execution_rpc = "https://mainnet.infura.io/v3/025779c23a2d44d1b1ecef2bfb4f2b29"})'