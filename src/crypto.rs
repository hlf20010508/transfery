/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use base64::engine::general_purpose::URL_SAFE as base64;
use base64::Engine;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};

use crate::error::Error;
use crate::error::ErrorType::InternalServerError;
use crate::error::Result;

const NONCE_SIZE: usize = 12;

#[derive(Debug, Clone)]
pub struct Crypto {
    key: LessSafeKey,
}

struct NoncePack {
    nonce: Nonce,
    raw: Vec<u8>,
}

impl Crypto {
    pub fn new(key_base64: &str) -> Result<Self> {
        let key_byte = base64.decode(key_base64).map_err(|e| {
            Error::context(
                InternalServerError,
                e,
                "failed to decode crypto key into Base64",
            )
        })?;

        let key_unbound = UnboundKey::new(&AES_256_GCM, &key_byte).map_err(|e| {
            Error::context(
                InternalServerError,
                e,
                "failed to create Crypto unbound key",
            )
        })?;

        let key = LessSafeKey::new(key_unbound);

        Ok(Self { key })
    }

    pub fn encrypt(&self, text: &str) -> Result<String> {
        let NoncePack {
            nonce,
            raw: nonce_raw,
        } = Self::gen_nonce_pack()?;

        let mut buffer = text.as_bytes().to_vec();

        self.key
            .seal_in_place_append_tag(nonce, Aad::from(b""), &mut buffer)
            .map_err(|e| Error::context(InternalServerError, e, "failed to encrypt in Crypto"))?;

        // insert nonce to the head of the output
        buffer.splice(..0, nonce_raw.into_iter());

        let result = base64.encode(buffer);

        Ok(result)
    }

    pub fn decrypt(&self, text: &str) -> Result<String> {
        let mut text_raw = base64.decode(text).map_err(|e| {
            Error::context(
                InternalServerError,
                e,
                "failed to decode encrypted text from Base64",
            )
        })?;

        let mut buffer = text_raw.split_off(NONCE_SIZE);
        let nonce_raw = text_raw;

        let nonce = Self::nonce_raw_to_nonce(&nonce_raw)?;

        let buffer = self
            .key
            .open_in_place(nonce, Aad::from(b""), &mut buffer)
            .map_err(|e| Error::context(InternalServerError, e, "failed to decrypt in Crypto"))?
            .to_vec();

        let result = String::from_utf8(buffer).map_err(|e| {
            Error::context(
                InternalServerError,
                e,
                "failed to convert decrypted text buffer Vec<u8> to String",
            )
        })?;

        Ok(result)
    }

    fn nonce_raw_to_nonce(nonce_raw: &Vec<u8>) -> Result<Nonce> {
        let nonce: [u8; NONCE_SIZE] = nonce_raw[..NONCE_SIZE].try_into().map_err(|e| {
            Error::context(InternalServerError, e, "failed to create nonce in Crypto")
        })?;

        let nonce = Nonce::assume_unique_for_key(nonce);

        Ok(nonce)
    }

    fn gen_nonce_pack() -> Result<NoncePack> {
        let nonce_raw = Self::gen_random_key(NONCE_SIZE)?;
        let nonce = Self::nonce_raw_to_nonce(&nonce_raw)?;

        Ok(NoncePack {
            nonce,
            raw: nonce_raw,
        })
    }

    pub fn gen_secret_key() -> Result<String> {
        let secret_key = Self::gen_random_key(32)?;
        let secret_key_str = base64.encode(secret_key);

        Ok(secret_key_str)
    }

    fn gen_random_key(size: usize) -> Result<Vec<u8>> {
        let rng = SystemRandom::new();

        let mut key = vec![0u8; size];

        rng.fill(&mut key)
            .map_err(|_| Error::new(InternalServerError, "failed to fill key"))?;

        Ok(key)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::utils::tests::sleep;

    pub fn get_crypto() -> Crypto {
        let key = Crypto::gen_secret_key().unwrap();
        Crypto::new(&key).unwrap()
    }

    #[test]
    fn test_crypto_gen_random_key() {
        let result = Crypto::gen_random_key(NONCE_SIZE);
        assert_eq!(result.unwrap().len(), NONCE_SIZE);

        sleep(1);
    }

    #[test]
    fn test_crypto_gen_secret_key() {
        let result = Crypto::gen_secret_key();
        assert_eq!(result.unwrap().len(), 44);

        sleep(1);
    }

    #[test]
    fn test_crypto_gen_nonce_pack() {
        Crypto::gen_nonce_pack().unwrap();

        sleep(1);
    }

    #[test]
    fn test_crypto_nonce_raw_to_nonce() {
        let NoncePack {
            nonce: _,
            raw: nonce_raw,
        } = Crypto::gen_nonce_pack().unwrap();

        Crypto::nonce_raw_to_nonce(&nonce_raw).unwrap();

        sleep(1);
    }

    #[test]
    fn test_crypto_new() {
        let key = Crypto::gen_secret_key().unwrap();
        Crypto::new(&key).unwrap();

        sleep(1);
    }

    #[test]
    fn test_crypto_encrypt_decrypt() {
        let crypto = get_crypto();

        let text = "This is a test for crypto.";
        let text_encrypted = crypto.encrypt(text).unwrap();
        let text_decrypted = crypto.decrypt(&text_encrypted).unwrap();

        assert_eq!(text, &text_decrypted);

        sleep(1);
    }
}
