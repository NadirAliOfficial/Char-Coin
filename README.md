# CHAR Coin Smart Contract (Solana)

## Revolutionizing Crypto with Transparent Donations, Staking & Governance

---

## Overview

CHAR Coin is a Solana-based SPL token designed to facilitate automated donations, dynamic staking rewards, and decentralized governance. Built with Rust and the Anchor framework, the smart contract ensures secure, transparent, and automated fund distribution. Users can stake tokens, vote on proposals, and participate in a deflationary ecosystem that supports charitable causes and community decisions.

This repository contains the complete Solana smart contract for CHAR Coin, featuring:

- **Token Minting & Transfers**
- **Automated Burn Mechanism**
- **Multi-tiered Staking with Dynamic Rewards**
- **Lottery-Based Rewards System**
- **Decentralized Voting for Charitable Donations**
- **DAO Governance for Ecosystem Decisions**
- **Multisig Security & Emergency Halt Mechanism**
- **Private Sale Vesting with Fund Deposit & Claim**

---

## Technology Stack

- **Blockchain:** Solana (SPL Token)
- **Smart Contract Language:** Rust (using Anchor Framework)
- **CLI Tools:** Solana CLI, Anchor CLI
- **Frontend Integration:** JavaScript/React (for PWA integration)
- **Security:** Multisig Wallets, DAO Voting, Smart Contract Audits

---

## Project Modules

- **burn.rs:** Implements buyback & burn functionality with deflationary token burning.
- **staking.rs:** Handles staking mechanics including dynamic rewards, interest rate calculation, and early withdrawal penalties.
- **governance.rs:** Supports DAO governance with proposal submission, stake-weighted voting, and finalization.
- **marketing.rs:** Manages multisig-controlled marketing wallet fund distribution.
- **donation.rs:** Contains the charity registration and weighted voting system for donations.
- **private_sale.rs:** Manages private sale vesting, fund deposit into a vault, and token claim after a 90-day lockup.
- **security.rs:** Provides multisig and additional security features.
- **rewards.rs:** (Optional) Additional rewards system and lottery-based mechanisms.

---

## Installation & Setup

### Prerequisites

Before running the CHAR Coin smart contract, ensure you have:
- **Rust & Cargo** (for smart contract development)
- **Solana CLI** (to interact with the Solana blockchain)
- **Anchor Framework** (for efficient contract development)
- **Node.js & Yarn** (for frontend integration and testing)

### Install Dependencies

```sh
# Install Rust & Cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Install Anchor CLI
cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked
```

---

## Deployment

### Build the Smart Contract

```sh
anchor build
```

### Deploy to Solana Devnet

```sh
solana config set --url devnet
anchor deploy
```

### Verify on Solscan

Retrieve your Program ID and verify the deployment on [Solscan](https://solscan.io):

```sh
solana program show --program-id <YOUR_PROGRAM_ID>
```

---

## Features & Functionalities

### Tokenomics & Fund Distribution

- **Transaction Fee:** 1% of every transaction is collected.
- **Allocation:**
  - **10%** for Buyback, Deflation, & Marketing.
  - **75%** for Charity Donations & Rewards.
  - **15%** for Staking Incentives.

### Automated Burn Mechanism

- A portion of transaction fees is automatically used to buy back and burn tokens.
- This deflationary process ensures long-term scarcity and value appreciation.

### Staking & Dynamic Rewards

- Users can lock CHAR tokens for rewards with lockup durations of **30, 90, or 180 days**.
- **Dynamic Interest Rates:** Calculated based on total transaction volume and total staked tokens.
- Early withdrawal incurs a 10% penalty.
- Reward distribution uses a fixed-point multiplier for precision.

### DAO Governance & Voting

- Community-driven decision-making with proposal creation and stake-weighted voting.
- Voting is open for a defined period, and proposals are finalized on-chain with detailed event logging.
- Only tokens in staking are valid for voting; a minimum of 15 days of staking is required for eligibility.

### Multisig Security

- Multisig wallets ensure that fund transfers (marketing and donation) are approved by multiple authorized parties.
- Emergency halt mechanisms are in place to prevent exploits.

### Private Sale & Vesting

- Investors participate in a private sale where tokens are locked in a vesting contract for 90 days.
- After the vesting period, investors can claim their tokens, ensuring controlled token distribution.

---

## Testing

Run unit tests to validate all functionalities:

```sh
anchor test
```

Tests cover:
- Token minting and transfers.
- Burn mechanism and buyback functionality.
- Staking, reward distribution, and dynamic interest rate calculations.
- DAO governance (proposal submission, voting, and finalization).
- Multisig-controlled fund distribution.
- Private sale vesting and claim processes.

---

## API & Documentation

Express API server (located in the `api` folder) is provided for interacting with the on-chain instructions, including:

- Creating proposals.
- Casting votes.
- Distributing rewards.
- Managing staking and vesting.

Refer to the IDL in the `target/idl` directory for a complete list of instructions and account structures.
