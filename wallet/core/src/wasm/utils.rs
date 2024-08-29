use crate::result::Result;
use js_sys::BigInt;
use karlsen_consensus_core::network::{NetworkType, NetworkTypeT};
use wasm_bindgen::prelude::*;
use workflow_wasm::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "bigint | number | HexString")]
    #[derive(Clone, Debug)]
    pub type ISompiToKarlsen;
}

/// Convert a Karlsen string to Sompi represented by bigint.
/// This function provides correct precision handling and
/// can be used to parse user input.
/// @category Wallet SDK
#[wasm_bindgen(js_name = "karlsenToSompi")]
pub fn karlsen_to_sompi(karlsen: String) -> Option<BigInt> {
    crate::utils::try_karlsen_str_to_sompi(karlsen)
        .ok()
        .flatten()
        .map(Into::into)
}

///
/// Convert Sompi to a string representation of the amount in Karlsen.
///
/// @category Wallet SDK
///
#[wasm_bindgen(js_name = "sompiToKarlsenString")]
pub fn sompi_to_karlsen_string(sompi: ISompiToKarlsen) -> Result<String> {
    let sompi = sompi.try_as_u64()?;
    Ok(crate::utils::sompi_to_karlsen_string(sompi))
}

///
/// Format a Sompi amount to a string representation of the amount in Karlsen with a suffix
/// based on the network type (e.g. `KLS` for mainnet, `TKLS` for testnet,
/// `SKLS` for simnet, `DKLS` for devnet).
///
/// @category Wallet SDK
///
#[wasm_bindgen(js_name = "sompiToKarlsenStringWithSuffix")]
pub fn sompi_to_karlsen_string_with_suffix(
    sompi: ISompiToKarlsen,
    network: &NetworkTypeT,
) -> Result<String> {
    let sompi = sompi.try_as_u64()?;
    let network_type = NetworkType::try_from(network)?;
    Ok(crate::utils::sompi_to_karlsen_string_with_suffix(
        sompi,
        &network_type,
    ))
}
