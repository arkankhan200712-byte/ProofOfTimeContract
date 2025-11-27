#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Env, String, Symbol,
};

const TIME_NS: Symbol = symbol_short!("TIMELOG");

#[contracttype]
#[derive(Clone)]
pub struct TimeEntry {
    pub entry_id: u64,      // unique id for this time record
    pub worker: Symbol,     // who worked
    pub task_ref: String,   // project / ticket / job id
    pub start_time: u64,    // unix timestamp from ledger
    pub end_time: u64,      // unix timestamp from ledger
    pub hours: u64,         // precomputed duration in seconds or minutes (here: seconds)
    pub approved: bool,     // true once manager/contract approves
}

#[contract]
pub struct ProofOfTimeContract;

#[contractimpl]
impl ProofOfTimeContract {
    /// Log a new time entry for a worker and task.
    /// `start_time` and `end_time` should be validated off-chain; here they are taken as inputs.
    pub fn log_time(
        env: Env,
        entry_id: u64,
        worker: Symbol,
        task_ref: String,
        start_time: u64,
        end_time: u64,
    ) {
        if end_time <= start_time {
            panic!("end_time must be greater than start_time");
        }

        let key = Self::entry_key(entry_id);
        let existing: Option<TimeEntry> = env.storage().instance().get(&key);
        if existing.is_some() {
            panic!("Entry id already exists");
        }

        let duration = end_time - start_time;

        let entry = TimeEntry {
            entry_id,
            worker,
            task_ref,
            start_time,
            end_time,
            hours: duration,
            approved: false,
        };

        env.storage().instance().set(&key, &entry);
    }

    /// Approve a time entry (e.g., by manager or payroll contract).
    pub fn approve_time(env: Env, entry_id: u64) {
        let key = Self::entry_key(entry_id);
        let mut entry: TimeEntry = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| panic!("Entry not found"));

        entry.approved = true;
        env.storage().instance().set(&key, &entry);
    }

    /// Check if a given time entry is approved.
    pub fn is_time_approved(env: Env, entry_id: u64) -> bool {
        let key = Self::entry_key(entry_id);
        let entry: Option<TimeEntry> = env.storage().instance().get(&key);

        match entry {
            Some(e) => e.approved,
            None => false,
        }
    }

    /// Get full time entry details (for audits, payroll, analytics).
    pub fn get_time_entry(env: Env, entry_id: u64) -> Option<TimeEntry> {
        let key = Self::entry_key(entry_id);
        env.storage().instance().get(&key)
    }

    /// Helper: composite storage key.
    fn entry_key(entry_id: u64) -> (Symbol, u64) {
        (TIME_NS, entry_id)
    }
}
