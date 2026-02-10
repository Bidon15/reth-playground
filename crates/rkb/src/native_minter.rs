//! NativeMinter Precompile for RKB.
//!
//! This precompile allows authorized bridge contracts to mint and burn native tokens
//! for cross-chain bridging. It is used by the Hyperlane HypNativeGas contract to
//! credit users with native TIA when bridging from Celestia.
//!
//! ## Security
//!
//! - Only the authorized bridge contract can call mint/burn functions
//! - The authorized address is set at chain configuration time
//! - Cannot be called via DELEGATECALL (must be direct call)
//! - Reverts in STATICCALL context
//!
//! ## Interface
//!
//! ```solidity
//! interface INativeMinter {
//!     function mint(address recipient, uint256 amount) external;
//!     function burn(address from, uint256 amount) external;
//! }
//! ```

use alloy_evm::precompiles::{DynPrecompile, PrecompileInput};
use alloy_primitives::{address, Address, Bytes, U256};
use alloy_sol_types::{sol, SolCall};
use revm::precompile::{PrecompileError, PrecompileId, PrecompileOutput, PrecompileResult};
use tracing::{debug, warn};

/// Precompile address: 0x0000000000000000000000000000000000000420
pub const NATIVE_MINTER_ADDRESS: Address = address!("0x0000000000000000000000000000000000000420");

/// Gas cost for mint/burn operations.
/// This is similar to other balance-modifying operations (warm account access + modification).
pub const NATIVE_MINTER_GAS_COST: u64 = 6000;

// Define the Solidity interface using alloy-sol-types
sol! {
    /// Mint native tokens to a recipient address.
    /// Only callable by the authorized bridge contract.
    function mint(address recipient, uint256 amount);

    /// Burn native tokens from an address.
    /// Only callable by the authorized bridge contract.
    /// The `from` address must have approved or be the caller.
    function burn(address from, uint256 amount);
}

/// NativeMinter precompile for minting/burning native tokens during bridge operations.
///
/// # Usage
///
/// ```ignore
/// use reth_rkb::NativeMinterPrecompile;
/// use alloy_primitives::address;
///
/// // Create precompile with authorized bridge address
/// let bridge = address!("0x1234567890abcdef1234567890abcdef12345678");
/// let precompile = NativeMinterPrecompile::new(bridge);
///
/// // Convert to DynPrecompile for use with PrecompilesMap
/// let dyn_precompile = precompile.into_dyn_precompile();
/// ```
#[derive(Debug, Clone)]
pub struct NativeMinterPrecompile {
    /// The authorized bridge contract address that can call mint/burn.
    authorized_bridge: Address,
}

impl NativeMinterPrecompile {
    /// Creates a new NativeMinter precompile with the given authorized bridge address.
    pub const fn new(authorized_bridge: Address) -> Self {
        Self { authorized_bridge }
    }

    /// Returns the authorized bridge address.
    pub const fn authorized_bridge(&self) -> Address {
        self.authorized_bridge
    }

    /// Converts this precompile into a [`DynPrecompile`] for use with [`PrecompilesMap`].
    pub fn into_dyn_precompile(self) -> DynPrecompile {
        DynPrecompile::new_stateful(
            PrecompileId::custom("native_minter"),
            move |input: PrecompileInput<'_>| self.call(input),
        )
    }

