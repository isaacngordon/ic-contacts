use ic_cdk::{api, query, update};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

mod contact;
mod user;
mod new_contact;
mod new_user;

use contact::Contact;
use user::User;
use new_contact::NewContact;
use new_user::NewUser;

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
    use ic_cdk::api::call::test::{set_caller, Call};
    use ic_cdk::export::Principal;

    #[test]
    fn test_create_account() {
        let user1 = NewUser {
            username: "user1".to_string(),
        };
        let user2 = NewUser {
            username: "user2".to_string(),
        };
        let user1_duplicate = NewUser {
            username: "user1".to_string(),
        };

        // Set a fake caller principal for testing.
        set_caller(Principal::anonymous());

        // Clear the USERS HashMap before testing.
        let mut users = USERS.lock().unwrap();
        users.clear();
        drop(users); // Explicitly drop to release the lock.

        // Test creating a new account.
        assert!(create_account(user1).is_ok());

        // Test creating another new account with a different username.
        assert!(create_account(user2).is_ok());

        // Test creating an account with a username that already exists.
        assert!(create_account(user1_duplicate).is_err());
    }
}
