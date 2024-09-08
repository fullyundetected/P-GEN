use chacha20::ChaCha20;
use chacha20::cipher::{KeyIvInit, StreamCipher};

pub fn encrypt_data(key: Vec<u8>, iv: Vec<u8>, data: &mut Vec<u8>) {
    let key = key.as_slice();
    let iv = iv.as_slice();

    let mut cipher = ChaCha20::new(key.into(), iv.into());
    cipher.apply_keystream(data);
}