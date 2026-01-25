use std::{marker::PhantomData, mem::MaybeUninit};

struct Bool<const X: bool>;
trait True {}
impl True for Bool<true> {}

pub trait DefaultPackable: Copy {}

impl DefaultPackable for bool {}
impl DefaultPackable for u8 {}
impl DefaultPackable for u16 {}
impl DefaultPackable for u32 {}
impl DefaultPackable for i8 {}
impl DefaultPackable for i16 {}
impl DefaultPackable for i32 {}
impl<T: DefaultPackable> DefaultPackable for Option<T> {}
impl<T: BitPackable> DefaultPackable for BitPacked<T> {}

pub trait Packable: Copy {
    const LEN: usize;
    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>);
    fn read_packed(buffer: &mut PackedStreamReader) -> Self;
}

const fn is_smaller_than<T, U>() {
    assert!(size_of::<T>() <= size_of::<U>());
}

impl<T: Copy> Packable for T
where
    T: DefaultPackable,
{
    const LEN: usize = size_of::<Self>().div_ceil(size_of::<u32>());

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        let _: () = const { is_smaller_than::<T, u32>() };
        unsafe {
            let mut value: u32 = 0;
            (&raw mut value).cast::<T>().write_unaligned(self);
            buffer.write_u32(value);
        }
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        let _: () = const { is_smaller_than::<T, u32>() };
        let value = buffer.read_u32();
        unsafe { std::mem::transmute_copy(&value) }
    }
}

pub trait BitPackable: Copy {
    const BITS: usize;
    fn pack(self) -> u32;
    fn unpack(value: u32) -> Self;
}

impl BitPackable for bool {
    const BITS: usize = 1;

    fn pack(self) -> u32 {
        self as u32
    }

    fn unpack(value: u32) -> Self {
        value != 0
    }
}

impl BitPackable for u8 {
    const BITS: usize = 8;

    fn pack(self) -> u32 {
        self as u32
    }

    fn unpack(value: u32) -> Self {
        value as _
    }
}

impl BitPackable for u16 {
    const BITS: usize = 16;

    fn pack(self) -> u32 {
        self as u32
    }

    fn unpack(value: u32) -> Self {
        value as _
    }
}

impl BitPackable for u32 {
    const BITS: usize = 32;

    fn pack(self) -> u32 {
        self
    }

    fn unpack(value: u32) -> Self {
        value
    }
}

impl BitPackable for i8 {
    const BITS: usize = 8;

    fn pack(self) -> u32 {
        self as u32
    }

    fn unpack(value: u32) -> Self {
        value as _
    }
}

impl BitPackable for i16 {
    const BITS: usize = 16;

    fn pack(self) -> u32 {
        self as u32
    }

    fn unpack(value: u32) -> Self {
        value as _
    }
}

impl BitPackable for i32 {
    const BITS: usize = 32;

    fn pack(self) -> u32 {
        self as u32
    }

    fn unpack(value: u32) -> Self {
        value as _
    }
}

impl<T: BitPackable> BitPackable for Option<T>
where
    Bool<{ T::BITS < 32 }>: True,
{
    const BITS: usize = T::BITS + 1;

    fn pack(self) -> u32 {
        match self {
            Some(x) => x.pack() + 1,
            None => 0,
        }
    }

    fn unpack(value: u32) -> Self {
        match value {
            0 => None,
            _ => Some(T::unpack(value - 1)),
        }
    }
}

macro_rules! peel {
    ($name:ident: $Ty:ident, $($other:ident: $OTy:ident,)*) => (tuple! { $($other: $OTy,)* })
}

macro_rules! tuple_unpack {
    ($v:ident,) => {};
    ($v:ident, $name:ident: $Ty:ident, $($other:ident: $OTy:ident,)*) => {
        tuple_unpack! { $v, $($other: $OTy,)* }
        let $name = $Ty::unpack($v & ((1 << $Ty::BITS) - 1));
        let $v = $v >> $Ty::BITS;
    };
}

