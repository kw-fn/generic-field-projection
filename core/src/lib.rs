#![feature(const_fn_union, const_fn, specialization)]
// #![forbid(missing_docs)]
#![no_std]

/*!
This crate provides a generic interface to project to fields, think of it as an extended version
of `Deref` that handles all pointer types equally.
*/

mod project;
mod pin;
#[doc(hidden)]
pub mod macros;
mod chain;
mod set;

pub use self::pin::*;
pub use self::chain::*;
pub use self::set::FieldSet;
pub use gfp_derive::Field;

pub(crate) use self::set::tuple::*;

#[doc(hidden)]
pub mod derive {
    pub use core::marker::PhantomData;
    pub use core::iter::{Once, once};

    pub struct Invariant<T: ?Sized>(pub PhantomData<*mut T>);

    unsafe impl<T: ?Sized> Send for Invariant<T> {}
    unsafe impl<T: ?Sized> Sync for Invariant<T> {}
    
    impl<T: ?Sized> Clone for Invariant<T> { fn clone(&self) -> Self { *self } }
    impl<T: ?Sized> Copy for Invariant<T> {}
}

/// Projects a type to the given field
pub trait ProjectTo<F: Field> {
    /// The projection of the type, can be used to directly access the field
    type Projection;

    /// projects to the given field
    fn project_to(self, field: F) -> Self::Projection;
}

/// Projects a type to the given field
pub trait ProjectToSet<F: FieldSet> {
    /// The projection of the type, can be used to directly access the field
    type Projection;

    /// projects to the given field
    fn project_set_to(self, field: F) -> Self::Projection;
}

/// Represents a field of some `Parent` type
/// You can derive this trait for all fields of `Parent` by using
/// 
/// ```rust
/// # mod __ {
/// use gfp_core::Field;
/// 
/// #[derive(Field)]
/// struct Foo {
///     bar: u32,
///     val: String
/// }
/// # }
/// ```
/// 
/// # Safety
/// 
/// * `Parent` must represent the type where the field came from
/// * `Type` must represent the type of the field itself
/// * `project_raw` and `project_raw_mut` must only access the given field
/// * `name` must return an iterator that yields all of the fields from `Parent` to the given field,
/// 
/// ex.
/// 
/// ```rust
/// struct Foo {
///     bar: Bar
/// }
/// 
/// struct Bar {
///     tap: Tap
/// }
/// 
/// struct Tap {
///     val: u32
/// }
/// ```
/// 
/// if want to get field `val` from `Foo`,
/// 
/// you must implement field like so,
/// 
/// ```rust
/// # struct Foo {
/// #     bar: Bar
/// # }
/// # 
/// # struct Bar {
/// #     tap: Tap
/// # }
/// # 
/// # struct Tap {
/// #     val: u32
/// # }
/// use gfp_core::Field;
/// 
/// struct FieldVal;
/// 
/// unsafe impl Field for FieldVal {
///     type Parent = Foo;
///     type Type = u32;
///     type Name = std::iter::Copied<std::slice::Iter<'static, &'static str>>;
///     
///     fn name(&self) -> Self::Name {
///         ["bar", "tap", "val"].iter().copied()
///     }
///     
///     unsafe fn project_raw(&self, ptr: *const Self::Parent) -> *const Self::Type {
///         &(*ptr).bar.tap.val
///     }
///     
///     unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent) -> *mut Self::Type {
///         &mut (*ptr).bar.tap.val
///     }
/// }
/// ```
/// 
/// But it is better to just derive `Field` on the types that you need to, and then use the [`chain`](trait.Fields.html#method.chain)
/// combinator to project to the fields of fields
/// 
/// ```rust
/// # mod main {
/// use gfp_core::Field;
/// 
/// #[derive(Field)]
/// struct Foo {
///     bar: Bar
/// }
/// 
/// #[derive(Field)]
/// struct Bar {
///     tap: Tap
/// }
/// 
/// #[derive(Field)]
/// struct Tap {
///     val: u32
/// }
/// 
/// fn main() {
///     let foo_to_val = Foo::fields().bar.chain(
///         Bar::fields().tap
///     ).chain(
///         Tap::fields().val
///     );
/// 
///     // or if you are going to use that projection a lot
///     
///     use gfp_core::Chain;
///     
///     const FOO_TO_VAL: Chain<Chain<Foo_fields::bar::<Foo>, Bar_fields::tap::<Bar>>, Tap_fields::val::<Tap>> = Chain::new(
///         Chain::new(
///             Foo::FIELDS.bar,
///             Bar::FIELDS.tap,
///         ),
///         Tap::FIELDS.val,
///     );
/// }
/// # }
/// ```
/// 
pub unsafe trait Field {
    /// The type that the field comes from
    type Parent: ?Sized;

    /// The type of the field itself
    type Type: ?Sized;

    /// An iterator that returns the fuully qualified name of the field
    type Name: Iterator<Item = &'static str>;

    /// An iterator that returns the fully qualified name of the field
    /// 
    /// This must be unique for each field of the given `Parent` type
    fn name(&self) -> Self::Name;

    /// projects the raw pointer from the `Parent` type to the field `Type`
    /// 
    /// # Safety
    /// 
    /// * `ptr` must point to a valid, initialized allocation of `Parent`
    /// * the projection is not safe to write to
    unsafe fn project_raw(&self, ptr: *const Self::Parent) -> *const Self::Type;
    
    /// projects the raw pointer from the `Parent` type to the field `Type`
    /// 
    /// # Safety
    /// 
    /// `ptr` must point to a valid, initialized allocation of `Parent`
    unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent) -> *mut Self::Type;

    /// Chains the projection of this field with another field `F`
    fn chain<F: Field<Parent = Self::Type>>(self, f: F) -> Chain<Self, F> where Self: Sized {
        Chain::new(self, f)
    }
}

unsafe impl<F: ?Sized + Field> Field for &F {
    type Parent = F::Parent;
    type Type = F::Type;
    type Name = F::Name;

    #[inline]
    fn name(&self) -> Self::Name {
        F::name(self)
    }
    
    #[inline]
    unsafe fn project_raw(&self, ptr: *const Self::Parent) -> *const Self::Type {
        F::project_raw(self, ptr)
    }
    
    #[inline]
    unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent) -> *mut Self::Type {
        F::project_raw_mut(self, ptr)
    }
}

unsafe impl<F: ?Sized + Field> Field for &mut F {
    type Parent = F::Parent;
    type Type = F::Type;
    type Name = F::Name;

    #[inline]
    fn name(&self) -> Self::Name {
        F::name(self)
    }
 
    #[inline]   
    unsafe fn project_raw(&self, ptr: *const Self::Parent) -> *const Self::Type {
        F::project_raw(self, ptr)
    }
    
    #[inline]
    unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent) -> *mut Self::Type {
        F::project_raw_mut(self, ptr)
    }
}
