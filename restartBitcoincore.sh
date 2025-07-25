#!/bin/bash

echo "🛑 Stopping all Bitcoin Core processes..."
sudo pkill -f bitcoind 2>/dev/null || true
sudo pkill -f bitcoin-qt 2>/dev/null || true

# Wait for processes to completely stop
echo "⏳ Waiting for processes to stop..."
sleep 5

# Check if processes are still running
if pgrep -f bitcoin > /dev/null; then
    echo "⚠️  Force killing remaining Bitcoin processes..."
   sudo pkill -9 -f bitcoin
    sleep 3
fi

echo "🧹 Cleaning Bitcoin regtest data directory..."

# Remove the entire regtest directory (this removes all wallets and blockchain data)
if [ -d ~/.bitcoin/regtest ]; then
    echo "📁 Removing ~/.bitcoin/regtest directory..."
   sudo rm -rf ~/.bitcoin/regtest
    echo "✅ Regtest directory removed"
else
    echo "ℹ️  No regtest directory found"
fi

# Also clean any lock files in the main directory
echo "🔓 Cleaning lock files..."
rm -f ~/.bitcoin/.lock 2>/dev/null || true
rm -f ~/.bitcoin/bitcoind.pid 2>/dev/null || true

# Create fresh regtest directory
echo "📁 Creating fresh regtest directory..."
mkdir -p ~/.bitcoin/regtest

echo "🚀 Starting fresh Bitcoin Core in regtest mode..."
bitcoind -regtest -daemon -rpcuser=alice -rpcpassword=password -rpcport=18443

# Wait for Bitcoin Core to start
echo "⏳ Waiting for Bitcoin Core to start..."
sleep 10

# Test connection and show status
echo "🔍 Testing connection..."
if bitcoin-cli -regtest -rpcuser=alice -rpcpassword=password getblockchaininfo > /dev/null 2>&1; then
    echo "✅ Bitcoin Core is ready!"
    echo "📊 Current blockchain info:"
    bitcoin-cli -regtest -rpcuser=alice -rpcpassword=password getblockchaininfo | grep -E "(blocks|chain)"
    
    echo "💼 Current wallets:"
    bitcoin-cli -regtest -rpcuser=alice -rpcpassword=password listwallets
else
    echo "❌ Failed to connect to Bitcoin Core"
    exit 1
fi

echo ""
echo "🎉 Fresh Bitcoin Core regtest environment is ready!"
echo "🔧 You can now run your Rust program with: cargo run"