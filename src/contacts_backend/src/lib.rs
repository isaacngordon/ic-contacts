use candid::CandidType;
use ic_cdk::{api, update};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

// Data Structures
#[derive(CandidType, Deserialize, Serialize, Debug, Clone)]
pub struct Contact {
    id: u64,
    name: String,
    email: String,
    phone: String,
}

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub struct User {
    username: String,
    contacts: Vec<Contact>,
    shared_contacts: Vec<u64>, // Contact IDs
}

#[derive(CandidType, Deserialize)]
pub struct NewContact {
    name: String,
    email: String,
    phone: String,
}

#[derive(CandidType, Deserialize)]
pub struct NewUser {
    username: String,
}

// Global State
static USERS: Lazy<Mutex<HashMap<String, User>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// Helper Functions
fn get_user_id() -> String {
    api::caller().to_text()
}

// Canister Functions
#[update]
fn create_account(new_user: NewUser) -> Result<(), String> {
    let user_id = get_user_id();
    let mut users = USERS.lock().unwrap();

    if users
        .values()
        .any(|user| user.username == new_user.username)
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

#[update]
fn add_contact(new_contact: NewContact) -> Result<(), String> {
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
