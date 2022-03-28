use crate::emulator::emulator_heap::EmulatorHeap;

#[test]
fn test_malloc() {
    let mut heap = EmulatorHeap::new();
    let address_a = heap.malloc(256).unwrap();
    let address_b = heap.malloc(256).unwrap();
    let address_c = heap.malloc(256).unwrap();

    assert_eq!(address_a, 0x00_00_00_00__00_00_00_00);
    assert_eq!(address_b, 0x00_01_00_00__00_00_00_00);
    assert_eq!(address_c, 0x00_02_00_00__00_00_00_00);
}

#[test]
fn test_max_malloc_byte_size() {
    let mut heap = EmulatorHeap::new();
    heap.malloc((u64::pow(2, 48) + 1) as usize)
        .expect_err("Should fail to allocate the region");

    // Would test boundary but boundary is 256 TB.
}

#[test]
fn test_max_alloc_regions() {
    let mut heap = EmulatorHeap::new();

    for i in 0..usize::pow(2, 16) {
        heap.malloc(0).unwrap();
    }

    heap.malloc(0)
        .expect_err("Should fail to allocate the region");
}

#[test]
fn test_free_region() {
    let mut heap = EmulatorHeap::new();
    let memory_addr = heap.malloc(256).unwrap();
    heap.free(memory_addr).expect("Testing free region");
    heap.free(memory_addr).expect_err("Testing double free region");
}

#[test]
fn test_write_read() {
    let mut heap = EmulatorHeap::new();
    let address_start = heap.malloc(256).unwrap();
    let address_mid = address_start + 128;
    let address_end = address_start + 255;

    let expected_value: u8 = 42;
    heap.write(address_start, expected_value);
    heap.write(address_end, expected_value);

    assert_eq!(expected_value, heap.read(address_start).unwrap());
    assert_eq!(0, heap.read(address_mid).unwrap());
    assert_eq!(expected_value, heap.read(address_end).unwrap());
}

#[test]
fn test_invalid_write() {
    let mut heap = EmulatorHeap::new();
    let byte_count = 10;
    let address_start = heap.malloc(byte_count).unwrap();
    let valid_address = address_start + byte_count - 1;
    let invalid_address = valid_address + 1;

    heap.write(valid_address, 42).expect("Testing valid write");
    heap.write(invalid_address, 42).expect_err("Testing invalid write");
}


#[test]
fn test_invalid_read() {
    let mut heap = EmulatorHeap::new();
    let byte_count = 10;
    let address_start = heap.malloc(byte_count).unwrap();
    let valid_address = address_start + byte_count - 1;
    let invalid_address = valid_address + 1;

    heap.read(valid_address).expect("Testing valid read");
    heap.read(invalid_address).expect_err("Testing invalid read");
}

#[test]
fn test_memset() {
    let mut heap = EmulatorHeap::new();
    let address_start = heap.malloc(256).unwrap();
    let buffer_start = address_start + 128;
    let byte_count: usize = 64;
    heap.memset(buffer_start, 42, byte_count);

    for i in 0..byte_count {
        let value = heap.read(buffer_start + i).unwrap();
        assert_eq!(value, 42);
    }
    let value = heap.read(buffer_start+byte_count).unwrap();
    assert_eq!(value, 0);
}

#[test]
fn test_memcpy() {
    let mut heap = EmulatorHeap::new();
    let address_src = heap.malloc(256).unwrap();
    let address_dest = heap.malloc(256).unwrap();
    let byte_count: usize = 64;

    // Write bytes
    for i in 0..byte_count {
        heap.write(address_src+i, i as u8);
    }

    // Copy bytes
    heap.memcpy(address_dest, address_src, byte_count).unwrap();

    // Read bytes and check copied correctly
    for i in 0..byte_count {
        let value = heap.read(address_dest+i).unwrap();
        assert_eq!(value, i as u8);
    }

    // Check that value outside of buffer is not touched
    let value = heap.read(address_dest+byte_count).unwrap();
    assert_eq!(value, 0);
}