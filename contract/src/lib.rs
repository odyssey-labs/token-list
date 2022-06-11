use near_contract_standards::fungible_token::core::ext_ft_core;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::store::UnorderedSet;
use near_sdk::{
    env, ext_contract, near_bindgen, require, AccountId, Promise, PromiseError, PromiseOrValue,
};

#[ext_contract(ext_ft_metadata)]
trait FungibleTokenMetadataContract {
    fn ft_metadata(&self) -> FungibleTokenMetadata;
}

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
        let token_promise = self.get_add_token_to_list_promise(token);
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
            .filter_map(|token| self.get_add_token_to_list_promise(token))
            .reduce(|accum, p| accum.and(p));
        if let Some(promises) = promises {
            PromiseOrValue::Promise(
                promises.then(Self::ext(env::current_account_id()).add_tokens_callback()),
            )
        } else {
            PromiseOrValue::Value(0)
        }
    }

    pub fn get_tokens(&self, from_index: u64, limit: u64) -> Vec<&AccountId> {
        let keys: Vec<&AccountId> = self.tokens.iter().collect();
        (from_index..std::cmp::min(from_index + limit, self.tokens.len().into()))
            .map(|index| *keys.get(index as usize).unwrap())
            .collect()
    }

    fn get_add_token_to_list_promise(&self, token: AccountId) -> Option<Promise> {
        if !self.tokens.contains(&token) {
            Some(self.add_token_to_list(token))
        } else {
            None
        }
    }

    fn add_token_to_list(&self, token: AccountId) -> Promise {
        self.verify_account_is_token(&token)
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
    pub fn verify_account_is_token_callback(
        #[callback_result] balance: Result<U128, PromiseError>,
        #[callback_result] metadata: Result<FungibleTokenMetadata, PromiseError>,
    ) -> bool {
        metadata
            .expect("Provided token address does not have a ft_metadata method")
            .assert_valid();
        balance
            .expect("Provided token address does not have a ft_metadata method")
            .0
            >= std::u128::MIN
    }

    #[private]
    pub fn add_token_to_list_callback(
        &mut self,
        #[callback_result] is_token_account: Result<bool, PromiseError>,
        token: AccountId,
    ) -> bool {
        require!(
            is_token_account.expect("Unable to get result of token account verification"),
            format!("The account {} is not a valid token account", token)
        );
        self.tokens.insert(token);
        true
    }

    #[private]
    pub fn add_tokens_callback() -> u64 {
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

    #[test]
    fn get_tokens() {
        let context = get_context(vec![], None);
        testing_env!(context);
        let mut contract = TokenList::default();
        let tokens: Vec<AccountId> = vec![
            "linear-protocol.testnet".parse().unwrap(),
            "wrap.testnet".parse().unwrap(),
        ];
        tokens.iter().for_each(|token| {
            contract.tokens.insert(token.clone());
        });
        assert_eq!(
            vec![&tokens[0], &tokens[1]],
            contract.get_tokens(0, tokens.len() as u64)
        );
    }

    #[test]
    fn get_tokens_subset() {
        let context = get_context(vec![], None);
        testing_env!(context);
        let mut contract = TokenList::default();
        let tokens: Vec<AccountId> = vec![
            "linear-protocol.testnet".parse().unwrap(),
            "wrap.testnet".parse().unwrap(),
        ];
        tokens.iter().for_each(|token| {
            contract.tokens.insert(token.clone());
        });
        assert_eq!(vec![&tokens[0]], contract.get_tokens(0, 1));
    }

    #[test]
    fn get_tokens_out_of_bounds_index() {
        let context = get_context(vec![], None);
        testing_env!(context);
        let mut contract = TokenList::default();
        let tokens: Vec<AccountId> = vec![
            "linear-protocol.testnet".parse().unwrap(),
            "wrap.testnet".parse().unwrap(),
        ];
        tokens.iter().for_each(|token| {
            contract.tokens.insert(token.clone());
        });
        assert_eq!(vec![] as Vec<&AccountId>, contract.get_tokens(1000, 1));
    }
}
