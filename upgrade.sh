cargo build --release --target wasm32-unknown-unknown --package ethereum_lightclient_canister
candid-extractor target/wasm32-unknown-unknown/release/ethereum_lightclient_canister.wasm > src/ethereum-lightclient-canister/candid.did
dfx build ethereum_lightclient_canister
dfx canister install ethereum_lightclient_canister --wasm .dfx/local/canisters/ethereum_lightclient_canister/ethereum_lightclient_canister.wasm.gz --mode upgrade