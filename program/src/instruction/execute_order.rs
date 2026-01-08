use crate::error::SolverError;
use crate::state::order::Order;
use crate::utils::{DataLen, Unpackable};
use light_hasher::{Hasher, Keccak};
use pinocchio::pubkey::create_program_address;
use pinocchio::sysvars::clock::Clock;
use pinocchio::sysvars::Sysvar;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

pub struct ExecuteOrderContext<'a> {
    pub solver: &'a AccountInfo,
    pub order_account: &'a AccountInfo,
    pub owner: &'a AccountInfo,
    pub from_token_account: &'a AccountInfo,
    pub to_token_account: &'a AccountInfo,
    pub referral_token_account: &'a AccountInfo,
    pub order_program: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
    pub remaining_accounts: &'a [AccountInfo],
}

impl<'a> TryFrom<&'a [AccountInfo]> for ExecuteOrderContext<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [solver, order_account, owner, from_token_account, to_token_account, referral_token_account, order_program, token_program, remaining_accounts @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !solver.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(Self {
            solver,
            order_account,
            owner,
            from_token_account,
            to_token_account,
            referral_token_account,
            order_program,
            token_program,
            remaining_accounts,
        })
    }
}

pub fn process_execute_order(accounts: &[AccountInfo], args: &[u8]) -> ProgramResult {
    let context = ExecuteOrderContext::try_from(accounts)?;

    // 1. Parse arguments (bump + Order + CPI data)
    if args.len() < 1 + Order::LEN {
        return Err(SolverError::InvalidInstructionData.into());
    }

    let order_bump = args[0];
    let order_data = &args[1..1 + Order::LEN];
    // Remaining data is CPI instruction data
    let instruction_data = &args[1 + Order::LEN..];

    let order = Order::unpack(order_data)?;

    if !order.validate_order_accounts(
        context.from_token_account.key(),
        context.to_token_account.key(),
        context.referral_token_account.key(),
    ) {
        return Err(SolverError::InvalidOrderAccounts.into());
    }

    // 2. Validate Order PDA
    let intent_hash = Keccak::hashv(&[order_data]).unwrap();
    // Use correct slice type for address generation
    let intent_hash_bytes = &intent_hash as &[u8];

    let calculated_order_pubkey = create_program_address(
        &[
            b"order",
            context.owner.key().as_ref(),
            intent_hash_bytes,
            &[order_bump],
        ],
        &crate::ID,
    )
    .unwrap();

    if &calculated_order_pubkey != context.order_account.key() {
        return Err(SolverError::InvalidOrderAccount.into());
    }

    // 3. Confirm to_token_account matches order
    if context.to_token_account.key() != &order.to_token_account {
        return Err(SolverError::InvalidOrderAccount.into()); // Or a more specific error
    }

    // 4. Calculate expected buy amount
    let clock = Clock::get()?;
    let expected_buy_amount = order.calculate_current_buy_amount(clock.unix_timestamp as u64);

    // 5. Pre-balance check
    let pre_balance = {
        let data = context.to_token_account.try_borrow_data()?;
        if data.len() < 72 {
            return Err(ProgramError::InvalidAccountData);
        }
        let mut amount_bytes = [0u8; 8];
        amount_bytes.copy_from_slice(&data[64..72]);
        u64::from_le_bytes(amount_bytes)
    };

    // 6. Execute CPI
    todo!("Implement CPI to token swap program using instruction_data and remaining_accounts");

    // 7. Post-balance check
    let post_balance = {
        let data = context.to_token_account.try_borrow_data()?;
        if data.len() < 72 {
            return Err(ProgramError::InvalidAccountData);
        }
        let mut amount_bytes = [0u8; 8];
        amount_bytes.copy_from_slice(&data[64..72]);
        u64::from_le_bytes(amount_bytes)
    };

    if post_balance < pre_balance + expected_buy_amount {
        return Err(SolverError::SlippageExceeded.into());
    }

    Ok(())
}
