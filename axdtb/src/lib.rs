#![no_std]

use axconfig::align_up;
use core::str;

mod util;
pub use crate::util::SliceRead;

extern crate alloc;
use alloc::{borrow::ToOwned, string::String, vec::Vec};

const MAGIC_NUMBER: u32 = 0xd00dfeed;
const SUPPORTED_VERSION: u32 = 17;
const OF_DT_BEGIN_NODE: u32 = 0x00000001;
const OF_DT_END_NODE: u32 = 0x00000002;
const OF_DT_PROP: u32 = 0x00000003;

#[derive(Debug)]
pub enum DeviceTreeError {
    BadMagicNumber,
    SliceReadError,
    VersionNotSupported,
    ParseError(usize),
    Utf8Error,
}

pub type DeviceTreeResult<T> = Result<T, DeviceTreeError>;
pub type DeviceTreeProperty = (String, Vec<u8>);
pub type DeviceTreeCallback = dyn FnMut(String, usize, usize, Vec<DeviceTreeProperty>);

pub struct DeviceTree {
    ptr: usize,
    totalsize: usize,
    pub off_struct: usize,
    off_strings: usize,
}

impl DeviceTree {
    pub fn init(ptr: usize) -> DeviceTreeResult<Self> {
        let buf = unsafe { core::slice::from_raw_parts(ptr as *const u8, 24) };

        if buf.read_be_u32(0)? != MAGIC_NUMBER {
            return Err(DeviceTreeError::BadMagicNumber);
        }
        if buf.read_be_u32(20)? != SUPPORTED_VERSION {
            return Err(DeviceTreeError::VersionNotSupported);
        }

        let totalsize = buf.read_be_u32(4)? as usize;
        let off_struct = buf.read_be_u32(8)? as usize;
        let off_strings = buf.read_be_u32(12)? as usize;

        Ok(Self {
            ptr,
            totalsize,
            off_struct,
            off_strings,
        })
    }
}

impl DeviceTree {
    pub fn parse(
        &self,
        mut pos: usize,
        mut addr_cells: usize,
        mut size_cells: usize,
        cb: &mut DeviceTreeCallback,
    ) -> DeviceTreeResult<usize> {
        let buf = unsafe { core::slice::from_raw_parts(self.ptr as *const u8, self.totalsize) };
        // check for DT_BEGIN_NODE
        if buf.read_be_u32(pos)? != OF_DT_BEGIN_NODE {
            return Err(DeviceTreeError::ParseError(pos));
        }
        pos += 4;

        let raw_name = buf.read_bstring0(pos)?;
        pos = align_up(pos + raw_name.len() + 1, 4);
        // First, read all the props.
        let mut props = Vec::new();
        while buf.read_be_u32(pos)? == OF_DT_PROP {
            let val_size = buf.read_be_u32(pos + 4)? as usize;
            let name_offset = buf.read_be_u32(pos + 8)? as usize;
            // get value slice
            let val_start = pos + 12;
            let val_end = val_start + val_size;
            let val = buf.subslice(val_start, val_end)?;
            // lookup name in strings table
            let prop_name = buf.read_bstring0(self.off_strings + name_offset)?;
            let prop_name = str::from_utf8(prop_name)?.to_owned();
            if prop_name == "#address-cells" {
                addr_cells = val.read_be_u32(0)? as usize;
            } else if prop_name == "#size-cells" {
                size_cells = val.read_be_u32(0)? as usize;
            }
            props.push((prop_name, val.to_owned()));
            pos = align_up(val_end, 4);
        }
        // Callback for parsing dtb
        let name = str::from_utf8(raw_name)?.to_owned();
        cb(name, addr_cells, size_cells, props);
        // Then, parse all its children.
        while buf.read_be_u32(pos)? == OF_DT_BEGIN_NODE {
            pos = self.parse(pos, addr_cells, size_cells, cb)?;
        }
        if buf.read_be_u32(pos)? != OF_DT_END_NODE {
            return Err(DeviceTreeError::ParseError(pos));
        }
        pos += 4;
        Ok(pos)
    }
}

impl From<str::Utf8Error> for DeviceTreeError {
    fn from(_: str::Utf8Error) -> DeviceTreeError {
        DeviceTreeError::Utf8Error
    }
}
