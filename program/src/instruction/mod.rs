use bytemuck::Pod;
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
            0..=2 => Ok(unsafe { core::mem::transmute::<u8, Instruction>(value) }),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}