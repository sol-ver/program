use bytemuck::{Pod, Zeroable};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};

// Constants
const SPL_TOKEN_PROGRAM_ID: Pubkey =
    solana_program::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const ORDER_PROGRAM_ID: Pubkey =
    solana_program::pubkey!("7QP9vxNo7EEwTjrskup6n3F1dcwgUsVKgMFnJsXoyBde");
const SYSTEM_PROGRAM_ID: Pubkey = solana_program::pubkey!("11111111111111111111111111111111");
const TOKEN_ACCOUNT_LEN: u64 = 165;
const MINT_LEN: u64 = 82;

// System Instruction Helpers
fn system_create_account(
    from_pubkey: &Pubkey,
    to_pubkey: &Pubkey,
    lamports: u64,
    space: u64,
    owner_program_id: &Pubkey,
) -> Instruction {
    let mut data = Vec::with_capacity(4 + 8 + 8 + 32);
    data.extend_from_slice(&0u32.to_le_bytes()); // CreateAccount
    data.extend_from_slice(&lamports.to_le_bytes());
    data.extend_from_slice(&space.to_le_bytes());
    data.extend_from_slice(owner_program_id.as_ref());

    Instruction {
        program_id: SYSTEM_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*from_pubkey, true),
            AccountMeta::new(*to_pubkey, true),
        ],
        data,
    }
}

// SPL Token Instructions Builders
fn token_initialize_mint(
    mint_pubkey: &Pubkey,
    mint_authority: &Pubkey,
    decimals: u8,
) -> Instruction {
    let mut data = Vec::with_capacity(67);
    data.push(0); // InitializeMint discriminator
    data.push(decimals);
    data.extend_from_slice(mint_authority.as_ref());
    data.push(0); // Freeze authority option (None)
    data.extend_from_slice(&[0u8; 32]);

    Instruction {
        program_id: SPL_TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*mint_pubkey, false),
            AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
        ],
        data,
    }
}

fn token_initialize_account(account: &Pubkey, mint: &Pubkey, owner: &Pubkey) -> Instruction {
    Instruction {
        program_id: SPL_TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*account, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(*owner, false),
            AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
        ],
        data: vec![1], // InitializeAccount discriminator
    }
}

fn token_mint_to(mint: &Pubkey, dest: &Pubkey, authority: &Pubkey, amount: u64) -> Instruction {
    let mut data = Vec::with_capacity(9);
    data.push(7); // MintTo discriminator
    data.extend_from_slice(&amount.to_le_bytes());

    Instruction {
        program_id: SPL_TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*mint, false),
            AccountMeta::new(*dest, false),
            AccountMeta::new_readonly(*authority, true),
        ],
        data,
    }
}

// Order Layout
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct OrderLayout {
    is_initialized: u8,
    owner: [u8; 32],
    sell_token: [u8; 32],
    buy_token: [u8; 32],
    receiver_token_account: [u8; 32],
    sell_amount: u64,
    buy_amount: u64,
    referral_fee: u64,
    referral_token_account: [u8; 32],
    rent_payer: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct InitializeOrderArgs {
    sell_token: Pubkey,
    buy_token: Pubkey,
    sell_amount: u64,
    buy_amount: u64,
    receiver_token_account: Pubkey,
    referral_fee: u64,
    referral_token_account: Pubkey,
    order_nonce: [u8; 8],
}

impl InitializeOrderArgs {
    fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(160);
        data.push(0u8); // Discriminator
        data.extend_from_slice(self.sell_token.as_ref());
        data.extend_from_slice(self.buy_token.as_ref());
        data.extend_from_slice(&self.sell_amount.to_le_bytes());
        data.extend_from_slice(&self.buy_amount.to_le_bytes());
        data.extend_from_slice(self.receiver_token_account.as_ref());
        data.extend_from_slice(&self.referral_fee.to_le_bytes());
        data.extend_from_slice(self.referral_token_account.as_ref());
        data.extend_from_slice(&self.order_nonce);
        data
    }
}

