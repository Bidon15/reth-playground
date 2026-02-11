//! RKB Executor Builder - Builds EVM config with NativeMinter precompile.

use crate::RkbEvmFactory;
use alloy_primitives::Address;
use reth_chainspec::{EthereumHardforks, Hardforks};
use reth_ethereum::evm::EthEvmConfig;
use reth_ethereum_primitives::EthPrimitives;
use reth_evm::eth::spec::EthExecutorSpec;
use reth_node_api::NodeTypes;
use reth_node_builder::{components::ExecutorBuilder, node::FullNodeTypes, BuilderContext};

/// RKB Executor Builder - builds EVM config with NativeMinter precompile.
///
/// This executor builder creates an `EthEvmConfig` that uses `RkbEvmFactory`
/// instead of the default `EthEvmFactory`, adding the NativeMinter precompile
/// at address 0x420.
///
/// # Example
///
/// ```ignore
/// use reth_rkb::RkbExecutorBuilder;
/// use reth_ethereum_node::EthereumNode;
///
/// // Use with Ethereum node components
/// let components = EthereumNode::components()
///     .executor(RkbExecutorBuilder::new(bridge_address));
/// ```
#[derive(Debug, Clone)]
pub struct RkbExecutorBuilder {
    /// Authorized bridge address for NativeMinter.
    authorized_bridge: Address,
}

impl RkbExecutorBuilder {
    /// Creates a new RKB executor builder with the given authorized bridge address.
    pub const fn new(authorized_bridge: Address) -> Self {
        Self { authorized_bridge }
    }

    /// Creates a new RKB executor builder with zero address (for testing only).
    pub const fn testing() -> Self {
        Self {
            authorized_bridge: Address::ZERO,
        }
    }
}

impl Default for RkbExecutorBuilder {
    fn default() -> Self {
        Self::testing()
    }
}

impl<Types, Node> ExecutorBuilder<Node> for RkbExecutorBuilder
where
    Types: NodeTypes<
        ChainSpec: Hardforks + EthExecutorSpec + EthereumHardforks,
        Primitives = EthPrimitives,
    >,
    Node: FullNodeTypes<Types = Types>,
{
    type EVM = EthEvmConfig<Types::ChainSpec, RkbEvmFactory>;

    async fn build_evm(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::EVM> {
        tracing::info!(
            authorized_bridge = %self.authorized_bridge,
            "Building RKB EVM with NativeMinter precompile"
        );

        let factory = RkbEvmFactory::new(self.authorized_bridge);
        let evm_config = EthEvmConfig::new_with_evm_factory(ctx.chain_spec(), factory);

        Ok(evm_config)
    }
}
