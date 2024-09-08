use sha3::{Digest, Sha3_256, Sha3_512};
use whirlpool::Whirlpool;

pub fn sha3_256_compute(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    return hasher.finalize().as_slice().to_vec();
}

pub fn sha3_512_compute(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha3_512::new();
    hasher.update(data);
    return hasher.finalize().as_slice().to_vec();
}

pub fn blake3_generate_stream(data: &[u8], size: usize) -> Vec<u8> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(data);
    let mut output = vec![0u8; size];
    let mut output_reader = hasher.finalize_xof();
    output_reader.fill(&mut output);
    return output.to_vec();
}

pub fn blake3_256_compute(data: &[u8]) -> Vec<u8> {
    return blake3_generate_stream(data, 32);
}

pub fn blake3_512_compute(data: &[u8]) -> Vec<u8> {
    return blake3_generate_stream(data, 64);
}

pub fn whirlpool_512_compute(data: &[u8]) -> Vec<u8> {
    let mut hasher = Whirlpool::new();
    hasher.update(data);
    return hasher.finalize().as_slice().to_vec();
}

pub fn perform_joined_digest(data: Vec<u8>) -> Vec<u8> {
    let mut state: Vec<Vec<u8>> = Vec::new();
    
    for _ in 0..3 { state.push(data.clone()); }
    for _ in 0..8 {
        state[0].rotate_left(3);
        state[1].rotate_left(8);
        state[2].rotate_left(13);

        state[0] = sha3_512_compute(&state[0]);
        state[1] = blake3_512_compute(&state[1]);
        state[2] = whirlpool_512_compute(&state[2]);

        state.rotate_left(1);
    }

    state[0] = sha3_256_compute(&state[0]);
    state[1] = blake3_256_compute(&state[1]);
    state[2] = whirlpool_512_compute(&state[2]);
 
    let mut output_state = vec![0; 32];
    for i in 0..32 {
        output_state[i]             =  state[0][i].wrapping_add(state[1][i]).rotate_left(4 + (i as u32 % 2));
        output_state[i]             ^= state[2][i + 16].rotate_left(2 + ((i + 1) as u32 % 2));
    }

    return output_state;
}