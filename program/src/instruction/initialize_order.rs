use crate::utils::{DataLen, Unpackable};
use crate::{error::SolverError, state::order::Order};
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, sysvars::rent::Rent,
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, shank::ShankType, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InitializeOrderArgs {
    pub sell_token: Pubkey,
    pub buy_token: Pubkey,
    pub sell_amount: u64,
    pub buy_amount: u64,
    pub referral_fee: u64,
    pub referral_account: Pubkey,
}

pub struct InitializeOrderContext<'a> {
    pub payer: &'a AccountInfo,
    pub owner: &'a AccountInfo,
    pub order_account: &'a AccountInfo,
    pub sysvar_rent_acc: &'a AccountInfo,
    pub _system_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitializeOrderContext<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [payer, owner, order_account, sysvar_rent_acc, system_program] = accounts else {
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
            sysvar_rent_acc,
            _system_program: system_program,
        })
    }
}

pub fn process_initialize_order(accounts: &[AccountInfo], args: &[u8]) -> ProgramResult {
    let args = InitializeOrderArgs::unpack(args)?;
    let context = InitializeOrderContext::try_from(accounts)?;

    let rent = Rent::from_account_info(context.sysvar_rent_acc)?;

    CreateAccount {
        lamports: rent.minimum_balance(Order::LEN),
        space: Order::LEN as u64,
        owner: &crate::ID,
        from: context.payer,
        to: context.order_account,
    }
    .invoke()?;

    let order = Order::load_mut(context.order_account)?;

    order.init(
        *context.owner.key(),
        args.sell_token,
        args.buy_token,
        args.sell_amount,
        args.buy_amount,
        args.referral_fee,
        args.referral_account,
        *context.payer.key(),
    )?;

    Ok(())
}
