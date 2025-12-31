use crate::utils::{find_order_address, DataLen, Unpackable};
use crate::{error::SolverError, state::order::Order};
use pinocchio::instruction::{Seed, Signer};
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, sysvars::rent::Rent,
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::Transfer;
use pinocchio_token::state::TokenAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, shank::ShankType, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InitializeOrderArgs {
    pub sell_token: Pubkey,
    pub buy_token: Pubkey,
    pub sell_amount: u64,
    pub buy_amount: u64,
    pub referral_fee: u64,
    pub referral_token_account: Pubkey,
    pub order_nonce: [u8; 8],
}

pub struct InitializeOrderContext<'a> {
    pub payer: &'a AccountInfo,
    pub owner: &'a AccountInfo,
    pub order_account: &'a AccountInfo,
    pub from_token_account: &'a AccountInfo,
    pub to_token_account: &'a AccountInfo,
    pub sysvar_rent_acc: &'a AccountInfo,
    pub _token_program: &'a AccountInfo,
    pub _system_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitializeOrderContext<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [payer, owner, order_account, from_token_account, to_token_account, sysvar_rent_acc, token_program, system_program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        if !payer.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !owner.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !order_account.data_is_empty() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        if !order_account.is_writable() {
            return Err(SolverError::OrderAccountMustBeMut.into());
        }

        if !(system_program.key() == &pinocchio_system::ID) {
            return Err(ProgramError::IncorrectProgramId);
        }

        Ok(Self {
            payer,
            owner,
            order_account,
            from_token_account,
            to_token_account,
            sysvar_rent_acc,
            _token_program: token_program,
            _system_program: system_program,
        })
    }
}

pub fn process_initialize_order(accounts: &[AccountInfo], args: &[u8]) -> ProgramResult {
    let args = InitializeOrderArgs::unpack(args)?;
    let context = InitializeOrderContext::try_from(accounts)?;

    let rent = Rent::from_account_info(context.sysvar_rent_acc)?;

    let (order_drive_address, bump) = find_order_address(context.owner.key(), &args.order_nonce);

    // make sure that order account is created by this program
    if order_drive_address != *context.order_account.key() {
        return Err(SolverError::InvalidOrderAccount.into());
    }

    let signer_seeds = [
        Seed::from(b"order".as_slice()),
        Seed::from(context.owner.key().as_ref()),
        Seed::from(args.order_nonce.as_ref()),
        Seed::from(core::slice::from_ref(&bump)),
    ];

    let signer = Signer::from(&signer_seeds);

    // Create order account
    CreateAccount {
        lamports: rent.minimum_balance(Order::LEN),
        space: Order::LEN as u64,
        owner: &crate::ID,
        from: context.payer,
        to: context.order_account,
    }
    .invoke_signed(&[signer])?;

    {
        let to_token_account = TokenAccount::from_account_info(context.to_token_account).unwrap();
        // validate to token account owner
        if to_token_account.owner() != context.order_account.key() {
            return Err(SolverError::InvalidTokenAccountOwner.into());
        }

        // validate to token account mint
        if *to_token_account.mint() != args.buy_token {
            return Err(SolverError::InvalidTokenAccountMint.into());
        }
    }

    // Transfer buy token to order account
    Transfer {
        from: context.from_token_account,
        to: context.to_token_account,
        authority: context.owner,
        amount: args.buy_amount,
    }
    .invoke()?;

    let order = Order::load_mut(context.order_account)?;

    // Initialize order
    order.init(
        *context.owner.key(),
        args.sell_token,
        args.buy_token,
        args.sell_amount,
        args.buy_amount,
        args.referral_fee,
        args.referral_token_account,
        *context.payer.key(),
    )?;

    Ok(())
}
