use pinocchio::program_error::{ProgramError, ToStr};

/// Errors that may be returned by the Token program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SolverError {
    /// Invalid Instruction
    InvalidInstruction,
}

impl From<SolverError> for ProgramError {
    fn from(e: SolverError) -> Self {
        ProgramError::Custom(e as u32)
    }
}


impl ToStr for SolverError {
    fn to_str<E>(&self) -> &'static str
    where
        E: 'static + ToStr + TryFrom<u32>,
    {
        match self {
            SolverError::InvalidInstruction => "Invalid instruction",
        }
    }
}

impl TryFrom<u32> for SolverError {
    type Error = ProgramError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            x if x == SolverError::InvalidInstruction as u32 => Ok(SolverError::InvalidInstruction),
            _ => Err(ProgramError::Custom(value)),
        }
    }
}