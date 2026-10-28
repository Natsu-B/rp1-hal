use core::{cell::UnsafeCell, marker::PhantomData, ptr};

pub trait RegisterSpec {
    type Ux: Copy;
}

pub trait Readable: RegisterSpec {}
pub trait Modifiable: Readable {}
pub trait WriteOneToClear: RegisterSpec {}
pub trait WriteOneToSet: RegisterSpec {}
pub trait WriteOneToToggle: RegisterSpec {}
pub trait CommandWritable: RegisterSpec {}
pub trait FifoPush: RegisterSpec {}
pub trait FifoPop: RegisterSpec {}
pub trait UnsafeWritable: RegisterSpec {}

pub trait ResetValue: RegisterSpec<Ux = u32> {
    const RESET_VALUE: u32;
}

pub trait Resettable: ResetValue {}

pub trait TypedAtomicSet: RegisterSpec<Ux = u32> {
    const OFFSET: usize;
}
pub trait TypedAtomicClear: RegisterSpec<Ux = u32> {
    const OFFSET: usize;
}
pub trait TypedAtomicXor: RegisterSpec<Ux = u32> {
    const OFFSET: usize;
}
pub trait UnsafeAtomicSet: RegisterSpec<Ux = u32> {
    const OFFSET: usize;
}
pub trait UnsafeAtomicClear: RegisterSpec<Ux = u32> {
    const OFFSET: usize;
}
pub trait UnsafeAtomicXor: RegisterSpec<Ux = u32> {
    const OFFSET: usize;
}

#[repr(transparent)]
pub struct Reg<SPEC: RegisterSpec<Ux = u32>> {
    register: UnsafeCell<u32>,
    _spec: PhantomData<SPEC>,
}

unsafe impl<SPEC: RegisterSpec<Ux = u32>> Send for Reg<SPEC> {}

impl<SPEC: RegisterSpec<Ux = u32>> Reg<SPEC> {
    #[inline(always)]
    fn address(&self) -> *mut u32 {
        self.register.get()
    }

    #[inline(always)]
    unsafe fn write_at_offset(&self, offset: usize, bits: u32) {
        unsafe { ptr::write_volatile(self.address().byte_add(offset), bits) };
    }
}

impl<SPEC: Readable<Ux = u32>> Reg<SPEC> {
    #[inline(always)]
    pub fn read(&self) -> R<SPEC> {
        R {
            bits: unsafe { ptr::read_volatile(self.address()) },
            _spec: PhantomData,
        }
    }
}

impl<SPEC: Modifiable<Ux = u32>> Reg<SPEC> {
    #[inline(always)]
    pub fn modify(&self, f: impl for<'a> FnOnce(&R<SPEC>, &'a mut W<SPEC>) -> &'a mut W<SPEC>) {
        let bits = self.read().bits;
        let read = R {
            bits,
            _spec: PhantomData,
        };
        let mut write = W::from_bits(bits);
        f(&read, &mut write);
        unsafe { ptr::write_volatile(self.address(), write.bits) };
    }
}

impl<SPEC: Resettable<Ux = u32>> Reg<SPEC> {
    #[inline(always)]
    pub fn reset(&self) {
        unsafe { ptr::write_volatile(self.address(), SPEC::RESET_VALUE) };
    }
}

impl<SPEC: WriteOneToClear<Ux = u32>> Reg<SPEC> {
    #[inline(always)]
    pub fn clear(&self, f: impl FnOnce(&mut ClearW<SPEC>) -> &mut ClearW<SPEC>) {
        let mut value = ClearW::from_bits(0);
        f(&mut value);
        unsafe { ptr::write_volatile(self.address(), value.bits) };
    }
}

impl<SPEC: WriteOneToSet<Ux = u32>> Reg<SPEC> {
    #[inline(always)]
    pub fn set(&self, f: impl FnOnce(&mut SetW<SPEC>) -> &mut SetW<SPEC>) {
        let mut value = SetW::from_bits(0);
        f(&mut value);
        unsafe { ptr::write_volatile(self.address(), value.bits) };
    }
}

impl<SPEC: WriteOneToToggle<Ux = u32>> Reg<SPEC> {
    #[inline(always)]
    pub fn toggle(&self, f: impl FnOnce(&mut ToggleW<SPEC>) -> &mut ToggleW<SPEC>) {
        let mut value = ToggleW::from_bits(0);
        f(&mut value);
        unsafe { ptr::write_volatile(self.address(), value.bits) };
    }
}

