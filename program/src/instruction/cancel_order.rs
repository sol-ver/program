use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
};

use crate::{
    error::SolverError,
    state::order::Order,
};

pub struct CancelOrderContext<'a> {
    pub order_account: &'a AccountInfo,
    pub owner: &'a AccountInfo,
    pub rent_payer: &'a AccountInfo,
    pub _system_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for CancelOrderContext<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [order_account, owner, rent_payer, system_program] =
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

        if system_program.key() != &pinocchio_system::ID {
            return Err(ProgramError::IncorrectProgramId);
        }

        Ok(Self {
            order_account,
            owner,
            rent_payer,
            _system_program: system_program,
        })
    }
}

pub fn process_cancel_order(accounts: &[AccountInfo], _: &[u8]) -> Result<(), ProgramError> {
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

    // SAFETY close order account
    unsafe {
        *context.rent_payer.borrow_mut_lamports_unchecked() += context.order_account.lamports();
        context.order_account.close_unchecked();
    }

    Ok(())
}
