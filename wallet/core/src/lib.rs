//!
//! # Karlsen Wallet Core
//!
//! Multi-platform Rust framework for Karlsen Wallet.
//!
//! This framework provides a series of APIs and primitives
//! to simplify building applications that interface with
//! the Karlsen p2p network.
//!
//! For key generation and derivation, please see the
//! [`karlsen_wallet_keys`] crate.
//!
//! This crate included are low-level primitives
//! such as [`UtxoProcessor`](crate::utxo::UtxoProcessor)
//! and [`UtxoContext`](crate::utxo::UtxoContext) that provide
//! various levels of automation as well as higher-level
//! APIs such as [`Wallet`](crate::wallet::Wallet),
//! [`Account`](crate::account::Account) (managed via the
//! [`WalletApi`](crate::api::WalletApi) trait)
//! that offer a fully-featured wallet implementation
//! backed by a multi-platform data storage layer capable of
//! storing wallet data on a local file-system as well as
//! within the browser environment.
//!
//! The wallet framework also includes transaction
//! [`Generator`](crate::tx::generator::Generator)
//! that can be used to generate transactions from a set of
//! UTXO entries. The generator can be used to create
//! simple transactions as well as batch transactions
//! comprised of multiple chained transactions.  Batch
//! transactions (also known as compound transactions)
//! are needed when the total number of inputs required
//! to satisfy the requested amount exceeds the maximum
//! allowed transaction mass.
//!
//! Key generation and derivation is available in the
//! [`karlsen_wallet_keys`] crate.
//!
//! The framework can operate
//! within native Rust applications as well as within NodeJS, Bun
//! and browser environments via the WASM32 SDK.
//!
//! WASM32 SDK documentation is available at:
//! <https://karlsen.aspectron.org/docs/>
//!
//! For NodeJS JavaScript and TypeScript environments, there are two
//! available NPM modules:
//! - <https://www.npmjs.com/package/karlsen>
//! - <https://www.npmjs.com/package/karlsen-wasm>
//!
//! NOTE: for security reasons (to mitigate potential upstream vendor
//! attacks) it is always recommended to build WASM SDK from source or
//! download pre-built redistributables.
//!
//! Latest development builds of the WASM32 SDK can be found at:
//! <https://aspectron.org/en/projects/karlsen-wasm.html>
//!
//! The `karlsen-wasm` module is a pure WASM32 module that includes
//! the entire wallet framework, but does not support RPC due to an absence
//! of a native WebSocket in NodeJs environment, while
//! the `karlsen` module includes `websocket` module dependency simulating
//! the W3C WebSocket and thus supports RPC.
//!
//! JavaScript examples for using this framework can be found at:
//! <https://github.com/karlsen-network/rusty-karlsen/tree/master/wasm/nodejs>
//!
//! For pre-built browser-compatible WASM32 redistributables of this
//! framework please see the releases section of the Rusty Karlsen
//! repository at <https://github.com/karlsen-network/rusty-karlsen/releases>.
//!

extern crate alloc;
extern crate self as karlsen_wallet_core;

pub mod account;
pub mod api;
pub mod compat;
pub mod cryptobox;
pub mod derivation;
pub mod deterministic;
pub mod encryption;
pub mod error;
pub mod events;
pub mod factory;
mod imports;
pub mod message;
pub mod metrics;
pub mod prelude;
pub mod result;
pub mod rpc;
pub mod serializer;
pub mod settings;
pub mod storage;
pub mod tx;
pub mod utils;
pub mod utxo;
pub mod wallet;

#[cfg(any(feature = "wasm32-sdk", feature = "wasm32-core"))]
pub mod wasm;

/// Returns the version of the Wallet framework.
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Returns the version of the Wallet framework combined with short git hash.
pub fn version_with_git_hash() -> String {
    karlsen_utils::git::with_short_hash(env!("CARGO_PKG_VERSION")).to_string()
}

#[cfg(test)]
pub mod tests;
