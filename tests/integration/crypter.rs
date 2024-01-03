use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use divviup_api::Crypter;
use test_support::{assert_eq, test};

const AAD: &[u8] = b"aad";
const PLAINTEXT: &[u8] = b"plaintext";

#[test]
fn round_trip_with_current_key() {
    let crypter = Crypter::from(Crypter::generate_key());
    let encrypted = crypter.encrypt(AAD, PLAINTEXT).unwrap();
    assert_eq!(crypter.decrypt(AAD, &encrypted).unwrap(), PLAINTEXT);
}

#[test]
fn round_trip_with_old_key() {
    let old_key = Crypter::generate_key();
    let crypter = Crypter::from(old_key);
    let encrypted = crypter.encrypt(AAD, PLAINTEXT).unwrap();

    let crypter = Crypter::new(Crypter::generate_key(), [old_key]);
    assert_eq!(crypter.decrypt(AAD, &encrypted).unwrap(), PLAINTEXT);
}

#[test]
fn wrong_key() {
    let crypter = Crypter::from(Crypter::generate_key());
    let encrypted = crypter.encrypt(AAD, PLAINTEXT).unwrap();

    let crypter = Crypter::from(Crypter::generate_key());
    assert!(crypter.decrypt(AAD, &encrypted).is_err());
}

#[test]
fn wrong_aad() {
    let crypter = Crypter::from(Crypter::generate_key());
    let encrypted = crypter.encrypt(AAD, PLAINTEXT).unwrap();
    assert!(crypter.decrypt(b"different aad", &encrypted).is_err());
}

#[test]
fn short_input_does_not_panic() {
    let crypter = Crypter::from(Crypter::generate_key());
    assert!(crypter.decrypt(AAD, b"x").is_err());
}

#[test]
fn parsing() {
    let keys = std::iter::repeat_with(Crypter::generate_key)
        .take(5)
        .collect::<Vec<_>>();
    let encrypted = Crypter::from(keys[0]).encrypt(AAD, PLAINTEXT).unwrap();
    let crypter = keys
        .iter()
        .map(|k| URL_SAFE_NO_PAD.encode(k))
        .collect::<Vec<_>>()
        .join(",")
        .parse::<Crypter>()
        .unwrap();
    assert_eq!(crypter.decrypt(AAD, &encrypted).unwrap(), PLAINTEXT);
}
