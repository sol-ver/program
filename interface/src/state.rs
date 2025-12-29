use pinocchio::program_error::ProgramError;
use pinocchio::pubkey::Pubkey;

#[repr(C)]
pub struct Order {
    pub is_initialized: bool,
    pub owner: Pubkey,
    pub sell_token: Pubkey,
    pub buy_token: Pubkey,
    pub sell_amount: u64,
    pub buy_amount: u64,
    pub fee_amount: u64,

    pub rent_payer: Pubkey
}

impl Order {
    pub const LEN: usize = core::mem::size_of::<Order>();

    #[inline(always)]
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    #[inline(always)]
    pub unsafe fn load_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(&mut *(data.as_mut_ptr() as *mut Self))
    }

    #[inline(always)]
    pub unsafe fn load(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(&*(data.as_ptr() as *const Self))
    }

    #[inline(always)]
    pub fn init(&mut self) {
        self.is_initialized = true;
    }
}