#![no_std]

pub mod admin;
pub mod storage;
pub mod storage_types;
mod token;

use soroban_sdk::{contract, contractimpl, Address, Env};

use admin::AdminError;

// ---------------------------------------------------------------------------
// Contract entry point
// ---------------------------------------------------------------------------

/// The CreatorEventManager contract.
///
/// Call [`CreatorEventManagerContract::initialize`] exactly once after
/// deployment to configure the contract.  All other functions will panic
/// (or return an error) if called before initialization.
#[contract]
pub struct CreatorEventManagerContract;

#[contractimpl]
impl CreatorEventManagerContract {
    /// Initialise the contract for first use.
    ///
    /// Must be called exactly once after deployment.  Stores the admin,
    /// AI agent, treasury, XLM token address, and creation fee in persistent
    /// storage, resets all counters to zero, and emits an `initialized` event.
    ///
    /// # Panics
    /// Panics with a descriptive message on any validation failure so that
    /// Soroban surfaces a clear error to the caller:
    /// * `"already_initialized"` — called more than once.
    /// * `"invalid_address"` — one of the addresses equals the contract itself.
    /// * `"invalid_creation_fee"` — `initial_creation_fee` ≤ 0.
    pub fn initialize(
        env: Env,
        admin: Address,
        ai_agent: Address,
        treasury: Address,
        xlm_token: Address,
        initial_creation_fee: i128,
    ) {
        match admin::initialize(
            &env,
            admin,
            ai_agent,
            treasury,
            xlm_token,
            initial_creation_fee,
        ) {
            Ok(()) => {}
            Err(AdminError::AlreadyInitialized) => {
                panic!("already_initialized")
            }
            Err(AdminError::InvalidAddress) => {
                panic!("invalid_address")
            }
            Err(AdminError::InvalidCreationFee) => {
                panic!("invalid_creation_fee")
            }
        }
    }

    /// Returns `true` if the contract has been initialised.
    pub fn is_initialized(env: Env) -> bool {
        admin::is_initialized(&env)
    }

    /// Returns the current creation fee in stroops, or 0 if not initialised.
    pub fn get_creation_fee(env: Env) -> i128 {
        admin::get_creation_fee(&env).unwrap_or(0)
    }

    /// Returns `true` if the contract is currently paused.
    pub fn is_paused(env: Env) -> bool {
        admin::is_paused(&env)
    }
}
