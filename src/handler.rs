use chrono::Utc;
use ed25519_dalek::SigningKey;
use hex::ToHex;
use regex::Regex;

use std::fs;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Mutex;

/// Report every regex match instead of only those with a new record height.
pub static ALL_MATCHES: AtomicBool = AtomicBool::new(false);

pub fn handle_keypair(
    seed: &[u8],
    pk: &[u8],
    regex_sources: &[String],
    regexes: &[Regex],
    max_leading_zeros: &[AtomicU8],
    csv_file: &Option<Mutex<fs::File>>,
) {
    let leading_zeros = leading_zeros_of_pubkey(pk);

    let all_matches = ALL_MATCHES.load(Ordering::Relaxed);
    let mut str_addr = None;

    for ((re, mlz), re_src) in regexes
        .iter()
        .zip(max_leading_zeros.iter())
        .zip(regex_sources.iter())
    {
        if all_matches || mlz.load(Ordering::Relaxed) <= leading_zeros {
            let str_addr = str_addr.get_or_insert_with(|| address_for_pubkey(pk).to_string());

            if re.is_match(str_addr)
                && (all_matches || mlz.fetch_max(leading_zeros, Ordering::SeqCst) <= leading_zeros)
            {
                {
                    let mut fixed_seed = [0; 32];
                    fixed_seed.copy_from_slice(seed);
                    let signing_key = SigningKey::from_bytes(&fixed_seed);
                    let verifying_key_bytes = signing_key.verifying_key().to_bytes();
                    assert_eq!(verifying_key_bytes, pk);
                }

                let mut sk = [0u8; 64];
                sk[..32].copy_from_slice(seed);
                sk[32..].copy_from_slice(pk);
                let privkey_str: String = sk.encode_hex();

                {
                    let mut lock = std::io::stdout().lock();
                    writeln!(lock, "=======================================").unwrap();
                    writeln!(lock, "PrivateKey: {privkey_str}",).unwrap();
                    writeln!(lock, "Address: {}", str_addr).unwrap();
                    if !re_src.is_empty() {
                        writeln!(lock, "Regex: {}", re_src).unwrap();
                    }
                    writeln!(lock, "Height: {}", leading_zeros).unwrap();
                    writeln!(lock, "=======================================").unwrap();
                }

                if let Some(csv_file) = csv_file {
                    let mut csv_file = csv_file.lock().unwrap();
                    writeln!(
                        csv_file,
                        "{},{str_addr},{leading_zeros},{re_src},{privkey_str}",
                        Utc::now().timestamp()
                    )
                    .unwrap();
                }
            }
        }
    }
}

fn leading_zeros_of_pubkey(pk: &[u8]) -> u8 {
    let mut zeros = 0u8;
    for b in pk {
        let z = b.leading_zeros();
        zeros += z as u8;
        if z != 8 {
            break;
        }
    }
    zeros
}

fn address_for_pubkey(pk: &[u8]) -> std::net::Ipv6Addr {
    let zeros = leading_zeros_of_pubkey(pk);
    let mut buf = [0u8; 16];
    buf[0] = 0x02;
    buf[1] = zeros;
    for (src, trg) in pk[((zeros / 8) as usize)..]
        .windows(2)
        .zip(buf[2..].iter_mut())
    {
        *trg = src[0].wrapping_shl(((zeros + 1) % 8) as u32)
            ^ src[1].wrapping_shr(8 - ((zeros + 1) % 8) as u32)
            ^ 0xFF;
    }
    std::net::Ipv6Addr::from(buf)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::address_for_pubkey;

    #[test]
    fn test_address_for_pubkey() {
        assert_eq!(
            address_for_pubkey(
                hex::decode("000000000c4f58e09d19592f242951e6aa3185bd5ec6b95c0d56c93ae1268cbd")
                    .unwrap()
                    .as_slice()
            ),
            std::net::Ipv6Addr::from_str("224:7614:e3ec:5cd4:da1b:7ad5:c32a:b9cf").unwrap()
        )
    }
}
