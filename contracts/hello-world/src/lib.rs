#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol,
};

const RENT_NS: Symbol = symbol_short!("RENT");

#[contracttype]
#[derive(Clone)]
pub enum RentStatus {
    Created,
    Active,
    Completed,
    Cancelled,
}

#[contracttype]
#[derive(Clone)]
pub struct RentAgreement {
    pub rent_id: u64,
    pub landlord: Address,
    pub tenant: Address,
    pub monthly_rent: i128,
    pub security_deposit: i128,
    pub status: RentStatus,
    pub months_paid: u32,
}

#[contract]
pub struct SafeRentContract;

#[contractimpl]
impl SafeRentContract {
    // Landlord creates a rent agreement (tenant can be set later if needed)
    pub fn create_agreement(
        env: Env,
        rent_id: u64,
        landlord: Address,
        tenant: Address,
        monthly_rent: i128,
        security_deposit: i128,
    ) {
        if monthly_rent <= 0 || security_deposit < 0 {
            panic!("invalid amounts");
        }

        let inst = env.storage().instance();
        let key = Self::rent_key(rent_id);

        if inst.has(&key) {
            panic!("rent_id exists");
        }

        let agreement = RentAgreement {
            rent_id,
            landlord,
            tenant,
            monthly_rent,
            security_deposit,
            status: RentStatus::Created,
            months_paid: 0,
        };

        inst.set(&key, &agreement);
    }

    // Tenant confirms and activates the agreement (after off-chain payment of deposit)
    pub fn activate_agreement(env: Env, rent_id: u64, caller: Address) {
        let inst = env.storage().instance();
        let key = Self::rent_key(rent_id);

        let mut ag: RentAgreement =
            inst.get(&key).unwrap_or_else(|| panic!("agreement not found"));

        if let RentStatus::Created = ag.status {
        } else {
            panic!("not in Created state");
        }

        if caller != ag.tenant {
            panic!("only tenant can activate");
        }

        ag.status = RentStatus::Active;
        inst.set(&key, &ag);
    }

    // Tenant logs a rent payment for one month
    pub fn pay_rent(env: Env, rent_id: u64, caller: Address) {
        let inst = env.storage().instance();
        let key = Self::rent_key(rent_id);

        let mut ag: RentAgreement =
            inst.get(&key).unwrap_or_else(|| panic!("agreement not found"));

        if let RentStatus::Active = ag.status {
        } else {
            panic!("agreement not active");
        }

        if caller != ag.tenant {
            panic!("only tenant can pay rent");
        }

        // Real token transfer happens via separate token contract; here we just log it
        ag.months_paid += 1;
        inst.set(&key, &ag);
    }

    // Landlord marks agreement completed and deposit settled
    pub fn complete_agreement(env: Env, rent_id: u64, caller: Address) {
        let inst = env.storage().instance();
        let key = Self::rent_key(rent_id);

        let mut ag: RentAgreement =
            inst.get(&key).unwrap_or_else(|| panic!("agreement not found"));

        if caller != ag.landlord {
            panic!("only landlord can complete");
        }

        if let RentStatus::Active = ag.status {
        } else {
            panic!("must be active");
        }

        ag.status = RentStatus::Completed;
        inst.set(&key, &ag);
    }

    // Landlord cancels before activation (e.g., tenant never confirms)
    pub fn cancel_agreement(env: Env, rent_id: u64, caller: Address) {
        let inst = env.storage().instance();
        let key = Self::rent_key(rent_id);

        let mut ag: RentAgreement =
            inst.get(&key).unwrap_or_else(|| panic!("agreement not found"));

        if caller != ag.landlord {
            panic!("only landlord can cancel");
        }

        if let RentStatus::Created = ag.status {
        } else {
            panic!("can cancel only in Created state");
        }

        ag.status = RentStatus::Cancelled;
        inst.set(&key, &ag);
    }

    // View agreement
    pub fn get_agreement(env: Env, rent_id: u64) -> Option<RentAgreement> {
        let inst = env.storage().instance();
        let key = Self::rent_key(rent_id);
        inst.get(&key)
    }

    fn rent_key(id: u64) -> (Symbol, u64) {
        (RENT_NS, id)
    }
}
