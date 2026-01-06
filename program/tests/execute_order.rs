use light_hasher::{Hasher, Keccak};
use sol_ver::state::order::Order;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

// Helper to run the test properly with setup
#[tokio::test]
async fn test_execute_order_e2e() {
    let program_id = Pubkey::new_from_array(sol_ver::ID);
    // Use the adapter to bridge Solana SDK types (Test) -> Pinocchio types (Program)
    let mut program_test = ProgramTest::new("sol_ver", program_id, processor!(sol_ver_adapter));
    program_test.prefer_bpf(false);

    let mock_market_program_id = solana_sdk::pubkey!("MockMarket111111111111111111111111111111111");
    // Mock Market Processor that simulates a successful trade by increasing balance
    program_test.add_program(
        "mock_market",
        mock_market_program_id,
        processor!(mock_market_processor),
    );

    let from_token_account = Pubkey::new_unique();
    let to_token_account = Pubkey::new_unique();

    let order_struct = Order {
        from_token_account: pinocchio::pubkey::Pubkey::from(from_token_account.to_bytes()),
        to_token_account: pinocchio::pubkey::Pubkey::from(to_token_account.to_bytes()),
        sell_amount: 100,
        buy_amount: 100,
        referral_fee: 0,
        referral_token_account: pinocchio::pubkey::Pubkey::default(),
        minimun_buy_amount: 90,
        _padding: [0; 6],
        start_time: 0,
        deadline: 1000,
    };

    let order_bytes = unsafe {
        core::slice::from_raw_parts(
            &order_struct as *const Order as *const u8,
            core::mem::size_of::<Order>(),
        )
    };
    let intent_hash = Keccak::hashv(&[&order_bytes]).unwrap();
    let owner = Keypair::new();

    let (order_pda, bump) = Pubkey::find_program_address(
        &[b"order", owner.pubkey().as_ref(), intent_hash.as_ref()],
        &program_id,
    );

    // Add Order Account
    let account = Account {
        lamports: 1_000_000,
        data: order_bytes.to_vec(),
        owner: program_id,
        executable: false,
        rent_epoch: 0,
    };
    program_test.add_account(order_pda, account.into());

    // Add To Token Account (Simulate SPL Token Account)
    let token_data = vec![0u8; 165];
    let token_account = Account {
        lamports: 1_000_000,
        data: token_data,
        owner: solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
        executable: false,
        rent_epoch: 0,
    };
    program_test.add_account(to_token_account, token_account.into());

    let (banks_client, payer, recent_blockhash) = program_test.start().await;

    // Execute Instruction
    let solver = Keypair::new();
    let cpi_instruction_data = vec![1, 2, 3]; // Dummy data for market

    let mut instruction_data = vec![2]; // Discriminator Execute
    instruction_data.push(bump);
    instruction_data.extend_from_slice(order_bytes);
    instruction_data.extend_from_slice(&cpi_instruction_data);

    let accounts = vec![
        AccountMeta::new(solver.pubkey(), true),
        AccountMeta::new(order_pda, false),
        AccountMeta::new(owner.pubkey(), false),
        AccountMeta::new(from_token_account, false),
        AccountMeta::new(to_token_account, false), // Writable for balance change
        AccountMeta::new_readonly(mock_market_program_id, false),
        AccountMeta::new_readonly(
            solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
            false,
        ),
        AccountMeta::new(to_token_account, false),
    ];

    let instruction = Instruction {
        program_id,
        accounts,
        data: instruction_data,
    };

    let transfer_ix = system_transfer(&payer.pubkey(), &solver.pubkey(), 1_000_000_000);

    let mut fund_tx = Transaction::new_with_payer(&[transfer_ix], Some(&payer.pubkey()));
    fund_tx.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(fund_tx).await.unwrap();

    let mut tx = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    tx.sign(&[&payer, &solver], recent_blockhash);

    // This should succeed if Mock Market increases balance by at least 100.
    banks_client.process_transaction(tx).await.unwrap();
}

pub fn mock_market_processor(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _input: &[u8],
) -> ProgramResult {
    let account = &accounts[0]; // Should be to_token_account
                                // Increase balance (offset 64) by 100.
    let mut data = account.try_borrow_mut_data()?;
    let mut amount_bytes = [0u8; 8];
    amount_bytes.copy_from_slice(&data[64..72]);
    let amount = u64::from_le_bytes(amount_bytes);
    let new_amount = amount + 100;
    data[64..72].copy_from_slice(&new_amount.to_le_bytes());

    Ok(())
}

fn system_transfer(from: &Pubkey, to: &Pubkey, lamports: u64) -> Instruction {
    let account_metas = vec![AccountMeta::new(*from, true), AccountMeta::new(*to, false)];
    let mut data = vec![2, 0, 0, 0]; // Transfer instruction index = 2
    data.extend_from_slice(&lamports.to_le_bytes());
    Instruction {
        program_id: Pubkey::default(), // System Program ID is [0; 32]
        accounts: account_metas,
        data,
    }
}

pub fn sol_ver_adapter(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Transmute Solana SDK types to Pinocchio types
    // Pinocchio types are repr(C) compliant with Solana types
    let p_program_id: &pinocchio::pubkey::Pubkey = unsafe { std::mem::transmute(program_id) };
    let p_accounts: &[pinocchio::account_info::AccountInfo] =
        unsafe { std::mem::transmute(accounts) };

    sol_ver::instruction::process_instruction(p_program_id, p_accounts, instruction_data).map_err(
        |e| match e {
            pinocchio::program_error::ProgramError::Custom(c) => ProgramError::Custom(c),
            pinocchio::program_error::ProgramError::InvalidArgument => {
                ProgramError::InvalidArgument
            }
            pinocchio::program_error::ProgramError::InvalidInstructionData => {
                ProgramError::InvalidInstructionData
            }
            _ => ProgramError::Custom(999),
        },
    )
}
