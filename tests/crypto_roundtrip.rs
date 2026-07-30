//! End-to-end check of the encrypt/decrypt pipeline against the bundled
//! MuleSoft Secure Properties Tool jar.
//!
//! Ignored by default because it requires a JRE on PATH and the jar file.
//! Run explicitly with:
//!
//! ```sh
//! cargo test --test crypto_roundtrip -- --ignored
//! ```

use std::path::PathBuf;

use lazyprop::dencrypt::{decrypt, encrypt};
use lazyprop::environment::{Algorithm, Environment, State};

#[test]
#[ignore = "requires java + secure-properties-tool.jar"]
fn encrypt_then_decrypt_roundtrips() {
    let jar = PathBuf::from("secure-properties-tool.jar");
    let env = Environment::new(
        "Test",
        Algorithm::AES,
        State::CBC,
        false,
        "secret1234567890",
    );

    let plaintext = "helloWorld";
    let cipher = encrypt(plaintext, &env, &jar).expect("encrypt should succeed");
    assert_ne!(cipher, plaintext, "ciphertext must differ from plaintext");

    let recovered = decrypt(&cipher, &env, &jar).expect("decrypt should succeed");
    assert_eq!(recovered, plaintext, "decrypt must recover the plaintext");
}
