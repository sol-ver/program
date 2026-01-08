use crate::{error::SolverError, utils::Unpackable};
use bytemuck::{Pod, Zeroable};
use light_hasher::{Hasher, Keccak};
use pinocchio::{
    program_error::ProgramError,
    pubkey::{create_program_address, Pubkey},
};

#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct Order {
    pub from_token_account: Pubkey,
    pub to_token_account: Pubkey,
    pub sell_amount: u64,
    pub buy_amount: u64,
    pub referral_fee: u64,
    pub referral_token_account: Pubkey,
    pub minimun_buy_amount: u64,
    pub amount_decrease_per_second: u64,
    pub start_time: u64,
    pub deadline: u64,
}

impl Order {
    pub fn validate_and_unpack(
        data: &[u8],
        owner_key: &Pubkey,
        order_key: &Pubkey,
        order_bump: u8,
    ) -> Result<(Self, [u8; 32]), ProgramError> {
        // 2. Validate Order PDA
        let intent_hash = Keccak::hashv(&[data]).unwrap();
        // Use correct slice type for address generation
        let intent_hash_bytes = &intent_hash as &[u8];

        let calculated_order_pubkey = create_program_address(
            &[
                b"order",
                owner_key.as_ref(),
                intent_hash_bytes,
                &[order_bump],
            ],
            &crate::ID,
        )
        .unwrap();

        if &calculated_order_pubkey != order_key {
            return Err(SolverError::InvalidOrderAccount.into());
        }

        let order = Order::unpack(data).unwrap();

        Ok((order, intent_hash)) // Return the intent_hash for further use
    }

    pub fn calculate_current_buy_amount(&self, current_time: u64) -> u64 {
        // 1. If auction hasn't started, return the full starting buy_amount
        if current_time <= self.start_time {
            return self.buy_amount;
        }

        // 2. If auction has ended, return the floor (minimum)
        if current_time >= self.deadline {
            return self.minimun_buy_amount;
        }

        // 3. Calculate linear decay
        let total_duration = self.deadline.saturating_sub(self.start_time);
        let elapsed_time = current_time.saturating_sub(self.start_time);

        // Range of the auction price
        let total_decay_range = self.buy_amount.saturating_sub(self.minimun_buy_amount);

        // We calculate (Range * Elapsed) / Total to maintain precision with integers
        let reduction = (total_decay_range as u128)
            .saturating_mul(elapsed_time as u128)
            .saturating_div(total_duration as u128) as u64;

        self.buy_amount.saturating_sub(reduction)
    }

    pub fn validate_order_accounts(
        &self,
        from_token_account: &Pubkey,
        to_token_account: &Pubkey,
        referral_token_account: &Pubkey,
    ) -> bool {
        if from_token_account != &self.from_token_account {
            return false;
        }
        if to_token_account != &self.to_token_account {
            return false;
        }
        if self.referral_fee > 0 && referral_token_account != &self.referral_token_account {
            return false;
        }
        true
    }
}
