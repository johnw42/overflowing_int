use std::{marker::PhantomData, mem::ManuallyDrop, ops::Deref};

use crate::encoding::shifted::{Shiftable, Shifted};

#[derive(Debug, PartialEq, Eq)]
pub enum IntOrPtr<I, P> {
    Int(I),
    Ptr(P),
}

union IntOrPtrUnion<I, P>
where
    I: Copy,
{
    int: Shifted<I>,
    ptr: ManuallyDrop<P>,
}

/// A type that can store either an integer or a pointer.
///
/// SAFETY: This type is implemented using `unsafe` code, and the safety of its
/// operations relies on the following invariants:
/// 1. The alignment of the pointer type `P` must be greater than 1, which is
///    guaranteed by the compile-time assertion in the `IntOrPtrData` struct.
///    This ensures that the least significant bit of a valid pointer will
///    always be 0, allowing us to distinguish between integers and pointers
///    using the least significant bit.
/// 2. The integer value stored in the `int` field must always have the least
///    significant bit set to 1, which is guaranteed by the `Shifted` type.
/// 3. The integer type must be at least as large as a pointer, which is
///    guaranteed by the compile-time assertion in the `IntOrPtrData` struct.
///    Without this requirement, it would be possible for a pointer value to be
///    misinterpreted as an integer, which would violate the safety of the
///    operations on this type.
/// 4. The pointer type must be rerpresented internally as a single pointer,
///    which is (mostly) guaranteed by a compile-time assertion that `P` has the
///    same size as `*const T`.
/// 5. All access to the internal fields is gated by calling `Shifted::validate`
///    on the `int` field, which ensures that we only treat the value as an
///    integer if it is a valid shifted integer.  If `validate` returns `None`,
///    we know that the value is not a valid shifted integer, and therefore must
///    be a pointer.
pub struct IntOrPtrData<I, T, P>(IntOrPtrUnion<I, P>, PhantomData<T>)
where
    I: Shiftable,
    P: Deref<Target = T>;

impl<I, T, P> IntOrPtrData<I, T, P>
where
    I: Shiftable,
    P: Deref<Target = T>,
{
    // Compile-time assertions to meet the safety requirements of this type.
    const _ASSERTIONS: () = {
        assert!(std::mem::align_of::<T>() > 1);
        assert!(std::mem::size_of::<I>() >= std::mem::size_of::<P>());
        assert!(std::mem::size_of::<P>() == std::mem::size_of::<*const T>());
    };

    /// The zero value of this type.
    pub const ZERO: Self = Self(IntOrPtrUnion { int: Shifted::ZERO }, PhantomData);

    /// Creates a new `IntOrPtrData` from an integer, if it can be represented as
    /// such.  If the integer is too large to be represented as a shifted value,
    /// returns `None`.
    pub fn new_int(int: I) -> Option<Self> {
        let result = Self(
            IntOrPtrUnion {
                int: Shifted::new(int)?,
            },
            PhantomData,
        );
        debug_assert!(matches!(result.get(), IntOrPtr::Int(_)));
        Some(result)
    }

    /// Creates a new `IntOrPtrData` from a pointer.  The pointer must be
    /// properly aligned, but a compile-time assertion will catch misalignment.
    pub fn new_ptr(ptr: P) -> Self {
        let result = Self(
            IntOrPtrUnion {
                ptr: ManuallyDrop::new(ptr),
            },
            PhantomData,
        );
        debug_assert!(matches!(result.get(), IntOrPtr::Ptr(_)));
        result
    }

    /// Gets the value stored in this `IntOrPtrData`, returning either the integer
    /// or the pointer, depending on which one is stored.
    pub fn get(&self) -> IntOrPtr<I, &T> {
        unsafe {
            if let Some(int) = self.0.int.validate() {
                IntOrPtr::Int(int)
            } else {
                IntOrPtr::Ptr(&**self.0.ptr)
            }
        }
    }

    /// Gets the value stored in this `IntOrPtrData`, returning either the
    /// integer or the pointer, depending on which one is stored.  If the
    /// pointer is returned, it is returned as a mutable reference, and the
    /// caller is allowed to modify the value through the pointer.  This is safe
    /// because the caller is guaranteed to have exclusive access to the
    /// pointer.
    pub fn get_mut(&mut self) -> IntOrPtr<I, &mut P> {
        unsafe {
            if let Some(int) = self.0.int.validate() {
                IntOrPtr::Int(int)
            } else {
                IntOrPtr::Ptr(&mut *self.0.ptr)
            }
        }
    }

    /// Consumes this `IntOrPtrData`, returning either the integer or the pointer,
    /// depending on which one is stored.
    pub fn into_inner(mut self) -> IntOrPtr<I, P> {
        unsafe {
            if let Some(int) = self.0.int.validate() {
                IntOrPtr::Int(int)
            } else {
                let result = IntOrPtr::Ptr(ManuallyDrop::take(&mut self.0.ptr));
                // Prevent from trying to drop the pointer again in the `Drop`
                // impl, which would cause a double free.  Any non-pointer value
                // would work here.
                self.0.int = Shifted::ZERO;
                result
            }
        }
    }
}

