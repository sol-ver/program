use pinocchio::{account_info::AccountInfo, program_error::ProgramError};
use pinocchio::pubkey::Pubkey;

use crate::utils::{try_from_account_info_mut, Initialized};

#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)] 
#[repr(C)]
pub struct Order {
    pub owner: Pubkey,
    pub sell_token: Pubkey,
    pub buy_token: Pubkey,
    pub sell_amount: u64,
    pub buy_amount: u64,
    pub referral_fee: u64,
    pub referral_account: Pubkey,

    pub rent_payer: Pubkey,
    pub is_initialized: u8,
    pub _padding: [u8; 7],
}

impl Initialized for Order {
    #[inline(always)]
    fn is_initialized(&self) -> bool {
        self.is_initialized > 0
    }
}

impl Order {
    #[inline(always)]
    pub fn init(
        state_account: &AccountInfo,
        owner: Pubkey,
        sell_token: Pubkey,
        buy_token: Pubkey,
        sell_amount: u64,
        buy_amount: u64,
        referral_fee: u64,
        referral_account: Pubkey,
        rent_payer: Pubkey,
    ) -> Result<(), ProgramError> {
        let state = unsafe { try_from_account_info_mut::<Order>(state_account) }?;
        state.is_initialized = 1;
        state.owner = owner;
        state.sell_token = sell_token;
        state.buy_token = buy_token;
        state.sell_amount = sell_amount;
        state.buy_amount = buy_amount;
        state.referral_fee = referral_fee;
        state.referral_account = referral_account;
        state.rent_payer = rent_payer;
        Ok(())
    }
}
