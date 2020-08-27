use arrayvec::ArrayVec;
use crate::writer::IO;
use crate::writer::Writer;
use crate::writer::WriterBackend;
use std::fmt;
use std::io::Write;
use std::io;

pub trait MpegBox {
    fn len(&self) -> usize;
    fn write<B: WriterBackend>(&self, w: &mut Writer<B>) -> Result<(), B::Error>;
}

#[derive(Copy, Clone)]
pub struct FourCC(pub [u8; 4]);

impl fmt::Debug for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match std::str::from_utf8(&self.0) {
            Ok(s) => s.fmt(f),
            Err(_) => self.0.fmt(f)
        }
    }
}

#[derive(Debug, Clone)]
pub struct AvifFile<'data> {
    pub ftyp: FtypBox,
    pub meta: MetaBox,
    pub mdat: MdatBox<'data>,
}

impl AvifFile<'_> {
    /// Where the primary data starts inside the `mdat` box, for `iloc`'s offset
    fn mdat_payload_start_offset(&self) -> u32 {
        (self.ftyp.len() + self.meta.len()
            + BASIC_BOX_SIZE) as u32 // mdat head
    }

    /// `iloc` is mostly unnecssary, high risk of out-of-buffer accesses in parsers that don't pay attention,
    /// and also awkward to serialize, because its content depends on its own serialized byte size.
    fn fix_iloc_positions(&mut self) {
        let start_offset = self.mdat_payload_start_offset();
        for iloc_item in self.meta.iloc.items.iter_mut() {
            for ex in iloc_item.extents.iter_mut() {
                let abs = match ex.offset {
                    IlocOffset::Relative(ref mut n) => {
                        *n as u32 + start_offset
                    },
                    IlocOffset::Absolute(_) => continue,
                };
                ex.offset = IlocOffset::Absolute(abs);
            }
        }
    }

    pub fn write<W: Write>(&mut self, mut out: W) -> io::Result<()> {
        self.fix_iloc_positions();

        let mut tmp = Vec::with_capacity(self.ftyp.len() + self.meta.len());
        let mut w = Writer::new(&mut tmp);
        let _ = self.ftyp.write(&mut w);
        let _ = self.meta.write(&mut w);
        drop(w);
        out.write_all(&tmp)?;
        drop(tmp);

        let mut out = IO(out);
        let mut w = Writer::new(&mut out);
        self.mdat.write(&mut w)?;
        Ok(())
    }
}

const BASIC_BOX_SIZE: usize = 8;
const FULL_BOX_SIZE: usize = BASIC_BOX_SIZE + 4;

#[derive(Debug, Clone)]
pub struct FtypBox {
    pub major_brand: FourCC,
    pub minor_version: u32,
    pub compatible_brands: ArrayVec<[FourCC; 1]>,
}

/// File Type box (chunk)
impl MpegBox for FtypBox {
    #[inline(always)]
    fn len(&self) -> usize {
        BASIC_BOX_SIZE
        + 4 // brand
        + 4 // ver
        + 4 * self.compatible_brands.len()
    }

    fn write<B: WriterBackend>(&self, w: &mut Writer<B>) -> Result<(), B::Error> {
        let mut b = w.new_box(self.len());
        b.basic_box(*b"ftyp")?;
        b.push(&self.major_brand.0)?;
        b.u32(self.minor_version)?;
        for cb in &self.compatible_brands {
            b.push(&cb.0)?;
        }
        Ok(())
    }
}

/// Metadata box
#[derive(Debug, Clone)]
pub struct MetaBox {
    pub iloc: IlocBox,
    pub iinf: IinfBox,
    pub pitm: PitmBox,
    pub iprp: IprpBox,
    pub iref: Option<IrefBox>,
}

impl MpegBox for MetaBox {
    #[inline]
    fn len(&self) -> usize {
        FULL_BOX_SIZE
        + self.pitm.len()
        + self.iloc.len()
        + self.iinf.len()
        + self.iprp.len()
        + self.iref.as_ref().map_or(0, |b| b.len())
    }

