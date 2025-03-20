use std::collections::HashMap;

pub use rec_partial_macros::HasPartial;

pub trait HasPartial {
    type Partial;
}

macro_rules! impl_iden {
    ($($ty:ty),* $(,)?) => {
        $(
            impl HasPartial for $ty {
                type Partial = $ty;
            }
        )*
    };
}

impl_iden!(
    bool,
    char,
    f32,
    f64,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    (),
    String,
);

impl<T: HasPartial> HasPartial for Vec<T> {
    type Partial = Vec<T::Partial>;
}

impl<K, V: HasPartial> HasPartial for HashMap<K, V> {
    type Partial = HashMap<K, V::Partial>;
}

impl<T: HasPartial> HasPartial for Option<T> {
    type Partial = T::Partial;
}
