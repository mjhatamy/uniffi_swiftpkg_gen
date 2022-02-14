use std::sync::Arc;
use crate::utils::keys_utils::{key_eq, key_to_base64, key_from_base64, key_from_hex, key_to_hex};
use crate::utils::x25519::{ curve25519_derive_public_key };
pub mod errors;
pub mod utils;
use rand::*;
pub use errors::GixTunnelErrorKind;
use self::utils::constants::{GXT_KEY_LEN};//, GXT_KEY_LEN_BASE64, GXT_KEY_LEN_HEX};

pub trait BaseKey {
    fn new(raw_value: Vec<u8>) -> Result<Self, GixTunnelErrorKind> where Self: Sized;
    fn raw_value(&self) -> Vec<u8>;

    fn is_eq(&self, other: &Self) -> bool {
        key_eq(Some(&self.raw_value()), Some(&other.raw_value()))
    }
    fn from_hex_key<T: BaseKey>(hex_key: String) -> Result<T, GixTunnelErrorKind> {
        match key_from_hex(Some(hex_key.as_bytes().to_vec())) {
            Ok(raw_value) => T::new(raw_value),
            Err(e) => Err(e)
        }
    }

    fn from_base64_str<T: BaseKey>(base64_key: String) -> Result<T, GixTunnelErrorKind> {
        match key_from_base64(Some(base64_key)) {
            Ok(raw_value) => T::new(raw_value),
            Err(e) => Err(e)
        }
    }

    /// Instance of baseKey is always valid, so, we expect it to return correct value.
    fn base64_key(&self) -> String {
        key_to_base64(Some(&self.raw_value())).unwrap_or_else(|_| "".to_string())
    }

    /// Instance of baseKey is always valid, so, we expect it to return correct value.
    fn hex_key(&self) -> String {
        //String::from_utf8(key_to_hex(Some(&self.raw_value())).unwrap_or(vec![])).unwrap_or("".to_string())
        String::from_utf8(key_to_hex(Some(&self.raw_value())).unwrap_or_default()).unwrap_or_else(|_| "".to_string())
    }

    fn mock<T: BaseKey>() -> T {
        let mut rng = rand::thread_rng();
        let rand_vals = (0..(GXT_KEY_LEN)).map(|_|rng.gen_range(0..255)).collect::<Vec<u8>>();
        T::new(rand_vals).unwrap()
    }
    
}

#[allow(unused)]
pub struct PrivateKey {
    raw_value: Vec<u8>
}

impl PrivateKey {
    pub fn public_key(&self) -> Arc<PublicKey> {
        let  mut public_key:[u8; GXT_KEY_LEN] = [0; GXT_KEY_LEN];
        let mut private_key: [u8; GXT_KEY_LEN] = [0; GXT_KEY_LEN];
        self.raw_value.iter().enumerate().for_each(|(i,c) | {
            private_key[i] = *c;
        });
        curve25519_derive_public_key(&mut public_key, &private_key);
        Arc::new(PublicKey {
            raw_value: public_key.to_vec()
        })
    }
}

impl BaseKey for PrivateKey {
    fn new(raw_value: Vec<u8>) -> Result<PrivateKey, GixTunnelErrorKind> {
        Ok(PrivateKey{
            raw_value
        })
    }

    fn raw_value(&self) -> Vec<u8> {
        self.raw_value.clone()
    }
}

pub struct PublicKey {
    raw_value: Vec<u8>
}

impl BaseKey for PublicKey {
    fn new(raw_value: Vec<u8>) -> Result<PublicKey, GixTunnelErrorKind> {
        Ok(PublicKey {
            raw_value
        })
    }

    fn raw_value(&self) -> Vec<u8> {
        self.raw_value.clone()
    }
}

pub struct PreSharedKey {
    raw_value: Vec<u8>
}

impl BaseKey for PreSharedKey {
    fn new(raw_value: Vec<u8>) -> Result<PreSharedKey, GixTunnelErrorKind> {
        Ok(PreSharedKey {
            raw_value
        })
    }

    fn raw_value(&self) -> Vec<u8> {
        self.raw_value.clone()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_key_test() {
        let p_key: PrivateKey = PrivateKey::mock();
        println!("pub key: {:?}", p_key.public_key().raw_value)

    }
}

include!(concat!(env!("OUT_DIR"), "/gix_guard.uniffi.rs"));
