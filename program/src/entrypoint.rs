use crate::{
    error::SolverError,
    instruction::{
        cancel_order::process_cancel_order, initialize_order::process_initialize_order, Instruction,
    },
};
use pinocchio::{ProgramResult, account_info::AccountInfo, entrypoint, log::sol_log_compute_units, msg, no_allocator, pubkey::Pubkey};

entrypoint!(process_instruction);

#[inline(always)]
pub fn process_instruction(
    _: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    sol_log_compute_units();
    let [discriminator, instruction_data @ ..] = instruction_data else {
        msg!("Instruction data too short");
        return Err(SolverError::InvalidInstruction.into());
    };

    sol_log_compute_units();

    let instruction = Instruction::try_from(*discriminator)?;
    match instruction {
        Instruction::Initialize => process_initialize_order(accounts, instruction_data),
        Instruction::Cancel => process_cancel_order(accounts, instruction_data),
        _ => Err(SolverError::InvalidInstruction.into()),
    }
}
