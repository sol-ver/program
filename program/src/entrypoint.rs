use crate::{
    error::SolverError,
    instruction::{
        cancel_order::process_cancel_order, initialize_order::process_initialize_order, Instruction,
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
    let [descriminator, instruction_data @ ..] = instruction_data else {
        msg!("Instruction data too short");
        return Err(SolverError::InvalidInstruction.into());
    };

    let instruction = Instruction::try_from(*descriminator)?;
    match instruction {
        Instruction::InitializeOrder => process_initialize_order(accounts, instruction_data),
        Instruction::CancelOrder => process_cancel_order(accounts, instruction_data),
        _ => Err(SolverError::InvalidInstruction.into()),
    }
}
