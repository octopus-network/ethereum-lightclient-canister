dfx canister create ethereum_oracle
cargo build --release --target wasm32-unknown-unknown --package ethereum_oracle
candid-extractor target/wasm32-unknown-unknown/release/ethereum_oracle.wasm > src/ethereum-oracle/candid.did
dfx build ethereum_oracle
dfx deploy ethereum_oracle