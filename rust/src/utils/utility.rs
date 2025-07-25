use bitcoin::{address::NetworkUnchecked, transaction, Address, TxIn, TxOut, Script};
use bitcoincore_rpc::{
    bitcoin::{Amount, BlockHash, Network, Txid},
    Auth, Client, RpcApi,
};
use serde::{de::value, Deserialize};
use serde_json::json;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
};
use std::{io::Write, str::FromStr};

use crate::utils::transaction_data::TransactionData;


use bitcoincore_rpc::json::GetTransactionResult;


pub fn create_client_for_wallet(wallet_name: &str) -> bitcoincore_rpc::Result<Client> {
    // Create a new wallet with the given name
    let wallet_rpc = Client::new(
        format!("http://127.0.0.1:18443/wallet/{wallet_name}").as_str(),
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
    println!("Adresse du portefeuille Miner : {miner_address:?}");

    if (transaction_data.miner_input_address.is_empty()) {
        transaction_data.set_miner_input_address(miner_address.assume_checked_ref().to_string());
    } else {
        transaction_data.set_miner_change_address(miner_address.assume_checked_ref().to_string());
    }
    match miner_address.require_network(Network::Regtest) {
        Ok(adresse) => {
            // Générer 101 blocs pour avoir des fonds dépensables
            let block_hashes = rpc.generate_to_address(value, &adresse)?;
            if (block_hashes.len() == 1) {
                transaction_data.set_block_hash(block_hashes[0].to_string());
                let block_height = rpc.get_block_count()?;
                println!("hauteur du blocs : {block_height}");
                transaction_data.set_block_height(block_height.to_string());
            }
            println!("Généré {} blocs", block_hashes.len());
        }
        Err(e) => {
            eprintln!("L'adresse n'est pas valide pour le réseau Regtest : {e}");
        }
    };

    // Vérifier le solde dépensable
    let balance = rpc.get_balance(None, None)?;
    if (transaction_data.miner_input_amount.is_empty()) {
        transaction_data.set_miner_input_amount(format!("{}", balance.to_btc()));
    } else {
        println!("Le montant d'entrée du portefeuille Miner est déjà défini.");
    }
    println!("Solde dépensable du Mineur : {balance}");

    Ok(())
}

// pub fn send_20_btc_to(
//     trader_rpc: &Client,
//     miner_rpc: &Client,
//     value: u64,
//     transaction_data: &mut TransactionData,
// ) -> bitcoincore_rpc::Result<Txid> {
//     // Obtenir une adresse du portefeuille Miner
//     //ensure_wallet(rpc, "Miner")?;
//     let trader_address = trader_rpc.get_new_address(None, None)?;
//     //Verifier l'adresse
//     println!("Adresse du portefeuille Trader : {trader_address:?}");
//     transaction_data.set_trader_output_address(trader_address.assume_checked_ref().to_string());
//     //charger l'adresse du portefeuille Miner
//     // rpc.load_wallet("Miner")?;
//     // Envoyer 20 BTC du portefeuille Miner au portefeuille Trader
//     // let change_transaction =
//     //     miner_rpc.get_raw_change_address(Some(bitcoincore_rpc::json::AddressType::Legacy))?;
//     // println!("Transaction details: {:?}", change_transaction);
//     // transaction_data.set_miner_change_address(change_transaction.assume_checked_ref().to_string());
//     let txid = miner_rpc.send_to_address(
//         &trader_address.assume_checked(),
//         Amount::from_btc(20.0)?,
//         None,
//         None,
//         None,
//         None,
//         None,
//         None,
//     )?;
//     println!("Transaction envoyée avec succès, ID de transaction : {txid}");
//     // Mettre à jour les données de la transaction
//     transaction_data.set_transaction_id(txid.to_string());
//     transaction_data.set_trader_output_amount(format!("{value}"));
//     // checker les frais de transaction
//     let fee = miner_rpc.get_mempool_entry(&txid)?.fees;
//     transaction_data.set_transaction_fees(format!("-{:e}", fee.base.to_btc()));
//     // transaction_data.set_transaction_fee(fee.to_string());
//     println!("Frais de transaction : {}", fee.base.to_btc());
//     // Vérifier le solde dépensable
//     let transaction_info_raw = miner_rpc.get_raw_transaction(&txid, None)?;
//     println!("=============================================");

//     let mut change_amount = Amount::ZERO;

//     println!("Utilisation de la méthode de détection par montant...");
//     for output in &transaction_info_raw.output {
//         // Le montant exact de 20 BTC va au trader
//         if output.value == Amount::from_btc(20.0)? {
//             // trader_received_amount = output.value;
//             println!("Trader a reçu exactement : {} BTC", output.value.to_btc());
//         } else {
//             // L'autre output est le change (avec les frais déduits)
//             change_amount = output.value;
//             transaction_data.set_miner_change_amount(format!("{}", output.value.to_btc()));
//             println!("Change du Miner : {} BTC", output.value.to_btc());
//         }
//     }

//     println!("=============================================");

//     //    // let amount = output.value;
//     //     println!("Transaction Amount: {}", amount.to_btc());
//     //     transaction_data.set_miner_change_amount(format!("{}", amount.to_btc()));
//     Ok(txid)
// }



pub fn send_20_btc_to(
    trader_rpc: &Client,
    miner_rpc: &Client,
    value: u64,
    transaction_data: &mut TransactionData,
) -> bitcoincore_rpc::Result<Txid> {
    // Obtenir une adresse du portefeuille Trader
    let trader_address = trader_rpc.get_new_address(None, None)?;
    println!("Adresse du portefeuille Trader : {:?}", trader_address);
    transaction_data.set_trader_output_address(trader_address.assume_checked_ref().to_string());

    // Obtenir une adresse de change du portefeuille Miner
    let change_address = miner_rpc.get_raw_change_address(Some(bitcoincore_rpc::json::AddressType::Legacy))?;
    println!("Adresse de change du Miner : {:?}", change_address);
    transaction_data.set_miner_change_address(change_address.assume_checked_ref().to_string());

    // Envoyer 20 BTC du portefeuille Miner au portefeuille Trader
    let txid = miner_rpc.send_to_address(
        &trader_address.clone().assume_checked(),
        Amount::from_btc(20.0)?,
        None,
        None,
        None,
        None,
        None,
        None,
    )?;
    
    println!("Transaction envoyée avec succès, ID : {}", txid);
    transaction_data.set_transaction_id(txid.to_string());
    transaction_data.set_trader_output_amount(format!("{}", 20.0));

    // Attendre que la transaction soit dans la mempool pour obtenir les frais
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Utiliser gettransaction comme dans les tests
    let tx_info: GetTransactionResult = miner_rpc.get_transaction(&txid, Some(true))?;
    
    // Stocker les frais - fee est de type Option<SignedAmount>
    if let Some(fee) = tx_info.fee {
        transaction_data.set_transaction_fees(format!("{:e}", fee.to_btc()));
        println!("Frais de transaction : {} BTC", fee.to_btc());
    }

    // Décoder la transaction pour analyser les outputs
    // tx_info.hex est de type Vec<u8>
    let decoded = miner_rpc.decode_raw_transaction(&tx_info.hex, None)?;
    
    println!("=============================================");
    println!("Analyse de la transaction :");
    println!("- Nombre d'inputs: {}", decoded.vin.len());
    println!("- Nombre d'outputs: {}", decoded.vout.len());
    
    let trader_addr_str = trader_address.assume_checked_ref().to_string();
    let miner_change_addr_str = change_address.assume_checked_ref().to_string();
    
    for (index, vout) in decoded.vout.iter().enumerate() {
        // Utiliser l'adresse directement depuis le vout si disponible
        let output_addr_str_opt = if let Some(addr) = &vout.script_pub_key.address {
            Some(addr.assume_checked_ref().to_string())
        } else if !vout.script_pub_key.addresses.is_empty() {
            Some(vout.script_pub_key.addresses[0].assume_checked_ref().to_string())
        } else {
            // Fallback: convertir le script en adresse
            Address::from_script(
                &Script::from_bytes(&vout.script_pub_key.hex), 
                bitcoin::Network::Regtest
            ).ok().map(|addr| addr.to_string())
        };
        
        if let Some(output_addr_str) = output_addr_str_opt {
            println!("Output {} : {} BTC -> {}", index, vout.value.to_btc(), output_addr_str);
            
            // Vérifier si c'est l'adresse du trader
            if output_addr_str == trader_addr_str {
                println!("→ Output pour le Trader confirmé");
            }
            // Vérifier si c'est l'adresse de change du miner
            else if output_addr_str == miner_change_addr_str {
                println!("→ Output de change pour le Miner confirmé : {} BTC", vout.value.to_btc());
                transaction_data.set_miner_change_amount(format!("{}", vout.value.to_btc()));
            }
            // Fallback: vérifier si l'adresse appartient au portefeuille Miner
            else {
                // Créer une adresse à partir de la string pour get_address_info
                if let Ok(check_addr) = output_addr_str.parse::<Address<NetworkUnchecked>>() {
                    let addr = check_addr.assume_checked_ref();
                    match miner_rpc.get_address_info(addr) {
                        Ok(addr_info) if addr_info.is_mine == Some(true) => {
                            println!("→ Output de change alternatif pour le Miner : {} BTC", vout.value.to_btc());
                            transaction_data.set_miner_change_amount(format!("{}", vout.value.to_btc()));
                            // Mettre à jour l'adresse de change réelle utilisée
                            transaction_data.set_miner_change_address(output_addr_str.clone());
                        }
                        _ => {
                            println!("→ Output vers adresse externe ou erreur");
                        }
                    }
                }
            }
        } else {
            println!("Output {} : {} BTC -> (adresse non décodable)", index, vout.value.to_btc());
        }
    }
    
    println!("=============================================");

    Ok(txid)
}





// fn get_change_amont(
//     output_txid: Vec<TxOut>,
// ) -> bitcoincore_rpc::Result<Amount> {
//     for output in output_txid {
//        let script = output.script_pubkey.as_mut_script();
//        let adresse = script.
//         if let Some(addr) = address {
//             if addr.to_string().contains("change") {
//                 return Ok(output.value);
//             }
//         }
//     }
//   Ok(Amount::from_sat(0))
// }

pub fn ensure_wallet(rpc: &Client, name: &str) -> bitcoincore_rpc::Result<()> {
    // First try to unload the wallet if it's loaded
    let _ = rpc.unload_wallet(Some(name));

    // Check if wallet exists by trying to load it
    match rpc.load_wallet(name) {
        Ok(_) => {
            println!("{name} wallet loaded successfully");
            Ok(())
        }
        Err(e) if e.to_string().contains("Wallet file verification failed") => {
            // If wallet is corrupted or locked, try creating a new one
            println!("{name} wallet appears locked or corrupted, creating fresh wallet...");
            rpc.create_wallet(name, None, None, None, None)?;
            Ok(())
        }
        Err(e) if e.to_string().contains("not found") => {
            // Wallet doesn't exist, create it
            println!("{name} wallet not found, creating...");
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
        writeln!(file, "{field_value}")?;
    }

    Ok(())
}
