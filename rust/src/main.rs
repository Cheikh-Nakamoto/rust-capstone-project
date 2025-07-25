#![allow(unused)]
use bitcoin::hex::DisplayHex;
use bitcoin::{transaction};
use bitcoincore_rpc::bitcoin::{Amount, BlockHash, Txid};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use serde::Deserialize;
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;
mod utils; // This declares the 'utils' module
use crate::utils::transaction_data::TransactionData;
use crate::utils::utils::{
    create_client_for_wallet, ensure_wallet, generate_spendable_balance, send_20_btc_to,
    write_transaction_to_file,
};

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

    //Initialisation of the TransactionData struct
    let mut transaction_data = TransactionData::default();

    // Get blockchain info
    let blockchain_info = rpc.get_blockchain_info()?;
    println!("Blockchain Info: {:?}", blockchain_info);
    //====================================================================

    // Create/Load the wallets with error handling
    println!("<=================Creation de Wallets=========================>");
    ensure_wallet(&rpc, "Miner")?;
    ensure_wallet(&rpc, "Trader")?;

    // Generate spendable balances in the Miner wallet
    println!("<=================Creation de Balance========================>");

    // Create a client for the Miner wallet
    println!("Loading Miner wallet client...");
    // Create wallet-specific clients
    println!("Creating wallet clients...");
    let miner_client = create_client_for_wallet("Miner")?;

    println!("Generating spendable balance for Miner wallet...");
    generate_spendable_balance(&miner_client, 101, &mut transaction_data)?;

    // Load Trader wallet and generate a new address
    println!("Loading Trader wallet client...");
    let trader_client = create_client_for_wallet("Trader")?;
    println!("Generating new address for Trader wallet...");
    let trader_wallet_info = trader_client.get_wallet_info()?;
    println!("Trader wallet info: {:?}", trader_wallet_info);
    let trader_address = trader_client.get_new_address(None, None)?;
    println!("Trader's new address: {}", trader_address.assume_checked());

    // Send 20 BTC from Miner to Trader
    let tx_id = send_20_btc_to(&trader_client,&miner_client, 20, &mut transaction_data)?;

    // Check spendable balance
    let balance = trader_client.get_balance(Some(0), None)?;
    println!(
        "Solde dépensable traders before mempool confirmation : {}",
        balance
    );

    // Check transaction in mempool
    let mempool = rpc.get_raw_mempool()?;
    println!("Mempool transactions: {:?}", mempool);

    // Mine 1 block to confirm the transaction
    println!("Minage d'un bloc pour confirmer la transaction...");
    generate_spendable_balance(&miner_client, 1, &mut transaction_data)?;

    // Extract all required transaction details
    //load the trader wallet to get the balance after the transaction
    println!("Loading Trader wallet to check balance after transaction...");
    
    let balance_after = trader_client.get_balance(Some(0), None)?;
    println!(
        "Trader's balance after mempool confirmation: {}",
        balance_after
    );

    // Check the transaction ID
    println!("Transaction ID: {}", tx_id);

    // Write the data to ../out.txt in the specified format given in readme.md
    println!("<=================Ecriture dans le fichier========================>");
    write_transaction_to_file(&transaction_data)?;
    println!("Transaction data written to ../out.txt");

    Ok(())
}
