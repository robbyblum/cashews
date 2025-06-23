use std::{hash::Hasher, io::Write};

pub struct HashingWriter<T>
where
    T: Hasher,
{
    hasher: T,
}

impl<T> HashingWriter<T>
where
    T: Hasher,
{
    pub fn new(hasher: T) -> HashingWriter<T> {
        HashingWriter { hasher }
    }
}

impl<T> Write for HashingWriter<T>
where
    T: Hasher,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.hasher.write(&buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
