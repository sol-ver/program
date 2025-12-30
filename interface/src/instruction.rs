use pinocchio::program_error::ProgramError;

#[repr(u8)]
#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    /// Initializes a new order account
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[writable, signer]` The order account to initialize
    /// 1. `[signer]` The owner of the order account
    /// 2. `[signer]` payer of the account creation rent
    /// 1. `[]` The rent sysvar
    InitOrder,

    /// Cancle an existing order
    /// Accounts expected by this instruction:
    /// 0. `[writable]` The order account to cancle
    /// 1. `[signer]` The owner of the order account
    CancleOrder,

    /// Settle an existing order
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[writable]` The order account to fulfill
    /// 1. `[]` The owner of the order account
    SettleOrder,
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