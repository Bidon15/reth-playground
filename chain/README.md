# Bootcamp Chain Configuration

This directory contains the custom chain configuration for the bootcamp rollup.

## Chain Details
- **Chain ID**: 1234
- **Network**: Custom Ethereum-compatible rollup
- **Consensus**: Proof of Stake (post-merge)

## Pre-funded Accounts

The following accounts are pre-funded with 100,000 ETH each for testing:

### Account 1
- **Address**: `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266`
- **Private Key**: `0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80`
- **Balance**: 100,000 ETH

### Account 2
- **Address**: `0x70997970C51812dc3A010C7d01b50e0d17dc79C8`
- **Private Key**: `0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d`
- **Balance**: 100,000 ETH

### Account 3
- **Address**: `0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC`
- **Private Key**: `0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a`
- **Balance**: 100,000 ETH

### Account 4
- **Address**: `0x90F79bf6EB2c4f870365E785982E1f101E93b906`
- **Private Key**: `0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6`
- **Balance**: 100,000 ETH

### Account 5
- **Address**: `0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65`
- **Private Key**: `0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a`
- **Balance**: 100,000 ETH

## Usage

Students can import any of these private keys into MetaMask or other wallets to start testing immediately.

### MetaMask Setup
1. Add custom network with RPC URL: `http://localhost:8545`
2. Chain ID: `1234`
3. Currency Symbol: `ETH`
4. Import any of the private keys above

### Using with CLI tools
```bash
# Example with cast
cast send --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --rpc-url http://localhost:8545 \
  0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
  --value 1ether
```

## Security Notice
⚠️ **These private keys are for testing only!** Never use these accounts on mainnet or with real funds.