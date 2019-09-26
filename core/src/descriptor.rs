use std::marker::PhantomData;

pub struct FieldDescriptor<Parent: ?Sized, Type: ?Sized> {
    offset: usize,
    
    #[allow(clippy::type_complexity)]
    field: PhantomData<(*mut Parent, *mut Type)>,
}

unsafe impl<Parent: ?Sized, Type: ?Sized> Send for FieldDescriptor<Parent, Type> {}
unsafe impl<Parent: ?Sized, Type: ?Sized> Sync for FieldDescriptor<Parent, Type> {}

impl<Parent: ?Sized, Type: ?Sized> Copy for FieldDescriptor<Parent, Type> {}
impl<Parent: ?Sized, Type: ?Sized> Clone for FieldDescriptor<Parent, Type> { 
    fn clone(&self) -> Self { *self }
}

union Pointer<T: ?Sized, U: ?Sized> {
    fat_ptr: *const T,
    fat_ptr_mut: *mut T,
    fat_ptr_out: *const U,
    fat_ptr_out_mut: *mut U,
    ptr: *const u8,
    ptr_mut: *mut u8,
    int: usize
}

impl<Parent: ?Sized, Type: ?Sized> FieldDescriptor<Parent, Type> {
    pub const unsafe fn from_offset(offset: usize) -> Self {
        Self { offset, field: PhantomData }
    }

    // `from_pointers` relies on the layout of fat pointers,
    // 
    // * `Sized` types have no metadata, so they are fine
    //     * This will never change
    // * `[_]` have the pointer in the first `std::mem::size_of::<usize>()` bytes, so they are fine
    //     * This is subject to change in the future
    // * `dyn Trait` have the pointer in the first `std::mem::size_of::<usize>()` bytes, so they are fine
    //     * This is subject to change in the future
    pub const unsafe fn from_pointers(parent: *mut Parent, field: *mut Type) -> Self {
        let parent = Pointer::<_, ()> { fat_ptr_mut: parent };
        let field = Pointer::<_, ()> { fat_ptr_mut: field };
        
        Self::from_offset(field.int - parent.int)
    }

    // `project_raw_unchecked` relies on the layout of fat pointers,
    // 
    // * `Sized` types have no metadata, so they are fine
    //     * This will never change
    // * `[_]` have the pointer in the first `std::mem::size_of::<usize>()` bytes, so they are fine
    //     * This is subject to change in the future
    // * `dyn Trait` have the pointer in the first `std::mem::size_of::<usize>()` bytes, so they are fine
    //     * This is subject to change in the future
    pub unsafe fn project_raw_unchecked(self, parent: *const Parent) -> *const Type {
        let mut pointer = Pointer { fat_ptr: parent };

        // offset in bytes
        pointer.ptr = pointer.ptr.add(self.offset);

        pointer.fat_ptr_out
    }

    // `project_raw_mut_unchecked` relies on the layout of fat pointers,
    // 
    // * `Sized` types have no metadata, so they are fine
    //     * This will never change
    // * `[_]` have the pointer in the first `std::mem::size_of::<usize>()` bytes, so they are fine
    //     * This is subject to change in the future
    // * `dyn Trait` have the pointer in the first `std::mem::size_of::<usize>()` bytes, so they are fine
    //     * This is subject to change in the future
    pub unsafe fn project_raw_mut_unchecked(self, parent: *mut Parent) -> *mut Type {
        let mut pointer = Pointer { fat_ptr_mut: parent };

        // offset in bytes
        pointer.ptr_mut = pointer.ptr_mut.add(self.offset);

        pointer.fat_ptr_out_mut
    }

    // `project_raw` relies on the layout of fat pointers,
    // 
    // * `Sized` types have no metadata, so they are fine
    //     * This will never change
    // * `[_]` have the pointer in the first `std::mem::size_of::<usize>()` bytes, so they are fine
    //     * This is subject to change in the future
    // * `dyn Trait` have the pointer in the first `std::mem::size_of::<usize>()` bytes, so they are fine
    //     * This is subject to change in the future
    pub fn project_raw(self, parent: *const Parent) -> *const Type {
        unsafe {
            let mut pointer = Pointer { fat_ptr: parent };

            // offset in bytes
            pointer.ptr = pointer.ptr.wrapping_add(self.offset);

            pointer.fat_ptr_out
        }
    }

    // `project_raw_mut` relies on the layout of fat pointers,
    // 
    // * `Sized` types have no metadata, so they are fine
    //     * This will never change
    // * `[_]` have the pointer in the first `std::mem::size_of::<usize>()` bytes, so they are fine
    //     * This is subject to change in the future
    // * `dyn Trait` have the pointer in the first `std::mem::size_of::<usize>()` bytes, so they are fine
    //     * This is subject to change in the future
    pub fn project_raw_mut(self, parent: *mut Parent) -> *mut Type {
        unsafe {
            let mut pointer = Pointer { fat_ptr_mut: parent };

            // offset in bytes
            pointer.ptr_mut = pointer.ptr_mut.wrapping_add(self.offset);

            pointer.fat_ptr_out_mut
        }
    }
}