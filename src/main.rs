use std::{env, process, str::FromStr};

use miniscript::bitcoin::{
    self, Address, Network, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Witness,
    absolute,
    address::NetworkUnchecked,
    hex::DisplayHex,
    opcodes,
    script::{self},
    transaction::Version,
};

const CSV2_FLAG: u32 = 1 << 21;
const NETWORK: bitcoin::Network = Network::Regtest;

fn main() {
    let args = env::args().collect::<Vec<_>>();

    if args.len() > 1 && (args[1] == "-help" || args[1] == "-h") {
        help();
        process::exit(0);
    }
    let args = env::args().collect::<Vec<_>>();
    match command(args) {
        Command::Address { timelock } => {
            let addr = generate_p2wsh_address(timelock, NETWORK);
            println!("{addr}");
        }
        Command::Spend {
            outpoint,
            timelock,
            spend_amount,
            address,
        } => {
            if !address.is_valid_for_network(NETWORK) {
                let msg = format!("Address not valid for network {}", NETWORK);
                exit_with_message(&msg);
            }
            let address = address.assume_checked();
            let tx = spend(outpoint, timelock, spend_amount, address);
            let str_tx =
                bitcoin::consensus::serialize(&tx).to_hex_string(bitcoin::hex::Case::Lower);
            println!("{str_tx}");
        }
    }
}

pub enum Command {
    Address {
        timelock: u16,
    },
    Spend {
        outpoint: OutPoint,
        timelock: u16,
        spend_amount: bitcoin::Amount,
        address: Address<NetworkUnchecked>,
    },
}

fn exit() -> ! {
    help();
    process::exit(1);
}

fn exit_with_message(msg: &str) -> ! {
    eprintln!("{msg}");
    process::exit(1);
}

fn help() {
    let bin = env::args().next().expect("binary name");
    println!(
        r#"Usage:
    - {bin} address <timelock>  Generate a P2WSH address with the specified timelock
    - {bin} spend <outpoint> <sat_amount_to_spend> <address>  Spend from the specified outpoint
    - {bin} -help or {bin} -h  Show this help message

This binary is a helper to build & spend "anyone can spend after timelock" coins.
It's not intended to be used in real-world conditions but as a helper for experimenting
with the possibility to extend Bitcoin's relative locktime by adding a new flag to the nSequence field.
"#
    );
}

fn command(mut args: Vec<String>) -> Command {
    args.remove(0);
    match args.len() {
        2 => parse_address_command(args),
        5 => parse_spend_command(args),
        _ => exit(),
    }
}

fn parse_address_command(args: Vec<String>) -> Command {
    if &args[0] != "address" {
        exit()
    }
    match args[1].parse::<u16>() {
        Ok(timelock) => Command::Address { timelock },
        _ => {
            let msg = format!("Invalid timelock value {}", &args[1]);
            exit_with_message(&msg);
        }
    }
}

fn parse_spend_command(args: Vec<String>) -> Command {
    if &args[0] != "spend" {
        exit()
    }
    let outpoint = match OutPoint::from_str(&args[1]) {
        Ok(op) => op,
        Err(_) => {
            let msg = format!("Invalid outpoint {}", &args[1]);
            exit_with_message(&msg);
        }
    };
    let timelock = match args[2].parse::<u16>() {
        Ok(timelock) => timelock,
        _ => {
            let msg = format!("Invalid timelock value {}", &args[2]);
            exit_with_message(&msg);
        }
    };
    let spend_amount = match args[3].parse::<u32>() {
        Ok(sats) => bitcoin::Amount::from_sat(sats.into()),
        _ => {
            let msg = format!("Invalid amount value {}", &args[3]);
            exit_with_message(&msg);
        }
    };
    let address = match Address::<NetworkUnchecked>::from_str(&args[4]) {
        Ok(addr) => addr,
        Err(_) => {
            let msg = format!("Invalid address {}", &args[4]);
            exit_with_message(&msg);
        }
    };

    Command::Spend {
        outpoint,
        timelock,
        spend_amount,
        address,
    }
}

fn csv2_sequence(timelock: u16) -> Sequence {
    Sequence::from_consensus(timelock as u32 | CSV2_FLAG)
}

fn csv2_script(sequence: Sequence) -> script::Builder {
    script::Builder::new()
        .push_sequence(sequence)
        .push_opcode(opcodes::all::OP_CSV)
}

/// generate an p2wsh address where the redeem script is OP_CHECKSEQUENCEVERIFY <timelock>
fn generate_p2wsh_address(timelock: u16, network: Network) -> bitcoin::Address {
    let script = csv2_script(csv2_sequence(timelock));
    Address::p2wsh(script.as_script(), network)
}

fn spend(
    outpoint: OutPoint,
    timelock: u16,
    amount: bitcoin::Amount,
    address: bitcoin::Address,
) -> Transaction {
    let script = csv2_script(csv2_sequence(timelock));
    let mut witness = Witness::new();
    witness.push(script.as_bytes());
    let input = TxIn {
        previous_output: outpoint,
        script_sig: ScriptBuf::new(),
        sequence: csv2_sequence(timelock),
        witness,
    };

    bitcoin::Transaction {
        version: Version(2),
        lock_time: absolute::LockTime::ZERO,
        input: vec![input],
        output: vec![TxOut {
            value: amount,
            script_pubkey: address.script_pubkey(),
        }],
    }
}
