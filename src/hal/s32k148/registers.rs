use core::ptr::{read_volatile, write_volatile};

/// Register access traits
pub trait ReadOnly {
    type Type;
    
    /// Read from the register
    fn read(&self) -> Self::Type;
}

pub trait WriteOnly {
    type Type;
    
    /// Write to the register
    fn write(&self, value: Self::Type);
}

pub trait ReadWrite: ReadOnly + WriteOnly {}

/// Register definition
#[repr(transparent)]
pub struct Register<T, A> {
    /// Memory address of the register
    address: *mut T,
    
    /// Register access type (read-only, write-only, read-write)
    _access: core::marker::PhantomData<A>,
}

/// Read-only register implementation
impl<T: Copy> ReadOnly for Register<T, ReadOnlyAccess> {
    type Type = T;
    
    fn read(&self) -> T {
        unsafe { read_volatile(self.address) }
    }
}

/// Write-only register implementation
impl<T> WriteOnly for Register<T, WriteOnlyAccess> {
    type Type = T;
    
    fn write(&self, value: T) {
        unsafe { write_volatile(self.address, value) }
    }
}

/// Read-write register implementation
impl<T: Copy> ReadOnly for Register<T, ReadWriteAccess> {
    type Type = T;
    
    fn read(&self) -> T {
        unsafe { read_volatile(self.address) }
    }
}

impl<T> WriteOnly for Register<T, ReadWriteAccess> {
    type Type = T;
    
    fn write(&self, value: T) {
        unsafe { write_volatile(self.address, value) }
    }
}

impl<T: Copy> ReadWrite for Register<T, ReadWriteAccess> {}

/// Register access type markers
pub struct ReadOnlyAccess;
pub struct WriteOnlyAccess;
pub struct ReadWriteAccess;