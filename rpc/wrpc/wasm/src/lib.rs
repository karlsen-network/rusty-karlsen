//!
//! WASM bindings for the [Rusty Karlsen p2p Node wRPC Client](karlsen-wrpc-client)
//!

#![allow(unused_imports)]

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "wasm32-sdk")] {
        mod imports;
        pub mod client;
        pub use client::*;
        pub mod resolver;
        pub use resolver::*;
        pub mod notify;
        pub use notify::*;
    }

}
