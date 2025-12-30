use crate::{error::SolverError, instruction::initialize_order::process_initialize_order};
use pinocchio::{account_info::AccountInfo, entrypoint, msg, pubkey::Pubkey, ProgramResult};

entrypoint!(process_instruction);

#[inline(always)]
pub fn process_instruction(
    _: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [descriminator, instruction_data @ ..] = instruction_data else {
        msg!("Instruction data too short");
        return Err(SolverError::InvalidInstruction.into());
    };

    match *descriminator {
        0 => {
            msg!("Processing InitializeOrder instruction");
            process_initialize_order(accounts, instruction_data)
        }
        _ => {
            return Err(SolverError::InvalidInstruction.into());
        }
    }
}
