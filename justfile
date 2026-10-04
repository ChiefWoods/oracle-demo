# Devnet integration tests

# Build the program with devnet-only relaxed staleness checks.
build:
    anchor build -- --features devnet,disable-staleness-check

# Deploy the program to the configured devnet cluster without publishing an IDL.
deploy:
    anchor deploy -p oracle-demo --provider.cluster d --program-keypair target/deploy/oracle_demo-keypair.json --provider.wallet dev-wallet.json --no-idl

# Run a named devnet test and keep test output visible.
test test_name:
    cargo test -p oracle-demo --features devnet --test devnet {{test_name}} -- --nocapture
