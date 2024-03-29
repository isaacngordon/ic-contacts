mod contact;
mod new_contact;
mod new_user;
mod user;

use ic_cdk::{api, query, update};

use contact::Contact;
use new_contact::NewContact;
use new_user::NewUser;
use user::User;

// Data Structures
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use candid::Principal;
use std::cell::RefCell;

// Global State
type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    // The memory manager is used for simulating multiple memories. Given a `MemoryId` it can
    // return a memory that can be used by stable structures.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    // Initialize a `StableBTreeMap` with `MemoryId(0)`.
    static USER_MAP: RefCell<StableBTreeMap<Principal, String, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );
}
    
// Helper Functions
fn get_user_id() -> Principal {
    api::caller()
}

// Retrieves the value associated with the given key if it exists.
#[query]
fn get() -> Option<String> {
    let key = get_user_id();
    USER_MAP.with(|p| p.borrow().get(&key))
}

// Inserts an entry into the map and returns the previous value of the key if it exists.
#[update]
fn insert(value: String) -> Option<String> {
    let key = get_user_id();
    USER_MAP.with(|p| p.borrow_mut().insert(key, value))
}

/// Create a new user account by providing a unique username.
#[update]
fn create_account(new_user: NewUser) -> Result<(), String> {
    let principal = get_user_id();
    println!("principal: {}", principal);

    let user_exists: bool = USER_MAP.with(|p| p.borrow().contains_key(&principal));
    println!("user_exists: {}", user_exists);
    if user_exists {
        return Err("User already has an account".to_string());
    }

    // this seems like it would be slow as it iterates over all users to check if the username is taken
    let username_taken: bool = USER_MAP.with(|p| {
        p.borrow().iter().any(|(_, existing_username)| *existing_username == new_user.username)
    });
    println!("username_taken: {}", username_taken);
    if username_taken {
        return Err("Username already taken".to_string());
    }

    let user = User {
        username: new_user.username,
        contacts: Vec::new(),
        shared_contacts: Vec::new(),
    };

    USER_MAP.with(|p| p.borrow_mut().insert(principal, user.username.clone()));
    println!("user inserted: {}", user.username);

    Ok(())
}

/// Get the list of contacts for the current user.
// #[query]
// fn get_contacts() -> Result<Vec<Contact>, String> {
//     let user_id = get_user_id();
//     let users = USER_MAP.with(|p| p.borrow());

//     if let Some(user) = users.get(&user_id) {
//         Ok(user.contacts.clone())
//     } else {
//         Err("User not found".to_string())
//     }
// }

/// Create a new contact for the current user.
// #[update(name = "create_contact")]
// fn create_contact(new_contact: NewContact) -> Result<(), String> {
//     let user_id = get_user_id();
//     let mut users = USER_MAP.lock().unwrap();

//     if let Some(user) = users.get_mut(&user_id) {
//         let new_id = user.contacts.len() as u64 + 1;
//         let contact = Contact {
//             id: new_id,
//             name: new_contact.name,
//             email: new_contact.email,
//             phone: new_contact.phone,
//         };
//         user.contacts.push(contact);
//         Ok(())
//     } else {
//         Err("User not found".to_string())
//     }
// }

// #[update]
// fn edit_contact(contact_id: u64, updated_contact: NewContact) -> Result<(), String> {
//     let user_id = get_user_id();
//     let mut users = USER_MAP.lock().unwrap();

//     if let Some(user) = users.get_mut(&user_id) {
//         if let Some(contact) = user.contacts.iter_mut().find(|c| c.id == contact_id) {
//             contact.name = updated_contact.name;
//             contact.email = updated_contact.email;
//             contact.phone = updated_contact.phone;
//             Ok(())
//         } else {
//             Err("Contact not found".to_string())
//         }
//     } else {
//         Err("User not found".to_string())
//     }
// }

// #[update]
// fn delete_contact(contact_id: u64) -> Result<(), String> {
//     let user_id = get_user_id();
//     let mut users = USER_MAP.lock().unwrap();

//     if let Some(user) = users.get_mut(&user_id) {
//         if user.contacts.iter().any(|c| c.id == contact_id) {
//             user.contacts.retain(|c| c.id != contact_id);
//             Ok(())
//         } else {
//             Err("Contact not found".to_string())
//         }
//     } else {
//         Err("User not found".to_string())
//     }
// }

// #[update]
// fn share_contact(contact_id: u64, recipient_username: String) -> Result<(), String> {
//     let owner_id = get_user_id();
//     let mut users = USER_MAP.lock().unwrap();

//     if let Some(owner) = users.get(&owner_id) {
//         if owner.contacts.iter().any(|c| c.id == contact_id) {
//             if let Some(recipient) = users
//                 .values_mut()
//                 .find(|u| u.username == recipient_username)
//             {
//                 recipient.shared_contacts.push(contact_id);
//                 Ok(())
//             } else {
//                 Err("Recipient not found".to_string())
//             }
//         } else {
//             Err("Contact not found".to_string())
//         }
//     } else {
//         Err("Owner not found".to_string())
//     }
// }

// #[update]
// fn revoke_shared_contact(contact_id: u64, recipient_username: String) -> Result<(), String> {
//     let owner_id = get_user_id();
//     let mut users = USER_MAP.lock().unwrap();

//     // Check if the owner exists and has the contact in their list.
//     if let Some(owner) = users.get(&owner_id) {
//         if owner.contacts.iter().any(|c| c.id == contact_id) {
//             // Now, find the recipient by username and get a mutable reference to modify their shared_contacts.
//             if let Some(recipient) = users
//                 .values_mut()
//                 .find(|user| user.username == recipient_username)
//             {
//                 // Check if the contact is indeed shared with the recipient and remove it if so.
//                 if recipient.shared_contacts.contains(&contact_id) {
//                     recipient.shared_contacts.retain(|&id| id != contact_id);
//                     return Ok(());
//                 } else {
//                     return Err("Contact not shared with this user".to_string());
//                 }
//             } else {
//                 return Err("Recipient not found".to_string());
//             }
//         } else {
//             return Err("Contact not found in owner's contact list".to_string());
//         }
//     } else {
//         return Err("Owner not found".to_string());
//     }
// }

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
            WasmResult::Reply(x) => {
                eprintln!("Reply: {:?}", x);
            WasmResult::Reply(reply_bytes) => {
                let decoded: Result<(), String> = candid::decode_one(&reply_bytes)
                    .expect("Failed to decode reply");
                decoded
            },
            WasmResult::Reject(reject_message) => {
                eprintln!("Reject message: {}", reject_message);
                Err(reject_message)
            },
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
        assert!(first_account_create.is_ok(), "First account creation failed when it should not have. Expected `Ok` but got `Err`.");

        // Test another user creates a new account with a different username. (Requirement 1)
        println!("Creating account for principal2...");
        let second_account_create = call_create_account(&pic, canister_id, principal2, user2);
        assert!(second_account_create.is_ok(), "Second account creation failed when it should not have. Expected `Ok` but got `Err`.");

        // Test creating an account with a username that already exists. (Requirement 2)
        println!("Creating account for principal3 with an already claimed username...");
        let already_registered_username =
            call_create_account(&pic, canister_id, principal3, user1_duplicate);
        assert!(already_registered_username.is_err(), "Username should already be taken. Expected `Err` but got `Ok`.");

        // Test creating an account when user already has one. (Requirement 3)
        println!("Creating account for principal2 when they already have one...");
        let already_registered_user = call_create_account(&pic, canister_id, principal2, user3);
        assert!(already_registered_user.is_err(), "User should already have an account. Expected `Err` but got `Ok`.");
    }
}
