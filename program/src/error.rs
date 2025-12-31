use pinocchio::program_error::{ProgramError, ToStr};

/// Errors that may be returned by the Token program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SolverError {
    /// Invalid Instruction
    InvalidInstruction,
    InvalidInstructionData,
    OrderAccountMustBeMut,
    InvalidOrderAccount,
    InvalidTokenAccountOwner,
    InvalidTokenAccountMint,
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
            SolverError::InvalidInstructionData => "Invalid instruction data",
            SolverError::OrderAccountMustBeMut => "Order account must be mutable",
            SolverError::InvalidOrderAccount => "Invalid order account",
            SolverError::InvalidTokenAccountOwner => "Invalid token account owner",
            SolverError::InvalidTokenAccountMint => "Invalid token account mint",
        }
    }
}

impl TryFrom<u32> for SolverError {
    type Error = ProgramError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            x if x == SolverError::InvalidInstruction as u32 => Ok(SolverError::InvalidInstruction),
            x if x == SolverError::InvalidInstructionData as u32 => {
                Ok(SolverError::InvalidInstructionData)
            }
            x if x == SolverError::OrderAccountMustBeMut as u32 => {
                Ok(SolverError::OrderAccountMustBeMut)
            }
            x if x == SolverError::InvalidOrderAccount as u32 => {
                Ok(SolverError::InvalidOrderAccount)
            }
            x if x == SolverError::InvalidTokenAccountOwner as u32 => {
                Ok(SolverError::InvalidTokenAccountOwner)
            }
            x if x == SolverError::InvalidTokenAccountMint as u32 => {
                Ok(SolverError::InvalidTokenAccountMint)
            }
            _ => Err(ProgramError::Custom(value)),
        }
    }
}
