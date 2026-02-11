#![allow(missing_docs)]

#[global_allocator]
static ALLOC: reth_cli_util::allocator::Allocator = reth_cli_util::allocator::new_allocator();

#[cfg(all(feature = "jemalloc-prof", unix))]
#[unsafe(export_name = "_rjem_malloc_conf")]
static MALLOC_CONF: &[u8] = b"prof:true,prof_active:true,lg_prof_sample:19\0";

use clap::Parser;
use reth::{args::RessArgs, cli::Cli, ress::install_ress_subprotocol};
use reth_ethereum_cli::chainspec::EthereumChainSpecParser;
use reth_node_builder::NodeHandle;
use reth_node_ethereum::{EthereumAddOns, EthereumNode};
use alloy_primitives::Address;
use reth_rkb::RkbExecutorBuilder;
use tracing::info;

fn main() {
    reth_cli_util::sigsegv_handler::install();

    // Enable backtraces unless a RUST_BACKTRACE value has already been explicitly provided.
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }

    if let Err(err) =
        Cli::<EthereumChainSpecParser, RessArgs>::parse().run(async move |builder, ress_args| {
            // Get authorized bridge address from environment variable
            // Falls back to Address::ZERO if not set (for testing/development)
            let authorized_bridge: Address = std::env::var("RKB_AUTHORIZED_BRIDGE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(Address::ZERO);

            info!(target: "reth::cli", %authorized_bridge, "Launching RKB node with NativeMinter precompile");

            let NodeHandle { node, node_exit_future } = builder
                .with_types::<EthereumNode>()
                .with_components(
                    EthereumNode::components().executor(RkbExecutorBuilder::new(authorized_bridge)),
                )
                .with_add_ons(EthereumAddOns::default())
                .launch_with_debug_capabilities()
                .await?;

            // Install ress subprotocol.
            if ress_args.enabled {
                install_ress_subprotocol(
                    ress_args,
                    node.provider,
                    node.evm_config,
                    node.network,
                    node.task_executor,
                    node.add_ons_handle.engine_events.new_listener(),
                )?;
            }

            node_exit_future.await
        })
    {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}
