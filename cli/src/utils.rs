use crate::error::Error;
use crate::result::Result;
use karlsen_consensus_core::constants::SOMPI_PER_KARLSEN;
use std::fmt::Display;

pub fn try_parse_required_nonzero_karlsen_as_sompi_u64<S: ToString + Display>(
    karlsen_amount: Option<S>,
) -> Result<u64> {
    if let Some(karlsen_amount) = karlsen_amount {
        let sompi_amount = karlsen_amount.to_string().parse::<f64>().map_err(|_| {
            Error::custom(format!(
                "Supplied Karlsen amount is not valid: '{karlsen_amount}'"
            ))
        })? * SOMPI_PER_KARLSEN as f64;
        if sompi_amount < 0.0 {
            Err(Error::custom(
                "Supplied Karlsen amount is not valid: '{karlsen_amount}'",
            ))
        } else {
            let sompi_amount = sompi_amount as u64;
            if sompi_amount == 0 {
                Err(Error::custom(
                    "Supplied required karlsen amount must not be a zero: '{karlsen_amount}'",
                ))
            } else {
                Ok(sompi_amount)
            }
        }
    } else {
        Err(Error::custom("Missing Karlsen amount"))
    }
}

pub fn try_parse_required_karlsen_as_sompi_u64<S: ToString + Display>(
    karlsen_amount: Option<S>,
) -> Result<u64> {
    if let Some(karlsen_amount) = karlsen_amount {
        let sompi_amount = karlsen_amount.to_string().parse::<f64>().map_err(|_| {
            Error::custom(format!(
                "Supplied Karlsen amount is not valid: '{karlsen_amount}'"
            ))
        })? * SOMPI_PER_KARLSEN as f64;
        if sompi_amount < 0.0 {
            Err(Error::custom(
                "Supplied Karlsen amount is not valid: '{karlsen_amount}'",
            ))
        } else {
            Ok(sompi_amount as u64)
        }
    } else {
        Err(Error::custom("Missing Karlsen amount"))
    }
}

pub fn try_parse_optional_karlsen_as_sompi_i64<S: ToString + Display>(
    karlsen_amount: Option<S>,
) -> Result<Option<i64>> {
    if let Some(karlsen_amount) = karlsen_amount {
        let sompi_amount = karlsen_amount.to_string().parse::<f64>().map_err(|_e| {
            Error::custom(format!(
                "Supplied Karlsen amount is not valid: '{karlsen_amount}'"
            ))
        })? * SOMPI_PER_KARLSEN as f64;
        if sompi_amount < 0.0 {
            Err(Error::custom(
                "Supplied Karlsen amount is not valid: '{karlsen_amount}'",
            ))
        } else {
            Ok(Some(sompi_amount as i64))
        }
    } else {
        Ok(None)
    }
}
