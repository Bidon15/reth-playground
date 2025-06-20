#!/bin/bash
# celestia-fund.sh - Fund the Celestia wallet and get JWT token

set -e

echo "💰 Funding Celestia wallet and getting JWT token..."

# Wait for node to be ready
echo "⏳ Waiting for Celestia node to be ready..."
sleep 10

# Get JWT token
echo "🔑 Getting JWT token..."
jwt_token=""
retry_count=0
max_retries=20

while [ -z "$jwt_token" ] && [ $retry_count -lt $max_retries ]; do
    if celestia light auth admin --node.store /home/celestia > /tmp/jwt.txt 2>/dev/null; then
        jwt_token=$(cat /tmp/jwt.txt | tr -d '\n')
        break
    fi
    echo "Waiting for JWT token generation... (attempt $((retry_count + 1)))"
    sleep 3
    retry_count=$((retry_count + 1))
done

if [ -z "$jwt_token" ]; then
    echo "❌ Failed to get JWT token after $max_retries attempts"
    exit 1
fi

# Store JWT token in shared volume
echo "$jwt_token" > /shared/jwt/celestia-jwt.token
chmod 644 /shared/jwt/celestia-jwt.token

echo "✅ JWT Token obtained: $jwt_token"

# Test Celestia RPC
echo "🔍 Testing Celestia RPC..."
if curl -s -X POST http://localhost:26658 \
    -H "Authorization: Bearer $jwt_token" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"header.NetworkHead","params":[],"id":1}' \
    | grep -q "result"; then
    echo "✅ Celestia RPC is ready!"
else
    echo "⚠️  Celestia RPC not responding, but JWT token is available"
fi

# Get wallet address
echo "🔑 Getting wallet address through JSON-RPC..."
max_address_retries=10
address_retry=0

while [ $address_retry -lt $max_address_retries ]; do
    address_response=$(curl -s -X POST http://localhost:26658 \
        -H "Authorization: Bearer $jwt_token" \
        -H "Content-Type: application/json" \
        -d '{"id": 1, "jsonrpc": "2.0", "method": "state.AccountAddress", "params": []}' 2>/dev/null)
    
    address=$(echo "$address_response" | jq -r '.result // empty' 2>/dev/null | grep -o 'celestia[a-z0-9]\{39\}' | head -1)
    
    if [ -n "$address" ]; then
        echo "Wallet address: $address"
        break
    fi
    echo "Waiting for wallet address... (attempt $((address_retry + 1)))"
    sleep 3
    address_retry=$((address_retry + 1))
done

if [ -z "$address" ]; then
    echo "❌ Failed to get wallet address"
    exit 1
fi

# Check balance and fund if needed
echo "💰 Checking balance..."
balance_response=$(curl -s "https://api-mocha-4.consensus.celestia-mocha.com/cosmos/bank/v1beta1/balances/$address" || echo '{"balances":[]}')
balance=$(echo "$balance_response" | jq -r '.balances[] | select(.denom == "utia") | .amount // "0"')

if [[ "$balance" =~ ^[0-9]+$ ]] && [ "$balance" -gt 1000000 ]; then
    balance_tia=$(echo "scale=6; $balance/1000000" | bc)
    echo "✅ Current balance: $balance_tia TIA (sufficient)"
else
    echo "🚰 Requesting funds from faucet..."
    curl -X POST 'https://faucet.celestia-mocha.com/api/v1/faucet/give_me' \
        -H 'Content-Type: application/json' \
        -d "{\"address\": \"$address\", \"chainId\": \"mocha-4\"}" \
        --max-time 30 || echo "Faucet request sent"
    
    echo "⏳ Waiting for funds to arrive..."
    sleep 15
fi


echo ""
echo "🎉 Celestia Setup Complete!"
echo "=================================="
echo "📡 RPC Endpoint: http://localhost:26658"
echo "🔑 JWT Token: $jwt_token"
echo "💾 Token saved to: /shared/jwt/celestia-jwt.token"
echo ""
echo "🧪 Test with curl:"
echo "curl -X POST http://localhost:26658 \\"
echo "  -H 'Authorization: Bearer $jwt_token' \\"
echo "  -H 'Content-Type: application/json' \\"
echo "  -d '{\"jsonrpc\":\"2.0\",\"method\":\"header.NetworkHead\",\"params\":[],\"id\":1}'"