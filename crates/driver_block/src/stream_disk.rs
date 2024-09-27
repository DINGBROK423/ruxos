use std::{
    fs::{self, File},
    io::{self, Read, Seek, Write},
};

use driver_common::{BaseDriverOps, DevError, DevResult, DeviceType};
use fscommon::BufStream;

use crate::BlockDriverOps;

const BLOCK_SIZE: usize = 512;

/// A RAM disk that stores data in a BufStream.
pub struct RamDisk {
    inner: BufStream<File>,
    size: usize,
}

impl RamDisk {
    pub fn new(image: &str) -> Self {
        let size_hint = fs::metadata(image).unwrap().len() as usize;
        let size = align_up(size_hint);

        let mut config = std::fs::File::options();
        let image_file = config.read(true).write(true).open(image).unwrap();

        let inner = BufStream::new(image_file);
        Self { inner, size }
    }

    /// Returns the size of the RAM disk in bytes.
    pub const fn size(&self) -> usize {
        self.size
    }
}

impl const BaseDriverOps for RamDisk {
    fn device_type(&self) -> DeviceType {
        DeviceType::Block
    }

    fn device_name(&self) -> &str {
        "ramdisk"
    }
}

impl BlockDriverOps for RamDisk {
    #[inline]
    fn num_blocks(&self) -> u64 {
        (self.size / BLOCK_SIZE) as u64
    }

    #[inline]
    fn block_size(&self) -> usize {
        BLOCK_SIZE
    }

    fn read_block(&mut self, block_id: u64, buf: &mut [u8]) -> DevResult {
        let offset = block_id as usize * BLOCK_SIZE;
        if offset + buf.len() > self.size {
            return Err(DevError::Io);
        }
        if buf.len() % BLOCK_SIZE != 0 {
            return Err(DevError::InvalidParam);
        }
        // buf.copy_from_slice(&self.data[offset..offset + buf.len()]);
        self.inner
            .seek(io::SeekFrom::Start(offset as u64))
            .map_err(|_| DevError::Io)?;
        self.inner.read(buf).map_err(|_| DevError::Io)?;
        Ok(())
    }

    fn write_block(&mut self, block_id: u64, buf: &[u8]) -> DevResult {
        let offset = block_id as usize * BLOCK_SIZE;
        if offset + buf.len() > self.size {
            return Err(DevError::Io);
        }
        if buf.len() % BLOCK_SIZE != 0 {
            return Err(DevError::InvalidParam);
        }
        // self.data[offset..offset + buf.len()].copy_from_slice(buf);
        self.inner
            .seek(io::SeekFrom::Start(offset as u64))
            .map_err(|_| DevError::Io)?;
        self.inner.write(buf).map_err(|_| DevError::Io)?;
        Ok(())
    }

    fn flush(&mut self) -> DevResult {
        Ok(())
    }
}

impl Default for RamDisk {
    fn default() -> Self {
        Self {
            size: Default::default(),
            inner: unimplemented!(),
        }
    }
}

const fn align_up(val: usize) -> usize {
    (val + BLOCK_SIZE - 1) & !(BLOCK_SIZE - 1)
}
