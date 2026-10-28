use crate::{Error, FourCC, Result};

pub(crate) struct Reader<'a> {
    file: &'static str,
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub(crate) fn new(file: &'static str, data: &'a [u8]) -> Self {
        Self { file, data, pos: 0 }
    }

    pub(crate) fn pos(&self) -> usize {
        self.pos
    }

    pub(crate) fn since(&self, start: usize) -> Vec<u8> {
        self.data[start..self.pos].to_vec()
    }

    pub(crate) fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    pub(crate) fn damaged(&self, reason: &str) -> Error {
        Error::Invalid(format!(
            "{} is damaged at byte {}: {reason}",
            self.file, self.pos
        ))
    }

    pub(crate) fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.pos.checked_add(n).filter(|&e| e <= self.data.len());
        let Some(end) = end else {
            return Err(self.damaged("the file ends early"));
        };
        let bytes = &self.data[self.pos..end];

        self.pos = end;
        Ok(bytes)
    }

    pub(crate) fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    pub(crate) fn u32(&mut self) -> Result<u32> {
        let mut bytes = [0u8; 4];

        bytes.copy_from_slice(self.take(4)?);
        Ok(u32::from_le_bytes(bytes))
    }

    pub(crate) fn i32(&mut self) -> Result<i32> {
        let mut bytes = [0u8; 4];

        bytes.copy_from_slice(self.take(4)?);
        Ok(i32::from_le_bytes(bytes))
    }

    pub(crate) fn f32(&mut self) -> Result<f32> {
        let mut bytes = [0u8; 4];

        bytes.copy_from_slice(self.take(4)?);
        Ok(f32::from_le_bytes(bytes))
    }

    pub(crate) fn fourcc(&mut self) -> Result<FourCC> {
        let mut bytes = [0u8; 4];

        bytes.copy_from_slice(self.take(4)?);
        Ok(FourCC(bytes))
    }

    pub(crate) fn count(&mut self, entry_size: usize) -> Result<usize> {
        let n = self.u32()? as usize;

        if n.saturating_mul(entry_size) > self.remaining() {
            return Err(self.damaged(&format!("count {n} exceeds the remaining data")));
        }

        Ok(n)
    }

    pub(crate) fn c_string(&mut self) -> Result<&'a [u8]> {
        let rest = &self.data[self.pos..];
        let Some(len) = rest.iter().position(|&b| b == 0) else {
            return Err(self.damaged("unterminated string"));
        };

        self.pos += len + 1;
        Ok(&rest[..len])
    }
}

pub(crate) fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

pub(crate) fn put_i32(out: &mut Vec<u8>, value: i32) {
    out.extend_from_slice(&value.to_le_bytes());
}

pub(crate) fn put_count(out: &mut Vec<u8>, len: usize) {
    // Counts in map files are 32-bit; maps never come close to the limit.
    put_u32(out, u32::try_from(len).unwrap_or(u32::MAX));
}
