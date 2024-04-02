import React, { useState, useEffect } from 'react';
import { AuthClient } from "@dfinity/auth-client";
import { HttpAgent } from "@dfinity/agent";
import {
    createActor,
    ii_integration_backend,
} from "../../declarations/ii_integration_backend";

let actor = ii_integration_backend;

console.log(process.env.CANISTER_ID_INTERNET_IDENTITY);
import { contacts_backend } from '../../declarations/contacts_backend';

const emptyContact = { name: '', email: '', phone: '' };

function App() {
  const [username, setUsername] = useState('');
  const [contact, setContact] = useState(emptyContact);
  const [contacts, setContacts] = useState([]);
  const [selectedContactId, setSelectedContactId] = useState(null);
  const [authClient, setAuthClient] = useState(null);

  useEffect(() => {
    AuthClient.create().then(setAuthClient);
  }, []);

  const handleCreateAccount = async () => {
    await contacts_backend.create_account({ username });
    // Fetch contacts or update state as needed
  };

  const handleAddContact = async () => {
    await contacts_backend.add_contact(contact);
    setContact(emptyContact);
    // Fetch contacts or update state as needed
  };

  const handleEditContact = async () => {
    if (selectedContactId !== null) {
      await contacts_backend.edit_contact(selectedContactId, contact);
      setContact(emptyContact);
      setSelectedContactId(null);
      // Fetch contacts or update state as needed
    }
  };

  const handleDeleteContact = async (contactId) => {
    await contacts_backend.delete_contact(contactId);
    // Fetch contacts or update state as needed
  };

  const handleShareContact = async (contactId, shareWithUsername) => {
    await contacts_backend.share_contact(contactId, shareWithUsername);
    // Fetch contacts or update state as needed
  };

  const handleRevokeSharedContact = async (contactId, revokeFromUsername) => {
    await contacts_backend.revoke_shared_contact(contactId, revokeFromUsername);
    // Fetch contacts or update state as needed
  };


  // Placeholder for the new UI components and interactions
  return (
    <main>
      <h1>Contacts Manager</h1>
      <img src="logo2.svg" alt="DFINITY logo" />
      <br />
      <br />
      <form>
        <button onClick={handleLogin}>Login!</button>
      </form>
      <br />
      <form>
        <button onClick={handleWhoAmI}>Who Am I</button>
      </form>
      <section id="principal"></section>
      {/* UI components for account creation */}
      <section>
        <input
          type="text"
          placeholder="Username"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
        />
        <button onClick={handleCreateAccount}>Create Account</button>
      </section>
      {/* UI components for contact management */}
      {/* ... Add form elements and buttons for add, edit, delete, share, and revoke operations ... */}
    </main>
  );
}

export default App;
