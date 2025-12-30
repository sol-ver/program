use pinocchio::pubkey::Pubkey;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

use crate::utils::{try_from_account_info, try_from_account_info_mut, DataLen, Initialized};

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Order {
    pub is_initialized: bool,
    pub owner: Pubkey,
    pub sell_token: Pubkey,
    pub buy_token: Pubkey,
    pub sell_amount: u64,
    pub buy_amount: u64,
    pub referral_fee: u64,
    pub referral_account: Pubkey,
    pub rent_payer: Pubkey,
}

impl Initialized for Order {
    #[inline(always)]
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl DataLen for Order {
    const LEN: usize = core::mem::size_of::<Order>();
}

impl Order {
    #[inline(always)]
    pub fn init(
        &mut self,
        owner: Pubkey,
        sell_token: Pubkey,
        buy_token: Pubkey,
        sell_amount: u64,
        buy_amount: u64,
        referral_fee: u64,
        referral_account: Pubkey,
        rent_payer: Pubkey,
    ) -> Result<(), ProgramError> {
        self.is_initialized = true;
        self.owner = owner;
        self.sell_token = sell_token;
        self.buy_token = buy_token;
        self.sell_amount = sell_amount;
        self.buy_amount = buy_amount;
        self.referral_fee = referral_fee;
        self.referral_account = referral_account;
        self.rent_payer = rent_payer;
        Ok(())
    }

    /// Reads the data from the account.
    /// This returns a reference to the data, so it does not allocate new memory (Zero Copy).
    #[inline(always)]
    pub fn load(account: &AccountInfo) -> Result<&Self, ProgramError> {
        unsafe { try_from_account_info::<Order>(account) }
    }

    /// Writes/Modifies the data in the account.
    /// This returns a mutable reference to the data. Any changes made to the returned struct
    /// are directly applied to the account's data buffer (Zero Copy).
    #[inline(always)]
    pub fn load_mut(account: &AccountInfo) -> Result<&mut Self, ProgramError> {
        unsafe { try_from_account_info_mut::<Order>(account) }
    }
}
