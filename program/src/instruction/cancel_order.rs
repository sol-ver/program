use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
};
use pinocchio_token::instructions::{CloseAccount, Transfer};

use crate::{
    error::SolverError,
    state::order::Order,
    utils::{find_order_address, Unpackable},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, shank::ShankType, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CancelOrderArgs {
    pub order_nonce: [u8; 8],
}

pub struct CancelOrderContext<'a> {
    pub order_account: &'a AccountInfo,
    pub owner: &'a AccountInfo,
    pub from_token_account: &'a AccountInfo,
    pub to_token_account: &'a AccountInfo,
    pub rent_payer: &'a AccountInfo,
    pub _token_program: &'a AccountInfo,
    pub _system_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for CancelOrderContext<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [order_account, owner, from_token_account, to_token_account, rent_payer, token_program, system_program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
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
            order_account,
            owner,
            from_token_account,
            to_token_account,
            rent_payer,
            _token_program: token_program,
            _system_program: system_program,
        })
    }
}

pub fn process_cancel_order(accounts: &[AccountInfo], args: &[u8]) -> Result<(), ProgramError> {
    let args = CancelOrderArgs::unpack(args)?;
    let context = CancelOrderContext::try_from(accounts)?;

    let order = Order::load(context.order_account)?;
    // validate owner
    if order.owner != *context.owner.key() {
        return Err(SolverError::InvalidOrderAccountOwner.into());
    }

    // validate rent payer
    if order.rent_payer != *context.rent_payer.key() {
        return Err(SolverError::InvalidRentPayer.into());
    }

    let (_, bump) = find_order_address(context.owner.key(), &args.order_nonce);

    let signer_seeds = [
        Seed::from(b"order".as_slice()),
        Seed::from(context.owner.key().as_ref()),
        Seed::from(args.order_nonce.as_ref()),
        Seed::from(core::slice::from_ref(&bump)),
    ];

    let signer = Signer::from(&signer_seeds);

    // Transfer all token to rent payer
    Transfer {
        from: context.from_token_account,
        to: context.to_token_account,
        authority: context.order_account,
        amount: order.sell_amount,
    }
    .invoke_signed(&[signer.clone()])?;

    // Close from token account
    CloseAccount {
        account: context.from_token_account,
        destination: context.rent_payer,
        authority: context.order_account,
    }
    .invoke_signed(&[signer])?;

    // SAFETY close order account
    unsafe {
        *context.rent_payer.borrow_mut_lamports_unchecked() += context.order_account.lamports();
        context.order_account.close_unchecked();
    }

    Ok(())
}
