use std::ops::Deref;

#[cfg(feature = "serde")]
use {
    serde::de::{Error, SeqAccess, Visitor},
    serde::ser::SerializeTuple,
    serde::{Deserialize, Deserializer, Serialize, Serializer},
    std::marker::PhantomData,
};

/// Serde doesn't know how to handle constant generics. [Self] serves as a zero-cost wrapper over the array.
/// It implements [Deref], the underlying array is therefore exposed and reachable directly.
#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub struct Array<T, const N: usize> {
    values: [T; N],
}

impl<T, const N: usize> Array<T, N> {
    pub fn wrap(values: [T; N]) -> Self {
        Self { values }
    }
}

impl<T, const N: usize> From<[T; N]> for Array<T, N> {
    fn from(value: [T; N]) -> Self {
        Array::wrap(value)
    }
}

impl<T, const N: usize> Deref for Array<T, N> {
    type Target = [T; N];

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}
#[cfg(feature = "serde")]
impl<T: Serialize, const N: usize> Serialize for Array<T, N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_tuple(N)?;

        for element in self.values.iter() {
            state.serialize_element(element)?;
        }

        state.end()
    }
}

#[cfg(feature = "serde")]
pub(crate) struct ArrayVisitor<T, const N: usize>(pub(crate) PhantomData<T>);

#[cfg(feature = "serde")]
impl<'de, T, const N: usize> Visitor<'de> for ArrayVisitor<T, N>
where
    T: Deserialize<'de>,
{
    type Value = Array<T, N>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(&format!("An array of length {N}"))
    }

    #[inline]
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut data = Vec::<T>::with_capacity(N);
        for _ in 0..N {
            match seq.next_element()? {
                Some(val) => data.push(val),
                None => return Err(Error::invalid_length(N, &self)),
            }
        }

        let backing_array: [T; N] = data.try_into().unwrap_or_else(|v: Vec<T>| {
            panic!("Data of length {} expected, obtained {}", N, v.len())
        });
        Ok(Array::wrap(backing_array))
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Deserialize<'de>, const N: usize> Deserialize<'de> for Array<T, N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(N, ArrayVisitor::<T, N>(PhantomData))
    }
}
