use std::convert::{Infallible, TryFrom};
use std::io;

pub trait WriterBackend {
    type Error;
    fn extend_from_slice(&mut self, data: &[u8]) -> Result<(), Self::Error>;
}

/// `io::Write` generates bloated code (with backtrace for every byte written),
/// so small boxes are written infallibly.
impl WriterBackend for Vec<u8> {
    type Error = Infallible;

    #[inline(always)]
    fn extend_from_slice(&mut self, data: &[u8]) -> Result<(), Infallible> {
        self.extend_from_slice(data);
        Ok(())
    }
}

pub struct IO<W>(pub W);

impl<W: io::Write> WriterBackend for IO<W> {
    type Error = io::Error;

    #[inline(always)]
    fn extend_from_slice(&mut self, data: &[u8]) -> io::Result<()> {
        self.0.write_all(data)
    }
}

pub struct Writer<'p, 'w, B> {
    parent: Option<&'p mut usize>,
    left: usize,
    out: &'w mut B,
}

impl<'w, B> Writer<'static, 'w, B> {
    #[inline]
    pub fn new(out: &'w mut B) -> Self {
        Self {
            parent: None,
            left: 0,
            out,
        }
    }
}

impl<B: WriterBackend> Writer<'_, '_, B> {
    #[inline]
    pub fn new_box(&mut self, len: usize) -> Writer<'_, '_, B> {
        Writer {
            parent: if self.left > 0 {
                Some(&mut self.left)
            } else {
                debug_assert!(self.parent.is_none());
                None
            },
            left: len,
            out: self.out,
        }
    }

    #[inline(always)]
    pub fn full_box(&mut self, typ: [u8; 4], version: u8) -> Result<(), B::Error> {
        self.basic_box(typ)?;
        self.push(&[version, 0, 0, 0])
    }

    #[inline]
    pub fn basic_box(&mut self, typ: [u8; 4]) -> Result<(), B::Error> {
        let len = self.left;
        if let Some(parent) = &mut self.parent {
            **parent -= len;
        }
        if let Ok(len) = u32::try_from(len) {
            self.u32(len)?;
        } else {
            self.u32(1)?;
            self.u64(len as u64)?;
        }
        self.push(&typ)
    }

    #[inline(always)]
    pub fn push(&mut self, data: &[u8]) -> Result<(), B::Error> {
        self.left -= data.len();
        self.out.extend_from_slice(data)
    }

    #[inline(always)]
    pub fn u8(&mut self, val: u8) -> Result<(), B::Error> {
        self.push(std::slice::from_ref(&val))
    }

    #[inline(always)]
    pub fn u16(&mut self, val: u16) -> Result<(), B::Error> {
        self.push(&val.to_be_bytes())
    }

    #[inline(always)]
    pub fn u32(&mut self, val: u32) -> Result<(), B::Error> {
        self.push(&val.to_be_bytes())
    }

    #[inline(always)]
    pub fn u64(&mut self, val: u64) -> Result<(), B::Error> {
        self.push(&val.to_be_bytes())
    }
}

#[cfg(debug_assertions)]
impl<B> Drop for Writer<'_, '_, B> {
    fn drop(&mut self) {
        assert_eq!(self.left, 0);
    }
}
