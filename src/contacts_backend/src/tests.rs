#[cfg(test)]
mod tests {
    use crate::new_user;

    use candid::{self, encode_one, Principal};
    use ic_cdk::api::management_canister::main::CanisterId;
    use pocket_ic::{PocketIc, WasmResult};

    use new_user::NewUser;

    fn load_contacts_backend_wasm() -> Vec<u8> {
        let wasm_path = "/Users/isaacgordon/Documents/ic/contacts/target/wasm32-unknown-unknown/release/contacts_backend.wasm";
        std::fs::read(wasm_path).unwrap_or_else(|_| panic!("Failed to read contacts_backend.wasm"))
    }

    /// Helper function to call the create_account function on the canister, and return a Result that can be checked immediately.
    fn call_create_account(
        pic: &PocketIc,
        canister_id: CanisterId,
        principal: Principal,
        new_user: NewUser,
    ) -> Result<(), String> {
        let wasm_result = pic
            .update_call(
                canister_id,
                principal,
                "create_account",
                encode_one(new_user).unwrap(),
            )
            .expect("Failed to call create_account");

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

    /// Testing the create_account function and its adherence to the requirements.
    /// The requirements are:
    /// 1. A user can create an account with a unique username.
    /// 2. A user cannot create an account with a username that already exists.
    /// 3. A user cannot create an account if they already have one.
    #[test]
    fn test_create_account() {
        // Set up 4 users, 3 with unique usernames and 1 duplicate username.
        let user1 = NewUser {
            username: "user1".to_string(),
        };
        let user2 = NewUser {
            username: "user2".to_string(),
        };
        let user3 = NewUser {
            username: "user3".to_string(),
        };
        let user1_duplicate = NewUser {
            username: "user1".to_string(),
        };

        // init pocket-ic canister
        let pic = PocketIc::new();
        let canister_id = pic.create_canister();
        pic.add_cycles(canister_id, 2_000_000_000_000);

        // install wasm on canister
        let wasm_bytes = load_contacts_backend_wasm();
        pic.install_canister(canister_id, wasm_bytes, vec![], None);

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
}
