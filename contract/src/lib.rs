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
use near_contract_standards::fungible_token::metadata::FT_METADATA_SPEC;
// To conserve gas, efficient serialization is achieved through Borsh (http://borsh.io/)
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedSet;
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::from_slice;
use near_sdk::{env, ext_contract, near_bindgen, require, AccountId, Promise, PromiseResult};

// TODO: Determine why I can't import FungibleTokenMetadata directly from near_contract_standards
#[derive(BorshDeserialize, BorshSerialize, Clone, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FungibleTokenMetadata {
    pub spec: String,
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub reference: Option<String>,
    pub reference_hash: Option<Base64VecU8>,
    pub decimals: u8,
}

impl FungibleTokenMetadata {
    pub fn assert_valid(&self) {
        require!(self.spec == FT_METADATA_SPEC);
        require!(self.reference.is_some() == self.reference_hash.is_some());
        if let Some(reference_hash) = &self.reference_hash {
            require!(reference_hash.0.len() == 32, "Hash has to be 32 bytes");
        }
    }
}

#[ext_contract(ext_ft_metadata)]
trait FungibleTokenMetadataContract {
    fn ft_metadata(&self) -> FungibleTokenMetadata;
}

#[ext_contract(ext_self)]
trait TokenListCallbacks {
    fn verify_account_is_token_callback(&self) -> bool;
    fn add_token_to_list_callback(&self, token: AccountId) -> String;
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
    pub fn add_token(&mut self, token: AccountId) -> Promise {
        self.add_token_to_list(token)
    }

    // TODO: Use add_token for adding multiple tokens at once into the list
    pub fn add_tokens(&mut self, tokens: Vec<AccountId>) {
        // Use env::log to record logs permanently to the blockchain!
        // env::log(format!("Adding tokens '{:?}' to token list", tokens));

        // for token in tokens {
        //     self.tokens.insert(&token);
        // }
        tokens.into_iter().for_each(|token| {
            self.tokens.insert(&token);
        });
    }

    pub fn get_tokens(&self, from_index: u64, limit: u64) -> Vec<AccountId> {
        let keys = self.tokens.as_vector();
        (from_index..std::cmp::min(from_index + limit, self.tokens.len()))
            .map(|index| keys.get(index).unwrap())
            .collect()
    }

    fn add_token_to_list(&self, token: AccountId) -> Promise {
        self.verify_account_is_token(&token)
            .then(ext_self::ext(env::current_account_id()).add_token_to_list_callback(token))
    }

    fn verify_account_is_token(&self, token: &AccountId) -> Promise {
        env::log_str(&format!("Adding token '{}' to token list", token));
        let account_id: AccountId = env::signer_account_id();
        ext_ft_core::ext(token.clone())
            .ft_balance_of(account_id)
            .and(ext_ft_metadata::ext(token.clone()).ft_metadata())
            .then(ext_self::ext(env::current_account_id()).verify_account_is_token_callback())
    }

    #[private]
    pub fn verify_account_is_token_callback(&self) -> bool {
        assert_eq!(env::promise_results_count(), 2, "This is a callback method");
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
    pub fn add_token_to_list_callback(&mut self, token: AccountId) {
        assert_eq!(env::promise_results_count(), 1, "This is a callback method");

        // handle the result from the cross contract call this method is a callback for
        let is_token_account = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                env::panic_str("Unable to get result of token account verification")
            }
            PromiseResult::Successful(result) => from_slice::<bool>(&result)
                .expect("Unable to deserialize bool for is_token_account, invalid"),
        };

        assert!(is_token_account);
        self.tokens.insert(&token);
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

    #[test]
    fn set_then_get_token() {
        let context = get_context(vec![], None);
        testing_env!(context);
        let mut contract = TokenList::default();
        let token: AccountId = "wrap.near".parse().unwrap();
        contract.add_token(token.clone());
        assert_eq!(&token, contract.get_tokens(0, 1).get(0).unwrap());
    }

    #[test]
    fn set_then_get_tokens() {
        let context = get_context(vec![], None);
        testing_env!(context);
        let mut contract = TokenList::default();
        let tokens: Vec<AccountId> = vec![
            "wrap.near".parse().unwrap(),
            "meta-pool.sputnik2.testnet".parse().unwrap(),
        ];
        contract.add_tokens(tokens.clone());
        assert_eq!(tokens, contract.get_tokens(0, 2));
    }

    // #[test]
    // fn get_default_greeting() {
    //     let context = get_context(vec![], true);
    //     testing_env!(context);
    //     let contract = TokenList::default();
    //     // this test did not call set_greeting so should return the default "Hello" greeting
    //     assert_eq!(
    //         "Hello".to_string(),
    //         contract.get_greeting("francis.near".to_string())
    //     );
    // }
}
