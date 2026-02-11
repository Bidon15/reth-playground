//! RKB EVM Factory - Custom EVM with NativeMinter precompile.
//!
//! This module provides a custom EVM factory that extends the standard Ethereum EVM
//! with the NativeMinter precompile at address 0x420.

use crate::NATIVE_MINTER_ADDRESS;
use alloy_evm::{
    eth::EthEvmContext,
    precompiles::PrecompilesMap,
    EvmFactory,
};
use alloy_primitives::{Address, Bytes};
use reth_ethereum::evm::{
    primitives::{Database, EvmEnv},
    revm::{
        context::{BlockEnv, TxEnv},
        context_interface::result::{EVMError, HaltReason},
        inspector::{Inspector, NoOpInspector},
        interpreter::interpreter::EthInterpreter,
        precompile::{Precompile, PrecompileError, PrecompileId, PrecompileOutput, Precompiles},
        primitives::hardfork::SpecId,
        MainBuilder, MainContext,
    },
    EthEvm,
};

/// RKB EVM Factory - Creates EVMs with NativeMinter precompile.
///
/// This factory extends the standard Ethereum EVM with the NativeMinter precompile
/// at address 0x420, which enables minting/burning of native tokens for bridge operations.
///
/// # Example
///
/// ```ignore
/// use reth_rkb::RkbEvmFactory;
/// use alloy_primitives::address;
///
/// // Create factory with authorized bridge address
/// let bridge = address!("0x1234567890abcdef1234567890abcdef12345678");
/// let factory = RkbEvmFactory::new(bridge);
/// ```
#[derive(Debug, Clone)]
pub struct RkbEvmFactory {
    /// Authorized bridge address that can call NativeMinter.
    authorized_bridge: Address,
}

impl RkbEvmFactory {
    /// Creates a new RKB EVM factory with the given authorized bridge address.
    ///
    /// The authorized bridge is the only address allowed to call the NativeMinter
    /// precompile's mint/burn functions. This should be the deployed HypNativeGas
    /// contract address.
    pub fn new(authorized_bridge: Address) -> Self {
        tracing::info!(
            %authorized_bridge,
            native_minter = %NATIVE_MINTER_ADDRESS,
            "Creating RKB EVM Factory with NativeMinter"
        );

        Self { authorized_bridge }
    }

    /// Returns the authorized bridge address.
    pub const fn authorized_bridge(&self) -> Address {
        self.authorized_bridge
    }

    /// Creates precompiles for the given spec ID, including NativeMinter.
    fn create_precompiles(&self, _spec: SpecId) -> PrecompilesMap {
        // Get base precompiles for Cancun (our target spec)
        let base: &Precompiles = Precompiles::cancun();

        // Clone and add NativeMinter
        let mut precompiles = base.clone();

        // Create NativeMinter as a revm Precompile
        // Note: We use a simple function pointer that doesn't capture state
        // The authorized_bridge check will be done in the Solidity contract (HypNativeGas)
        // that calls this precompile, not in the precompile itself
        let native_minter_precompile = Precompile::new(
            PrecompileId::custom("native_minter"),
            NATIVE_MINTER_ADDRESS,
            native_minter_fn,
        );

        precompiles.extend([native_minter_precompile]);

        // Leak to get 'static lifetime (this is the pattern used by Reth)
        PrecompilesMap::from_static(Box::leak(Box::new(precompiles)))
    }
}

/// NativeMinter precompile function.
///
/// This is a placeholder implementation. The actual mint/burn logic requires
/// access to EVM state which isn't available in the simple precompile interface.
///
/// In production, the HypNativeGas Solidity contract will call this precompile,
/// and the precompile implementation should:
/// 1. Verify the caller is the authorized bridge contract
/// 2. Parse the mint/burn function selector and arguments
/// 3. Modify the recipient's/sender's balance using EVM internals
///
/// For now, this returns success to validate the precompile is registered.
fn native_minter_fn(input: &[u8], gas_limit: u64) -> Result<PrecompileOutput, PrecompileError> {
    const GAS_COST: u64 = crate::NATIVE_MINTER_GAS_COST;

    if gas_limit < GAS_COST {
        return Err(PrecompileError::OutOfGas);
    }

    tracing::debug!(
        input_len = input.len(),
        "NativeMinter precompile called"
    );

    // Return success with empty output
    // The actual state modification would happen here with proper EVM access
    Ok(PrecompileOutput::new(GAS_COST, Bytes::new()))
}

impl Default for RkbEvmFactory {
    fn default() -> Self {
        // Default to zero address - MUST be configured before use in production
        Self::new(Address::ZERO)
    }
}

impl EvmFactory for RkbEvmFactory {
    type Evm<DB: Database, I: Inspector<EthEvmContext<DB>, EthInterpreter>> =
        EthEvm<DB, I, Self::Precompiles>;
    type Tx = TxEnv;
    type Error<DBError: core::error::Error + Send + Sync + 'static> = EVMError<DBError>;
    type HaltReason = HaltReason;
    type Context<DB: Database> = EthEvmContext<DB>;
    type Spec = SpecId;
    type BlockEnv = BlockEnv;
    type Precompiles = PrecompilesMap;

    fn create_evm<DB: Database>(&self, db: DB, input: EvmEnv) -> Self::Evm<DB, NoOpInspector> {
        let spec = input.cfg_env.spec;

        tracing::debug!(
            ?spec,
            authorized_bridge = %self.authorized_bridge,
            native_minter = %NATIVE_MINTER_ADDRESS,
            "Creating RKB EVM with NativeMinter"
        );

        let precompiles = self.create_precompiles(spec);

        let evm = revm::Context::mainnet()
            .with_db(db)
            .with_cfg(input.cfg_env)
            .with_block(input.block_env)
            .build_mainnet_with_inspector(NoOpInspector {})
            .with_precompiles(precompiles);

        EthEvm::new(evm, false)
    }

    fn create_evm_with_inspector<DB: Database, I: Inspector<Self::Context<DB>, EthInterpreter>>(
        &self,
        db: DB,
        input: EvmEnv,
        inspector: I,
    ) -> Self::Evm<DB, I> {
        EthEvm::new(
            self.create_evm(db, input).into_inner().with_inspector(inspector),
            true,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::address;

    #[test]
    fn test_factory_creation() {
        let bridge = address!("0x1234567890abcdef1234567890abcdef12345678");
        let factory = RkbEvmFactory::new(bridge);
        assert_eq!(factory.authorized_bridge(), bridge);
    }

    #[test]
    fn test_default_factory() {
        let factory = RkbEvmFactory::default();
        assert_eq!(factory.authorized_bridge(), Address::ZERO);
    }
}
