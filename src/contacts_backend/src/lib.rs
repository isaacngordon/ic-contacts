use ic_cdk::{api, query, update};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

mod contact;
mod new_contact;
mod new_user;
mod user;

use contact::Contact;
use new_contact::NewContact;
use new_user::NewUser;
use user::User;

// Data Structures
// Global State
static USERS: Lazy<Mutex<HashMap<String, User>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// Helper Functions
fn get_user_id() -> String {
    api::caller().to_text()
}

// Canister Functions

/// Create a new user account by providing a unique username.
#[update]
fn create_account(new_user: NewUser) -> Result<(), String> {
    let user_id = get_user_id();
    let mut users = USERS.lock().unwrap();

    if users.contains_key(&user_id) {
        return Err("User already has an account".to_string());
    }

    let username_taken = users
        .values()
        .any(|existing_user| existing_user.username == new_user.username);

    if username_taken {
    {
        return Err("Username already taken".to_string());
    }

    let user = User {
        username: new_user.username,
        contacts: Vec::new(),
        shared_contacts: Vec::new(),
    };

    users.insert(user_id, user);
    Ok(())
}

/// Get the list of contacts for the current user.
#[query]
fn get_contacts() -> Result<Vec<Contact>, String> {
    let user_id = get_user_id();
    let users = USERS.lock().unwrap();

    if let Some(user) = users.get(&user_id) {
        Ok(user.contacts.clone())
    } else {
        Err("User not found".to_string())
    }
}

/// Create a new contact for the current user.
#[update(name = "create_contact")]
fn create_contact(new_contact: NewContact) -> Result<(), String> {
    let user_id = get_user_id();
    let mut users = USERS.lock().unwrap();

    if let Some(user) = users.get_mut(&user_id) {
        let new_id = user.contacts.len() as u64 + 1;
        let contact = Contact {
            id: new_id,
            name: new_contact.name,
            email: new_contact.email,
            phone: new_contact.phone,
        };
        user.contacts.push(contact);
        Ok(())
    } else {
        Err("User not found".to_string())
    }
}

#[update]
fn edit_contact(contact_id: u64, updated_contact: NewContact) -> Result<(), String> {
    let user_id = get_user_id();
    let mut users = USERS.lock().unwrap();

    if let Some(user) = users.get_mut(&user_id) {
        if let Some(contact) = user.contacts.iter_mut().find(|c| c.id == contact_id) {
            contact.name = updated_contact.name;
            contact.email = updated_contact.email;
            contact.phone = updated_contact.phone;
            Ok(())
        } else {
            Err("Contact not found".to_string())
        }
    } else {
        Err("User not found".to_string())
    }
}

#[update]
fn delete_contact(contact_id: u64) -> Result<(), String> {
    let user_id = get_user_id();
    let mut users = USERS.lock().unwrap();

    if let Some(user) = users.get_mut(&user_id) {
        if user.contacts.iter().any(|c| c.id == contact_id) {
            user.contacts.retain(|c| c.id != contact_id);
            Ok(())
        } else {
            Err("Contact not found".to_string())
        }
    } else {
        Err("User not found".to_string())
    }
}

#[update]
fn share_contact(contact_id: u64, recipient_username: String) -> Result<(), String> {
    let owner_id = get_user_id();
    let mut users = USERS.lock().unwrap();

    if let Some(owner) = users.get(&owner_id) {
        if owner.contacts.iter().any(|c| c.id == contact_id) {
            if let Some(recipient) = users
                .values_mut()
                .find(|u| u.username == recipient_username)
            {
                recipient.shared_contacts.push(contact_id);
                Ok(())
            } else {
                Err("Recipient not found".to_string())
            }
        } else {
            Err("Contact not found".to_string())
        }
    } else {
        Err("Owner not found".to_string())
    }
}

#[update]
fn revoke_shared_contact(contact_id: u64, recipient_username: String) -> Result<(), String> {
    let owner_id = get_user_id();
    let mut users = USERS.lock().unwrap();

    // Check if the owner exists and has the contact in their list.
    if let Some(owner) = users.get(&owner_id) {
        if owner.contacts.iter().any(|c| c.id == contact_id) {
            // Now, find the recipient by username and get a mutable reference to modify their shared_contacts.
            if let Some(recipient) = users
                .values_mut()
                .find(|user| user.username == recipient_username)
            {
                // Check if the contact is indeed shared with the recipient and remove it if so.
                if recipient.shared_contacts.contains(&contact_id) {
                    recipient.shared_contacts.retain(|&id| id != contact_id);
                    return Ok(());
                } else {
                    return Err("Contact not shared with this user".to_string());
                }
            } else {
                return Err("Recipient not found".to_string());
            }
        } else {
            return Err("Contact not found in owner's contact list".to_string());
        }
    } else {
        return Err("Owner not found".to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::{self, encode_one, Principal};
    use ic_cdk::api::management_canister::main::CanisterId;
    use pocket_ic::{PocketIc, WasmResult};

    fn load_contacts_backend_wasm() -> Vec<u8> {
        let wasm_path = "/Users/isaacgordon/Documents/ic/contacts/target/wasm32-unknown-unknown/release/contacts_backend.wasm";
        std::fs::read(wasm_path).unwrap_or_else(|_| {
            panic!("Failed to read contacts_backend.wasm")
        })
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
            WasmResult::Reply(_) => Ok(()),
            WasmResult::Reject(reject_message) => Err(reject_message),
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
        let first_account_create = call_create_account(&pic, canister_id, principal1, user1);
        assert!(first_account_create.is_ok(), "First account creation failed when it should not have. Expected `Ok` but got `Err`.");

        // Test another user creates a new account with a different username. (Requirement 1)
        let second_account_create = call_create_account(&pic, canister_id, principal2, user2);
        assert!(second_account_create.is_ok(), "Second account creation failed when it should not have. Expected `Ok` but got `Err`.");

        // Test creating an account with a username that already exists. (Requirement 2)
        let already_registered_username =
            call_create_account(&pic, canister_id, principal3, user1_duplicate);
        assert!(already_registered_username.is_err(), "Username should already be taken. Expected `Err` but got `Ok`.");

        // Test creating an account when user already has one. (Requirement 3)
        let already_registered_user = call_create_account(&pic, canister_id, principal2, user3);
        assert!(already_registered_user.is_err(), "User should already have an account. Expected `Err` but got `Ok`.");
    }
}