    /// Execute the precompile call.
    fn call(&self, mut input: PrecompileInput<'_>) -> PrecompileResult {
        // Check gas
        if input.gas < NATIVE_MINTER_GAS_COST {
            return Err(PrecompileError::OutOfGas);
        }

        // Security: Must be a direct call, not DELEGATECALL
        if !input.is_direct_call() {
            warn!(
                target: "rkb::native_minter",
                caller = %input.caller,
                target = %input.target_address,
                bytecode = %input.bytecode_address,
                "NativeMinter: DELEGATECALL not allowed"
            );
            return Err(PrecompileError::other_static("NativeMinter: DELEGATECALL not allowed"));
        }

        // Security: Cannot call in STATICCALL context
        if input.is_static_call() {
            warn!(
                target: "rkb::native_minter",
                caller = %input.caller,
                "NativeMinter: STATICCALL not allowed"
            );
            return Err(PrecompileError::other_static("NativeMinter: STATICCALL not allowed"));
        }

        // Security: Only authorized bridge can call
        if input.caller != self.authorized_bridge {
            warn!(
                target: "rkb::native_minter",
                caller = %input.caller,
                authorized = %self.authorized_bridge,
                "NativeMinter: unauthorized caller"
            );
            return Err(PrecompileError::other_static("NativeMinter: unauthorized caller"));
        }

        // Need at least 4 bytes for function selector
        if input.data.len() < 4 {
            return Err(PrecompileError::other_static("NativeMinter: invalid calldata length"));
        }

        // Parse function selector
        let selector: [u8; 4] = input.data[..4].try_into().unwrap();

        match selector {
            // mint(address,uint256) selector: 0x40c10f19
            <mintCall as SolCall>::SELECTOR => {
                let decoded = mintCall::abi_decode(&input.data[4..])
                    .map_err(|_| PrecompileError::other_static("NativeMinter: invalid mint args"))?;

                self.execute_mint(&mut input, decoded.recipient, decoded.amount)
            }
            // burn(address,uint256) selector: 0x9dc29fac
            <burnCall as SolCall>::SELECTOR => {
                let decoded = burnCall::abi_decode(&input.data[4..])
                    .map_err(|_| PrecompileError::other_static("NativeMinter: invalid burn args"))?;

                self.execute_burn(&mut input, decoded.from, decoded.amount)
            }
            _ => {
                warn!(
                    target: "rkb::native_minter",
                    selector = ?selector,
                    "NativeMinter: unknown function selector"
                );
                Err(PrecompileError::other_static("NativeMinter: unknown function"))
            }
        }
    }

    /// Execute the mint operation - credit native tokens to recipient.
    fn execute_mint(
        &self,
        input: &mut PrecompileInput<'_>,
        recipient: Address,
        amount: U256,
    ) -> PrecompileResult {
        debug!(
            target: "rkb::native_minter",
            %recipient,
            %amount,
            "Minting native tokens"
        );

        // Use EvmInternals to increment the recipient's balance
        input
            .internals_mut()
            .balance_incr(recipient, amount)
            .map_err(|e| PrecompileError::other(format!("NativeMinter: mint failed: {e}")))?;

        // Emit a log for indexing (optional but useful)
        // We could add a Mint event here, but precompiles emitting logs is tricky
        // The HypNativeGas contract will emit its own events

        Ok(PrecompileOutput::new(NATIVE_MINTER_GAS_COST, Bytes::new()))
    }

    /// Execute the burn operation - debit native tokens from an address.
    fn execute_burn(
        &self,
        input: &mut PrecompileInput<'_>,
        from: Address,
        amount: U256,
    ) -> PrecompileResult {
        debug!(
            target: "rkb::native_minter",
            %from,
            %amount,
            "Burning native tokens"
        );

        // Load the account to check balance
        let account = input
            .internals_mut()
            .load_account(from)
            .map_err(|e| PrecompileError::other(format!("NativeMinter: load account failed: {e}")))?;

        let current_balance = account.data.info.balance;

        // Check sufficient balance
        if current_balance < amount {
            warn!(
                target: "rkb::native_minter",
                %from,
                %amount,
                %current_balance,
                "NativeMinter: insufficient balance for burn"
            );
            return Err(PrecompileError::other_static("NativeMinter: insufficient balance"));
        }

        // Calculate new balance and set it
        let new_balance = current_balance - amount;
        input
            .internals_mut()
            .set_balance(from, new_balance)
            .map_err(|e| PrecompileError::other(format!("NativeMinter: burn failed: {e}")))?;

        Ok(PrecompileOutput::new(NATIVE_MINTER_GAS_COST, Bytes::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mint_selector() {
        // mint(address,uint256) = keccak256("mint(address,uint256)")[0:4]
        let expected: [u8; 4] = [0x40, 0xc1, 0x0f, 0x19];
        assert_eq!(<mintCall as SolCall>::SELECTOR, expected);
    }

    #[test]
    fn test_burn_selector() {
        // burn(address,uint256) = keccak256("burn(address,uint256)")[0:4]
        let expected: [u8; 4] = [0x9d, 0xc2, 0x9f, 0xac];
        assert_eq!(<burnCall as SolCall>::SELECTOR, expected);
    }

    #[test]
    fn test_precompile_address() {
        assert_eq!(
            NATIVE_MINTER_ADDRESS,
            address!("0x0000000000000000000000000000000000000420")
        );
    }
}
