use light_hasher::{Hasher, Keccak};
use sol_ver::state::order::Order;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    signature::{Keypair, Signer},
    transaction::Transaction,
};

#[tokio::test]
async fn test_calculate_current_buy_amount() {
    let order = Order {
        from_token_account: pinocchio::pubkey::Pubkey::default(),
        to_token_account: pinocchio::pubkey::Pubkey::default(),
        sell_amount: 1000,
        buy_amount: 1000, // Start price
        referral_fee: 0,
        referral_token_account: pinocchio::pubkey::Pubkey::default(),
        minimun_buy_amount: 500, // Floor price
        start_time: 100,         // Start Time
        deadline: 200,           // End Time (Duration: 100)
    };
    // 1. Before start time
    assert_eq!(order.calculate_current_buy_amount(50), 1000);
    // 2. After deadline
    assert_eq!(order.calculate_current_buy_amount(250), 500);
    // 3. Midway through
    assert_eq!(order.calculate_current_buy_amount(150), 750);
    // 4. Near the end
    assert_eq!(order.calculate_current_buy_amount(190), 550);
}

#[tokio::test]
async fn test_initialize_order() {
    let program_id = Pubkey::new_from_array(sol_ver::ID);

    let mut program_test = ProgramTest::new("sol_ver", program_id, None);
    program_test.prefer_bpf(false);

    let token_program_id = solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
    program_test.add_program(
        "spl_token",
        token_program_id,
        processor!(mock_token_processor),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // 1. Setup Accounts
    let owner = Keypair::new();
    let sell_token_mint = Keypair::new();
    let buy_token_mint = Keypair::new();
    let receiver_token_account = Keypair::new();
    let referral_token_account = Keypair::new();
    let rent_payer = Keypair::new();

    // Fund owner
    let transfer_tx = system_transfer(&payer.pubkey(), &owner.pubkey(), 1_000_000_000);
    let mut tx = Transaction::new_with_payer(&[transfer_tx], Some(&payer.pubkey()));
    tx.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(tx).await.unwrap();

    // 2. Construct Order
    let p_owner = pinocchio::pubkey::Pubkey::from(owner.pubkey().to_bytes());
    let p_sell_token = pinocchio::pubkey::Pubkey::from(sell_token_mint.pubkey().to_bytes());
    let p_buy_token = pinocchio::pubkey::Pubkey::from(buy_token_mint.pubkey().to_bytes());
    let p_receiver = pinocchio::pubkey::Pubkey::from(receiver_token_account.pubkey().to_bytes());
    let p_referral = pinocchio::pubkey::Pubkey::from(referral_token_account.pubkey().to_bytes());
    let p_rent_payer = pinocchio::pubkey::Pubkey::from(rent_payer.pubkey().to_bytes());

    let order = Order {
        from_token_account: p_sell_token,
        to_token_account: p_buy_token,
        sell_amount: 100,
        buy_amount: 50,
        referral_fee: 1,
        referral_token_account: p_referral,
        minimun_buy_amount: 45,
        start_time: 1_600_000_000,
        deadline: 1_700_000_000,
    };

    // Unsafe serialization because Order is repr(C) but not Pod
    let order_bytes = unsafe {
        core::slice::from_raw_parts(
            &order as *const Order as *const u8,
            core::mem::size_of::<Order>(),
        )
    };
    let mut intent_body = Vec::new();
    intent_body.extend_from_slice(order_bytes);

    let intent_hash = Keccak::hashv(&[&intent_body]).unwrap();

    let (order_pda, bump) = Pubkey::find_program_address(
        &[b"order", owner.pubkey().as_ref(), intent_hash.as_ref()],
        &program_id,
    );

    // 3. Construct Instruction Data
    // [discriminator] + [bump] + [nonce] + [Order]
    let mut instruction_data = Vec::new();
    instruction_data.push(0); // Instruction::Initialize discriminator
    instruction_data.push(bump);
    instruction_data.extend_from_slice(&intent_body);

    let from_token_account = Keypair::new();

    let accounts = vec![
        AccountMeta::new(owner.pubkey(), true),
        AccountMeta::new(order_pda, false),
    ];

    let instruction = Instruction {
        program_id,
        accounts,
        data: instruction_data,
    };

    let mut tx = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    tx.sign(&[&payer, &owner], recent_blockhash);

    banks_client.process_transaction(tx).await.unwrap();
}

pub fn mock_token_processor(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
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
