use std::collections::HashMap;
use std::{io, fmt};
use std::io::{ErrorKind, Write};
use std::cell::RefCell;
use std::borrow::Borrow;
use endiannezz::{Primitive, NativeEndian, LittleEndian};
use endiannezz::ext::{EndianWriter, EndianReader};
use std::mem::size_of;

/// Memory heap used by MathStack program code
/// ## Summary
/// Virtualized as a vector of memory regions. To simplify page allocation that the gpu would
/// be using the pointers given for memory are stored with the first 16 bits storing the memory
/// region address (First vector) and the later 48 bits storing the byte address (Second Vector)
/// in a memory region. This gives a maximum of 65536 malloc allocations without free and a
/// maximum of 256 TB for each memory region.
/// ## Limitations
/// + The heap is limited to 2^16 memory regions allocated.
/// + Virtual address are not fully emulated but approximated
/// + Currently assumes that previous mallocs will be freed somewhat sequentially. i.e when checking
///   if all regions are in use instead of checking for a single free region out of 2^16. It will
///   roll back around and if it meets a region address already in use it will assume there is no
///   free address. Not a perfect solution but suitable for the current application.
pub(crate) struct EmulatorHeap {
    heap: HashMap<u16, RefCell<Vec<u8>>>, // Stored as refcell to borrow individual elements
    next_region_address: HeapAddress
}


const HEAP_ADDRESS_BYTE_MASK: usize = 0x0000FFFF_FFFFFFFF;
struct HeapAddress {
    region_index: u16,
    byte_index: u64 // Only 48bits should be used.
}

impl HeapAddress {
    fn max_region_count() -> u64 {
       u64::pow(2, 16)
    }

    fn max_byte_count() -> u64 {
        u64::pow(2, 48)
    }

    fn from_real(region_index: u16, byte_address: u64) -> Self {
        HeapAddress {
            region_index,
            byte_index: (byte_address & HEAP_ADDRESS_BYTE_MASK as u64) as u64
        }
    }

    fn from_virtual(pointer: usize) -> Self {
        HeapAddress {
            region_index: (pointer >> 48) as u16,
            byte_index: (pointer & HEAP_ADDRESS_BYTE_MASK) as u64
        }
    }

    fn region_index(&self) -> u16 {
        self.region_index
    }

    fn byte_index(&self) -> u64 {
        self.byte_index
    }

    fn virtual_addr(&self) -> usize {
        (((self.region_index as u64) << 48) | self.byte_index) as usize
    }

    fn next_region(&self) -> Self {
        match self.region_index.checked_add(1) {
            Some(next_index) => HeapAddress {
                region_index: next_index,
                byte_index: 0
            },
            None => HeapAddress {
                region_index: 0,
                byte_index: 0
            }
        }
    }
}

impl fmt::Debug for HeapAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VirtAddress({:#018X})", self.virtual_addr())
    }
}

impl EmulatorHeap {
    pub(crate) fn new() -> Self {
        EmulatorHeap {
            heap: HashMap::new(),
            next_region_address: HeapAddress::from_real(0,0)
        }
    }

    /// Allocates a memory region within the heap and returns a virtual address to refer to when
    /// writing or reading data. The region assigned is of size @num_bytes. A maximum of 2^16 memory
    /// regions can be allocated at a time.
    /// @num_bytes: Number of bytes to allocate in new region
    /// @return: A virtual address of memory region if Ok, otherwise an OutOfMemory error if all
    /// regions are allocated.
    pub(crate) fn malloc(&mut self, num_bytes: usize) -> Result<usize, io::Error> {
        if self.heap.contains_key(&self.next_region_address.region_index) {
            return Err(io::Error::new(ErrorKind::OutOfMemory, "Malloc was attempted while all memory regions were allocated"))
        }

        if num_bytes >= HeapAddress::max_byte_count() as usize {
            return Err(io::Error::new(ErrorKind::OutOfMemory, "Malloc was attempted with a capacity greater than max possible."))
        }

        self.heap.insert(self.next_region_address.region_index, RefCell::new(vec![0; num_bytes]));
        let region_pointer = self.next_region_address.virtual_addr();
        self.next_region_address = self.next_region_address.next_region();

        Ok(region_pointer)
    }

    /// Frees a previously allocated region.
    /// @virtual_pointer: A pointer to any byte within the memory region to be freed.
    /// @return: Nothing if Ok, A NotFound error if region is already free.
    pub(crate) fn free(&mut self, virtual_pointer: usize) -> Result<(), io::Error> {
        let pointer = HeapAddress::from_virtual(virtual_pointer);

        match self.heap.remove(&pointer.region_index) {
            Some(_) => Ok(()),
            // Although this can be safely done inside the emulator. It can cause hard to debug bugs
            // on real hardware and therefore should be alerted if attempted in the emulator.
            None => Err(io::Error::new(ErrorKind::NotFound, "Free was attempted on an already free memory region"))
        }
    }

