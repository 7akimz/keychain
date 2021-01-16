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

// To conserve gas, efficient serialization is achieved through Borsh (http://borsh.io/)
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::wee_alloc;
use near_sdk::{env, near_bindgen};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


const LOWER_CASE_LETTERS: &str = "abcdefghijklmnopqrstuvwxyz";
const UPPER_CASE_LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const NUMBERS: &str = "0123456789";
const SPECIAL_CHARS: &str = "~!@#$%^&*()_-+=[]{}/\\|?,.<>'\"";


// Structs in Rust are similar to other languages, and may include impl keyword as shown below
// Note: the names of the structs are not important when calling the smart contract, but the function names are
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Key {
    identifier: String,
    enc_password: String
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Keychain {
    keys: HashMap<String, HashMap<String, Key>>,
}

#[near_bindgen]
impl Keychain {
    pub fn generate_new_password(&mut self, resource: String, identifier: String) {
        let account_id = env::signer_account_id();
        env::log(format!("started executing", ).as_bytes());

        if self.get_password(&account_id, &resource).is_empty()
        {
            let password_len = 12;

            let mut selected_set: String = "".to_string();
            selected_set.push_str(LOWER_CASE_LETTERS);
            selected_set.push_str(UPPER_CASE_LETTERS);
            selected_set.push_str(NUMBERS);
            selected_set.push_str(SPECIAL_CHARS);

            let selected_set_len = selected_set.len();

            let mut password = "".to_string();
            for _n in 0..password_len {
                password.push(selected_set.chars().nth(Keychain::generate_random_number(selected_set_len)).unwrap());
            }

            let mut record: HashMap<String, Key> = HashMap::new();
            record.insert(resource, Key { identifier, enc_password: password });

            self.keys.insert(account_id, record);   
        }
        env::log(format!("finished executing", ).as_bytes());
    }

    fn generate_random_number(less_than_or_equal_to: usize) -> usize {
        let microseconds: usize = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_micros() as usize;
        microseconds % less_than_or_equal_to
    }

    // `match` is similar to `switch` in other languages; here we use it to default to "Hello" if
    // self.records.get(&account_id) is not yet defined.
    // Learn more: https://doc.rust-lang.org/book/ch06-02-match.html#matching-with-optiont
    pub fn get_password(&self, account_id: &String, resource: &String) -> &str {
        let result =
        match self.keys.get(account_id) {
            Some(record) => match record.get(resource) {
                Some(key) => &key.enc_password,
                None => ""
            },
            None => "",
        };
        env::log(format!("Saving result '{}' for account '{}'", result, account_id,).as_bytes());

        result
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
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    // mock the context for testing, notice "signer_account_id" that was accessed above from env::
    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice_near".to_string(),
            signer_account_id: "bob_near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol_near".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn generate_then_check_password_length() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = Keychain::default();
        contract.generate_new_password("email".to_string(), "bob@email.com".to_string());
        assert_eq!(
            12,
            contract.get_password(&"bob_near".to_string(), &"email".to_string()).len()
        );
    }

    #[test]
    fn generate_then_check_if_password_not_empty() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = Keychain::default();
        contract.generate_new_password("email".to_string(), "bob@email.com".to_string());
        assert_ne!(
            "",
            contract.get_password(&"bob_near".to_string(), &"email".to_string())
        );
    }

    #[test]
    fn get_default_key() {
        let context = get_context(vec![], true);
        testing_env!(context);
        let contract = Keychain::default();
        // this test did not call set_greeting so should return the default "Hello" greeting
        assert_eq!(
            "".to_string(),
            contract.get_password(&"francis.near".to_string(), &"".to_string())
        );
    }
}
