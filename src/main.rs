
extern crate elementsd;

use std::str::FromStr;
use core::fmt::Write;
use elementsd::{bitcoind, ElementsD};
use elementsd::bitcoincore_rpc::jsonrpc::serde_json;
use elementsd::bitcoincore_rpc::jsonrpc::serde_json::{json};
use elementsd::bitcoincore_rpc::RpcApi;

fn main() {

    // setup bitcoin
    let bitcoind_exe = bitcoind::exe_path().unwrap();
    let bitcoind_conf = bitcoind::Conf::default();
    
    let bitcoind = bitcoind::BitcoinD::with_conf(&bitcoind_exe, &bitcoind_conf).unwrap();

    // setup elements
    let elementsd_conf = elementsd::Conf::new(Some(&bitcoind));
    let elementsd = ElementsD::with_conf(elementsd::exe_path().unwrap(), &elementsd_conf).unwrap();

    // run bitcoin
    let info = bitcoind.client.get_blockchain_info().unwrap();
    assert_eq!(0, info.blocks);

    // Get RT-BTC to peg-in (Bitcoin Core)
    let address = bitcoind.client.get_new_address(None, None).unwrap();
    let _ = bitcoind.client.generate_to_address(101, &address).unwrap();
    let info = bitcoind.client.get_blockchain_info().unwrap();
    assert_eq!(101, info.blocks);

    // Verify we have 50 RT-BTC to spend (Bitcoin Core)
    let balance = bitcoind.client.get_balance(None, None).unwrap();
    println!("balance: {}", balance);
    assert_eq!(50_f64, balance.as_btc());
    
    // Generate a peg-in address (Elements Core)
    let pegin_address = elementsd
        .client()
        .call::<serde_json::Value>("getpeginaddress", &[])
        .unwrap();
    println!("pegin_address: {}", pegin_address);

    // Send 1 RT-BTC to our peg-in address (Bitcoin Core)
    let mainchain_address = pegin_address.get("mainchain_address").unwrap();
    let addr = bitcoin::Address::from_str(mainchain_address.as_str().unwrap()).unwrap();
    let amount = bitcoin::Amount::from_btc(1.0).unwrap();
    let tx = bitcoind.client.send_to_address(
        &addr,
        amount,
        None,
        None,
        None,
        None,
        None,
        None,
    ).unwrap();
    println!("tx: {}", tx);

    // Give our peg-in transaction sufficient confirmations (Bitcoin Core)
    let blockhashes = bitcoind.client.generate_to_address(10, &address).unwrap();

    // Prove our peg-in transaction was included in a block (Bitcoin Core)
    let tx_proof_bytes = bitcoind.client.get_tx_out_proof(&[tx], None).unwrap();
    let mut tx_proof = String::with_capacity(2 * tx_proof_bytes.len());
    for byte in tx_proof_bytes {
        _ = write!(tx_proof, "{:02X}", byte);
    }
    println!("tx_proof: {}", tx_proof);

    // Get the peg-in's raw transaction hex (Bitcoin Core)
    let raw_tx = bitcoind.client.get_raw_transaction_hex(&tx, Some(&blockhashes[0])).unwrap();
    println!("raw_tx: {}", raw_tx);
    
    // Generate a block including our peg-in transaction (Elements Core)
    let element_address = elementsd
        .client()
        .call::<serde_json::Value>("getnewaddress", &[])
        .unwrap();
    println!("element_address: {}", element_address);

    let generate_block = elementsd
        .client()
        .call::<serde_json::Value>("generatetoaddress", &[json!(1), json!(element_address)])
        .unwrap();
    println!("generate_block: {}", generate_block);

    // Claim peg-in funds on the sidechain (Elements Core)
    let claim_tx_id = elementsd
        .client()
        .call::<serde_json::Value>("claimpegin", &[json!(raw_tx), json!(tx_proof)])
        .unwrap();
    println!("claim_tx_id: {}", claim_tx_id);
    
    // View peg-in transaction details (Elements Core)
    let claim_tx = elementsd
        .client()
        .call::<serde_json::Value>("getrawtransaction", &[json!(claim_tx_id), json!(1)])
        .unwrap();
    println!("claim_tx: {}", claim_tx);

    // See peg-in transaction in the mempool (Elements Core)
    let mempool = elementsd
    .client()
    .call::<serde_json::Value>("getrawmempool", &[])
    .unwrap();
    println!("mempool: {}", mempool);

    // Check your pre-peg-in LRT-BTC (Elements Core)
    let balances = elementsd
        .client()
        .call::<serde_json::Value>("getbalances", &[])
        .unwrap();
    println!("balances: {}", balances);

    // Generate a block including our peg-in transaction (Elements Core)
    let generate_block = elementsd
        .client()
        .call::<serde_json::Value>("generatetoaddress", &[json!(1), json!(element_address)])
        .unwrap();
    println!("generate_block: {}", generate_block);

    // Check your post-peg-in LRT-BTC (Elements Core)
    let balances = elementsd
        .client()
        .call::<serde_json::Value>("getbalances", &[])
        .unwrap();
    println!("balances: {}", balances);
    
}
