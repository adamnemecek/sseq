use std::io::{self, Read, Write};
use std::mem::size_of;
use std::sync::Arc;

use crate::{Load, Save};

impl Save for bool {
    fn save(&self, buffer: &mut impl Write) -> io::Result<()> {
        if *self {
            buffer.write_all(&[1])?;
        } else {
            buffer.write_all(&[0])?;
        }
        Ok(())
    }
}

impl Load for bool {
    type AuxData = ();

    fn load(buffer: &mut impl Read, _: &()) -> io::Result<Self> {
        let mut bytes: [u8; 1] = [0; 1];
        buffer.read_exact(&mut bytes)?;
        if bytes[0] == 1 {
            Ok(true)
        } else if bytes[0] == 0 {
            Ok(false)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid encoding of boolean",
            ))
        }
    }
}

macro_rules! impl_num {
    ( $( $x:ty ), * ) => {
        $(
            impl Save for $x {
                fn save(&self, buffer : &mut impl Write) -> io::Result<()> {
                    let bytes = self.to_le_bytes();
                    buffer.write_all(&bytes)?;
                    Ok(())
                }
            }

            impl Load for $x {
                type AuxData = ();

                fn load(buffer : &mut impl Read, _ : &()) -> io::Result<Self> {
                    let mut bytes : [u8; size_of::<$x>()] = [0; size_of::<$x>()];
                    buffer.read_exact(&mut bytes)?;
                    Ok(<$x>::from_le_bytes(bytes))
                }
            }
        )*
    }
}

impl_num!(i32, i64, i128, isize, u32, u64, u128, usize);

impl<T: Save> Save for Vec<T> {
    fn save(&self, buffer: &mut impl Write) -> io::Result<()> {
        self.len().save(buffer)?;
        for x in self.iter() {
            x.save(buffer)?;
        }
        Ok(())
    }
}

impl<T: Load> Load for Vec<T> {
    type AuxData = T::AuxData;

    fn load(buffer: &mut impl Read, data: &Self::AuxData) -> io::Result<Self> {
        let len = usize::load(buffer, &())?;

        let mut result: Vec<T> = Vec::with_capacity(len);

        for _ in 0..len {
            result.push(T::load(buffer, data)?);
        }
        Ok(result)
    }
}

impl<T: Save> Save for Arc<T> {
    fn save(&self, buffer: &mut impl Write) -> io::Result<()> {
        let x: &T = &*self;
        x.save(buffer)
    }
}

impl<T: Load> Load for Arc<T> {
    type AuxData = T::AuxData;

    fn load(buffer: &mut impl Read, data: &Self::AuxData) -> io::Result<Self> {
        Ok(Arc::new(T::load(buffer, data)?))
    }
}

impl<T: Save> Save for std::sync::Mutex<T> {
    fn save(&self, buffer: &mut impl Write) -> io::Result<()> {
        let x: &T = &*self.lock().unwrap();
        x.save(buffer)
    }
}

impl<T: Load> Load for std::sync::Mutex<T> {
    type AuxData = T::AuxData;

    fn load(buffer: &mut impl Read, data: &Self::AuxData) -> io::Result<Self> {
        Ok(std::sync::Mutex::new(T::load(buffer, data)?))
    }
}

impl<T: Save> Save for Option<T> {
    fn save(&self, buffer: &mut impl Write) -> io::Result<()> {
        match self {
            None => false.save(buffer),
            Some(x) => {
                true.save(buffer)?;
                x.save(buffer)
            }
        }
    }
}

impl<T: Load> Load for Option<T> {
    type AuxData = Option<T::AuxData>;

    fn load(buffer: &mut impl Read, data: &Self::AuxData) -> io::Result<Self> {
        let is_some = bool::load(buffer, &())?;
        if is_some {
            Ok(Some(T::load(buffer, data.as_ref().unwrap())?))
        } else {
            Ok(None)
        }
    }
}

impl<A: Save, B: Save> Save for (A, B) {
    fn save(&self, buffer: &mut impl Write) -> io::Result<()> {
        self.0.save(buffer)?;
        self.1.save(buffer)
    }
}

impl<A: Load, B: Load> Load for (A, B) {
    type AuxData = (A::AuxData, B::AuxData);

    fn load(buffer: &mut impl Read, data: &Self::AuxData) -> io::Result<Self> {
        Ok((A::load(buffer, &data.0)?, B::load(buffer, &data.1)?))
    }
}
impl<K: Save + Eq + std::hash::Hash, V: Save> Save for dashmap::DashMap<K, V> {
    fn save(&self, buffer: &mut impl Write) -> io::Result<()> {
        self.len().save(buffer)?;
        for r in self.iter() {
            r.key().save(buffer)?;
            r.value().save(buffer)?;
        }
        Ok(())
    }
}

impl<K: Load + Eq + std::hash::Hash, V: Load> Load for dashmap::DashMap<K, V> {
    type AuxData = (K::AuxData, V::AuxData);

    fn load(buffer: &mut impl Read, data: &Self::AuxData) -> io::Result<Self> {
        let len: usize = usize::load(buffer, &())?;
        let dm = dashmap::DashMap::with_capacity(len);
        for _ in 0..len {
            let k = K::load(buffer, &data.0)?;
            let v = V::load(buffer, &data.1)?;
            dm.insert(k, v);
        }
        Ok(dm)
    }
}
