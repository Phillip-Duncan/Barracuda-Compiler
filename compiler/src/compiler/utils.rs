pub fn pack_string_to_f64_array(input: &str, precision: usize) -> Vec<f64> {
    let mut result = Vec::new();
    let mut processed_input = String::new();
    
    // Process escape sequences
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next_c) = chars.peek() {
                match next_c {
                    'a' => {
                        processed_input.push('\x07');
                        chars.next(); // consume 'a'
                    }
                    'b' => {
                        processed_input.push('\x08');
                        chars.next(); // consume 'b'
                    }
                    'f' => {
                        processed_input.push('\x0C');
                        chars.next(); // consume 'f'
                    }
                    'n' => {
                        processed_input.push('\n');
                        chars.next(); // consume 'n'
                    }
                    'r' => {
                        processed_input.push('\r');
                        chars.next(); // consume 'r'
                    }
                    't' => {
                        processed_input.push('\t');
                        chars.next(); // consume 't'
                    }
                    'v' => {
                        processed_input.push('\x0B');
                        chars.next(); // consume 'v'
                    }
                    '\\' => {
                        processed_input.push('\\');
                        chars.next(); // consume '\\'
                    }
                    '\'' => {
                        processed_input.push('\'');
                        chars.next(); // consume '\''
                    }
                    '"' => {
                        processed_input.push('"');
                        chars.next(); // consume '"'
                    }
                    '?' => {
                        processed_input.push('?');
                        chars.next(); // consume '?'
                    }
                    _ => {
                        processed_input.push(c);
                    }
                }
            } else {
                processed_input.push(c);
            }
        } else {
            processed_input.push(c);
        }
    }

    let bytes = processed_input.as_bytes();
    
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
