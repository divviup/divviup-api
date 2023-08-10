use std::{ops::Deref, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kem(pub hpke_dispatch::Kem);
impl Deref for Kem {
    type Target = hpke_dispatch::Kem;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Kem {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &*s.to_lowercase() {
            "dhp256hkdfsha256" | "dh-p256-hkdf-sha256" | "dhkem(p-256, hkdf-sha256)" => {
                Ok(Self(hpke_dispatch::Kem::DhP256HkdfSha256))
            }
            "x25519hkdfsha256" | "x25519-hkdf-sha256" | "dhkem(x25519, hkdf-sha256)" => {
                Ok(Self(hpke_dispatch::Kem::X25519HkdfSha256))
            }
            _ => Err(ParseError("kem", s.into())),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kdf(pub hpke_dispatch::Kdf);
impl FromStr for Kdf {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &*s.to_lowercase() {
            "hkdfsha256" | "sha256" | "sha-256" => Ok(Self(hpke_dispatch::Kdf::Sha256)),
            "hkdfsha384" | "sha384" | "sha-384" => Ok(Self(hpke_dispatch::Kdf::Sha384)),
            "hkdfsha512" | "sha512" | "sha-512" => Ok(Self(hpke_dispatch::Kdf::Sha512)),
            _ => Err(ParseError("kdf", s.into())),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Aead(pub hpke_dispatch::Aead);
impl FromStr for Aead {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &*s.to_lowercase() {
            "aes128gcm" | "aesgcm128" | "aes-gcm-128" | "aes-128-gcm" => {
                Ok(Self(hpke_dispatch::Aead::AesGcm128))
            }
            "aes256gcm" | "aesgcm256" | "aes-gcm-256" | "aes-256-gcm" => {
                Ok(Self(hpke_dispatch::Aead::AesGcm256))
            }
            "chacha20poly1305"
            | "chacha-20-poly-1305"
            | "cha-cha-20-poly-1305"
            | "chacha20-poly1305" => Ok(Self(hpke_dispatch::Aead::ChaCha20Poly1305)),
            _ => Err(ParseError("aead", s.into())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseError(&'static str, String);
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} `{}` not recognized", self.0, self.1))
    }
}
impl std::error::Error for ParseError {}
