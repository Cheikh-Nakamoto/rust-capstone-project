#![allow(unused)]
use bitcoin::hex::DisplayHex;
use bitcoincore_rpc::bitcoin::Amount;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use serde::Deserialize;
use serde_json::json;
use std::fs::File;
use std::io::Write;
mod utils; // This declares the 'utils' module
use crate::utils::utils::{create_client_for_wallet, generate_spendable_balance, send_20_btc_to};

// Node access params
const RPC_URL: &str = "http://127.0.0.1:18443"; // Default regtest RPC port
const RPC_USER: &str = "alice";
const RPC_PASS: &str = "password";

// You can use calls not provided in RPC lib API using the generic `call` function.
// An example of using the `send` RPC call, which doesn't have exposed API.
// You can also use serde_json `Deserialize` derivation to capture the returned json result.
fn send(rpc: &Client, addr: &str) -> bitcoincore_rpc::Result<String> {
    let args = [
        json!([{addr : 100 }]), // recipient address
        json!(null),            // conf target
        json!(null),            // estimate mode
        json!(null),            // fee rate in sats/vb
        json!(null),            // Empty option object
    ];

    #[derive(Deserialize)]
    struct SendResult {
        complete: bool,
        txid: String,
    }
    let send_result = rpc.call::<SendResult>("send", &args)?;
    assert!(send_result.complete);
    Ok(send_result.txid)
}

fn main() -> bitcoincore_rpc::Result<()> {
    // Connect to Bitcoin Core RPC
    let rpc = Client::new(
        RPC_URL,
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )?;

    // Get blockchain info
    let blockchain_info = rpc.get_blockchain_info()?;
    println!("Blockchain Info: {:?}", blockchain_info);

    // Create/Load the wallets, named 'Miner' and 'Trader'. Have logic to optionally create/load them if they do not exist or not loaded already.

    //rpc.create_wallet("Miner", None, None, None, None)?;

    // rpc.create_wallet("Trader", None, None, None, None)?;

    // Generate spendable balances in the Miner wallet. How many blocks needs to be mined?
    println!("<==========================================>");
    // Create a client for the Miner wallet
    let miner_wallet = create_client_for_wallet("Miner")?;

    // generate_spendable_balance(&rpc, &miner_wallet, 101)?;
    // Load Trader wallet and generate a new address
    let trader_wallet = create_client_for_wallet("Trader")?;
    // rpc.load_wallet("Trader")?;
    // let trader_address = rpc.get_new_address(None, None)?;
    // println!("Trader's new address: {}", trader_address.assume_checked());
    // Send 20 BTC from Miner to Trader

    // let txId =  send_20_btc_to(&rpc, &miner_wallet, &trader_wallet, 20)?;
    let balance_before = miner_wallet.get_balance(None, None)?;
    println!("Miner's balance after sending: {}", balance_before);
    // Vérifier le solde dépensable
    let balance = trader_wallet.get_balance(Some(0), None)?;
    println!("Solde dépensable traders before  mempool confirmation : {}", balance);
    // Check transaction in mempool
    let mempool = rpc.get_raw_mempool()?;
    println!("Mempool transactions: {:?}", mempool);
    // Mine 1 block to confirm the transaction
    // IMPORTANT: Miner un bloc pour confirmer la transaction
    println!("Minage d'un bloc pour confirmer la transaction...");

    generate_spendable_balance(&rpc, &miner_wallet, 1, "Miner")?;

    // Extract all required transaction details
    let balance_after = trader_wallet.get_balance(Some(0), None)?;
    println!("Trader's balance after mempool confirmation: {}", balance_after);
    // Check the transaction ID
  

    // Write the data to ../out.txt in the specified format given in readme.md

    Ok(())
}
