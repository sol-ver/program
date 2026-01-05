use pinocchio::program_error::ProgramError;

pub mod cancel_order;
pub mod initialize_order;

#[repr(u8)]
pub enum Instruction {
    Initialize,
    Cancel,
    Execute,
}

impl TryFrom<u8> for Instruction {
    type Error = ProgramError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Instruction::Initialize),
            1 => Ok(Instruction::Cancel),
            2 => Ok(Instruction::Execute),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
