//! RKB EVM Factory - Custom EVM with NativeMinter precompile.
//!
//! This module provides a custom EVM factory that extends the standard Ethereum EVM
//! with the NativeMinter precompile at address 0x420, enabling minting/burning of
//! native tokens for Hyperlane bridge operations.

use crate::{NativeMinterPrecompile, NATIVE_MINTER_ADDRESS};
use alloy_evm::{eth::EthEvmContext, precompiles::PrecompilesMap, revm::handler::EthPrecompiles, Evm, EvmFactory};
use alloy_primitives::Address;
use reth_ethereum::evm::{
    primitives::{Database, EvmEnv},
    revm::{
        context::{BlockEnv, TxEnv},
        context_interface::result::{EVMError, HaltReason},
        inspector::{Inspector, NoOpInspector},
        interpreter::interpreter::EthInterpreter,
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
/// The NativeMinter precompile is a **stateful precompile** that can modify account
/// balances during execution, enabling the Hyperlane HypNativeGas contract to mint
/// native tokens when bridging from Celestia.
///
/// # Security
///
/// - Only the `authorized_bridge` address can call mint/burn functions
/// - DELEGATECALL is not allowed (must be direct call)
/// - STATICCALL is not allowed (state modification required)
///
/// # Example
///
/// ```ignore
/// use reth_rkb::RkbEvmFactory;
/// use alloy_primitives::address;
///
/// // Create factory with authorized bridge address (HypNativeGas contract)
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

        // Create base EVM with standard Ethereum precompiles
        let evm = revm::Context::mainnet()
            .with_db(db)
            .with_cfg(input.cfg_env)
            .with_block(input.block_env)
            .build_mainnet_with_inspector(NoOpInspector {})
            .with_precompiles(PrecompilesMap::from_static(EthPrecompiles::default().precompiles));

        let mut evm = EthEvm::new(evm, false);

        // Add the NativeMinter stateful precompile
        // This precompile has access to EVM internals and can modify account balances
        let native_minter = NativeMinterPrecompile::new(self.authorized_bridge);
        let native_minter_dyn = native_minter.into_dyn_precompile();

        evm.precompiles_mut()
            .apply_precompile(&NATIVE_MINTER_ADDRESS, |_| Some(native_minter_dyn));

        evm
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
