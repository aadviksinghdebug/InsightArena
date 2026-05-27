/// Admin module — contract initialization and privileged configuration.
///
/// The `initialize` function is the single entry point that must be called
/// exactly once after deployment.  It stores every piece of global config in
/// persistent storage and sets the counters to zero.
use soroban_sdk::{Address, Env, Symbol};

use crate::storage::TTL_LEDGERS;
use crate::storage_types::DataKey;

// ---------------------------------------------------------------------------
// Error codes
// ---------------------------------------------------------------------------

/// Errors that can be returned by admin operations.
///
/// Represented as `u32` so they can be used as Soroban contract error codes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AdminError {
    /// `initialize` was called on an already-initialised contract.
    AlreadyInitialized = 1,
    /// One of the required addresses is the zero / default address.
    InvalidAddress = 2,
    /// `creation_fee` must be strictly positive.
    InvalidCreationFee = 3,
}

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

/// Initialise the contract for first use.
///
/// # Parameters
/// | Name | Description |
/// |---|---|
/// | `admin` | Contract administrator — the only address that can call privileged functions. |
/// | `ai_agent` | Oracle address authorised to submit match results. |
/// | `treasury` | Recipient of all creation fees. |
/// | `xlm_token` | Address of the native XLM token contract. |
/// | `initial_creation_fee` | Fee (in stroops) charged to creators; must be > 0. |
///
/// # Errors
/// * [`AdminError::AlreadyInitialized`] — if the contract has already been initialised.
/// * [`AdminError::InvalidAddress`] — if any address equals the contract's own address
///   (used as a proxy for "zero / unset" since Soroban has no literal zero address).
/// * [`AdminError::InvalidCreationFee`] — if `initial_creation_fee` ≤ 0.
///
/// # Storage written
/// All values are stored in **persistent** storage with a one-year TTL.
///
/// | Key | Value |
/// |---|---|
/// | `DataKey::Initialized` | `true` |
/// | `DataKey::Admin(admin)` | `admin` |
/// | `DataKey::AIAgent(ai_agent)` | `ai_agent` |
/// | `DataKey::Treasury(treasury)` | `treasury` |
/// | `DataKey::XLMToken(xlm_token)` | `xlm_token` |
/// | `DataKey::CreationFee(0)` | `initial_creation_fee` |
/// | `DataKey::Paused(false)` | `false` |
///
/// Counters (`EventCounter`, `MatchCounter`, `PredictionCounter`) are written
/// to **instance** storage and set to `0`.
///
/// # Events
/// Emits a `(Symbol("admin"), Symbol("initialized"))` event with the topic
/// `[admin, ai_agent, treasury]` and data `initial_creation_fee`.
pub fn initialize(
    env: &Env,
    admin: Address,
    ai_agent: Address,
    treasury: Address,
    xlm_token: Address,
    initial_creation_fee: i128,
) -> Result<(), AdminError> {
    // ── Guard: prevent re-initialisation ────────────────────────────────────
    if is_initialized(env) {
        return Err(AdminError::AlreadyInitialized);
    }

    // ── Validate addresses ───────────────────────────────────────────────────
    // Soroban has no literal "zero address", so we use the contract's own
    // address as a sentinel for "caller passed a nonsensical value".
    // Any address that is equal to the current contract address is rejected
    // because it would create circular authority.
    let contract_self = env.current_contract_address();
    if admin == contract_self
        || ai_agent == contract_self
        || treasury == contract_self
        || xlm_token == contract_self
    {
        return Err(AdminError::InvalidAddress);
    }

    // ── Validate creation fee ────────────────────────────────────────────────
    if initial_creation_fee <= 0 {
        return Err(AdminError::InvalidCreationFee);
    }

    // ── Persist config ───────────────────────────────────────────────────────
    let storage = env.storage().persistent();

    // Initialization sentinel — checked by `is_initialized`
    storage.set(&DataKey::Initialized, &true);
    storage.extend_ttl(&DataKey::Initialized, TTL_LEDGERS, TTL_LEDGERS);

    // Admin address
    storage.set(&DataKey::Admin(admin.clone()), &admin);
    storage.extend_ttl(&DataKey::Admin(admin.clone()), TTL_LEDGERS, TTL_LEDGERS);

    // AI agent address
    storage.set(&DataKey::AIAgent(ai_agent.clone()), &ai_agent);
    storage.extend_ttl(
        &DataKey::AIAgent(ai_agent.clone()),
        TTL_LEDGERS,
        TTL_LEDGERS,
    );

    // Treasury address
    storage.set(&DataKey::Treasury(treasury.clone()), &treasury);
    storage.extend_ttl(
        &DataKey::Treasury(treasury.clone()),
        TTL_LEDGERS,
        TTL_LEDGERS,
    );

    // XLM token address
    storage.set(&DataKey::XLMToken(xlm_token.clone()), &xlm_token);
    storage.extend_ttl(
        &DataKey::XLMToken(xlm_token.clone()),
        TTL_LEDGERS,
        TTL_LEDGERS,
    );

    // Creation fee — stored under a canonical key with value 0 as placeholder
    // (the actual fee is the *value*, not the key discriminant)
    storage.set(&DataKey::CreationFee(0), &initial_creation_fee);
    storage.extend_ttl(&DataKey::CreationFee(0), TTL_LEDGERS, TTL_LEDGERS);

    // Paused flag — starts as false
    storage.set(&DataKey::Paused(false), &false);
    storage.extend_ttl(&DataKey::Paused(false), TTL_LEDGERS, TTL_LEDGERS);

    // ── Initialise counters to 0 (instance storage) ──────────────────────────
    let instance = env.storage().instance();
    instance.set(&DataKey::EventCounter(0), &0u64);
    instance.set(&DataKey::MatchCounter(0), &0u64);
    instance.set(&DataKey::PredictionCounter(0), &0u64);

    // ── Emit initialization event ────────────────────────────────────────────
    env.events().publish(
        (Symbol::new(env, "admin"), Symbol::new(env, "initialized")),
        (admin, ai_agent, treasury, initial_creation_fee),
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Read helpers (used by other modules)
// ---------------------------------------------------------------------------

/// Returns `true` if the contract has already been initialised.
pub fn is_initialized(env: &Env) -> bool {
    env.storage()
        .persistent()
        .get::<DataKey, bool>(&DataKey::Initialized)
        .unwrap_or(false)
}

/// Read the current creation fee (in stroops).
///
/// Returns `None` if the contract has not been initialised.
pub fn get_creation_fee(env: &Env) -> Option<i128> {
    env.storage()
        .persistent()
        .get::<DataKey, i128>(&DataKey::CreationFee(0))
}

/// Returns `true` if the contract is currently paused.
pub fn is_paused(env: &Env) -> bool {
    env.storage()
        .persistent()
        .get::<DataKey, bool>(&DataKey::Paused(false))
        .unwrap_or(false)
}
