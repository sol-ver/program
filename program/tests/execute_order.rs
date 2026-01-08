use light_hasher::{Hasher, Keccak};
use sol_ver::state::order::Order;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    signature::{Keypair, Signer},
    transaction::Transaction,
};

// Helper to run the test properly with setup
#[tokio::test]
async fn test_execute_order_e2e() {
    let program_id = Pubkey::new_from_array(sol_ver::ID);
    let mut program_test = ProgramTest::new("sol_ver", program_id, processor!(sol_ver_adapter));
    program_test.prefer_bpf(true);

    let from_token_account = Pubkey::new_unique();
    let to_token_account = Pubkey::new_unique();
    let referral_token_account = Pubkey::new_unique();

    // We need a Mint to create valid token accounts
    let mint_account = Pubkey::new_unique();
    let mint_authority = Keypair::new();

    let order_struct = Order {
        from_token_account: pinocchio::pubkey::Pubkey::from(from_token_account.to_bytes()),
        to_token_account: pinocchio::pubkey::Pubkey::from(to_token_account.to_bytes()),
        sell_amount: 100,
        buy_amount: 100, // We need to transfer >= 100
        referral_fee: 0,
        referral_token_account: pinocchio::pubkey::Pubkey::from(referral_token_account.to_bytes()),
        minimun_buy_amount: 90,
        amount_decrease_per_second: 0,
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

    let (order_pda, order_bump) = Pubkey::find_program_address(
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

    let token_program_id = solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

    // Setup Mint Manually
    let mut mint_data = vec![0u8; 82]; // Mint::LEN = 82
                                       // Mint Authority: Option::Some(mint_authority)
    mint_data[0..4].copy_from_slice(&1u32.to_le_bytes()); // Option::Some
    mint_data[4..36].copy_from_slice(mint_authority.pubkey().as_ref());
    // Supply: 1000
    mint_data[36..44].copy_from_slice(&1000u64.to_le_bytes());
    // Decimals: 6
    mint_data[44] = 6;
    // IsInitialized: true
    mint_data[45] = 1;
    // Freeze Authority: Option::None (0)

    program_test.add_account(
        mint_account,
        Account {
            lamports: 1_000_000,
            data: mint_data,
            owner: token_program_id,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // Setup To Token Account (Destination) - Initial Balance 0
    let mut to_token_data = vec![0u8; 165]; // TokenAccount::LEN = 165
                                            // Mint
    to_token_data[0..32].copy_from_slice(mint_account.as_ref());
    // Owner
    to_token_data[32..64].copy_from_slice(Pubkey::new_unique().as_ref());
    // Amount: 0
    to_token_data[64..72].copy_from_slice(&0u64.to_le_bytes());
    // Delegate: None
    // State: Initialized (1)
    to_token_data[108] = 1;
    // IsNative: None
    // Delegated Amount: 0
    // Close Authority: None

    program_test.add_account(
        to_token_account,
        Account {
            lamports: 1_000_000,
            data: to_token_data,
            owner: token_program_id,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // Setup Solver Token Account (Source) - Initial Balance 1000
    let solver = Keypair::new();
    let solver_token_account = Pubkey::new_unique();
    let mut solver_token_data = vec![0u8; 165];
    // Mint
    solver_token_data[0..32].copy_from_slice(mint_account.as_ref());
    // Owner: Solver
    solver_token_data[32..64].copy_from_slice(solver.pubkey().as_ref());
    // Amount: 1000
    solver_token_data[64..72].copy_from_slice(&1000u64.to_le_bytes());
    // State: Initialized (1)
    solver_token_data[108] = 1;

    program_test.add_account(
        solver_token_account,
        Account {
            lamports: 1_000_000,
            data: solver_token_data,
            owner: token_program_id,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Construct CPI Instruction: Transfer 100 tokens from solver_token_account to to_token_account
    // spl_token instruction: Transfer { amount }
    // 3 is Transfer
    let amount_to_transfer = 100u64;
    let mut cpi_instruction_data = vec![3];
    cpi_instruction_data.extend_from_slice(&amount_to_transfer.to_le_bytes());

    let mut instruction_data = vec![2]; // Discriminator Execute
    instruction_data.push(order_bump);
    instruction_data.extend_from_slice(order_bytes);
    instruction_data.extend_from_slice(&cpi_instruction_data);

    let accounts = vec![
        AccountMeta::new(solver.pubkey(), true), // Solver Signer
        AccountMeta::new(order_pda, false),
        AccountMeta::new(owner.pubkey(), false),
        AccountMeta::new(from_token_account, false),
        AccountMeta::new(to_token_account, false), // Writable for balance check
        AccountMeta::new(referral_token_account, false),
        AccountMeta::new_readonly(token_program_id, false), // Order Program = Token Program
        AccountMeta::new_readonly(token_program_id, false), // Token Program
        // Remaining Accounts for CPI Transfer
        // source, destination, authority
        AccountMeta::new(solver_token_account, false),
        AccountMeta::new(to_token_account, false),
        AccountMeta::new(solver.pubkey(), true), // Authority (Solver) - must be signer in CPI if not PDA
    ];

    let instruction = Instruction {
        program_id,
        accounts,
        data: instruction_data,
    };

    let transfer_ix = solana_program::system_instruction::transfer(
        &payer.pubkey(),
        &solver.pubkey(),
        1_000_000_000,
    );

    let mut fund_tx = Transaction::new_with_payer(&[transfer_ix], Some(&payer.pubkey()));
    fund_tx.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(fund_tx).await.unwrap();

    let mut tx = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    tx.sign(&[&payer, &solver], recent_blockhash);

    banks_client.process_transaction(tx).await.unwrap();

    // Verify balance
    let account = banks_client
        .get_account(to_token_account)
        .await
        .unwrap()
        .unwrap();
    let mut amount_bytes = [0u8; 8];
    amount_bytes.copy_from_slice(&account.data[64..72]);
    let amount = u64::from_le_bytes(amount_bytes);
    assert_eq!(amount, 100);
}

pub fn sol_ver_adapter(
    program_id: &Pubkey,
    accounts: &[solana_sdk::account_info::AccountInfo],
    instruction_data: &[u8],
) -> solana_program::entrypoint::ProgramResult {
    // Transmute Solana SDK types to Pinocchio types
    // Pinocchio types are repr(C) compliant with Solana types
    let p_program_id: &pinocchio::pubkey::Pubkey = unsafe { std::mem::transmute(program_id) };
    let p_accounts: &[pinocchio::account_info::AccountInfo] =
        unsafe { std::mem::transmute(accounts) };

    sol_ver::instruction::process_instruction(p_program_id, p_accounts, instruction_data).map_err(
        |e| match e {
            pinocchio::program_error::ProgramError::Custom(c) => {
                solana_sdk::program_error::ProgramError::Custom(c)
            }
            pinocchio::program_error::ProgramError::InvalidArgument => {
                solana_sdk::program_error::ProgramError::InvalidArgument
            }
            pinocchio::program_error::ProgramError::InvalidInstructionData => {
                solana_sdk::program_error::ProgramError::InvalidInstructionData
            }
            _ => solana_sdk::program_error::ProgramError::Custom(999),
        },
    )
}
