use bitcoin::TxIn;
use bitcoincore_rpc::{
    bitcoin::{Amount, Network, Txid},
    Auth, Client, RpcApi,
};
use serde::{de::value, Deserialize};
use serde_json::json;
use std::fs::File;

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
    wallet_rpc: &Client,
    value: u64,
    wallet_name: &str,
) -> bitcoincore_rpc::Result<()> {
    // Obtenir une adresse du portefeuille Miner
    println!("<==========================================>");
    //rpc.load_wallet(wallet_name)?;
    println!("<==========================================>");

    let miner_address = wallet_rpc.get_new_address(None, None)?;
    //Verifier l'adresse
    println!("Adresse du portefeuille Miner : {:?}", miner_address);
    match miner_address.require_network(Network::Regtest) {
        Ok(adresse) => {
            // Générer 101 blocs pour avoir des fonds dépensables
            let block_hashes = rpc.generate_to_address(value, &adresse)?;

            println!("Généré {} blocs", block_hashes.len());
        }
        Err(e) => {
            eprintln!("L'adresse n'est pas valide pour le réseau Regtest : {}", e);
        }
    };

    // Vérifier le solde dépensable
    let balance = wallet_rpc.get_balance(None, None)?;
    println!("Solde dépensable : {}", balance);

    Ok(())
}

pub fn send_20_btc_to(
    rpc: &Client,
    wallet_rpc_miners: &Client,
    wallet_rpc_trader: &Client,
    value: u64,
) -> bitcoincore_rpc::Result<Txid> {
    // Obtenir une adresse du portefeuille Miner
    println!("<==========================================>");
    //rpc.load_wallet("Trader")?;
    println!("<==========================================>");

    let trader_address = wallet_rpc_trader.get_new_address(None, None)?;
    //Verifier l'adresse
    println!("Adresse du portefeuille Trader : {:?}", trader_address);
    // Envoyer 20 BTC du portefeuille Miner au portefeuille Trader
    let txid = wallet_rpc_miners.send_to_address(
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

    // Vérifier le solde dépensable
    let balance = wallet_rpc_trader.get_balance(None, None)?;
    println!("Solde dépensable traders : {}", balance);
    let balance = wallet_rpc_miners.get_balance(None, None)?;
    println!("Solde dépensable mineurs : {}", balance);

    Ok(txid)
}
