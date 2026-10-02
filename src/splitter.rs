//! Splits an outbound ZEC amount into ZIP 318 canonical denominations (`{1,2,5} x 10^k`
//! zatoshi in `[0.01 ZEC, 10000 ZEC]`) so each crossing joins the same anonymity set as a
//! pool-migration transfer, and no single hold exceeds `max_part`.
use anyhow::{bail, Result};
use zcash_protocol::{value::Zatoshis, zip318::{is_canonical_denomination, largest_one_two_five}};

pub fn is_canonical(zat: u64) -> bool {
    Zatoshis::from_u64(zat).is_ok_and(is_canonical_denomination)
}

#[derive(Debug, PartialEq, Eq)]
pub struct Split {
    pub parts: Vec<u64>,
    /// Remainder below the smallest canonical denomination. It never crosses: it stays shielded.
    pub residual: u64,
}

pub const FLOOR: u64 = 1_000_000;

pub fn split(total: u64, max_part: u64, max_parts: usize) -> Result<Split> {
    if max_part < FLOOR {
        bail!("max_part {max_part} zat is below the smallest canonical denomination {FLOOR}");
    }
    let (mut rem, mut parts) = (total, Vec::new());
    loop {
        let p = largest_one_two_five(rem.min(max_part), FLOOR);
        if p == 0 {
            break;
        }
        parts.push(p);
        rem -= p;
        if parts.len() > max_parts {
            bail!("{total} zat at cap {max_part} needs more than {max_parts} parts");
        }
    }
    Ok(Split { parts, residual: rem })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ZEC: u64 = 100_000_000;

    fn canon(p: u64) -> bool {
        is_canonical(p)
    }

    #[test]
    fn splits_are_canonical_capped_and_conserve_value() {
        for total in [ZEC, 13 * ZEC / 10, 37 * ZEC / 10, 2 * ZEC + 12_345_678, 5_000_000] {
            let s = split(total, ZEC / 2, 64).unwrap();
            assert_eq!(s.parts.iter().sum::<u64>() + s.residual, total, "conserved {total}");
            assert!(s.parts.iter().all(|&p| canon(p) && p <= ZEC / 2), "{s:?}");
            assert!(s.residual < FLOOR);
        }
    }

    #[test]
    fn known_split() {
        let s = split(13 * ZEC / 10, ZEC / 2, 64).unwrap();
        assert_eq!(s.parts, vec![ZEC / 2, ZEC / 2, ZEC / 5, ZEC / 10]);
        assert_eq!(s.residual, 0);
    }

    #[test]
    fn dust_stays_shielded() {
        assert_eq!(split(999_999, ZEC, 8).unwrap(), Split { parts: vec![], residual: 999_999 });
    }

    #[test]
    fn part_count_is_bounded() {
        assert!(split(100 * ZEC, ZEC / 100, 8).is_err());
    }
}

#[cfg(test)]
mod floor_tests {
    #[test]
    fn floor_matches_zip318() {
        assert_eq!(super::FLOOR, u64::from(zcash_protocol::zip318::MAX_RESIDUAL_VALUE));
    }
}
