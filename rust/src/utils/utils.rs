use bitcoin::{address::NetworkUnchecked, Address, TxIn};
use bitcoincore_rpc::{
    bitcoin::{Amount, BlockHash, Network, Txid},
    Auth, Client, RpcApi,
};
use serde::{de::value, Deserialize};
use serde_json::json;
use std::io::Write;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
};

use crate::utils::transaction_data::TransactionData;

pub fn create_client_for_wallet(wallet_name: &str) -> bitcoincore_rpc::Result<Client> {
    // Create a new wallet with the given name
    let wallet_rpc = Client::new(
        format!("http://127.0.0.1:18443/wallet/{}", wallet_name).as_str(),
        Auth::UserPass("alice".to_owned(), "password".to_owned()),
    )?;
    Ok(wallet_rpc)
}

pub fn generate_spendable_balance(
    rpc: &Client,
    value: u64,

    transaction_data: &mut TransactionData,
) -> bitcoincore_rpc::Result<()> {
    // Obtenir une adresse du portefeuille Miner
    //ensure_wallet(rpc, wallet_name)?;
    let miner_address = rpc.get_new_address(None, None)?;
    //Verifier l'adresse
    println!("Adresse du portefeuille Miner : {:?}", miner_address);
    if (transaction_data.miner_input_address.is_empty()) {
        transaction_data.set_miner_input_address(miner_address.assume_checked_ref().to_string());
    }
    match miner_address.require_network(Network::Regtest) {
        Ok(adresse) => {
            // Générer 101 blocs pour avoir des fonds dépensables
            let block_hashes = rpc.generate_to_address(value, &adresse)?;
            if (block_hashes.len() == 1) {
                transaction_data.set_block_hash(block_hashes[0].to_string());
                let block_height = rpc.get_block_count()?;
                println!("hauteur du blocs : {}", block_height);
                transaction_data.set_block_height(block_height.to_string());
            }
            println!("Généré {} blocs", block_hashes.len());
        }
        Err(e) => {
            eprintln!("L'adresse n'est pas valide pour le réseau Regtest : {}", e);
        }
    };

    // Vérifier le solde dépensable
    let balance = rpc.get_balance(None, None)?;
   if (transaction_data.miner_input_amount.is_empty()) {
        transaction_data.set_miner_input_amount(format!("{}", balance.to_btc()));
    } else {
        println!("Le montant d'entrée du portefeuille Miner est déjà défini.");
    }
    println!("Solde dépensable du Mineur : {}", balance);

    Ok(())
}

pub fn send_20_btc_to(
    trader_rpc: &Client,
    miner_rpc: &Client,
    value: u64,
    transaction_data: &mut TransactionData,
) -> bitcoincore_rpc::Result<Txid> {
    // Obtenir une adresse du portefeuille Miner
    //ensure_wallet(rpc, "Miner")?;
    let trader_address = trader_rpc.get_new_address(None, None)?;
    //Verifier l'adresse
    println!("Adresse du portefeuille Trader : {:?}", trader_address);
    transaction_data.set_trader_output_address(trader_address.assume_checked_ref().to_string());
    //charger l'adresse du portefeuille Miner
    // rpc.load_wallet("Miner")?;
    // Envoyer 20 BTC du portefeuille Miner au portefeuille Trader
    let change_transaction =
        miner_rpc.get_raw_change_address(Some(bitcoincore_rpc::json::AddressType::Legacy))?;
    println!("Transaction details: {:?}", change_transaction);
    transaction_data.set_miner_change_address(change_transaction.assume_checked_ref().to_string());
    let txid = miner_rpc.send_to_address(
        &trader_address.assume_checked(),
        Amount::from_btc(20.0)?,
        None,
        None,
        None,
        None,
        None,
        None,
    )?;
    println!(
        "Transaction envoyée avec succès, ID de transaction : {}",
        txid
    );
    // Mettre à jour les données de la transaction
    transaction_data.set_transaction_id(txid.to_string());
    transaction_data.set_trader_output_amount(format!("{}", value));
    // checker les frais de transaction
    let fee = miner_rpc.get_mempool_entry(&txid)?.fees;
    transaction_data.set_transaction_fees(format!("-{:e}", fee.base.to_btc()));
    // transaction_data.set_transaction_fee(fee.to_string());
    println!("Frais de transaction : {}", fee.base.to_btc().to_string());
    // Vérifier le solde dépensable
    let balance = miner_rpc.get_balance(None, None)?;
    println!("Solde dépensable mineurs : {}", balance);
    transaction_data.set_miner_change_amount(format!("{}", balance.to_btc()));

    Ok(txid)
}


pub fn ensure_wallet(rpc: &Client, name: &str) -> bitcoincore_rpc::Result<()> {
    // First try to unload the wallet if it's loaded
    let _ = rpc.unload_wallet(Some(name));

    // Check if wallet exists by trying to load it
    match rpc.load_wallet(name) {
        Ok(_) => {
            println!("{} wallet loaded successfully", name);
            Ok(())
        }
        Err(e) if e.to_string().contains("Wallet file verification failed") => {
            // If wallet is corrupted or locked, try creating a new one
            println!(
                "{} wallet appears locked or corrupted, creating fresh wallet...",
                name
            );
            rpc.create_wallet(name, None, None, None, None)?;
            Ok(())
        }
        Err(e) if e.to_string().contains("not found") => {
            // Wallet doesn't exist, create it
            println!("{} wallet not found, creating...", name);
            rpc.create_wallet(name, None, None, None, None)?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub fn write_transaction_to_file(data: &TransactionData) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("../out.txt")?;

    let ordered_data = data.to_ordered_vec();

    for field_value in ordered_data {
        writeln!(file, "{}", field_value)?;
    }

    Ok(())
}
