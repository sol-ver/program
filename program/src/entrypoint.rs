use pinocchio::{ProgramResult, account_info::AccountInfo, entrypoint, msg, no_allocator, nostd_panic_handler, pubkey::Pubkey, program_error::ProgramError};
use sol_ver_interface::error::SolverError;

use crate::processor::initialize_order::process_initialize_order;

entrypoint!(process_instruction);
nostd_panic_handler!();

#[inline(always)]
pub fn process_instruction(
    program_id: &Pubkey,
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
            process_initialize_order(program_id, accounts, instruction_data)
        }
        _ => {
            return Err(SolverError::InvalidInstruction.into());
        }
    }
}