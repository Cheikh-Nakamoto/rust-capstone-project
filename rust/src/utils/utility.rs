use bitcoin::{address::NetworkUnchecked, Address, Script};
use bitcoincore_rpc::{
    bitcoin::{Amount, Network, Txid},
    json::GetTransactionResult,
    Auth, Client, RpcApi,
};
use std::fs::OpenOptions;
use std::io::Write;

use crate::utils::transaction_data::TransactionData;

pub fn create_client_for_wallet(wallet_name: &str) -> bitcoincore_rpc::Result<Client> {
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
    // Le README impose le label "Mining Reward" pour l'adresse de minage.
    let miner_address = rpc.get_new_address(Some("Mining Reward"), None)?;

    // La première adresse générée sert d'entrée ; les suivantes servent de change.
    if transaction_data.miner_input_address.is_empty() {
        transaction_data.set_miner_input_address(miner_address.assume_checked_ref().to_string());
    } else {
        transaction_data.set_miner_change_address(miner_address.assume_checked_ref().to_string());
    }

    let adresse = miner_address.assume_checked();
    let block_hashes = rpc.generate_to_address(value, &adresse)?;
    // Quand on ne mine qu'un seul bloc, c'est le bloc de confirmation de la transaction :
    // on enregistre sa hauteur et son hash.
    if block_hashes.len() == 1 {
        transaction_data.set_block_hash(block_hashes[0].to_string());
        let block_height = rpc.get_block_count()?;
        transaction_data.set_block_height(block_height.to_string());
    }

    let balance = rpc.get_balance(None, None)?;
    if transaction_data.miner_input_amount.is_empty() {
        transaction_data.set_miner_input_amount(format!("{}", balance.to_btc()));
    }
    println!("Solde dépensable du Mineur : {balance}");

    Ok(())
}

pub fn send_20_btc_to(
    trader_rpc: &Client,
    miner_rpc: &Client,
    value: u64,
    transaction_data: &mut TransactionData,
) -> bitcoincore_rpc::Result<Txid> {
    // Adresse de réception du Trader (label "Received" requis par le README).
    let trader_address = trader_rpc.get_new_address(Some("Received"), None)?;
    transaction_data.set_trader_output_address(trader_address.assume_checked_ref().to_string());

    // Adresse de change du Miner (mise à jour plus bas avec celle réellement utilisée).
    let change_address =
        miner_rpc.get_raw_change_address(Some(bitcoincore_rpc::json::AddressType::Legacy))?;
    transaction_data.set_miner_change_address(change_address.assume_checked_ref().to_string());

    // Envoyer `value` BTC du Miner au Trader.
    let txid = miner_rpc.send_to_address(
        &trader_address.clone().assume_checked(),
        Amount::from_btc(value as f64)?,
        None,
        None,
        None,
        None,
        None,
        None,
    )?;
    transaction_data.set_transaction_id(txid.to_string());
    transaction_data.set_trader_output_amount(format!("{value}"));

    // Laisser la transaction entrer dans la mempool avant de lire les frais.
    std::thread::sleep(std::time::Duration::from_millis(100));

    let tx_info: GetTransactionResult = miner_rpc.get_transaction(&txid, Some(true))?;
    if let Some(fee) = tx_info.fee {
        transaction_data.set_transaction_fees(format!("{:e}", fee.to_btc()));
    }

    // Décoder la transaction pour identifier l'output de change du Miner.
    let decoded = miner_rpc.decode_raw_transaction(&tx_info.hex, None)?;
    let trader_addr_str = trader_address.assume_checked_ref().to_string();
    let miner_change_addr_str = change_address.assume_checked_ref().to_string();

    for vout in &decoded.vout {
        let output_addr_str_opt = if let Some(addr) = &vout.script_pub_key.address {
            Some(addr.assume_checked_ref().to_string())
        } else if !vout.script_pub_key.addresses.is_empty() {
            Some(
                vout.script_pub_key.addresses[0]
                    .assume_checked_ref()
                    .to_string(),
            )
        } else {
            Address::from_script(
                Script::from_bytes(&vout.script_pub_key.hex),
                Network::Regtest,
            )
            .ok()
            .map(|addr| addr.to_string())
        };

        let Some(output_addr_str) = output_addr_str_opt else {
            continue;
        };

        // L'output du Trader est connu, on ne traite que le change du Miner.
        if output_addr_str == trader_addr_str {
            continue;
        }

        if output_addr_str == miner_change_addr_str {
            transaction_data.set_miner_change_amount(format!("{}", vout.value.to_btc()));
        } else if let Ok(check_addr) = output_addr_str.parse::<Address<NetworkUnchecked>>() {
            // `send_to_address` génère sa propre adresse de change : on détecte
            // l'output qui appartient au Miner et on enregistre l'adresse réelle.
            let addr = check_addr.assume_checked_ref();
            if let Ok(addr_info) = miner_rpc.get_address_info(addr) {
                if addr_info.is_mine == Some(true) {
                    transaction_data.set_miner_change_amount(format!("{}", vout.value.to_btc()));
                    transaction_data.set_miner_change_address(output_addr_str);
                }
            }
        }
    }

    Ok(txid)
}

pub fn ensure_wallet(rpc: &Client, name: &str) -> bitcoincore_rpc::Result<()> {
    // Décharger d'abord le wallet s'il est déjà chargé.
    let _ = rpc.unload_wallet(Some(name));

    match rpc.load_wallet(name) {
        Ok(_) => Ok(()),
        // Wallet corrompu/verrouillé ou inexistant : on en crée un neuf.
        Err(e)
            if e.to_string().contains("Wallet file verification failed")
                || e.to_string().contains("not found") =>
        {
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

    for field_value in data.to_ordered_vec() {
        writeln!(file, "{field_value}")?;
    }

    Ok(())
}
