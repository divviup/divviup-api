use aes_gcm::{
    aead::{AeadCore, AeadInPlace, KeyInit, OsRng},
    Aes128Gcm as AesGcm, Error, KeySizeUser, Nonce,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, DecodeError, Engine};
use std::{
    collections::VecDeque,
    fmt::{self, Debug, Formatter},
    iter,
    str::FromStr,
    sync::Arc,
};
use typenum::marker_traits::Unsigned;

pub type Key = aes_gcm::Key<AesGcm>;

#[derive(Clone)]
pub struct Crypter(Arc<CrypterInner>);

#[derive(thiserror::Error, Debug, Clone)]
pub enum CrypterParseError {
    #[error(transparent)]
    Base64(#[from] DecodeError),

    #[error("incorrect key length, must be {} bytes after base64 decode", <AesGcm as KeySizeUser>::KeySize::USIZE)]
    IncorrectLength,

    #[error("at least one key needed")]
    Missing,
}

impl FromStr for Crypter {
    type Err = CrypterParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut keys = s
            .split(',')
            .map(|s| {
                URL_SAFE_NO_PAD
                    .decode(s)
                    .map_err(CrypterParseError::Base64)
                    .and_then(|v| Key::from_exact_iter(v).ok_or(CrypterParseError::IncorrectLength))
            })
            .collect::<Result<VecDeque<Key>, _>>()?;
        let current_key = keys.pop_front().ok_or(CrypterParseError::Missing)?;
        Ok(Self::new(current_key, keys))
    }
}

impl Debug for Crypter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Crypter")
            .field("current_ciphers", &"..")
            .field("past_ciphers", &self.0.past_ciphers.len())
            .finish()
    }
}

#[derive(Clone)]
struct CrypterInner {
    current_cipher: AesGcm,
    past_ciphers: Vec<AesGcm>,
}

impl From<Key> for Crypter {
    fn from(key: Key) -> Self {
        Self::new(key, [])
    }
}

impl Crypter {
    pub fn new(current_key: Key, past_keys: impl IntoIterator<Item = Key>) -> Self {
        Self(Arc::new(CrypterInner {
            current_cipher: AesGcm::new(&current_key),
            past_ciphers: past_keys.into_iter().map(|key| AesGcm::new(&key)).collect(),
        }))
    }

    pub fn generate_key() -> Key {
        AesGcm::generate_key(OsRng)
    }

    pub fn encrypt(&self, associated_data: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, Error> {
        self.0.encrypt(associated_data, plaintext)
    }

    pub fn decrypt(
        &self,
        associated_data: &[u8],
        nonce_and_ciphertext: &[u8],
    ) -> Result<Vec<u8>, Error> {
        self.0.decrypt(associated_data, nonce_and_ciphertext)
    }
}

impl CrypterInner {
    fn encrypt(&self, associated_data: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, Error> {
        let nonce = AesGcm::generate_nonce(&mut OsRng);
        let mut buffer = plaintext.to_vec();
        self.current_cipher
            .encrypt_in_place(&nonce, associated_data, &mut buffer)?;
        let mut nonce_and_ciphertext = nonce.to_vec();
        nonce_and_ciphertext.append(&mut buffer);
        Ok(nonce_and_ciphertext)
    }

    fn decrypt(
        &self,
        associated_data: &[u8],
        nonce_and_ciphertext: &[u8],
    ) -> Result<Vec<u8>, Error> {
        let nonce_size = <AesGcm as AeadCore>::NonceSize::USIZE;
        if nonce_and_ciphertext.len() < nonce_size {
            return Err(Error);
        }

        let (nonce, ciphertext) = nonce_and_ciphertext.split_at(nonce_size);

        self.cipher_iter()
            .find_map(|cipher| {
                self.decrypt_with_key(cipher, associated_data, nonce, ciphertext)
                    .ok()
            })
            .ok_or(Error)
    }

    fn cipher_iter(&self) -> impl Iterator<Item = &AesGcm> {
        iter::once(&self.current_cipher).chain(self.past_ciphers.iter())
    }

    fn decrypt_with_key(
        &self,
        cipher: &AesGcm,
        associated_data: &[u8],
        nonce: &[u8],
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, Error> {
        let nonce = Nonce::from_slice(nonce);
        let mut bytes = ciphertext.to_vec();
        cipher.decrypt_in_place(nonce, associated_data, &mut bytes)?;
        Ok(bytes)
    }
}
