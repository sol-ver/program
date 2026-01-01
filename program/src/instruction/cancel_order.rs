use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

use crate::{error::SolverError, state::order::Order, utils::Unpackable};

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
    if order.owner != *context.owner.key() {
        return Err(SolverError::InvalidOrderAccountOwner.into());
    }

    Ok(())
}