impl<I, T, P> Clone for IntOrPtrData<I, T, P>
where
    I: Shiftable,
    P: Deref<Target = T> + Clone,
{
    fn clone(&self) -> Self {
        unsafe {
            if self.0.int.validate().is_some() {
                Self(IntOrPtrUnion { int: self.0.int }, PhantomData)
            } else {
                Self(
                    IntOrPtrUnion {
                        ptr: self.0.ptr.clone(),
                    },
                    PhantomData,
                )
            }
        }
    }
}

impl<I, T, P> Drop for IntOrPtrData<I, T, P>
where
    I: Shiftable,
    P: Deref<Target = T>,
{
    fn drop(&mut self) {
        unsafe {
            if self.0.int.validate().is_none() {
                ManuallyDrop::drop(&mut self.0.ptr);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestType = IntOrPtrData<usize, u128, Box<u128>>;

    #[test]
    fn test_new_int_get() {
        let int_data = TestType::new_int(42).unwrap();
        assert_eq!(int_data.get(), IntOrPtr::Int(42));
    }

    #[test]
    fn test_new_ptr_get() {
        let ptr_data = TestType::new_ptr(Box::new(42));
        assert_eq!(ptr_data.get(), IntOrPtr::Ptr(&42));
    }

    #[test]
    fn test_new_int_clone_get() {
        let int_data = TestType::new_int(42).unwrap().clone();
        assert_eq!(int_data.get(), IntOrPtr::Int(42));
    }

    #[test]
    fn test_new_ptr_clone_get() {
        let ptr_data = TestType::new_ptr(Box::new(42)).clone();
        assert_eq!(ptr_data.get(), IntOrPtr::Ptr(&42));
    }

    #[test]
    fn test_new_int_get_mut() {
        let mut int_data = TestType::new_int(42).unwrap();
        assert_eq!(int_data.get_mut(), IntOrPtr::Int(42));
    }

    #[test]
    #[allow(clippy::replace_box)]
    fn test_new_ptr_get_mut() {
        let boxed_int = Box::new(42);
        let int_ptr = boxed_int.as_ref() as *const _;
        let mut ptr_data = TestType::new_ptr(boxed_int);
        let IntOrPtr::Ptr(ptr_ref) = ptr_data.get_mut() else {
            panic!();
        };
        assert!(std::ptr::eq(ptr_ref.as_ref(), int_ptr));
        *ptr_ref = Box::new(43);
        assert_eq!(ptr_data.get(), IntOrPtr::Ptr(&43));
    }

    #[test]
    fn test_new_int_into_inner() {
        let int_data = TestType::new_int(42).unwrap();
        assert_eq!(int_data.into_inner(), IntOrPtr::Int(42));
    }

    #[test]
    #[allow(clippy::replace_box)]
    fn test_new_ptr_into_inner() {
        let boxed_int = Box::new(42);
        let int_ptr = boxed_int.as_ref() as *const _;
        let ptr_data = TestType::new_ptr(boxed_int);
        let IntOrPtr::Ptr(mut original_box) = ptr_data.into_inner() else {
            panic!();
        };
        assert!(std::ptr::eq(original_box.as_ref(), int_ptr));
        original_box = Box::new(43);
        let _ = original_box;
    }
}
