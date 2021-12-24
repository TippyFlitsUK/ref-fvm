use crate::sys;
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;
// no_std
use crate::error::{IntoSyscallResult, SyscallResult};
use fvm_shared::encoding::{from_slice, RawBytes, DAG_CBOR};
use fvm_shared::error::ExitCode::ErrIllegalArgument;
use fvm_shared::receipt::Receipt;
use fvm_shared::MethodNum;

/// Sends a message to another actor.
pub fn send(
    to: &Address,
    method: MethodNum,
    params: RawBytes,
    value: TokenAmount,
) -> SyscallResult<Receipt> {
    let recipient = to.to_bytes();
    let mut value_iter = value.iter_u64_digits();
    let value_lo = value_iter.next().unwrap();
    let value_hi = value_iter.next().unwrap_or(0);
    if value_iter.next().is_some() {
        return Err(ErrIllegalArgument);
    };
    unsafe {
        // Send the message.
        let params_id = sys::ipld::create(DAG_CBOR, params.as_ptr(), params.len() as u32)
            .into_syscall_result()?;
        let id = sys::send::send(
            recipient.as_ptr(),
            recipient.len() as u32,
            method,
            params_id,
            value_hi,
            value_lo,
        )
        .into_syscall_result()?;
        // Allocate a buffer to read the result.
        let (_, length) = sys::ipld::stat(id).into_syscall_result()?;
        let mut bytes = Vec::with_capacity(length as usize);
        // Now read the result.
        let read = sys::ipld::read(id, 0, bytes.as_mut_ptr(), length).into_syscall_result()?;
        assert_eq!(read, length);
        // Deserialize the receipt.
        let ret = from_slice::<Receipt>(bytes.as_slice()).expect("invalid receipt returned");
        Ok(ret)
    }
}