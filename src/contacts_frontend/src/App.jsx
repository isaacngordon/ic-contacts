import React, { useState, useEffect } from 'react';
import { AuthClient } from "@dfinity/auth-client";
import { HttpAgent } from "@dfinity/agent";
import {
    createActor,
    contacts_backend,
} from "../../declarations/contacts_backend";

let actor = contacts_backend;

console.log(process.env.CANISTER_ID_CONTACTS_BACKEND);

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

  const handleWhoAmI = async () => {
    const principal = await actor.whoami();
    document.getElementById("principal").innerText = principal.toString();
  };

  const handleLogin = async () => {
    let authClient = await AuthClient.create();
    await new Promise((resolve) => {
      authClient.login({
        identityProvider:
          process.env.DFX_NETWORK === "ic"
            ? "https://identity.ic0.app"
            : `http://rdmx6-jaaaa-aaaaa-aaadq-cai.localhost:4943`,
        onSuccess: resolve,
      });
    });
    const identity = authClient.getIdentity();
    const agent = new HttpAgent({ identity });
    actor = createActor(process.env.CANISTER_ID_CONTACTS_BACKEND, {
      agent,
    });
  };

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
        <button id="login" onClick={handleLogin}>Login!</button>
      </form>
      <br />
      <form>
        <button id="whoAmI" onClick={handleWhoAmI}>Who Am I</button>
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
