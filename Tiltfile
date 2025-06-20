# Tiltfile - Composable Rollup Development

# Allow Tilt to run with the current k8s context (safety check)
allow_k8s_contexts('admin@k8s-tools-internal')

# Configuration flags
config.define_bool('reth-only', args=False, usage='Run only Reth (disable Celestia)')

cfg = config.parse()

# Create shared network for all services
local('docker network create rollup-network || true')

# Build Reth using existing Makefile
local_resource('build-reth',
    'make build',
    deps=['./crates', './bin', './Cargo.toml', './Cargo.lock'],
    labels=['build']
)

# Always start Reth
docker_compose('./docker-compose.reth.yml')
dc_resource('reth-node', labels=['reth'])

# Celestia is enabled by default (unless --reth-only is specified)
if not cfg.get('reth-only'):
    print("🌟 Starting Celestia DA node with auto-funding and JWT token setup...")
    
    # Start Celestia with auto-setup
    docker_compose('./celestia-compose.yml')
    dc_resource('celestia-node', labels=['celestia'])
    
    # Wait for Celestia to start, then fund and get JWT token
    local_resource('celestia-fund',
        '''
        echo "🔑 Waiting for Celestia node to start..."
        
        # Wait for container to be ready
        timeout 60 bash -c '
        until docker exec celestia echo "Container ready" > /dev/null 2>&1; do
            echo "Waiting for container..."
            sleep 2
        done'
        
        echo "💰 Running funding and JWT setup..."
        docker exec celestia sh /fund.sh
        ''',
        resource_deps=['celestia-node'],
        labels=['celestia']
    )
    
    # Check that everything is working
    local_resource('celestia-ready',
        '''
        # Get the JWT token
        jwt_token=$(docker exec celestia cat /shared/jwt/celestia-jwt.token)
        
        echo "✅ JWT Token obtained!"
        echo "🔍 Testing Celestia RPC with JWT token..."
        
        # Test if Celestia is ready using the JWT token
        curl -s -X POST http://localhost:26658 \\
            -H "Authorization: Bearer $jwt_token" \\
            -H "Content-Type: application/json" \\
            -d '{"jsonrpc":"2.0","method":"header.NetworkHead","params":[],"id":1}' \\
            | jq .
        
        echo ""
        echo "🎉 Celestia is ready for Rollkit!"
        echo "🔑 JWT Token: $jwt_token"
        echo ""
        echo "🔗 Network connectivity:"
        echo "  Rollkit → Celestia: http://celestia-node:26658"
        echo "  Rollkit → Reth: http://reth-node:8545"
        ''',
        resource_deps=['celestia-fund'],
        labels=['celestia']
    )

# Manual health check from host
local_resource('reth-health',
    '''
    curl -s http://localhost:8545 \
        -X POST \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        | jq .
    ''',
    trigger_mode=TRIGGER_MODE_MANUAL,
    labels=['test']
)

# Print usage info
print("")
if not cfg.get('reth-only'):
    print("🌟 Celestia DA: http://localhost:26658")
    print("🔑 JWT Token: Check celestia-ready logs")

print("🔗 Reth RPC: http://localhost:8545")
print("📊 Tilt Dashboard: http://localhost:10350")
print("🌐 Shared Network: rollup-network")
print("")
print("💡 Usage:")
print("  tilt up             # Reth + Celestia DA on shared network")
print("  tilt up --reth-only # Just Reth on shared network")