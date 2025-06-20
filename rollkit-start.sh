#!/bin/sh
# rollkit-start.sh - Simple entrypoint for Rollkit EVM Single

set -e

echo "🔄 Starting Rollkit EVM Single Sequencer..."

# Wait for configuration to be available
echo "⏳ Waiting for rollkit configuration..."
timeout 120 sh -c '
until [ -f /shared/rollkit.env ]; do
    echo "Waiting for rollkit.env..."
    sleep 3
done'

if [ ! -f /shared/rollkit.env ]; then
    echo "❌ Rollkit configuration not found! Initialization may have failed."
    exit 1
fi

echo "✅ Configuration found, loading environment..."

# Source the environment file to load variables
source /shared/rollkit.env

# Export all variables so they're available to the rollkit process
export EVM_ENGINE_URL
export EVM_ETH_URL
export EVM_JWT_SECRET
export EVM_GENESIS_HASH
export EVM_BLOCK_TIME
export EVM_SIGNER_PASSPHRASE
export DA_ADDRESS
export DA_AUTH_TOKEN
export DA_NAMESPACE

echo "🚀 Starting Rollkit EVM Single..."

echo "Configuration:"
echo "  EVM Engine: $EVM_ENGINE_URL"
echo "  EVM RPC: $EVM_ETH_URL"
echo "  Genesis Hash: $EVM_GENESIS_HASH"
echo "  DA Address: $DA_ADDRESS"
echo "  DA Namespace: $DA_NAMESPACE"
echo "  Block Time: $EVM_BLOCK_TIME"
echo "  Reth JWT: ${EVM_JWT_SECRET:0:20}..."
echo "  Celestia JWT: ${DA_AUTH_TOKEN:0:20}..."

echo ""
echo "🔍 Checking Reth configuration..."
echo "Engine API should be available at: $EVM_ENGINE_URL"
echo "ETH API should be available at: $EVM_ETH_URL"
echo "JWT Secret length: ${#EVM_JWT_SECRET} characters"
echo ""

cd /usr/bin

sleep 5

# Create default rollkit config if missing
if [ ! -f "$HOME/.evm-single/config/signer.json" ]; then
  ./evm-single init --rollkit.node.aggregator=true --rollkit.signer.passphrase $EVM_SIGNER_PASSPHRASE
fi

# Conditionally add --rollkit.da.address if DA_ADDRESS is set
da_flag=""
if [ -n "$DA_ADDRESS" ]; then
  da_flag="--rollkit.da.address $DA_ADDRESS"
fi

# Conditionally add --rollkit.da.auth_token if DA_AUTH_TOKEN is set
da_auth_token_flag=""
if [ -n "$DA_AUTH_TOKEN" ]; then
  da_auth_token_flag="--rollkit.da.auth_token $DA_AUTH_TOKEN"
fi

# Conditionally add --rollkit.da.namespace if DA_NAMESPACE is set
da_namespace_flag=""
if [ -n "$DA_NAMESPACE" ]; then
  da_namespace_flag="--rollkit.da.namespace $DA_NAMESPACE"
fi

exec ./evm-single start \
  --evm.jwt-secret $EVM_JWT_SECRET \
  --evm.genesis-hash $EVM_GENESIS_HASH \
  --evm.engine-url $EVM_ENGINE_URL \
  --evm.eth-url $EVM_ETH_URL \
  --rollkit.node.block_time $EVM_BLOCK_TIME \
  --rollkit.node.aggregator=true \
  --rollkit.signer.passphrase $EVM_SIGNER_PASSPHRASE \
  $da_flag \
  $da_auth_token_flag \
  $da_namespace_flag