impl<SPEC: CommandWritable<Ux = u32>> Reg<SPEC> {
    #[inline(always)]
    pub fn command(&self, f: impl FnOnce(&mut CommandW<SPEC>) -> &mut CommandW<SPEC>) {
        let mut value = CommandW::from_bits(0);
        f(&mut value);
        unsafe { ptr::write_volatile(self.address(), value.bits) };
    }
}

impl<SPEC: FifoPush<Ux = u32>> Reg<SPEC> {
    #[inline(always)]
    pub fn push(&self, value: u32) {
        unsafe { ptr::write_volatile(self.address(), value) };
    }
}

impl<SPEC: FifoPop<Ux = u32>> Reg<SPEC> {
    #[inline(always)]
    pub fn pop(&self) -> u32 {
        unsafe { ptr::read_volatile(self.address()) }
    }
}

impl<SPEC: UnsafeWritable<Ux = u32>> Reg<SPEC> {
    /// # Safety
    /// The caller must preserve reserved bits and obey all register-specific semantics.
    #[inline(always)]
    pub unsafe fn write_bits_unchecked(&self, bits: u32) {
        unsafe { ptr::write_volatile(self.address(), bits) };
    }
}

macro_rules! typed_atomic {
    ($trait_:ident, $method:ident, $writer:ident) => {
        impl<SPEC: $trait_<Ux = u32>> Reg<SPEC> {
            #[inline(always)]
            pub fn $method(&self, f: impl FnOnce(&mut $writer<SPEC>) -> &mut $writer<SPEC>) {
                let mut value = $writer::from_bits(0);
                f(&mut value);
                unsafe { self.write_at_offset(SPEC::OFFSET, value.bits) };
            }
        }
    };
}

typed_atomic!(TypedAtomicSet, atomic_set, AtomicSetW);
typed_atomic!(TypedAtomicClear, atomic_clear, AtomicClearW);
typed_atomic!(TypedAtomicXor, atomic_xor, AtomicXorW);

macro_rules! unsafe_atomic {
    ($trait_:ident, $method:ident) => {
        impl<SPEC: $trait_<Ux = u32>> Reg<SPEC> {
            /// Performs a raw atomic alias write.
            ///
            /// # Safety
            ///
            /// `mask` may select reserved, inferred, command, or otherwise unsafe bits. The
            /// caller must prove every selected bit is valid for this register and operation.
            /// Prefer the typed atomic writer whenever possible.
            #[inline(always)]
            pub unsafe fn $method(&self, mask: u32) {
                unsafe { self.write_at_offset(SPEC::OFFSET, mask) };
            }
        }
    };
}

unsafe_atomic!(UnsafeAtomicSet, atomic_set_bits_unchecked);
unsafe_atomic!(UnsafeAtomicClear, atomic_clear_bits_unchecked);
unsafe_atomic!(UnsafeAtomicXor, atomic_xor_bits_unchecked);

pub struct R<SPEC: RegisterSpec<Ux = u32>> {
    bits: u32,
    _spec: PhantomData<SPEC>,
}

impl<SPEC: RegisterSpec<Ux = u32>> R<SPEC> {
    #[inline(always)]
    pub const fn bits(&self) -> u32 {
        self.bits
    }
}

pub trait WriterBits {
    fn bits_value(&self) -> u32;

    /// # Safety
    /// This bypasses generated field visibility and register semantics.
    unsafe fn bits_mut_unchecked(&mut self) -> &mut u32;
}

macro_rules! writer_type {
    ($name:ident) => {
        pub struct $name<SPEC: RegisterSpec<Ux = u32>> {
            bits: u32,
            _spec: PhantomData<SPEC>,
        }

        impl<SPEC: RegisterSpec<Ux = u32>> $name<SPEC> {
            #[inline(always)]
            fn from_bits(bits: u32) -> Self {
                Self {
                    bits,
                    _spec: PhantomData,
                }
            }

            #[inline(always)]
            pub const fn bits_value(&self) -> u32 {
                self.bits
            }

            /// # Safety
            /// The caller must preserve reserved bits and obey register-specific semantics.
            #[inline(always)]
            pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
                self.bits = bits;
                self
            }
        }

        impl<SPEC: RegisterSpec<Ux = u32>> WriterBits for $name<SPEC> {
            #[inline(always)]
            fn bits_value(&self) -> u32 {
                self.bits
            }

            #[inline(always)]
            unsafe fn bits_mut_unchecked(&mut self) -> &mut u32 {
                &mut self.bits
            }
        }
    };
}

