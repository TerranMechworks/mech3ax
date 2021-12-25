use mech3ax_common::PeError as Error;

type Result<T> = ::std::result::Result<T, Error>;

/// Types whose values can be initialized from bytes, regardless of the bit
/// patterns in the bytes.
///
/// This is quite subtle. Types must be at least Copy, so that std::ptr::read's
/// requirements are upheld. Copy is however not sufficient. For example, a
/// structure containing a bool wouldn't be able to implement this trait, since
/// only 0 or 1 are valid bit patterns for bool. The same is true for padding.
pub unsafe trait FromU8: Copy {}

pub trait StructAt {
    fn struct_at<S: FromU8>(&self, offset: usize) -> Result<S>;
}

impl StructAt for &[u8] {
    fn struct_at<S: FromU8>(&self, offset: usize) -> Result<S> {
        let size = std::mem::size_of::<S>();
        let end = offset
            .checked_add(size)
            .ok_or(Error::ReadOutOfBounds(offset))?;
        if end > self.len() {
            return Err(Error::ReadOutOfBounds(end));
        }
        // SAFETY:
        // > The caller must ensure that the slice outlives the pointer this
        // > function returns [...].
        // The slice is valid for the lifetime of the function; and the
        // pointer's lifetime is shorter than the function's.
        //
        // > The caller must also ensure that the memory the pointer
        // > (non-transitively) points to is never written to (except inside an
        // > UnsafeCell) using this pointer or any pointer derived from it.
        // Consider it done.
        //
        // > Modifying the container referenced by this slice may cause its
        // > buffer to be reallocated, which would also make any pointers to it
        // > invalid.
        // Since we hold a reference to the slice, it shouldn't be possible to
        // modify the data during the function. The data is copied before the
        // function returns.
        let base_ptr = self.as_ptr();
        // SAFETY:
        // > If any of the following conditions are violated, the result is
        // > Undefined Behavior:
        // > * Both the starting and resulting pointer must be either in bounds
        // >   or one byte past the end of the same allocated object.
        // The assert checks that this is the case (hopefully).
        // > * The computed offset, in bytes, cannot overflow an isize.
        // Since the standard library avoids allocating over isize::MAX, and
        // T is a u8 and so is one byte in size, the above assert whether the
        // end offset is smaller than the length of the slice should also cover
        // this case.
        // > * The offset being in bounds cannot rely on "wrapping around" the
        // > address space.
        // The checked_add should cover this also.
        let struct_ptr = unsafe { base_ptr.add(offset) };
        // This should be safe because of the end offset assert (?)
        let src = struct_ptr as *const _;
        // SAFETY:
        // > Behavior is undefined if any of the following conditions are
        // > violated:
        // > * src must be valid for reads.
        // The pointer should not be null, since it points to the slice. It
        // also should be dereferenceable.
        // > * src must point to a properly initialized value of type T.
        // > Like read, read_unaligned creates a bitwise copy of T, regardless
        // > of whether T is Copy. If T is not Copy, using both the returned
        // > value and the value at *src can violate memory safety.
        // The FromU8 trait bound requires implementers to guarantee that any
        // sequence of bytes is a valid bit pattern.
        Ok(unsafe { std::ptr::read_unaligned(src) })
    }
}
