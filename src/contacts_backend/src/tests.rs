#[cfg(test)]
mod tests {
    use crate::data;

    use candid::{self, encode_one, Principal};
    use ic_cdk::api::management_canister::main::CanisterId;
    use pocket_ic::{PocketIc, WasmResult};

    fn load_contacts_backend_wasm() -> Vec<u8> {
        let wasm_path = "/Users/isaacgordon/Documents/ic/contacts/target/wasm32-unknown-unknown/release/contacts_backend.wasm";
        std::fs::read(wasm_path).unwrap_or_else(|_| panic!("Failed to read contacts_backend.wasm"))
    }

    fn deploy_test_canister() -> (PocketIc, CanisterId) {
        // init pocket-ic canister
        let pic = PocketIc::new();
        let canister_id = pic.create_canister();
        pic.add_cycles(canister_id, 2_000_000_000_000);

        // install wasm on canister
        let wasm_bytes = load_contacts_backend_wasm();
        pic.install_canister(canister_id, wasm_bytes, vec![], None);

        (pic, canister_id)
    }

    ///Helper function to decode a WasmResult into a Result that can be checked immediately.
    fn decode_wasm_result(wasm_result: WasmResult) -> Result<(), String> {
        match wasm_result {
            WasmResult::Reply(reply_bytes) => {
                let decoded: Result<(), String> =
                    candid::decode_one(&reply_bytes).expect("Failed to decode reply");
                eprintln!("Reply: {:?}", decoded);
                decoded
            }
            WasmResult::Reject(reject_message) => {
                eprintln!("Reject message: {}", reject_message);
                Err(reject_message)
            }
        }
    }

    /// Helper function to call the create_account function on the canister, and return a Result that can be checked immediately.
    fn call_create_account(
        pic: &PocketIc,
        canister_id: CanisterId,
        principal: Principal,
        new_user: data::new_user::NewUser,
    ) -> Result<(), String> {
        let wasm_result = pic
            .update_call(
                canister_id,
                principal,
                "create_account",
                encode_one(new_user).unwrap(),
            )
            .expect("Failed to call create_account");

        decode_wasm_result(wasm_result)
    }

    /// Helper function to call create_contact on the canister, and return a Result that can be checked immediately.
    fn call_create_contact(
        pic: &PocketIc,
        canister_id: CanisterId,
        principal: Principal,
        new_contact: data::contact::Contact,
    ) -> Result<(), String> {
        let wasm_result = pic
            .update_call(
                canister_id,
                principal,
                "create_contact",
                encode_one(new_contact).unwrap(),
            )
            .expect("Failed to call create_contact");

        decode_wasm_result(wasm_result)
    }

    /// Helper function to call get_contacts on the canister, and return a Result that can be checked immediately.
    fn call_get_contacts(
        pic: &PocketIc,
        canister_id: CanisterId,
        principal: Principal,
    ) -> Result<Vec<data::contact::Contact>, String> {
        let wasm_result = pic
            .query_call(canister_id, principal, "get_contacts", vec![])
            .expect("Failed to call get_contacts");

        match wasm_result {
            WasmResult::Reply(reply_bytes) => {
                let decoded: Result<Vec<data::contact::Contact>, String> =
                    candid::decode_one(&reply_bytes).expect("Failed to decode reply");
                eprintln!("Reply: {:?}", decoded);
                decoded
            }
            WasmResult::Reject(reject_message) => {
                eprintln!("Reject message: {}", reject_message);
                Err(reject_message)
            }
        }
    }