#[tokio::test]
async fn test_initialize_order() {
    let program_test = ProgramTest::new("sol_ver", ORDER_PROGRAM_ID, None);

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Accounts
    let owner = Keypair::new();
    let sell_token_mint = Keypair::new();
    let buy_token_mint = Keypair::new();
    let referral_account = Keypair::new();

    // Create Mints
    let rent_mint = banks_client
        .get_rent()
        .await
        .unwrap()
        .minimum_balance(MINT_LEN as usize);

    // Create Sell Mint
    let create_sell_mint_tx = Transaction::new_signed_with_payer(
        &[
            system_create_account(
                &payer.pubkey(),
                &sell_token_mint.pubkey(),
                rent_mint,
                MINT_LEN,
                &SPL_TOKEN_PROGRAM_ID,
            ),
            token_initialize_mint(&sell_token_mint.pubkey(), &payer.pubkey(), 6),
        ],
        Some(&payer.pubkey()),
        &[&payer, &sell_token_mint],
        recent_blockhash,
    );
    banks_client
        .process_transaction(create_sell_mint_tx)
        .await
        .unwrap();

    // Create Buy Mint
    let create_buy_mint_tx = Transaction::new_signed_with_payer(
        &[
            system_create_account(
                &payer.pubkey(),
                &buy_token_mint.pubkey(),
                rent_mint,
                MINT_LEN,
                &SPL_TOKEN_PROGRAM_ID,
            ),
            token_initialize_mint(&buy_token_mint.pubkey(), &payer.pubkey(), 6),
        ],
        Some(&payer.pubkey()),
        &[&payer, &buy_token_mint],
        recent_blockhash,
    );
    banks_client
        .process_transaction(create_buy_mint_tx)
        .await
        .unwrap();

    // Setup Owner's Token Account (From Account)
    let from_token_account = Keypair::new();
    let rent_account = banks_client
        .get_rent()
        .await
        .unwrap()
        .minimum_balance(TOKEN_ACCOUNT_LEN as usize);

    let create_from_account_tx = Transaction::new_signed_with_payer(
        &[
            system_create_account(
                &payer.pubkey(),
                &from_token_account.pubkey(),
                rent_account,
                TOKEN_ACCOUNT_LEN,
                &SPL_TOKEN_PROGRAM_ID,
            ),
            token_initialize_account(
                &from_token_account.pubkey(),
                &buy_token_mint.pubkey(),
                &owner.pubkey(),
            ),
            token_mint_to(
                &buy_token_mint.pubkey(),
                &from_token_account.pubkey(),
                &payer.pubkey(),
                2000,
            ),
        ],
        Some(&payer.pubkey()),
        &[&payer, &from_token_account],
        recent_blockhash,
    );
    banks_client
        .process_transaction(create_from_account_tx)
        .await
        .unwrap();

    // Calculate Order PDA
    let order_nonce = [1u8; 8];
    let (order_account, _bump) = Pubkey::find_program_address(
        &[b"order", &owner.pubkey().as_ref(), &order_nonce],
        &ORDER_PROGRAM_ID,
    );

    // Setup To Token Account (Owned by Order PDA)
    let to_token_account = Keypair::new();
    let create_to_account_tx = Transaction::new_signed_with_payer(
        &[
            system_create_account(
                &payer.pubkey(),
                &to_token_account.pubkey(),
                rent_account,
                TOKEN_ACCOUNT_LEN,
                &SPL_TOKEN_PROGRAM_ID,
            ),
            token_initialize_account(
                &to_token_account.pubkey(),
                &buy_token_mint.pubkey(),
                &order_account,
            ),
        ],
        Some(&payer.pubkey()),
        &[&payer, &to_token_account],
        recent_blockhash,
    );
    banks_client
        .process_transaction(create_to_account_tx)
        .await
        .unwrap();

    // Args
    let args = InitializeOrderArgs {
        sell_token: sell_token_mint.pubkey(),
        buy_token: buy_token_mint.pubkey(),
        sell_amount: 1000,
        buy_amount: 2000,
        receiver_token_account: to_token_account.pubkey(),
        referral_fee: 50,
        referral_token_account: referral_account.pubkey(),
        order_nonce,
    };

    // Instruction
    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(owner.pubkey(), true),
        AccountMeta::new(order_account, false),
        AccountMeta::new(from_token_account.pubkey(), false),
        AccountMeta::new(to_token_account.pubkey(), false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
        AccountMeta::new_readonly(SPL_TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
    ];

    let instruction = Instruction {
        program_id: ORDER_PROGRAM_ID,
        accounts,
        data: args.to_bytes(),
    };

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &owner], recent_blockhash);

    banks_client.process_transaction(transaction).await.unwrap();

    // Verify Order Account
    let account = banks_client
        .get_account(order_account)
        .await
        .unwrap()
        .expect("Order account not found");
    let order_layout: &OrderLayout = unsafe { &*(account.data.as_ptr() as *const &OrderLayout) };

    assert_eq!(order_layout.is_initialized, 1);
    assert_eq!(Pubkey::new_from_array(order_layout.owner), owner.pubkey());
    assert_eq!(
        Pubkey::new_from_array(order_layout.sell_token),
        sell_token_mint.pubkey()
    );
    assert_eq!(
        Pubkey::new_from_array(order_layout.buy_token),
        buy_token_mint.pubkey()
    );
    assert_eq!(order_layout.sell_amount, 1000);
    assert_eq!(order_layout.buy_amount, 2000);
    assert_eq!(order_layout.referral_fee, 50);
    assert_eq!(
        Pubkey::new_from_array(order_layout.referral_token_account),
        referral_account.pubkey()
    );
    assert_eq!(
        Pubkey::new_from_array(order_layout.rent_payer),
        payer.pubkey()
    );

    // TODO: Verify approval
    todo!("Verify approval logic")
}

