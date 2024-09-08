use crate::crypto;
use crate::entropy;

static SALT_1: [u8; 32] = [
    0x40, 0xf0, 0xe4, 0xd9, 0x81, 0x80, 0x8d, 0x5a, 0xe6, 0x1a, 0x7c, 0xef, 0x10, 0xee, 0x8f, 0xd9,
    0x86, 0xc1, 0x2b, 0x1c, 0x3d, 0x02, 0x76, 0x46, 0x02, 0x3c, 0x2f, 0x3a, 0x6a, 0xf8, 0x54, 0x1a
];
static SALT_2: [u8; 32] = [
    0x25, 0x02, 0x13, 0xbb, 0x51, 0xb7, 0x9f, 0xe2, 0xb3, 0xaa, 0x41, 0xdf, 0xbf, 0x53, 0x5e, 0xdc,
    0x48, 0x4d, 0x48, 0x85, 0xa5, 0xe1, 0xe4, 0x6e, 0x34, 0x14, 0x14, 0xd1, 0x91, 0x1e, 0xb4, 0xab
];
static SALT_3: [u8; 32] = [
    0xbb, 0x9c, 0xe0, 0x46, 0x9a, 0x34, 0x29, 0x3f, 0x6f, 0x90, 0x7f, 0xb1, 0x6e, 0x5e, 0x2f, 0x1f,
    0xfb, 0xad, 0xfc, 0xff, 0xc4, 0xbc, 0xca, 0xdf, 0xc3, 0x02, 0x1a, 0x4e, 0x96, 0x61, 0x2b, 0x3c
];
static SALT_4: [u8; 32] = [
    0x3b, 0x20, 0x03, 0x89, 0x90, 0x8f, 0xc6, 0x47, 0x97, 0x4c, 0xe9, 0xf0, 0x72, 0xad, 0x9b, 0x57,
    0xd9, 0x76, 0x2f, 0x36, 0x31, 0x46, 0x08, 0xff, 0x3a, 0xff, 0xee, 0x4f, 0xa2, 0x92, 0x93, 0x8b
];
static SALT_5: [u8; 32] = [
    0x02, 0xc6, 0x88, 0x12, 0xcb, 0xf2, 0xeb, 0x86, 0x60, 0x32, 0xbb, 0x06, 0xe5, 0x4a, 0x05, 0x80,
    0x93, 0xe7, 0xb6, 0x78, 0x3f, 0xa8, 0x19, 0xbf, 0x52, 0x69, 0x89, 0x15, 0xf0, 0xb0, 0xe2, 0x5a
];

pub fn generate_stream(instance: &crate::PigeonInstance, size: u32) -> Vec<u8> {
    let mut entropy_vec: Vec<u8> = Vec::new();
    entropy_vec.append(&mut ((size % 65536) as u16).to_be_bytes().to_vec());
    entropy_vec.append(&mut instance.generated_streams_count.to_be_bytes().to_vec());
    entropy_vec.append(&mut instance.program_start_time.clone().to_be_bytes().to_vec());
    entropy_vec.append(&mut entropy::get_current_time_ns().to_be_bytes().to_vec());

    if let Ok(mouse_position_entropy) = instance.mouse_position_entropy.lock() {
        entropy_vec.push(1);
        entropy_vec.append(&mut crypto::hashes::whirlpool_512_compute(&mouse_position_entropy));
    } else {
        entropy_vec.push(0);
    }
    
    let mut last_used_entropy = instance.last_used_entropy.lock().unwrap();

    if !last_used_entropy.is_empty() {
        entropy_vec.push(1);
        entropy_vec.append(&mut last_used_entropy.clone());
    } else {
        entropy_vec.push(0);
    }

    entropy_vec.append(&mut entropy::generate_bytes(40));
    crypto::chacha20::encrypt_data(entropy::generate_bytes(32), entropy::generate_bytes(12), &mut entropy_vec);

    let mut last_used_entropy_vec = entropy_vec.clone();
    last_used_entropy_vec.append(&mut SALT_5.to_vec());
    *last_used_entropy = crypto::hashes::whirlpool_512_compute(&last_used_entropy_vec);

    let mut output_stream = Vec::new();
    let mut iterations = size / 32;
    if size % 32 != 0 {iterations += 1;}
    
    for i in 0..iterations {
        let mut salt = [SALT_1, SALT_2, SALT_3, SALT_4][iterations as usize % 4].to_vec();
        salt.append(&mut i.to_be_bytes().to_vec());
        salt.append(&mut output_stream.clone());
        salt = crypto::hashes::blake3_512_compute(&salt);

        let mut block = entropy_vec.clone();
        block.append(&mut salt);
        block.rotate_left(i as usize % 16 + 4);
        crypto::chacha20::encrypt_data(entropy::generate_bytes(32), entropy::generate_bytes(12), &mut block);
        
        output_stream.append(&mut crypto::hashes::perform_joined_digest(block));
        crypto::chacha20::encrypt_data(entropy::generate_bytes(32), entropy::generate_bytes(12), &mut output_stream);
    }

    output_stream.resize(size as usize, 0);
    return output_stream;
}

pub struct GeneratePasswordParams {
    pub size: u32,
    pub use_uppercase_chars: bool,
    pub use_lowercase_chars: bool,
    pub use_numbers: bool,
    pub use_logograms: bool,
    pub use_punctuation: bool,
    pub use_quotation_marks: bool,
    pub use_dashes_and_slashes: bool,
    pub use_maths_symbols: bool,
    pub use_brackets: bool
}

pub fn generate_password(instance: &crate::PigeonInstance, params: &GeneratePasswordParams) -> String {
    if params.size == 0 {
        return String::new();
    }

    let mut characters = String::new();
    if params.use_uppercase_chars    { characters += "ABCDEFGHIJKLMNOPQRSTUVWXYZ"; }
    if params.use_lowercase_chars    { characters += "abcdefghijklmnopqrstuvwxyz"; }
    if params.use_numbers            { characters += "0123456789";                 }
    if params.use_logograms          { characters += "#$%&@^`~";                   }
    if params.use_punctuation        { characters += ".,:;";                       }
    if params.use_quotation_marks    { characters += "\"'";                        }
    if params.use_dashes_and_slashes { characters += "\\/|_-";                     }
    if params.use_maths_symbols      { characters += "<>*+!?=";                    }
    if params.use_brackets           { characters += "()[]{}";                     }
    
    let characters_length = characters.len();
    if characters_length == 0 {
        return String::new();
    }
    let characters = characters.as_bytes();

    let mut password = String::new();
    let stream = generate_stream(instance, params.size);
    
    for i in 0..params.size {
        password.push(characters[stream[i as usize] as usize % characters_length] as char);
    }

    return password;
}