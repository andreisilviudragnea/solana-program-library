use solana_sdk::signature::{Keypair, Signer};
use spl_example_cross_program_invocation::entrypoint;
use {
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    },
    solana_program_test::*,
    solana_sdk::{account::Account, transaction::Transaction},
    std::str::FromStr,
};

#[tokio::test]
async fn test_cross_program_invocation() {
    // Initialize the program test environment
    let program_id = Pubkey::from_str("invoker111111111111111111111111111111111111").unwrap();
    let mut program_test = ProgramTest::new(
        "spl_example_cross_program_invocation",
        program_id,
        processor!(entrypoint::process_instruction),
    );

    // Add the account with 10 MiB of data
    let input_account_key = Keypair::new();
    let input_account_size = 10 * 1024 * 1024; // 10 MiB

    program_test.add_account(
        input_account_key.pubkey(),
        Account {
            lamports: solana_sdk::rent::Rent::default().minimum_balance(input_account_size),
            data: vec![0; input_account_size],
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    // Start the test environment
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let mut transaction_1 = Transaction::new_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &[0],
            vec![AccountMeta::new(input_account_key.pubkey(), false)],
        )],
        Some(&payer.pubkey()),
    );

    transaction_1.sign(&[&payer], recent_blockhash);

    banks_client
        .process_transaction(transaction_1)
        .await
        .unwrap();

    let mut transaction_2 = Transaction::new_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &[124],
            vec![AccountMeta::new(input_account_key.pubkey(), false)],
        )],
        Some(&payer.pubkey()),
    );

    transaction_2.sign(&[&payer], recent_blockhash);

    banks_client
        .process_transaction(transaction_2)
        .await
        .unwrap();

    let mut transaction_3 = Transaction::new_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &[1],
            vec![AccountMeta::new(input_account_key.pubkey(), false)],
        )],
        Some(&payer.pubkey()),
    );

    transaction_3.sign(&[&payer], recent_blockhash);

    banks_client
        .process_transaction(transaction_3)
        .await
        .unwrap();

    // Add assertions to verify the state of the account after the transactions
    let account = banks_client
        .get_account(input_account_key.pubkey())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(account.data.len(), input_account_size);
    // Add more assertions based on the expected changes after transactions
}

// #[tokio::test]
#[allow(dead_code)]
async fn test_two_identical_transactions_no_error() {
    // Initialize the program test environment
    let program_id = Pubkey::from_str("invoker111111111111111111111111111111111111").unwrap();
    let mut program_test = ProgramTest::new(
        "spl_example_cross_program_invocation",
        program_id,
        processor!(entrypoint::process_instruction),
    );

    // Add the account with 10 MiB of data
    let input_account_key = Keypair::new();
    let input_account_size = 10 * 1024 * 1024; // 10 MiB

    program_test.add_account(
        input_account_key.pubkey(),
        Account {
            lamports: solana_sdk::rent::Rent::default().minimum_balance(input_account_size),
            data: vec![0; input_account_size],
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    // Start the test environment
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let mut transaction_1 = Transaction::new_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &[111],
            vec![AccountMeta::new(input_account_key.pubkey(), false)],
        )],
        Some(&payer.pubkey()),
    );

    transaction_1.sign(&[&payer], recent_blockhash);

    banks_client
        .process_transaction(transaction_1)
        .await
        .unwrap();

    let mut transaction_2 = Transaction::new_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &[111],
            vec![AccountMeta::new(input_account_key.pubkey(), false)],
        )],
        Some(&payer.pubkey()),
    );

    transaction_2.sign(&[&payer], recent_blockhash);

    banks_client
        .process_transaction(transaction_2)
        .await
        .unwrap();

    // Add assertions to verify the state of the account after the transactions
    let account = banks_client
        .get_account(input_account_key.pubkey())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(account.data.len(), input_account_size);
    // Add more assertions based on the expected changes after transactions
}
