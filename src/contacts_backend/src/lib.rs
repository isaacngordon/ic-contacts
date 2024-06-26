mod data;
mod response;

use data::new_user::NewUser;
use ic_cdk::{api, query, update};

use data::contact::{Contact, ContactID};
use data::user::User;
use response::httpish;

// Data Structures
use candid::Principal;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, };
use std::cell::RefCell;

mod tests; 

// Global State
type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    // The memory manager is used for simulating multiple memories. Given a `MemoryId` it can
    // return a memory that can be used by stable structures.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    // Initialize a `StableBTreeMap` with `MemoryId(0)` for User storage.
    static USER_MAP: RefCell<StableBTreeMap<Principal, User, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );

    // Initialize a `StableBTreeMap` with `MemoryId(1)` Contact storage.
    static CONTACT_MAP: RefCell<StableBTreeMap<ContactID, Contact, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
        )
    );

    // Initialize a `StableBTreeMap` with `MemoryId(2)` for usernames to principal mappings.
    static USERNAME_MAP: RefCell<StableBTreeMap<String, Principal, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
        )
    );

}

// Helper Functions
fn get_user_id() -> Principal {
    api::caller()
}

/// whomai i call
#[query]
fn whoami() -> (Principal, Option<String>) {
    let user_id = get_user_id();
    ic_cdk::println!("/whoami [QUERY] - Principal={:?}", user_id.to_text());
    let user: Option<User> = USER_MAP.with(|user_map| user_map.borrow().get(&user_id));
    let username = user.map(|u| u.username);
    (user_id, username)
}

/// Create a new user account by providing a unique username.
#[update]
fn create_account(new_user: NewUser) -> httpish::BasicResponse {
    let principal = get_user_id();
    ic_cdk::println!(
        "/create_account [UPDATE] - Principal={:?} Username={}",
        principal.to_string(),
        new_user.username
    );

    // check if user already has an account
    let user_exists: bool = USER_MAP.with(|p| p.borrow().contains_key(&principal));
    if user_exists {
        ic_cdk::println!("/create_account [REJECT] - User already has an account");
        return httpish::BasicResponse::Conflict("User already has an account".into());
    }

    // check if username is already taken
    let username_taken: bool = USERNAME_MAP.with(|p| p.borrow().contains_key(&new_user.username));

    if username_taken {
        ic_cdk::println!("/create_account [REJECT] - Username already taken");
        return httpish::BasicResponse::Conflict("Username already taken".into());
    }

    // create new user
    ic_cdk::println!("/create_account [INFO] - Creating new user");
    let user = User {
        username: new_user.username.clone(),
        contacts: Vec::new(),
        shared_contacts: Vec::new(),
    };

    USER_MAP.with(|p| p.borrow_mut().insert(principal, user.clone()));
    USERNAME_MAP.with(|p| p.borrow_mut().insert(new_user.username.clone(), principal));

    ic_cdk::println!("/create_account [DONE] - User: {:?}", user);
    httpish::BasicResponse::Success("Account created successfully".into())
}

/// Get the list of contacts for the current user.
#[query]
fn get_contacts() -> (httpish::BasicResponse, Vec<Contact>) {
    let user_id = get_user_id();
    ic_cdk::println!(
        "/get_contacts [QUERY] - Principal={:?}",
        user_id.to_string()
    );

    let contact_ids: Result<Vec<ContactID>, httpish::BasicResponse> = USER_MAP.with(|user_map| {
        user_map
            .borrow()
            .get(&user_id)
            .map_or(Err(httpish::BasicResponse::Unauthorized), |u| {
                Ok(u.contacts.clone())
            })
    });

    if contact_ids.is_err() {
        ic_cdk::println!("/get_contacts [REJECT] - User not found");
        return (httpish::BasicResponse::Unauthorized, Vec::new());
    }

    let contacts = CONTACT_MAP.with(|contact_map| {
        let contacts = contact_map.borrow();
        contact_ids
            .unwrap()
            .iter()
            .map(|id| contacts.get(id).unwrap().clone())
            .collect()
    });

    ic_cdk::println!("/get_contacts [DONE] - Contacts: {:?}", contacts);
    (
        httpish::BasicResponse::Success("Contacts retrieved successfully".into()),
        contacts,
    )
}

/// Create a new contact for the current user.
#[update(name = "create_contact")]
fn create_contact(new_contact: Contact) -> httpish::BasicResponse {
    let user_id = get_user_id();
    ic_cdk::println!(
        "/create_contact [UPDATE] - Principal={:?} Contact={:?}",
        user_id.to_string(),
        new_contact
    );

    let user: Option<User> = USER_MAP.with(|p| p.borrow().get(&user_id));
    if user.is_none() {
        ic_cdk::println!("/create_contact [REJECT] - User not found");
        return httpish::BasicResponse::Unauthorized;
    }
    let user = user.unwrap();

    let new_contact_id = CONTACT_MAP.with(|p| {
        let mut contacts = p.borrow_mut();
        let new_id = contacts.len() as u64;
        contacts.insert(new_id, new_contact.clone());
        new_id
    });

    let mut updated_user = user.clone();
    updated_user.contacts.push(new_contact_id);
    USER_MAP.with(|p| p.borrow_mut().insert(user_id, updated_user));

    ic_cdk::println!("/create_contact [DONE] - Contact: {:?}", new_contact);
    httpish::BasicResponse::Success("Contact created successfully".into())
}

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