writer_type!(W);
writer_type!(ClearW);
writer_type!(SetW);
writer_type!(ToggleW);
writer_type!(CommandW);
writer_type!(AtomicSetW);
writer_type!(AtomicClearW);
writer_type!(AtomicXorW);

#[derive(Clone, Copy)]
pub struct FieldReader<T = u32>(T);

impl<T: Copy> FieldReader<T> {
    pub const fn new(bits: T) -> Self {
        Self(bits)
    }

    pub const fn bits(&self) -> T {
        self.0
    }
}

pub type BitReader = FieldReader<bool>;

fn field_mask<const WIDTH: u8>() -> u32 {
    if WIDTH == 32 {
        u32::MAX
    } else {
        (1u32 << WIDTH) - 1
    }
}

fn write_field<W: WriterBits, const O: u8, const WIDTH: u8>(writer: &mut W, value: u32) {
    let mask = field_mask::<WIDTH>();
    let bits = unsafe { writer.bits_mut_unchecked() };
    *bits = (*bits & !(mask << O)) | ((value & mask) << O);
}

pub struct FieldWriter<'a, W: WriterBits, const O: u8, const WIDTH: u8> {
    writer: &'a mut W,
}

impl<'a, W: WriterBits, const O: u8, const WIDTH: u8> FieldWriter<'a, W, O, WIDTH> {
    pub fn new(writer: &'a mut W) -> Self {
        Self { writer }
    }

    pub fn bits(self, value: u32) -> &'a mut W {
        write_field::<W, O, WIDTH>(self.writer, value);
        self.writer
    }
}

pub struct BitWriter<'a, W: WriterBits, const O: u8> {
    writer: &'a mut W,
}

impl<'a, W: WriterBits, const O: u8> BitWriter<'a, W, O> {
    pub fn new(writer: &'a mut W) -> Self {
        Self { writer }
    }

    pub fn bit(self, value: bool) -> &'a mut W {
        write_field::<W, O, 1>(self.writer, value as u32);
        self.writer
    }

    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }

    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
}

macro_rules! one_write_field {
    ($field:ident, $bit:ident, $bits_method:ident, $bit_method:ident) => {
        pub struct $field<'a, W: WriterBits, const O: u8, const WIDTH: u8> {
            writer: &'a mut W,
        }

        impl<'a, W: WriterBits, const O: u8, const WIDTH: u8> $field<'a, W, O, WIDTH> {
            pub fn new(writer: &'a mut W) -> Self {
                Self { writer }
            }

            pub fn $bits_method(self, value: u32) -> &'a mut W {
                write_field::<W, O, WIDTH>(self.writer, value);
                self.writer
            }
        }

        pub struct $bit<'a, W: WriterBits, const O: u8> {
            writer: &'a mut W,
        }

        impl<'a, W: WriterBits, const O: u8> $bit<'a, W, O> {
            pub fn new(writer: &'a mut W) -> Self {
                Self { writer }
            }

            pub fn $bit_method(self) -> &'a mut W {
                write_field::<W, O, 1>(self.writer, 1);
                self.writer
            }
        }
    };
}

one_write_field!(ClearFieldWriter, ClearBitWriter, clear_bits, clear);
one_write_field!(SetFieldWriter, SetBitWriter, set_bits, set);
one_write_field!(ToggleFieldWriter, ToggleBitWriter, toggle_bits, toggle);
one_write_field!(CommandFieldWriter, CommandBitWriter, bits, trigger);
one_write_field!(AtomicSetFieldWriter, AtomicSetBitWriter, set_bits, set);
one_write_field!(
    AtomicClearFieldWriter,
    AtomicClearBitWriter,
    clear_bits,
    clear
);
one_write_field!(
    AtomicXorFieldWriter,
    AtomicXorBitWriter,
    toggle_bits,
    toggle
);
