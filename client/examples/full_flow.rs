use std::{
    path::Path,
    time::{Duration, Instant},
};

use client::{initialize_client, setup_payer, setup_program, ClientError, Config};
use futures::future::join_all;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    instruction::{AccountMeta, Instruction},
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};
use solana_system_interface::instruction::create_account;
use stark::{
    felt::Felt,
    stark_proof::VerifyPublicInput,
    swiftness::stark::types::{cast_struct_to_slice, StarkProof},
};
use swiftness_proof_parser::{json_parser, transform::TransformTo, StarkProof as StarkProofParser};
use tokio::time::sleep;
use utils::AccountCast;
use utils::BidirectionalStack;
use utils::Executable;
use verifier::{instruction::VerifierInstruction, state::BidirectionalStackAccount};

pub const CHUNK_SIZE: usize = 1000;

pub struct Input {
    pub front_index: u32,
    pub back_index: u32,
    pub proof: StarkProof,
}
#[tokio::main]
#[allow(clippy::result_large_err)]
async fn main() -> client::Result<()> {
    let config = Config::parse_args();
    let client = initialize_client(&config).await?;

    let payer = setup_payer(&client, &config).await?;

    let program_path = Path::new("target/deploy/verifier.so");

    let program_id = setup_program(&client, &payer, &config, program_path).await?;

    println!("Using program ID: {program_id}");

    let stack_account = Keypair::new();

    println!("Creating new account: {}", stack_account.pubkey());

    let space = size_of::<BidirectionalStackAccount>();
    println!("Account space: {space} bytes");

    let create_account_ix = create_account(
        &payer.pubkey(),
        &stack_account.pubkey(),
        client.get_minimum_balance_for_rent_exemption(space).await?,
        space as u64,
        &program_id,
    );

    let create_account_tx = Transaction::new_signed_with_payer(
        &[create_account_ix],
        Some(&payer.pubkey()),
        &[&payer, &stack_account],
        client.get_latest_blockhash().await?,
    );

    let signature = client
        .send_and_confirm_transaction(&create_account_tx)
        .await?;

    let mut input: [u64; 2] = [0, 65536];
    let proof_bytes = cast_struct_to_slice(&mut input);
    let new_offset = proof_bytes.len();
    println!("Proof bytes in kb: {:?}", proof_bytes.len() / 1024);
    let instructions = proof_bytes
        .chunks(CHUNK_SIZE)
        .enumerate()
        .map(|(i, chunk)| {
            Instruction::new_with_borsh(
                program_id,
                &VerifierInstruction::SetAccountData(i * CHUNK_SIZE, chunk.to_vec()),
                vec![AccountMeta::new(stack_account.pubkey(), false)],
            )
        })
        .collect::<Vec<_>>();

    println!("Instructions number: {:?}", instructions.len());
    for (i, instruction) in instructions.iter().enumerate() {
        let set_proof_tx = Transaction::new_signed_with_payer(
            &[instruction.clone()],
            Some(&payer.pubkey()),
            &[&payer],
            client.get_latest_blockhash().await?,
        );
        let set_proof_signature: solana_sdk::signature::Signature =
            client.send_and_confirm_transaction(&set_proof_tx).await?;
        println!("Set proof: {i}: {set_proof_signature}");
    }

    println!("Account created successfully: {signature}");
    println!("\nSet Proof on Solana");
    println!("====================");
    let input = include_str!("../../example_proof/saya.json");
    let proof_json = serde_json::from_str::<json_parser::StarkProof>(input).unwrap();
    let proof = StarkProofParser::try_from(proof_json).unwrap();
    let mut proof_verifier = proof.transform_to();

    let proof_bytes = cast_struct_to_slice(&mut proof_verifier);
    println!("Proof bytes in kb: {:?}", proof_bytes.len() / 1024);
    let instructions = proof_bytes
        .chunks(CHUNK_SIZE)
        .enumerate()
        .map(|(i, chunk)| {
            Instruction::new_with_borsh(
                program_id,
                &VerifierInstruction::SetAccountData(new_offset + (i * CHUNK_SIZE), chunk.to_vec()),
                vec![AccountMeta::new(stack_account.pubkey(), false)],
            )
        })
        .collect::<Vec<_>>();

    println!("Instructions number: {instructions:?}");
    let mut transactions = Vec::new();
    for (i, instruction) in instructions.iter().enumerate() {
        let set_proof_tx = Transaction::new_signed_with_payer(
            &[instruction.clone()],
            Some(&payer.pubkey()),
            &[&payer],
            client.get_latest_blockhash().await?,
        );
        let set_proof_signature: solana_sdk::signature::Signature =
            client.send_transaction(&set_proof_tx).await?;
        println!("Set proof: {i}: {set_proof_signature}");
        transactions.push(set_proof_signature);
    }

    wait_for_all_confirmations(
        &client,
        &transactions,
        Duration::from_millis(10), // poll every 10ms
        Duration::from_secs(120),  // timeout after 2 minutes
    )
    .await;

    let task = VerifyPublicInput::new();

    let verify_public_input_ix = Instruction::new_with_borsh(
        program_id,
        &VerifierInstruction::PushTask(task.to_vec_with_type_tag()),
        vec![AccountMeta::new(stack_account.pubkey(), false)],
    );

    let verify_public_input_tx = Transaction::new_signed_with_payer(
        &[verify_public_input_ix],
        Some(&payer.pubkey()),
        &[&payer],
        client.get_latest_blockhash().await?,
    );
    let verify_public_input_signature: solana_sdk::signature::Signature = client
        .send_and_confirm_transaction(&verify_public_input_tx)
        .await?;

    println!("Verify public input: {verify_public_input_signature:?}");

    let limit_instructions = ComputeBudgetInstruction::set_compute_unit_limit(800_000);

    let mut account_data = client
        .get_account_data(&stack_account.pubkey())
        .await
        .map_err(ClientError::SolanaClientError)?;
    let stack = BidirectionalStackAccount::cast_mut(&mut account_data);
    let simulation_steps = stack.simulate();

    println!("Simulation steps: {simulation_steps}");

    let mut transactions = Vec::new();
    for i in 0..simulation_steps {
        // Execute the task
        let execute_ix = Instruction::new_with_borsh(
            program_id,
            &VerifierInstruction::Execute(i as u32),
            vec![AccountMeta::new(stack_account.pubkey(), false)],
        );

        let execute_tx = Transaction::new_signed_with_payer(
            &[limit_instructions.clone(), execute_ix],
            Some(&payer.pubkey()),
            &[&payer],
            client.get_latest_blockhash().await?,
        );
        let execute_signature = client.send_transaction(&execute_tx).await?;
        transactions.push(execute_signature);
        println!("execute signature: {execute_signature:?}");
        println!("i: {i}");
    }

    let results = wait_for_all_confirmations(
        &client,
        &transactions,
        Duration::from_millis(10), // poll every 50ms
        Duration::from_secs(120),  // timeout after 2 minutes
    )
    .await;

    for (transaction, confirmed) in results {
        if confirmed {
            println!("Transaction confirmed: {transaction:?}");
        } else {
            println!("Transaction NOT confirmed (timeout): {transaction:?}");
        }
    }

    // Read and display the result
    let mut account_data = client
        .get_account_data(&stack_account.pubkey())
        .await
        .map_err(ClientError::SolanaClientError)?;
    let stack = BidirectionalStackAccount::cast_mut(&mut account_data);
    let result_program_hash = Felt::from_bytes_be_slice(stack.borrow_front());
    stack.pop_front();
    let result_output_hash = Felt::from_bytes_be_slice(stack.borrow_front());
    stack.pop_front();

    println!("\nProgram Hash: {result_program_hash:?}");
    println!("Output Hash: {result_output_hash:?}");
    println!("Stack front index: {}", stack.front_index);
    println!("Stack back index: {}", stack.back_index);
    println!("\nHash Public Inputs successfully executed on Solana!");

    Ok(())
}

async fn wait_for_all_confirmations(
    client: &RpcClient,
    transactions: &[solana_sdk::signature::Signature],
    poll_interval: Duration,
    timeout: Duration,
) -> Vec<(solana_sdk::signature::Signature, bool)> {
    let start = Instant::now();
    let mut pending: Vec<_> = transactions.to_vec();
    let mut results = Vec::new();

    while !pending.is_empty() && start.elapsed() < timeout {
        let futures = pending.iter().map(|tx| {
            let client = client;
            async move {
                let confirmed = client.confirm_transaction(tx).await.unwrap_or(false);
                (tx.clone(), confirmed)
            }
        });
        let statuses = join_all(futures).await;

        // Retain unconfirmed, collect confirmed
        pending = statuses
            .iter()
            .filter_map(|(tx, confirmed)| if !confirmed { Some(tx.clone()) } else { None })
            .collect();
        results.extend(statuses.into_iter().filter(|(_, c)| *c));

        if !pending.is_empty() {
            sleep(poll_interval).await;
        }
    }

    // Mark any still-pending as not confirmed
    results.extend(pending.into_iter().map(|tx| (tx, false)));
    results
}
