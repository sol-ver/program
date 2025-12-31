use crate::error::SolverError;
use bytemuck::{try_from_bytes, Pod};
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{find_program_address, Pubkey},
};

pub trait DataLen {
    const LEN: usize;
}

impl<T: Pod> DataLen for T {
    const LEN: usize = core::mem::size_of::<T>();
}

pub trait Unpackable: Pod {
    fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(bytemuck::try_pod_read_unaligned(data).unwrap())
    }
}

impl<T: Pod> Unpackable for T {}

pub trait Initialized {
    fn is_initialized(&self) -> bool;
}

#[inline(always)]
pub unsafe fn load_acc<T: DataLen + Initialized>(bytes: &[u8]) -> Result<&T, ProgramError> {
    load_acc_unchecked::<T>(bytes).and_then(|acc| {
        if acc.is_initialized() {
            Ok(acc)
        } else {
            Err(ProgramError::UninitializedAccount)
        }
    })
}

#[inline(always)]
pub unsafe fn load_acc_unchecked<T: DataLen>(bytes: &[u8]) -> Result<&T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(&*(bytes.as_ptr() as *const T))
}

#[inline(always)]
pub unsafe fn load_acc_mut<T: DataLen + Initialized>(
    bytes: &mut [u8],
) -> Result<&mut T, ProgramError> {
    load_acc_mut_unchecked::<T>(bytes).and_then(|acc| {
        if acc.is_initialized() {
            Ok(acc)
        } else {
            Err(ProgramError::UninitializedAccount)
        }
    })
}

#[inline(always)]
pub unsafe fn load_acc_mut_unchecked<T: DataLen>(bytes: &mut [u8]) -> Result<&mut T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(&mut *(bytes.as_mut_ptr() as *mut T))
}

#[inline(always)]
pub unsafe fn load_ix_data<T: DataLen>(bytes: &[u8]) -> Result<&T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(SolverError::InvalidInstructionData.into());
    }
    Ok(&*(bytes.as_ptr() as *const T))
}

pub unsafe fn to_bytes<T: DataLen>(data: &T) -> &[u8] {
    core::slice::from_raw_parts(data as *const T as *const u8, T::LEN)
}

pub unsafe fn to_mut_bytes<T: DataLen>(data: &mut T) -> &mut [u8] {
    core::slice::from_raw_parts_mut(data as *mut T as *mut u8, T::LEN)
}

pub unsafe fn try_from_account_info<T: DataLen>(acc: &AccountInfo) -> Result<&T, ProgramError> {
    if acc.owner() != &crate::ID {
        return Err(ProgramError::IllegalOwner);
    }
    let bytes = acc.try_borrow_data()?;

    if bytes.len() != T::LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(&*(bytes.as_ptr() as *const T))
}

pub unsafe fn try_from_account_info_mut<T: DataLen>(
    acc: &AccountInfo,
) -> Result<&mut T, ProgramError> {
    if acc.owner() != &crate::ID {
        return Err(ProgramError::IllegalOwner);
    }

    let mut bytes = acc.try_borrow_mut_data()?;

    if bytes.len() != T::LEN {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(&mut *(bytes.as_mut_ptr() as *mut T))
}

#[inline(always)]
pub fn find_order_address(owner: &Pubkey, order_nonce: &[u8; 8]) -> (Pubkey, u8) {
    find_program_address(
        &[b"order", owner.as_ref(), order_nonce.as_ref()],
        &crate::ID,
    )
}
