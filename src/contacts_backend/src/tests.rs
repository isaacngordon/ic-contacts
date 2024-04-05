#[cfg(test)]
mod tests {
    use crate::{data, response::httpish};

    use candid::{self, decode_args, encode_one, utils::ArgumentDecoder, CandidType, Principal};
    use ic_cdk::api::management_canister::main::CanisterId;
    use pocket_ic::{PocketIc, WasmResult};
    use serde::Deserialize;

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

    /// Helper function for calling pocket ic update call
    pub fn update<T>(
        ic: &PocketIc,
        sender: Principal, 
        receiver: Principal, 
        method: &str, 
        args: Vec<u8>
    ) -> Result<T, String> 
    where
        T: CandidType + for<'a> Deserialize<'a> + for<'a> ArgumentDecoder<'a>,
    {
        match ic.update_call(receiver, sender, method, args) {
            Ok(WasmResult::Reply(data)) => {
                // Directly decode the data to T, leveraging the fact that T's
                // Deserialize implementation is valid for any lifetime 'a.
                // This aligns with the decode_args function's requirement.
                let tuple: T = decode_args(&data).expect("Failed to decode reply");
                Ok(tuple)
            },
            Ok(WasmResult::Reject(error_message)) => Err(error_message.to_string()),
            Err(user_error) => Err(user_error.to_string()),
        }
    }
    

    /// Helper function to call the create_account function on the canister, and return a Result that can be checked immediately.
    fn call_create_account(
        pic: &PocketIc,
        canister_id: CanisterId,
        principal: Principal,
        new_user: data::new_user::NewUser,
    ) -> Result<(httpish::BasicResponse,), String> {
        update::<(httpish::BasicResponse,)>
        (
            &pic, 
            principal, 
            canister_id, 
            "create_account", 
            encode_one(new_user).unwrap()
        )
    }

    /// Helper function to call create_contact on the canister, and return a Result that can be checked immediately.
    fn call_create_contact(
        pic: &PocketIc,
        canister_id: CanisterId,
        principal: Principal,
        new_contact: data::contact::Contact,
    ) ->  Result<(httpish::BasicResponse,), String>{
        update::<(httpish::BasicResponse,)>(
            &pic, 
            principal, 
            canister_id, 
            "create_contact", 
            encode_one(new_contact).unwrap()
        )
    }

    /// Helper function to call get_contacts on the canister, and return a Result that can be checked immediately.
    fn call_get_contacts(
        pic: &PocketIc,
        canister_id: CanisterId,
        principal: Principal,
    ) -> Result<(httpish::BasicResponse, Vec<data::contact::Contact>), String> {
        update(
            &pic, 
            principal, 
            canister_id, 
            "get_contacts", 
            encode_one(()).unwrap()
        )   
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
            first_account_create.is_ok_and(|response| 
                matches!(response.0, httpish::BasicResponse::Success(_))
            ),
            "First account creation failed when it should not have. Expected a `Success` response, but didn't get one."
        );

        // Test another user creates a new account with a different username. (Requirement 1)
        println!("Creating account for principal2...");
        let second_account_create = call_create_account(&pic, canister_id, principal2, user2);
        assert!(
            second_account_create.is_ok_and(|response| 
                matches!(response.0, httpish::BasicResponse::Success(_))
            ),
            "Second account creation failed when it should not have. Expected `Ok` but got `Err`."
        );

        // Test creating an account with a username that already exists. (Requirement 2)
        println!("Creating account for principal3 with an already claimed username...");
        let already_registered_username =
            call_create_account(&pic, canister_id, principal3, user1_duplicate);
        assert!(
            already_registered_username.is_ok_and(|response| 
                matches!(response.0, httpish::BasicResponse::Conflict(_))
            ),
            "Username should already be taken. Expected `Err` but got `Ok`."
        );

        // Test creating an account when user already has one. (Requirement 3)
        println!("Creating account for principal2 when they already have one...");
        let already_registered_user = call_create_account(&pic, canister_id, principal2, user3);
        assert!(
            already_registered_user.is_ok_and(|response| 
                matches!(response.0, httpish::BasicResponse::Conflict(_))
            ),
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
        
        let new_contact = data::contact::Contact {
            name: "John Doe".to_string(),
            email: "johndoe@example.com".to_string(),
            phone: "123-456-7890".to_string(),
        };

        // init pocket-ic canister
        let (pic, canister_id) = deploy_test_canister();
        
        // Set up a mock principal for testing.
        let principal = Principal::from_slice(&[0x04]);

        // Test creating a contact when the user does not have an account. (Requirement 4)
        println!("Creating contact for principal1 without an account...");
        let create_contact_no_account = call_create_contact(&pic, canister_id, principal, new_contact.clone());
        assert!(
            create_contact_no_account.is_ok_and(|response| 
                matches!(response.0, httpish::BasicResponse::Unauthorized)
            ),
            "User should not have been able to create a contact without an account. Expected `Unauthorized`."
        );

        // Test creating an account.
        println!("Ensuring principal has an account...");
        let user = data::new_user::NewUser {
            username: principal.to_text(),
        };
        let _ = call_create_account(&pic, canister_id, principal, user);

        // Test creating a contact. (Requirement 1)
        println!("Creating contact for principal1...");
        let create_contact = call_create_contact(&pic, canister_id, principal, new_contact.clone());
        assert!(
            create_contact.is_ok_and(|response| 
                matches!(response.0, httpish::BasicResponse::Success(_))
            ),
            "Contact creation failed when it should not have. Expected `Ok` but got `Err`."
        );

        // Test retrieving contacts. (Requirement 2)
        println!("Retrieving contacts for principal1...");
        let contacts = call_get_contacts(&pic, canister_id, principal);
        assert!(
            contacts.as_ref().is_ok_and(|response| 
                matches!(response.0, httpish::BasicResponse::Success(_))
            ),
            "Failed to retrieve contacts. Expected `Ok` but got `Err`."
        );

        // Test that contacts contain the created contact
        let retrieved_contacts = contacts.unwrap().1;
        println!("Number of retrieved contacts: {}", retrieved_contacts.len());
        assert!(
            retrieved_contacts.contains(&new_contact),
            "Retrieved contacts do not contain the created contact."
        );
        
    }
}
