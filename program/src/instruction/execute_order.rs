use crate::error::SolverError;
use crate::state::order::Order;
use crate::utils::DataLen;
use alloc::vec::Vec;
use pinocchio::cpi::{invoke_signed, slice_invoke_signed};
use pinocchio::instruction::{AccountMeta, Instruction, Seed, Signer};
use pinocchio::sysvars::clock::Clock;
use pinocchio::sysvars::Sysvar;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};
use pinocchio_token::state::TokenAccount;

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

    let (order, intend_hash) = Order::validate_and_unpack(
        order_data,
        context.owner.key(),
        context.order_account.key(),
        order_bump,
    )?;

    if !order.validate_order_accounts(
        context.from_token_account.key(),
        context.to_token_account.key(),
        context.referral_token_account.key(),
    ) {
        return Err(SolverError::InvalidOrderAccounts.into());
    }
    let clock = Clock::get()?;
    let expected_buy_amount = order.calculate_current_buy_amount(clock.unix_timestamp as u64);

    let pre_balance = {
        let token_account = TokenAccount::from_account_info(context.to_token_account).unwrap();
        token_account.amount()
    };

    // Remaining data is CPI instruction data
    let instruction_data = &args[1 + Order::LEN..];

    let instruction = Instruction {
        program_id: context.order_program.key(),
        accounts: &context
            .remaining_accounts
            .iter()
            .map(|acc| AccountMeta {
                pubkey: acc.key(),
                is_signer: acc.is_signer(),
                is_writable: acc.is_writable(),
            })
            .collect::<Vec<AccountMeta>>(),
        data: instruction_data,
    };

    let seeds = [
        Seed::from(b"order".as_slice()),
        Seed::from(context.owner.key()),
        Seed::from(intend_hash.as_ref()),
        Seed::from(core::slice::from_ref(&order_bump)),
    ];

    let signer = Signer::from(&seeds);

    let account_refs: Vec<&AccountInfo> = context.remaining_accounts.iter().collect();

    slice_invoke_signed(&instruction, &account_refs, &[signer]).unwrap();

    let post_balance = {
        let token_account = TokenAccount::from_account_info(context.to_token_account).unwrap();
        token_account.amount()
    };

    if post_balance < pre_balance + expected_buy_amount {
        return Err(SolverError::SlippageExceeded.into());
    }

    Ok(())
}
