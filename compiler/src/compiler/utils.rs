pub fn pack_string_to_f64_array(input: &str, precision: usize) -> Vec<f64> {
    let mut result = Vec::new();
    let bytes = input.as_bytes();
    
    let chunk_size = match precision {
        32 => 4,  // 4 bytes for 32-bit precision
        64 => 8,  // 8 bytes for 64-bit precision
        _ => panic!("Unsupported precision: {}", precision),
    };

    for chunk in bytes.chunks(chunk_size) {
        let mut packed: u64 = 0;
        for (i, &byte) in chunk.iter().enumerate() {
            packed |= (byte as u64) << (i * 8);
        }

        // Shift the packed value to the left to fill the remaining bits with 0s
        packed <<= 8 * (chunk_size - chunk.len());

        match precision {
            32 => result.push(f64::from(f32::from_bits(packed as u32))),
            64 => result.push(f64::from_bits(packed)),
            _ => unreachable!(),
        }
    }

    result
}

