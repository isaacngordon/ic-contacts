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

mod tests; // This line will be modified to point to the new tests.rs file.

// Global State
type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    // The memory manager is used for simulating multiple memories. Given a `MemoryId` it can
    // return a memory that can be used by stable structures.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    // Initialize a `StableBTreeMap` with `MemoryId(0)`.
    static USER_MAP: RefCell<StableBTreeMap<Principal, User, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );
}
    
// Helper Functions
fn get_user_id() -> Principal {
    api::caller()
}

/// Create a new user account by providing a unique username.
#[update]
fn create_account(new_user: NewUser) -> Result<(), String> {
    let principal = get_user_id();

    // check if user already has an account
    let user_exists: bool = USER_MAP.with(|p| p.borrow().contains_key(&principal));
    if user_exists {
        return Err("User already has an account".to_string());
    }

    // check if username is already taken
    let username_taken: bool = USER_MAP.with(|p| {
        p.borrow().iter().any(|(_, existing_user)| *existing_user.username == new_user.username)
    });
    if username_taken {
        return Err("Username already taken".to_string());
    }

    // create new user
    let user = User {
        username: new_user.username.clone(),
        contacts: Vec::new(),
        shared_contacts: Vec::new(),
    };
    USER_MAP.with(|p| p.borrow_mut().insert(principal, user.clone()));
    
    Ok(())
}

// Get the list of contacts for the current user.
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

// Create a new contact for the current user.
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
