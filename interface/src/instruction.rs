use pinocchio::program_error::ProgramError;

#[repr(u8)]
#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    /// Initializes a new order account
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[writable, signer]` The order account to initialize
    /// 1. `[]` The rent sysvar
    InitOrder,

    /// Settle an existing order
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[writable]` The order account to fulfill
    /// 1. `[]` The owner of the order account
    SettleOrder,
}