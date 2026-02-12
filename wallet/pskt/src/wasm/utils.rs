use karlsen_consensus_core::constants::*;
use karlsen_consensus_core::network::NetworkType;
use separator::{separated_float, separated_int, separated_uint_with_output, Separatable};

#[inline]
pub fn sompi_to_karlsen(sompi: u64) -> f64 {
    sompi as f64 / SOMPI_PER_KARLSEN as f64
}

#[inline]
pub fn karlsen_to_sompi(karlsen: f64) -> u64 {
    (karlsen * SOMPI_PER_KARLSEN as f64) as u64
}

#[inline]
pub fn sompi_to_karlsen_string(sompi: u64) -> String {
    sompi_to_karlsen(sompi).separated_string()
}

#[inline]
pub fn sompi_to_karlsen_string_with_trailing_zeroes(sompi: u64) -> String {
    separated_float!(format!("{:.8}", sompi_to_karlsen(sompi)))
}

pub fn karlsen_suffix(network_type: &NetworkType) -> &'static str {
    match network_type {
        NetworkType::Mainnet => "KLS",
        NetworkType::Testnet => "TKLS",
        NetworkType::Simnet => "SKLS",
        NetworkType::Devnet => "DKLS",
    }
}

#[inline]
pub fn sompi_to_karlsen_string_with_suffix(sompi: u64, network_type: &NetworkType) -> String {
    let kls = sompi_to_karlsen_string(sompi);
    let suffix = karlsen_suffix(network_type);
    format!("{kls} {suffix}")
}
