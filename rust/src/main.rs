use bitcoincore_rpc::{Auth, Client, RpcApi};
use serde::Deserialize;
use serde_json::json;
mod utils; // This declares the 'utils' module
use crate::utils::transaction_data::TransactionData;
use crate::utils::utility::{
    create_client_for_wallet, ensure_wallet, generate_spendable_balance, send_20_btc_to,
    write_transaction_to_file,
};

// Node access params
const RPC_URL: &str = "http://127.0.0.1:18443"; // Default regtest RPC port
const RPC_USER: &str = "alice";
const RPC_PASS: &str = "password";

// Exemple (fourni par le template) d'appel d'un RPC non exposé par la lib via la
// fonction générique `call`, avec désérialisation du résultat via serde.
#[allow(dead_code)]
fn send(rpc: &Client, addr: &str, amount_btc: f64) -> bitcoincore_rpc::Result<String> {
    // Seul argument positionnel : le tableau `outputs` `[{ adresse: montant_en_btc }]`.
    // conf_target / estimate_mode / fee_rate / options gardent leurs valeurs par défaut.
    let args = [json!([{ addr: amount_btc }])];

    #[derive(Deserialize)]
    struct SendResult {
        complete: bool,
        txid: String,
    }
    let send_result = rpc.call::<SendResult>("send", &args)?;
    if !send_result.complete {
        return Err(bitcoincore_rpc::Error::ReturnedError(
            "send did not complete (transaction not fully signed)".to_string(),
        ));
    }
    Ok(send_result.txid)
}

fn main() -> bitcoincore_rpc::Result<()> {
    // Connexion au RPC de Bitcoin Core.
    let rpc = Client::new(
        RPC_URL,
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )?;

    let mut transaction_data = TransactionData::default();

    let blockchain_info = rpc.get_blockchain_info()?;
    println!("Blockchain Info: {blockchain_info:?}");

    // Création/chargement des wallets Miner et Trader.
    ensure_wallet(&rpc, "Miner")?;
    ensure_wallet(&rpc, "Trader")?;

    let miner_client = create_client_for_wallet("Miner")?;
    let trader_client = create_client_for_wallet("Trader")?;

    // Pourquoi 101 blocs ? La récompense de bloc (coinbase) est soumise à la règle
    // de maturité : une sortie coinbase ne devient dépensable qu'après 100
    // confirmations. En minant 101 blocs, le coinbase du bloc 1 obtient ses 100
    // confirmations (blocs 2..=101) et devient donc dépensable : le solde passe de 0
    // à 50 BTC. Les 100 blocs les plus récents restent immatures, d'où ce solde de 50.
    generate_spendable_balance(&miner_client, 101, &mut transaction_data)?;

    // Envoyer 20 BTC du Miner au Trader.
    let tx_id = send_20_btc_to(&trader_client, &miner_client, 20, &mut transaction_data)?;

    let balance = trader_client.get_balance(Some(0), None)?;
    println!("Solde dépensable du Trader avant confirmation : {balance}");

    // Récupérer la transaction non confirmée depuis la mempool.
    let mempool = rpc.get_raw_mempool()?;
    println!("Mempool transactions: {mempool:?}");

    // Confirmer la transaction en minant 1 bloc.
    generate_spendable_balance(&miner_client, 1, &mut transaction_data)?;

    let balance_after = trader_client.get_balance(Some(0), None)?;
    println!("Solde du Trader après confirmation : {balance_after}");
    println!("Transaction ID: {tx_id}");

    // Écrire les données dans ../out.txt au format demandé dans le README.
    write_transaction_to_file(&transaction_data)?;
    println!("Données de la transaction écrites dans ../out.txt");

    Ok(())
}