    /// Copies data from one part of the heap to another using virtual addresses. The underlying
    /// code also uses Rust's implementation of memcpy and so should be just as fast.
    /// @dest_virtual: Destination virtual address at first byte of byffer
    /// @src_virtual: Source virtual address at first byte of buffer
    /// @byte_count: Number of bytes to copy from source to destination.
    /// @return: Nothing if Ok, Otherwise NotFound error if address regions are out of bounds.
    pub(crate) fn memcpy(&mut self, dest_virtual: usize, src_virtual: usize, byte_count: usize) -> Result<(), io::Error>{

        // Get source and destination byte ranges
        let src = HeapAddress::from_virtual(src_virtual);
        let src_start = src.byte_index as usize;
        let src_end = src_start + byte_count;

        let dest = HeapAddress::from_virtual(dest_virtual);
        let dest_start = dest.byte_index as usize;
        let dest_end = dest_start + byte_count;

        // Get source and destination byte slices while checking for out of bounds errors
        let src_region = self.heap.get(&src.region_index)
            .ok_or(io::Error::new(ErrorKind::NotFound, format!("Memory copy source region was not found {:?}", src)))?;
        let src_region = src_region.borrow();
        let src_bytes = src_region.get(src_start..src_end)
            .ok_or(io::Error::new(ErrorKind::NotFound, format!("Memory copy source byte region invalid {:?}..+{}", src, byte_count)))?;

        let dest_region = self.heap.get(&dest.region_index)
            .ok_or(io::Error::new(ErrorKind::NotFound, format!("Memory copy destination region was not found {:?}", dest)))?;
        let mut dest_region = dest_region.borrow_mut();
        let dest_bytes = dest_region.get_mut(dest_start..dest_end)
            .ok_or(io::Error::new(ErrorKind::NotFound, format!("Memory copy destination byte region invalid {:?}..+{}", dest, byte_count)))?;

        dest_bytes.copy_from_slice(&src_bytes);
        Ok(())
    }

    /// Fills memory buffer with @fill_value
    /// @dest_virtual: Virtual address of first byte of memory buffer
    /// @fill_value: value to set each byte in buffer into
    /// @byte_count: Number of bytes to fill in buffer.
    /// @return: Nothing if Ok, otherwise NotFound if buffer region invalid
    pub(crate) fn memset(&mut self, dest_virtual: usize, fill_value: u8, byte_count: usize) -> Result<(), io::Error> {
        let dest = HeapAddress::from_virtual(dest_virtual);
        let dest_start = dest.byte_index as usize;
        let dest_end = dest_start + byte_count;

        let dest_region = self.heap.get(&dest.region_index)
            .ok_or(io::Error::new(ErrorKind::NotFound, format!("Memory copy destination region was not found {:?}", dest)))?;
        let mut dest_region = dest_region.borrow_mut();
        let dest_bytes = dest_region.get_mut(dest_start..dest_end)
            .ok_or(io::Error::new(ErrorKind::NotFound, format!("Memory copy destination byte region invalid {:?}..+{}", dest, byte_count)))?;

        dest_bytes.fill(fill_value);
        Ok(())
    }

    /// Reads byte from virtual heap
    /// @address_virtual: Virtual address of byte pointer
    /// @return byte value if Ok, otherwise NotFound error
    pub(crate) fn read(&mut self, address_virtual: usize) -> Result<u8, io::Error> {
        let address = HeapAddress::from_virtual(address_virtual);

        let memory_region = self.heap.get(&address.region_index)
            .ok_or(io::Error::new(ErrorKind::NotFound, format!("Memory read region was not found {:?}", address)))?;
        let memory_region = memory_region.borrow();

        let value = memory_region.get(address.byte_index as usize)
            .ok_or(io::Error::new(ErrorKind::NotFound, format!("Memory read byte was not found {:?}", address)))?;

        Ok(value.clone())
    }

    /// Writes byte into virtual heap
    /// @address_virtual: Virtual address of byte pointer
    /// @value: Value to assign byte
    /// @return: Nothing if Ok, otherwise NotFound error
    pub(crate) fn write(&mut self, address_virtual: usize, value: u8) -> Result<(), io::Error> {
        let address = HeapAddress::from_virtual(address_virtual);

        let memory_region = self.heap.get(&address.region_index)
            .ok_or(io::Error::new(ErrorKind::NotFound, format!("Memory read region was not found {:?}", address)))?;
        let mut memory_region = memory_region.borrow_mut();

        let mut memory_byte = memory_region.get_mut(address.byte_index as usize)
            .ok_or(io::Error::new(ErrorKind::NotFound, format!("Memory read byte was not found {:?}", address)))?;

        *memory_byte = value;
        Ok(())
    }

    pub(crate) fn write_word<T: Primitive>(&mut self, address_virtual: usize, value: T) -> Result<(), io::Error> {
        let mut byte_buffer = Vec::new();
        byte_buffer.try_write::<LittleEndian, T>(value)?;
        let byte_count = byte_buffer.len();

        let address_start = HeapAddress::from_virtual(address_virtual);
        let address_end = HeapAddress::from_virtual(address_virtual + byte_count);

        let memory_region = self.heap.get(&address_start.region_index)
            .ok_or(io::Error::new(ErrorKind::NotFound, format!("Memory read region was not found {:?}", address_start)))?;
        let mut memory_region = memory_region.borrow_mut();

        let mut memory_bytes = memory_region.get_mut((address_start.byte_index as usize)..(address_end.byte_index as usize))
            .ok_or(io::Error::new(ErrorKind::NotFound, format!("Memory read byte was not found {:?}", address_start)))?;


        memory_bytes.clone_from_slice(&byte_buffer);
        Ok(())
    }

    pub(crate) fn read_word<T: Primitive>(&mut self, address_virtual: usize) -> Result<T, io::Error> {
        let byte_count = size_of::<T>();
        let address_start = HeapAddress::from_virtual(address_virtual);
        let address_end = HeapAddress::from_virtual(address_virtual + byte_count);

        let memory_region = self.heap.get(&address_start.region_index)
            .ok_or(io::Error::new(ErrorKind::NotFound, format!("Memory read region was not found {:?}", address_start)))?;
        let mut memory_region = memory_region.borrow_mut();

        let mut memory_bytes = memory_region.get((address_start.byte_index as usize)..(address_end.byte_index as usize))
            .ok_or(io::Error::new(ErrorKind::NotFound, format!("Memory read byte was not found {:?}", address_start)))?;


        let value: T = memory_bytes.try_read::<LittleEndian, T>()?;
        Ok(value)
    }

}