/*
 * This is an example of a Rust smart contract with two simple, symmetric functions:
 *
 * 1. set_greeting: accepts a greeting, such as "howdy", and records it for the user (account_id)
 *    who sent the request
 * 2. get_greeting: accepts an account_id and returns the greeting saved for it, defaulting to
 *    "Hello"
 *
 * Learn more about writing NEAR smart contracts with Rust:
 * https://github.com/near/near-sdk-rs
 *
 */

use near_contract_standards::fungible_token::core::ext_ft_core;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
// To conserve gas, efficient serialization is achieved through Borsh (http://borsh.io/)
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedSet;
use near_sdk::json_types::U128;
use near_sdk::serde_json::from_slice;
use near_sdk::{
    env, ext_contract, near_bindgen, require, AccountId, Promise, PromiseOrValue, PromiseResult,
};

#[ext_contract(ext_ft_metadata)]
trait FungibleTokenMetadataContract {
    fn ft_metadata(&self) -> FungibleTokenMetadata;
}

// Structs in Rust are similar to other languages, and may include impl keyword as shown below
// Note: the names of the structs are not important when calling the smart contract, but the function names are
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenList {
    tokens: UnorderedSet<AccountId>,
}

impl Default for TokenList {
    fn default() -> Self {
        Self {
            tokens: UnorderedSet::new(b"t".to_vec()),
        }
    }
}

#[near_bindgen]
impl TokenList {
    pub fn add_token(&mut self, token: AccountId) -> PromiseOrValue<bool> {
        let token_promise = self.get_add_token_to_list_promise(&token);
        if let Some(token_promise) = token_promise {
            PromiseOrValue::Promise(token_promise)
        } else {
            PromiseOrValue::Value(false)
        }
    }

    // TODO: Figure out mut tokens warning
    pub fn add_tokens(&mut self, mut tokens: Vec<AccountId>) -> PromiseOrValue<u64> {
        tokens.sort_unstable();
        tokens.dedup();
        let num_of_tokens = tokens.len();
        require!(num_of_tokens.gt(&0), "No tokens provided");

        let promises = tokens
            .into_iter()
            .filter_map(|token| self.get_add_token_to_list_promise(&token))
            .reduce(|accum, p| accum.and(p));
        if let Some(promises) = promises {
            PromiseOrValue::Promise(
                promises.then(Self::ext(env::current_account_id()).add_tokens_callback()),
            )
        } else {
            PromiseOrValue::Value(0)
        }
    }

    pub fn get_tokens(&self, from_index: u64, limit: u64) -> Vec<AccountId> {
        let keys = self.tokens.as_vector();
        (from_index..std::cmp::min(from_index + limit, self.tokens.len()))
            .map(|index| keys.get(index).unwrap())
            .collect()
    }

    fn get_add_token_to_list_promise(&self, token: &AccountId) -> Option<Promise> {
        if !self.tokens.contains(&token) {
            Some(self.add_token_to_list(&token))
        } else {
            None
        }
    }

    fn add_token_to_list(&self, token: &AccountId) -> Promise {
        self.verify_account_is_token(token)
            .then(Self::ext(env::current_account_id()).add_token_to_list_callback(token))
    }

    fn verify_account_is_token(&self, token: &AccountId) -> Promise {
        env::log_str(&format!("Adding token '{}' to token list", token));
        let account_id: AccountId = env::signer_account_id();
        ext_ft_core::ext(token.clone())
            .ft_balance_of(account_id)
            .and(ext_ft_metadata::ext(token.clone()).ft_metadata())
            .then(Self::ext(env::current_account_id()).verify_account_is_token_callback())
    }

    #[private]
    pub fn verify_account_is_token_callback(&self) -> bool {
        require!(
            env::promise_results_count() == 2,
            "Invalid number of promise results"
        );
        let balance = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                env::panic_str("Provided token address does not have a ft_balance_of method")
            }
            PromiseResult::Successful(result) => from_slice::<U128>(&result)
                .expect("Unable to deserialize ft_balance_of into U128, invalid"),
        };

        let metadata = match env::promise_result(1) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                env::panic_str("Provided token address does not have a ft_metadata method")
            }
            PromiseResult::Successful(result) => from_slice::<FungibleTokenMetadata>(&result)
                .expect("Unable to deserialize ft_metadata, invalid"),
        };

        metadata.assert_valid();
        balance.0 >= std::u128::MIN
    }

    #[private]
    pub fn add_token_to_list_callback(&mut self, token: &AccountId) -> bool {
        require!(
            env::promise_results_count() == 1,
            "Invalid number of promise results"
        );

        // handle the result from the cross contract call this method is a callback for
        let is_token_account = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                env::panic_str("Unable to get result of token account verification")
            }
            PromiseResult::Successful(result) => from_slice::<bool>(&result)
                .expect("Unable to deserialize bool for is_token_account, invalid"),
        };

        require!(
            is_token_account,
            format!("The account {} is not a valid token account", token)
        );
        self.tokens.insert(&token);
        true
    }

    #[private]
    pub fn add_tokens_callback(&self) -> u64 {
        let num_of_tokens = env::promise_results_count();
        env::log_str(&format!("Saved {} tokens to list", num_of_tokens));
        num_of_tokens
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 *
 * To run from contract directory:
 * cargo test -- --nocapture
 *
 * From project root, to run in combination with frontend tests:
 * yarn test
 *
 */
#[cfg(test)]
mod tests {
    use super::*;
    use near_primitives_core::config::ViewConfig;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};

    // mock the context for testing, notice "signer_account_id" that was accessed above from env::
    fn get_context(input: Vec<u8>, view_config: Option<ViewConfig>) -> VMContext {
        VMContext {
            view_config,
            input,
            ..VMContextBuilder::new().context
        }
    }
}
