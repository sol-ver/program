use pinocchio::{ProgramResult, account_info::AccountInfo, entrypoint, msg, no_allocator, nostd_panic_handler, pubkey::Pubkey};

entrypoint!(process_instruction);
nostd_panic_handler!();

#[inline(always)]
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Hello, Intent Solana!");

    Ok(())
}