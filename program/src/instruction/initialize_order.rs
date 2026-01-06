use crate::utils::DataLen;
use crate::{error::SolverError, state::order::Order};
use pinocchio::pubkey::create_program_address;
use pinocchio::{
    account_info::AccountInfo, msg, program_error::ProgramError, ProgramResult,
};
use pinocchio_token::instructions::Approve;
use light_hasher::{Hasher, Keccak};

pub struct InitializeOrderContext<'a> {
    pub owner: &'a AccountInfo,
    pub order_account: &'a AccountInfo,
    pub from_token_account: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitializeOrderContext<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, order_account, from_token_account, token_program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !owner.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(Self {
            owner,
            order_account,
            from_token_account,
            token_program,
        })
    }
}

pub fn process_initialize_order(accounts: &[AccountInfo], args: &[u8]) -> ProgramResult {
    let context = InitializeOrderContext::try_from(accounts)?;
    if args.len() != 1 + 8 + Order::LEN {
        // 1 byte order_bump + 8 bytes order_nonce + Order data
        return Err(SolverError::InvalidInstructionData.into());
    }
    let order_bump = &args[0];
    let intent_body = &args[..args.len() - 1];
    let buy_amount = u64::from_le_bytes(args[153..161].try_into().unwrap());
    let intent_hash = Keccak::hashv(&[intent_body]).unwrap();

    let calculated_order_pubkey = create_program_address(
        &[b"order", context.owner.key().as_ref(), intent_hash.as_ref(), &[*order_bump]],
        &crate::ID,
    ).unwrap();

    if &calculated_order_pubkey != context.order_account.key() {
        return Err(SolverError::InvalidOrderAccount.into());
    }

    Approve {
        source: context.from_token_account,
        delegate: context.order_account,
        authority: context.owner,
        amount: buy_amount,
    }
    .invoke()?;

    msg!("Order created");

    Ok(())
}
