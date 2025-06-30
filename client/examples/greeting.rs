use client::{
    initialize_client, interact_with_program_instructions, setup_account, setup_payer,
    setup_program, ClientError, Config,
};
use greeting::{instruction::GreetingInstruction, state::GreetingAccount};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signer::Signer,
};
use std::{mem::size_of, path::Path};
use utils::AccountCast;

/// Main entry point for the Solana program client
#[tokio::main]
#[allow(clippy::result_large_err)]
async fn main() -> client::Result<()> {
    // Parse command-line arguments
    let config = Config::parse_args();

    // Initialize the Solana client
    let client = initialize_client(&config).await?;

    // Setup the payer account
    let payer = setup_payer(&client, &config).await?;

    // Define program path
    let program_path = Path::new("target/deploy/greeting.so");

    // Deploy or use existing program
    let program_id = setup_program(&client, &payer, &config, program_path).await?;

    // Setup greeting account
    let space = size_of::<GreetingAccount>();
    println!("Greeting account space: {space}");
    let greeting_account = setup_account(
        &client,
        &payer,
        &program_id,
        &config,
        space,
        "greeting-account",
    )
    .await?;

    let instructions = vec![Instruction::new_with_borsh(
        program_id,
        &GreetingInstruction::IncrementCounter,
        vec![AccountMeta::new(greeting_account.pubkey(), false)],
    )];

    // Interact with the program using the instructions directly
    interact_with_program_instructions(
        &client,
        &payer,
        &program_id,
        &greeting_account,
        &instructions,
    )
    .await?;

    println!("Greeting program interaction completed successfully!");
    let mut account_data = client
        .get_account_data(&greeting_account.pubkey())
        .await
        .map_err(ClientError::SolanaClientError)?;
    let greeting_account = GreetingAccount::cast_mut(&mut account_data);
    println!("Greeting counter: {}", greeting_account.counter);
    println!(
        "Greeting double counter: {}",
        greeting_account.double_counter
    );

    Ok(())
}