    fn write<B: WriterBackend>(&self, w: &mut Writer<B>) -> Result<(), B::Error> {
        let mut b = w.new_box(self.len());
        b.full_box(*b"meta", 0)?;
        self.pitm.write(&mut b)?;
        self.iloc.write(&mut b)?;
        self.iinf.write(&mut b)?;
        if let Some(iref) = &self.iref {
            iref.write(&mut b)?;
        }
        self.iprp.write(&mut b)
    }
}

/// Item Info box
#[derive(Debug, Clone)]
pub struct IinfBox {
    pub items: ArrayVec<[InfeBox; 2]>,
}

impl MpegBox for IinfBox {
    #[inline]
    fn len(&self) -> usize {
        FULL_BOX_SIZE
        + 2 // num entries
        + self.items.iter().map(|item| item.len()).sum::<usize>()
    }

    fn write<B: WriterBackend>(&self, w: &mut Writer<B>) -> Result<(), B::Error> {
        let mut b = w.new_box(self.len());
        b.full_box(*b"iinf", 0)?;
        b.u16(self.items.len() as _)?;
        for infe in self.items.iter() {
            infe.write(&mut b)?;
        }
        Ok(())
    }
}

/// Item Info Entry box
#[derive(Debug, Copy, Clone)]
pub struct InfeBox {
    pub id: u16,
    pub typ: FourCC,
    pub name: &'static str,
}

impl MpegBox for InfeBox {
    #[inline(always)]
    fn len(&self) -> usize {
        FULL_BOX_SIZE
        + 2 // id
        + 2 // item_protection_index
        + 4 // type
        + self.name.as_bytes().len() + 1 // nul-terminated
    }

    fn write<B: WriterBackend>(&self, w: &mut Writer<B>) -> Result<(), B::Error> {
        let mut b = w.new_box(self.len());
        b.full_box(*b"infe", 2)?;
        b.u16(self.id)?;
        b.u16(0)?;
        b.push(&self.typ.0)?;
        b.push(self.name.as_bytes())?;
        b.u8(0)
    }
}

/// Item properties + associations
#[derive(Debug, Clone)]
pub struct IprpBox {
    pub ipco: IpcoBox,
    pub ipma: IpmaBox,
}

impl MpegBox for IprpBox {
    #[inline(always)]
    fn len(&self) -> usize {
        BASIC_BOX_SIZE
            + self.ipco.len()
            + self.ipma.len()
    }

    fn write<B: WriterBackend>(&self, w: &mut Writer<B>) -> Result<(), B::Error> {
        let mut b = w.new_box(self.len());
        b.basic_box(*b"iprp")?;
        self.ipco.write(&mut b)?;
        self.ipma.write(&mut b)
    }
}

/// Item Property Container box
#[derive(Debug, Clone)]
pub struct IpcoBox {
    pub av1c: ArrayVec<[Av1CBox; 2]>,
    pub ispe: IspeBox,
    pub auxc: Option<AuxCBox>,
}

impl MpegBox for IpcoBox {
    #[inline]
    fn len(&self) -> usize {
        BASIC_BOX_SIZE
        + self.ispe.len()
        + self.av1c.iter().map(|a| a.len()).sum::<usize>()
        + self.auxc.map_or(0, |a| a.len())
    }

