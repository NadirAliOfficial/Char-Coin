
# CharCoin Smart Contract (Solana)

### Transparent Donations • DAO Governance • Dynamic Staking • Deflationary Tokenomics

---

## 📌 Overview

CHAR Coin is a Solana-based smart contract system built using the Anchor framework and Rust. It implements a decentralized ecosystem for:

- Stake-based DAO governance  
- Transparent, automated donation fund allocation  
- Buyback and burn deflation mechanics  
- Dynamic staking with time-based rewards  
- Multisig-secured marketing and treasury fund control  
- Emergency halt for contract protection

All modules have passed internal audits and comprehensive unit testing. For more details, please refer to the Official White Paper: https://charcoin.org/charcoin-white-paper/

---

## ✅ Development Environment

- **Rust:** `1.83.0`
- **Anchor CLI:** `0.30.1`
- **Solana CLI:** `2.1.17`
- **Test Path:** `tests/charcoin.ts`

---

## 🧩 Program Modules

Each logic component is modularized for clarity and maintainability:

| Module            | Description                                                                            |
|-------------------|----------------------------------------------------------------------------------------|
| `lib.rs`          | Entry point and global instruction router                                              |
| `burn.rs`         | Buyback and deflationary burn functions                                                |
| `staking.rs`      | Locking, staking, and dynamic reward distribution                                      |
| `governance.rs`   | Proposal lifecycle, vote casting, and finalization                                     |
| `donation.rs`     | Charity registration, voting, and distribution logic                                   |
| `marketing.rs`    | Multisig marketing wallet management and release rules                                 |
| `security.rs`     | Emergency halt mechanism and multisig verification                                     |
| `rewards.rs`      | Additional reward systems, including lottery-style and volume-based bonuses (optional) |

---

## 🧪 Unit Testing

Run:

```bash
anchor test
````

**Test Summary:**
All 20 core instructions and flows are passing:

* Stake / Unstake / Claim / Penalty logic
* Emergency halt triggers
* Buyback + burn execution
* DAO: Proposal → Vote → Finalize
* Marketing + donation fund release (multisig)
* DAO treasury: withdraw / approve / execute

**Execution Time:** ≈ 42 seconds

---

## 🔐 Security Features

* **Multisig Authorization:** All fund releases (marketing, donations, DAO treasury) are protected by multisig thresholds.
* **Emergency Halt:** Critical instructions can be paused upon detection of suspicious behavior.
* **DAO Governance:** Voting power is locked to staking participation; only staked tokens (min 15 days) are eligible to vote.
* **Audit Readiness:** Fully modular code, deterministic execution paths, and descriptive events for audit traceability.

---

## 📈 Tokenomics Structure

CHAR Coin uses a fixed **1% transaction fee**, distributed as:

| Category                    | Allocation | Description                                                           |
| --------------------------- | ---------- | --------------------------------------------------------------------- |
| Buyback & Marketing         | 10%        | Includes burn wallet and two multisig marketing funds                 |
| Charity Donations & Rewards | 75%        | Community-driven voting for recipients; includes lottery reward pools |
| Staking Incentives          | 15%        | Distributed across time-locked staking pools                          |

Reward rates are **dynamically adjusted** based on:

* Total staked token volume
* Daily/weekly transaction activity
* Lock-in duration (30/90/180 days)

---

## 🗳️ DAO Governance

Governance follows a proposal–vote–finalize structure. Key mechanics:

* **Proposals**: Any eligible staker can submit
* **Voting Period**: Defined by system configuration
* **Weight**: Proportional to staked tokens
* **Finalization**: On-chain vote tally and execution

---

## ❤️ Charity Voting System

* **Cause Registration**: Any verified organization can be listed
* **User Voting**: Monthly vote window; 1 vote per wallet
* **Distribution**: Weighted fund release after vote closure
* **Annual Reserve**: 20% of funds are pooled for yearly distribution

---

## 🛠 Setup Instructions

### Dependencies

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Install Anchor CLI
cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked
```

### Build & Deploy

```bash
anchor build
solana config set --url devnet
anchor deploy
```

Verify deployment:

```bash
solana program show --program-id <PROGRAM_ID>
```

---

## 📁 Repository Structure

```
charcoin-programs/
├── programs/
│   └── charcoin/
│       └── src/
│           ├── lib.rs
│           ├── burn.rs
│           ├── staking.rs
│           ├── governance.rs
│           ├── marketing.rs
│           ├── donation.rs
│           ├── security.rs
│           └── rewards.rs
├── tests/
│   └── charcoin.ts
├── Anchor.toml
├── Cargo.toml
└── README.md
```

---

## 📦 Audit Readiness

The repository is structured for third-party audit access:

* Full instruction coverage with unit tests
* Clear event logs for transaction tracing
* Multisig-secured flows for critical operations
* Configurable constants for fee and reward logic

For questions or integration support, please open an issue on this repository.