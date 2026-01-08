use crate::{
    error::SolverError,
    instruction::{
        execute_order::process_execute_order, initialize_order::process_initialize_order,
        Instruction,
    },
};
use pinocchio::{account_info::AccountInfo, entrypoint, msg, pubkey::Pubkey, ProgramResult};

entrypoint!(process_instruction);

#[inline(always)]
pub fn process_instruction(
    _: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [discriminator, instruction_data @ ..] = instruction_data else {
        msg!("Instruction data too short");
        return Err(SolverError::InvalidInstruction.into());
    };

    let instruction = Instruction::try_from(*discriminator)?;
    match instruction {
        Instruction::Initialize => process_initialize_order(accounts, instruction_data),
        Instruction::Execute => process_execute_order(accounts, instruction_data),
    }
}
