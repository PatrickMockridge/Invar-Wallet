//! Canonical DarkWow genesis contracts, configured as `[GENESIS]` (standard).
//!
//! The wallet's trust model (`doc/src/arch/manifest.md` "Trust Model") has four tiers —
//! Genesis, SelfDeployed, Attested, Unverified. The nine genesis contracts are the only
//! compile-time-known ContractIds; every other contract is discovered from its on-chain
//! manifest during scan and starts as Unverified. This module is the single source of truth
//! for "which contracts are standard" in Invar's UI.

use dwow_sdk::crypto::ContractId;
use dwow_sdk::manifest::TrustTier;

pub struct GenesisContract {
    pub name: &'static str,
    pub id: ContractId,
}

/// The nine genesis contracts deployed at block 1 (README "Genesis & Manifests").
pub fn genesis_contracts() -> Vec<GenesisContract> {
    vec![
        GenesisContract {
            name: "native_token",
            id: *dwow_sdk::crypto::NATIVE_TOKEN_CONTRACT_ID,
        },
        GenesisContract {
            name: "deployooor",
            id: *dwow_sdk::crypto::DEPLOYOOOR_CONTRACT_ID,
        },
        GenesisContract {
            name: "promissory_note",
            id: *dwow_sdk::crypto::PROMISSORY_NOTE_CONTRACT_ID,
        },
        GenesisContract {
            name: "identity",
            id: *dwow_sdk::crypto::IDENTITY_CONTRACT_ID,
        },
        GenesisContract {
            name: "oracle",
            id: *dwow_sdk::crypto::ORACLE_CONTRACT_ID,
        },
        GenesisContract {
            name: "attestation",
            id: *dwow_sdk::crypto::ATTESTATION_CONTRACT_ID,
        },
        GenesisContract {
            name: "purse",
            id: *dwow_sdk::crypto::PURSE_CONTRACT_ID,
        },
        GenesisContract {
            name: "box",
            id: *dwow_sdk::crypto::BOX_CONTRACT_ID,
        },
        GenesisContract {
            name: "multisig",
            id: *dwow_sdk::crypto::MULTISIG_CONTRACT_ID,
        },
    ]
}

pub fn is_genesis(id: &ContractId) -> bool {
    let bytes = id.to_bytes();
    genesis_contracts().iter().any(|g| g.id.to_bytes() == bytes)
}

/// Resolve the trust tier for a contract id: Genesis for the nine standard contracts,
/// Unverified otherwise (SelfDeployed/Attested require wallet identity / on-chain
/// attestation queries — deferred to the contracts milestone).
pub fn trust_tier(id: &ContractId) -> TrustTier {
    if is_genesis(id) {
        TrustTier::Genesis
    } else {
        TrustTier::Unverified
    }
}

/// Human name for a known genesis contract; "unknown" otherwise.
pub fn contract_name(id: &ContractId) -> &'static str {
    let bytes = id.to_bytes();
    genesis_contracts()
        .iter()
        .find(|g| g.id.to_bytes() == bytes)
        .map(|g| g.name)
        .unwrap_or("unknown")
}

/// Base58-encode a 32-byte crypto value for display.
pub fn b58(bytes: [u8; 32]) -> String {
    bs58::encode(bytes).into_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_nine_genesis_contracts_are_standard() {
        let contracts = genesis_contracts();
        assert_eq!(contracts.len(), 9);

        let names: Vec<&str> = contracts.iter().map(|c| c.name).collect();
        assert_eq!(
            names,
            vec![
                "native_token",
                "deployooor",
                "promissory_note",
                "identity",
                "oracle",
                "attestation",
                "purse",
                "box",
                "multisig",
            ]
        );

        // Every genesis contract resolves to the Genesis (standard) tier, not Unverified.
        for c in &contracts {
            assert!(is_genesis(&c.id), "{} should be genesis", c.name);
            assert_eq!(trust_tier(&c.id), TrustTier::Genesis);
            assert_eq!(contract_name(&c.id), c.name);
        }
    }

    #[test]
    fn unknown_contract_is_unverified() {
        // A zero ContractId is not one of the nine genesis contracts.
        let zero = ContractId::from_bytes([0u8; 32]).unwrap();
        assert!(!is_genesis(&zero));
        assert_eq!(trust_tier(&zero), TrustTier::Unverified);
        assert_eq!(contract_name(&zero), "unknown");
    }
}