    /// Testing the create_account function and its adherence to the requirements.
    /// The requirements are:
    /// 1. A user can create an account with a unique username.
    /// 2. A user cannot create an account with a username that already exists.
    /// 3. A user cannot create an account if they already have one.
    #[test]
    fn test_create_account() {
        // Set up 4 users, 3 with unique usernames and 1 duplicate username.
        let user1 = data::new_user::NewUser {
            username: "user1".to_string(),
        };
        let user2 = data::new_user::NewUser {
            username: "user2".to_string(),
        };
        let user3 = data::new_user::NewUser {
            username: "user3".to_string(),
        };
        let user1_duplicate = data::new_user::NewUser {
            username: "user1".to_string(),
        };

        // init pocket-ic canister
        let (pic, canister_id) = deploy_test_canister();

        // Set up 3 distinct mock principals for testing.
        let principal1 = Principal::from_slice(&[0x01]);
        let principal2 = Principal::from_slice(&[0x02]);
        let principal3 = Principal::from_slice(&[0x03]);

        // Test creating a new account. (Requirement 1)
        println!("Creating account for principal1...");
        let first_account_create = call_create_account(&pic, canister_id, principal1, user1);
        assert!(
            first_account_create.is_ok(),
            "First account creation failed when it should not have. Expected `Ok` but got `Err`."
        );

        // Test another user creates a new account with a different username. (Requirement 1)
        println!("Creating account for principal2...");
        let second_account_create = call_create_account(&pic, canister_id, principal2, user2);
        assert!(
            second_account_create.is_ok(),
            "Second account creation failed when it should not have. Expected `Ok` but got `Err`."
        );

        // Test creating an account with a username that already exists. (Requirement 2)
        println!("Creating account for principal3 with an already claimed username...");
        let already_registered_username =
            call_create_account(&pic, canister_id, principal3, user1_duplicate);
        assert!(
            already_registered_username.is_err(),
            "Username should already be taken. Expected `Err` but got `Ok`."
        );

        // Test creating an account when user already has one. (Requirement 3)
        println!("Creating account for principal2 when they already have one...");
        let already_registered_user = call_create_account(&pic, canister_id, principal2, user3);
        assert!(
            already_registered_user.is_err(),
            "User should already have an account. Expected `Err` but got `Ok`."
        );
    }

    /// Testing the ability to create and retrieve contacts.
    /// The requirements are:
    /// 1. A user can create a contact.
    /// 2. A user can retrieve their contacts.
    /// 3. A user cannot retrieve contacts if they do not have an account.
    /// 4. A user cannot create a contact if they do not have an account.
    #[test]
    fn test_create_and_retrieve_contacts() {
        // Set up a user and a contact.
        let user = data::new_user::NewUser {
            username: "user".to_string(),
        };
        let new_contact = data::contact::Contact {
            name: "John Doe".to_string(),
            email: "johndoe@example.com".to_string(),
            phone: "123-456-7890".to_string(),
        };

        // init pocket-ic canister
        let (pic, canister_id) = deploy_test_canister();
        
        // Set up a mock principal for testing.
        let principal = Principal::from_slice(&[0x01]);

        // Test creating a contact when the user does not have an account. (Requirement 4)
        println!("Creating contact for principal1 without an account...");
        let create_contact_no_account = call_create_contact(&pic, canister_id, principal, new_contact.clone());
        assert!(
            create_contact_no_account.is_err(),
            "User should not have been able to create a contact without an account. Expected `Err` but got `Ok`."
        );

        // Test creating an account.
        println!("Creating account for principal1...");
        let account_create = call_create_account(&pic, canister_id, principal, user);
        assert!(
            account_create.is_ok(),
            "Account creation failed when it should not have. Expected `Ok` but got `Err`."
        );

        // Test creating a contact. (Requirement 1)
        println!("Creating contact for principal1...");
        let create_contact = call_create_contact(&pic, canister_id, principal, new_contact.clone());
        assert!(
            create_contact.is_ok(),
            "Contact creation failed when it should not have. Expected `Ok` but got `Err`."
        );

        // Test retrieving contacts. (Requirement 2)
        println!("Retrieving contacts for principal1...");
        let contacts = call_get_contacts(&pic, canister_id, principal);
        assert!(
            contacts.is_ok(),
            "Failed to retrieve contacts. Expected `Ok` but got `Err`."
        );
    }
}
