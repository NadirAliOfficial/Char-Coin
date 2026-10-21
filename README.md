# CharCoin — Solana Token

A Solana-based token implementing a 1% transaction fee with auto-distribution to charity and staking pools.

## Description
CharCoin is a custom SPL token on the Solana blockchain that applies a 1% fee on every transfer. The fee is automatically split between a charity wallet and staking reward pools, enabling continuous support for charitable causes while rewarding token holders who stake their tokens.

## Features
- SPL token with custom transfer hook
- 1% fee on every transaction
- Automatic charity wallet allocation
- Staking reward distribution

## Tech Stack
- Rust + Anchor framework
- Solana Program Library (SPL)

## Requirements
- Node.js (compatible with the versions used in `package.json`)
- Rust toolchain with `cargo` and `anchor` installed
- Solana CLI configured for the target cluster

## Installation
```bash
# Install Node.js dependencies
npm install

# Build the Solana program
anchor build
```

## Configuration
The scripts rely on a configuration file `scripts/config.js` which expects the following environment variables to be set (values are placeholders):
```
PRIVATE_KEY=your_private_key_here
RPC_URL=https://api.mainnet-beta.solana.com
```
Create a `.env` file with these variables or export them in your shell before running scripts.

## Usage
- Build and deploy the program:
```bash
anchor build
anchor deploy --provider.cluster mainnet-beta
```
- Run utility scripts located in the `scripts/` directory, for example:
```bash
node scripts/Donation.js
node scripts/Security.js
node scripts/Treasury.js
```
- Interact with the client example:
```bash
node client/client.ts
```

## Project Structure
```
.DS_Store
.gitignore
Anchor.toml
Cargo.toml
LICENSE
README.md
client/
  client.ts
migrations/
  deploy.ts
scripts/
  Donation.js
  Security.js
  Treasury.js
  claimRewardHandler.js
  config.js
  createSPLToken.js
  dump.js
  index.js
  init.ts
  initialize.js
  proposal_finalize.js
  proposal_submit.js
  proposal_vote.js
  stake-tokens.js
  stakeTokensHandler.js
  stake_init.ts
  stakingInitialize.js
  unstakeTokensHandler.js
tests/
  charcoin.ts
programs/
  charcoin/
    Cargo.toml
    Xargo.toml
    src/
      burn.rs
      donation.rs
      errors.rs
      lib.rs
      marketing.rs
      rewards.rs
      security.rs
      staking.rs
```

## License
MIT
<!-- updated: 2025-11-17-r01 -->
