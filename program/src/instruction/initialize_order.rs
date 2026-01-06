use crate::utils::DataLen;
use crate::{error::SolverError, state::order::Order};
use light_hasher::{Hasher, Keccak};
use pinocchio::pubkey::create_program_address;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

pub struct InitializeOrderContext<'a> {
    pub owner: &'a AccountInfo,
    pub order_account: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitializeOrderContext<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, order_account] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !owner.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(Self {
            owner,
            order_account,
        })
    }
}

pub fn process_initialize_order(accounts: &[AccountInfo], args: &[u8]) -> ProgramResult {
    let context = InitializeOrderContext::try_from(accounts)?;
    if args.len() != 1 + Order::LEN {
        // 1 byte order_bump + 8 bytes order_nonce + Order data
        return Err(SolverError::InvalidInstructionData.into());
    }
    let order_bump = &args[0];
    let intent_body = &args[1..];
    let intent_hash = Keccak::hashv(&[intent_body]).unwrap();

    let calculated_order_pubkey = create_program_address(
        &[
            b"order",
            context.owner.key().as_ref(),
            intent_hash.as_ref(),
            &[*order_bump],
        ],
        &crate::ID,
    )
    .unwrap();

    if &calculated_order_pubkey != context.order_account.key() {
        return Err(SolverError::InvalidOrderAccount.into());
    }

    Ok(())
}