    fn write<B: WriterBackend>(&self, w: &mut Writer<B>) -> Result<(), B::Error> {
        let mut b = w.new_box(self.len());
        b.basic_box(*b"ipco")?;
        self.ispe.write(&mut b)?;
        for a in self.av1c.iter() {
            a.write(&mut b)?;
        }
        if let Some(a) = &self.auxc {
            a.write(&mut b)?;
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct AuxCBox {
    pub urn: &'static str,
}

impl AuxCBox {
    pub fn len(&self) -> usize {
        FULL_BOX_SIZE
            + self.urn.len() + 1
    }

    pub fn write<B: WriterBackend>(&self, w: &mut Writer<B>) -> Result<(), B::Error> {
        let mut b = w.new_box(self.len());
        b.full_box(*b"auxC", 0)?;
        b.push(self.urn.as_bytes())?;
        b.u8(0)
    }
}

// /// Pixies, I guess.
// #[derive(Debug, Copy, Clone)]
// pub struct PixiBox {
//     depth: u8,
//     channels: u8,
// }

// impl PixiBox {
//     pub fn len(&self) -> usize {
//         BASIC_BOX_SIZE
//             + 1 + self.channels as usize
//     }

//     pub fn write<B: WriterBackend>(&self, w: &mut Writer<B>) -> Result<(), B::Error> {
//         let mut b = w.new_box(self.len());
//         b.basic_box(*b"pixi")?;
//         b.u8(self.channels)?;
//         for _ in 0..self.channels {
//             b.u8(self.depth)?;
//         }
//         Ok(())
//     }
// }

/// This is HEVC-specific and not for AVIF, but Chrome wants it :(
#[derive(Debug, Copy, Clone)]
pub struct IspeBox {
    pub width: u32,
    pub height: u32,
}

impl MpegBox for IspeBox {
    #[inline(always)]
    fn len(&self) -> usize {
        FULL_BOX_SIZE + 4 + 4
    }

    fn write<B: WriterBackend>(&self, w: &mut Writer<B>) -> Result<(), B::Error> {
        let mut b = w.new_box(self.len());
        b.full_box(*b"ispe", 0)?;
        b.u32(self.width)?;
        b.u32(self.height)
    }
}

/// Property→image associations
#[derive(Debug, Clone)]
pub struct IpmaEntry {
    pub item_id: u16,
    pub prop_ids: ArrayVec<[u8; 3]>,
}

#[derive(Debug, Clone)]
pub struct IpmaBox {
    pub entries: ArrayVec<[IpmaEntry; 2]>,
}

impl MpegBox for IpmaBox {
    #[inline]
    fn len(&self) -> usize {
        FULL_BOX_SIZE + 4 + self.entries.iter().map(|e| 2 + 1 + e.prop_ids.len()).sum::<usize>()
    }

    fn write<B: WriterBackend>(&self, w: &mut Writer<B>) -> Result<(), B::Error> {
        let mut b = w.new_box(self.len());
        b.full_box(*b"ipma", 0)?;
        b.u32(self.entries.len() as _)?; // entry count

        for e in &self.entries {
            b.u16(e.item_id)?;
            b.u8(e.prop_ids.len() as u8)?; // assoc count
            for &p in e.prop_ids.iter() {
                b.u8(p)?;
            }
        }
        Ok(())
    }
}

/// Item Reference box
#[derive(Debug, Copy, Clone)]
pub struct IrefEntryBox {
    pub from_id: u16,
    pub to_id: u16,
    pub typ: FourCC,
}

impl MpegBox for IrefEntryBox {
    #[inline(always)]
    fn len(&self) -> usize {
        BASIC_BOX_SIZE
            + 2 // from
            + 2 // refcount
            + 2 // to
    }

    fn write<B: WriterBackend>(&self, w: &mut Writer<B>) -> Result<(), B::Error> {
        let mut b = w.new_box(self.len());
        b.basic_box(self.typ.0)?;
        b.u16(self.from_id)?;
        b.u16(1)?;
        b.u16(self.to_id)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct IrefBox {
    pub entry: IrefEntryBox,
}

impl MpegBox for IrefBox {
    #[inline(always)]
    fn len(&self) -> usize {
        FULL_BOX_SIZE + self.entry.len()
    }

    fn write<B: WriterBackend>(&self, w: &mut Writer<B>) -> Result<(), B::Error> {
        let mut b = w.new_box(self.len());
        b.full_box(*b"iref", 0)?;
        self.entry.write(&mut b)
    }
}



/// Auxiliary item (alpha or depth map)
#[derive(Debug, Copy, Clone)]
pub struct AuxlBox {
}

impl MpegBox for AuxlBox {
    #[inline(always)]
    fn len(&self) -> usize {
        FULL_BOX_SIZE
    }

    fn write<B: WriterBackend>(&self, w: &mut Writer<B>) -> Result<(), B::Error> {
        let mut b = w.new_box(self.len());
        b.full_box(*b"auxl", 0)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Av1CBox {
    pub seq_profile: bool,
    pub seq_level_idx_0: u8,
    pub seq_tier_0: bool,
    pub high_bitdepth: bool,
    pub twelve_bit: bool,
    pub monochrome: bool,
    pub chroma_subsampling_x: bool,
    pub chroma_subsampling_y: bool,
    pub chroma_sample_position: u8,
}

impl MpegBox for Av1CBox {
    #[inline(always)]
    fn len(&self) -> usize {
        BASIC_BOX_SIZE + 4
    }

    fn write<B: WriterBackend>(&self, w: &mut Writer<B>) -> Result<(), B::Error> {
        let mut b = w.new_box(self.len());
        b.basic_box(*b"av1C")?;
        let flags1 =
            (self.seq_tier_0 as u8) |
            (self.high_bitdepth as u8) << 1 |
            (self.twelve_bit as u8) << 2 |
            (self.monochrome as u8) << 3 |
            (self.chroma_subsampling_x as u8) << 4 |
            (self.chroma_subsampling_y as u8) << 5 |
            (self.chroma_sample_position as u8) << 6;

        b.push(&[
            0x81, // marker and version
            ((self.seq_profile as u8) << 5) | self.seq_level_idx_0, // x2d == 45
            flags1,
            0,
        ])
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PitmBox(pub u16);

impl MpegBox for PitmBox {
    #[inline(always)]
    fn len(&self) -> usize {
        FULL_BOX_SIZE + 2
    }

    fn write<B: WriterBackend>(&self, w: &mut Writer<B>) -> Result<(), B::Error> {
        let mut b = w.new_box(self.len());
        b.full_box(*b"pitm", 0)?;
        b.u16(self.0)
    }
}

#[derive(Debug, Clone)]
pub struct IlocBox {
    pub items: ArrayVec<[IlocItem; 2]>,
}

#[derive(Debug, Clone)]
pub struct IlocItem {
    pub id: u16,
    pub extents: ArrayVec<[IlocExtent; 2]>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum IlocOffset {
    Relative(usize),
    Absolute(u32),
}

#[derive(Debug, Copy, Clone)]
pub struct IlocExtent {
    pub offset: IlocOffset,
    pub len: usize,
}

impl MpegBox for IlocBox {
    #[inline(always)]
    fn len(&self) -> usize {
        FULL_BOX_SIZE
        + 1 // offset_size, length_size
        + 1 // base_offset_size, reserved
        + 2 // num items
        + self.items.iter().map(|i| ( // for each item
            2 // id
            + 2 // dat ref idx
            + 0 // base_offset_size
            + 2 // extent count
            + i.extents.len() * ( // for each extent
               4 // extent_offset
               + 4 // extent_len
            )
        )).sum::<usize>()
    }

    fn write<B: WriterBackend>(&self, w: &mut Writer<B>) -> Result<(), B::Error> {
        let mut b = w.new_box(self.len());
        b.full_box(*b"iloc", 0)?;
        b.push(&[4 << 4 | 4, 0])?; // offset and length are 4 bytes

        b.u16(self.items.len() as _)?; // num items
        for item in self.items.iter() {
            b.u16(item.id)?;
            b.u16(0)?;
            b.u16(item.extents.len() as _)?; // num extents
            for ex in &item.extents {
                b.u32(match ex.offset {
                    IlocOffset::Absolute(val) => val,
                    IlocOffset::Relative(_) => panic!("absolute offset must be set"),
                })?;
                b.u32(ex.len as _)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MdatBox<'data> {
    pub data_chunks: &'data[&'data [u8]],
}

impl MpegBox for MdatBox<'_> {
    #[inline(always)]
    fn len(&self) -> usize {
        BASIC_BOX_SIZE + self.data_chunks.iter().map(|c| c.len()).sum::<usize>()
    }

    fn write<B: WriterBackend>(&self, w: &mut Writer<B>) -> Result<(), B::Error> {
        let mut b = w.new_box(self.len());
        b.basic_box(*b"mdat")?;
        for ch in self.data_chunks {
            b.push(ch)?;
        }
        Ok(())
    }
}

