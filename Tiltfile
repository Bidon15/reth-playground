# Tiltfile - Rollup Development Environment

# Allow Tilt to run with the current k8s context (safety check)
allow_k8s_contexts('admin@k8s-tools-internal')

# Configuration flags
config.define_bool('reth-only', args=False, usage='Run only Reth (disable full rollup stack)')

cfg = config.parse()

# Create shared network and JWT secret for all services
local('docker network create rollup-network || true')

# Generate JWT secret for Reth (before it starts)
local_resource('generate-jwt-secret',
    '''
    echo "🔐 Generating JWT secret for Reth..."
    docker run --rm -v jwt-tokens:/shared alpine:latest sh -c "
    apk add --no-cache openssl &&
    openssl rand -hex 32 > /shared/reth-jwt-secret.txt &&
    echo 'JWT secret generated'
    "
    ''',
    labels=['init']
)

# Build Reth using existing Makefile
local_resource('build-reth',
    'make build',
    deps=['./crates', './bin', './Cargo.toml', './Cargo.lock'],
    resource_deps=['generate-jwt-secret'],
    labels=['build']
)

# Always start Reth
docker_compose('./docker-compose.reth.yml')
dc_resource('reth-node', labels=['reth'])

# Full rollup stack is enabled by default (unless --reth-only is specified)
if not cfg.get('reth-only'):
    print("🚀 Starting Full Rollup Stack (Reth + Celestia + Rollkit)...")
    
    # Start Celestia DA node
    docker_compose('./celestia-compose.yml')
    dc_resource('celestia-node', labels=['celestia'])
    
    # Fund Celestia and get its JWT token
    local_resource('celestia-fund',
        '''
        echo "🔑 Waiting for Celestia node to start..."
        
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
    
    # Initialize rollup with all dependencies
    local_resource('celestia-ready',
        '''
        jwt_token=$(docker exec celestia cat /shared/jwt/celestia-jwt.token)
        
        echo "✅ Celestia JWT Token obtained!"
        echo "🔍 Testing Celestia RPC with JWT token..."
        
        curl -s -X POST http://localhost:26658 \\
            -H "Authorization: Bearer $jwt_token" \\
            -H "Content-Type: application/json" \\
            -d '{"jsonrpc":"2.0","method":"header.NetworkHead","params":[],"id":1}' \\
            | jq .
        
        echo ""
        echo "🎉 Celestia is ready for rollup!"
        echo "🔑 Celestia JWT: $jwt_token"
        ''',
        resource_deps=['celestia-fund'],
        labels=['celestia']
    )
    
    # Start Rollkit sequencer
    docker_compose('./rollkit-compose.yml')
    dc_resource('rollup-init', labels=['rollkit'])
    dc_resource('rollkit-sequencer', labels=['rollkit'])
    
    # Final rollup stack validation
    local_resource('rollup-ready',
        '''
        echo "🔄 Waiting for complete rollup stack..."
        
        timeout 120 bash -c '
        until curl -s http://localhost:26657/status | grep -q "result"; do
            echo "Waiting for Rollkit RPC..."
            sleep 5
        done'
        
        echo ""
        echo "🎉 COMPLETE ROLLUP STACK IS READY!"
        echo "========================================="
        echo "🔧 Reth (Execution): http://localhost:8545"
        echo "🌟 Celestia (DA): http://localhost:26658"  
        echo "🔄 Rollkit (Sequencer): http://localhost:26657"
        echo ""
        echo "🧪 Test the rollup:"
        echo "curl -s http://localhost:26657/status | jq ."
        ''',
        resource_deps=['rollkit-sequencer'],
        labels=['rollkit']
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
    print("🚀 FULL ROLLUP STACK ENABLED")
    print("🔧 Reth: http://localhost:8545")
    print("🌟 Celestia: http://localhost:26658")
    print("🔄 Rollkit: http://localhost:26657")
else:
    print("🔧 RETH-ONLY MODE")
    print("🔗 Reth RPC: http://localhost:8545")

print("📊 Tilt Dashboard: http://localhost:10350")
print("🌐 Shared Network: rollup-network")
print("")
print("💡 Usage:")
print("  tilt up           # Full rollup stack (DEFAULT)")
print("  tilt up --reth-only # Just Reth for development")