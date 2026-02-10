//! RKB (Rauh-Konsens Begriff) - Celestia-native PoA EVM sequencer extensions for Reth.
//!
//! This crate provides custom precompiles and EVM configuration for RKB chains,
//! including the NativeMinter precompile for Hyperlane bridge integration.

#![cfg_attr(not(test), warn(unused_crate_dependencies))]

mod native_minter;

pub use native_minter::{
    NativeMinterPrecompile, NATIVE_MINTER_ADDRESS, NATIVE_MINTER_GAS_COST,
};
