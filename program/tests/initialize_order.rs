use bytemuck::{Pod, Zeroable};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
struct InitializeOrderArgs {
    sell_token: Pubkey,
    buy_token: Pubkey,
    sell_amount: u64,
    buy_amount: u64,
    referral_fee: u64,
    referral_account: Pubkey,
}

#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
struct Order {
    is_initialized: u8, // bool is not Pod, use u8
    owner: Pubkey,
    sell_token: Pubkey,
    buy_token: Pubkey,
    // There will be padding here due to alignment if next field is u64
    // 1 + 32 + 32 + 32 = 97.
    // u64 align is 8.
    // 97 -> 104 (7 bytes padding)
    // bytemuck derives should handle this?
    // Wait, bytemuck::Pod requires explicit padding or "no padding bytes".
    // "Pod" trait cannot be derived for structs with padding unless Zeroable is also derived and the padding is guaranteed zero?
    // Actually, bytemuck 1.14 allows derive(Pod) on repr(C) structs ONLY if they have no padding?
    // Safe bet: "The struct must not have any padding bytes." - standard rule.
    // So I must adding padding explicitly.
    _padding: [u8; 7],
    sell_amount: u64,
    buy_amount: u64,
    referral_fee: u64,
    referral_account: Pubkey,
    rent_payer: Pubkey,
}

// Address of the deployed program (matches declare_id! in lib.rs)
// 7QP9vxNo7EEwTjrskup6n3F1dcwgUsVKgMFnJsXoyBde
const PROGRAM_ID: Pubkey = solana_sdk::pubkey!("7QP9vxNo7EEwTjrskup6n3F1dcwgUsVKgMFnJsXoyBde");

#[tokio::test]
async fn test_initialize_order() {
    let program_test = ProgramTest::new("sol_ver", PROGRAM_ID, None);

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Accounts
    let owner = Keypair::new();
    let order_account = Keypair::new();
    let sell_token = Pubkey::new_unique();
    let buy_token = Pubkey::new_unique();
    let referral_account = Pubkey::new_unique();

    // Arguments
    let args = InitializeOrderArgs {
        sell_token,
        buy_token,
        sell_amount: 1000,
        buy_amount: 2000,
        referral_fee: 50,
        referral_account,
    };

    // Serialize args manually to match C representation
    let mut data = Vec::with_capacity(1 + std::mem::size_of::<InitializeOrderArgs>());
    data.push(0u8); // Discriminator for InitializeOrder
    data.extend_from_slice(args.sell_token.as_ref());
    data.extend_from_slice(args.buy_token.as_ref());
    data.extend_from_slice(&args.sell_amount.to_le_bytes());
    data.extend_from_slice(&args.buy_amount.to_le_bytes());
    data.extend_from_slice(&args.referral_fee.to_le_bytes());
    data.extend_from_slice(args.referral_account.as_ref());

    // Transaction to create and initialize order
    // Note: The instruction implementation calls CreateAccount.
    // The order_account must be a signer because it is being created.
    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(owner.pubkey(), true),
        AccountMeta::new(order_account.pubkey(), true),
        AccountMeta::new_readonly(solana_sdk::sysvar::rent::id(), false),
        AccountMeta::new_readonly(Pubkey::default(), false), // System Program ID
    ];

    let instruction = Instruction {
        program_id: PROGRAM_ID,
        accounts,
        data,
    };

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));

    transaction.sign(&[&payer, &owner, &order_account], recent_blockhash);

    banks_client.process_transaction(transaction).await.unwrap();

    // Verify account data
    let account = banks_client
        .get_account(order_account.pubkey())
        .await
        .unwrap()
        .expect("Account not found");

    // Check data length
    assert_eq!(account.data.len(), 192);

    // Use bytemuck to read data
    // Note: account.data might contain garbage in padding bytes if not initialized?
    // But `CreateAccount` zeroes memory.
    // And `Order::init` sets fields.
    // `Order::init` does NOT set padding bytes explicitly?
    // If it uses assignment `self.field = val`, padding is preserved (if existing) or undefined?
    // Actually, `Order::load_mut` gets mutable reference to zeroed account data.
    // Rust assignment to fields leaves padding bytes untouched (so they remain 0).
    // So explicit padding of 0 should match.

    let order_on_chain: &Order = bytemuck::from_bytes(&account.data);

    assert_eq!(order_on_chain.is_initialized, 1);
    assert_eq!(order_on_chain.owner, owner.pubkey());
    assert_eq!(order_on_chain.sell_token, sell_token);
    assert_eq!(order_on_chain.buy_token, buy_token);
    assert_eq!(order_on_chain.sell_amount, args.sell_amount);
    assert_eq!(order_on_chain.buy_amount, args.buy_amount);
    assert_eq!(order_on_chain.referral_fee, args.referral_fee);
    assert_eq!(order_on_chain.referral_account, referral_account);
    assert_eq!(order_on_chain.rent_payer, payer.pubkey());
}
