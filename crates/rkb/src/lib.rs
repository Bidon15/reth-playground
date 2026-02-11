//! RKB (Rauh-Konsens Begriff) - Celestia-native PoA EVM sequencer extensions for Reth.
//!
//! This crate provides custom precompiles and EVM configuration for RKB chains,
//! including the NativeMinter precompile for Hyperlane bridge integration.
//!
//! ## Components
//!
//! - [`NativeMinterPrecompile`]: Precompile at `0x420` for minting/burning native TIA
//! - [`RkbEvmFactory`]: Custom EVM factory with NativeMinter
//! - [`RkbExecutorBuilder`]: Executor builder for node integration
//!
//! ## Usage
//!
//! ```ignore
//! use reth_rkb::{RkbExecutorBuilder, NATIVE_MINTER_ADDRESS};
//! use reth_ethereum_node::EthereumNode;
//! use alloy_primitives::address;
//!
//! // Build node with NativeMinter precompile
//! let bridge = address!("0x1234567890abcdef1234567890abcdef12345678");
//! let components = EthereumNode::components()
//!     .executor(RkbExecutorBuilder::new(bridge));
//! ```

#![cfg_attr(not(test), warn(unused_crate_dependencies))]

mod native_minter;
mod evm;
mod executor;

pub use native_minter::{
    NativeMinterPrecompile, NATIVE_MINTER_ADDRESS, NATIVE_MINTER_GAS_COST,
};
pub use evm::RkbEvmFactory;
pub use executor::RkbExecutorBuilder;
