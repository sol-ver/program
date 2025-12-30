use pinocchio::program_error::ProgramError;

pub mod initialize_order;

#[repr(u8)]
pub enum Instruction {
    InitializeOrder,
    CancelOrder,
    ExecuteOrder,
}

impl TryFrom<u8> for Instruction {
    type Error = ProgramError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Instruction::InitializeOrder),
            1 => Ok(Instruction::CancelOrder),
            2 => Ok(Instruction::ExecuteOrder),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