macro_rules! tuple {
    () => {};
    ($($name:ident: $Ty:ident,)+) => {
        impl<$($Ty:BitPackable),+> BitPackable for ($($Ty),+,)
        where
            Bool<{ ($(($Ty::BITS) +)+ 0) < 32 }>: True,
        {
            const BITS: usize = $(($Ty::BITS) +)+ 0;
            fn pack(self) -> u32 {
                let ($($name,)+) = self;
                let value: u32 = 0;
                $(
                    let value = (value << $Ty::BITS) | $name.pack();
                )+
                value
            }
            fn unpack(value: u32) -> Self {
                tuple_unpack! { value, $($name: $Ty,)+ }
                _ = value;
                ($($name,)+)
            }
        }

        impl<$($Ty:Packable),+> Packable for ($($Ty),+,) {
            const LEN: usize = $(($Ty::LEN) +)+ 0;
            fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
                let ($($name,)+) = self;
                $(
                    buffer.write($name);
                )+
            }
            fn read_packed(buffer: &mut PackedStreamReader) -> Self {
                $(
                    let $name = buffer.read();
                )+
                ($($name,)+)
            }
        }

        peel! { $($name: $Ty,)+ }
    };
}

tuple! {
    t1: T1, t2: T2, t3: T3, t4: T4, t5: T5, t6: T6, t7: T7, t8: T8, t9: T9, t10: T10,
    t11: T11, t12: T12, t13: T13, t14: T14, t15: T15, t16: T16, t17: T17, t18: T18, t19: T19, t20: T20,
    t21: T21, t22: T22, t23: T23, t24: T24, t25: T25, t26: T26, t27: T27, t28: T28, t29: T29, t30: T30,
    t31: T31, t32: T32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BitPacked<T: BitPackable>(u32, PhantomData<T>);

impl<T: BitPackable> BitPacked<T> {
    pub fn pack_bits(value: T) -> Self {
        Self(value.pack(), PhantomData)
    }

    pub fn unpack(self) -> T {
        T::unpack(self.0)
    }
}

pub struct PackedStreamWriter<'a> {
    buffer: &'a mut Vec<u32>,
}

impl<'a> PackedStreamWriter<'a> {
    pub fn new(buffer: &'a mut Vec<u32>) -> Self {
        Self { buffer }
    }

    pub fn write<T: Packable>(&mut self, value: T) {
        value.write_packed(self);
    }

    pub fn write_slice<T: Packable>(&mut self, values: &[T]) {
        for value in values {
            value.write_packed(self);
        }
    }

    pub fn write_array<const N: usize, T: Packable>(&mut self, values: [T; N]) {
        for value in values {
            value.write_packed(self);
        }
    }

    pub fn write_u32(&mut self, value: u32) {
        self.buffer.push(value);
    }

    pub fn write_u32_array<const N: usize>(&mut self, values: [u32; N]) {
        self.buffer.extend(values);
    }
}

pub struct PackedStreamReader<'a> {
    buffer: &'a [u32],
    position: usize,
}

impl<'a> PackedStreamReader<'a> {
    pub fn new(buffer: &'a [u32]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    pub fn read<T: Packable>(&mut self) -> T {
        T::read_packed(self)
    }

    pub fn read_array<const N: usize, T: Packable>(&mut self) -> [T; N] {
        let mut values = [MaybeUninit::uninit(); N];
        for value in &mut values {
            value.write(T::read_packed(self));
        }
        unsafe { MaybeUninit::array_assume_init(values) }
    }

    pub fn read_u32(&mut self) -> u32 {
        let value = self.buffer[self.position];
        self.position += 1;
        value
    }

    pub fn write_u32_array<const N: usize>(&mut self) -> [u32; N] {
        let values = self.buffer[self.position..self.position + N]
            .as_array()
            .copied()
            .unwrap();
        self.position += N;
        values
    }
}
