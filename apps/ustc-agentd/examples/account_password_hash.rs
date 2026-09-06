//! Local operator helper. Capture stdout directly into the private configuration;
//! stdin carries the exact password bytes, without a newline or normalization.
use argon2::{Algorithm, Argon2, Params, PasswordHasher, Version, password_hash::SaltString};
use std::io::{Read, Write};

fn hash() -> Result<String, ()> {
    let mut bytes = Vec::new();
    std::io::stdin()
        .take(1025)
        .read_to_end(&mut bytes)
        .map_err(|_| ())?;
    let password = String::from_utf8(bytes).map_err(|_| ())?;
    if password.len() > 1024 || !(12..=256).contains(&password.chars().count()) {
        return Err(());
    }
    let mut salt = [0; 16];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut salt))
        .map_err(|_| ())?;
    let salt = SaltString::encode_b64(&salt).map_err(|_| ())?;
    let params = Params::new(19_456, 2, 1, Some(32)).map_err(|_| ())?;
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| ())
}
fn main() {
    match hash() {
        Ok(hash) => {
            if std::io::stdout().write_all(hash.as_bytes()).is_err() {
                std::process::exit(1);
            }
        }
        Err(()) => {
            eprintln!("account password preparation failed");
            std::process::exit(1);
        }
    }
}
