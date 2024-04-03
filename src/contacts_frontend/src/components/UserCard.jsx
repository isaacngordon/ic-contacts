import React from 'react';
import { AuthClient } from "@dfinity/auth-client";
import { HttpAgent } from "@dfinity/agent";
import { createActor, contacts_backend } from "../../../declarations/contacts_backend";

let actor = contacts_backend;

function UserCard({ setActor }) {
  const handleWhoAmI = async (event) => {
    event.preventDefault();
    const [principal, username] = await actor.whoami();
    document.getElementById("principal").innerText = principal.toString();
    
    if (username)
      document.getElementById("username").innerText = username.toString();
    else
      document.getElementById("username").innerText = "No Account";
  };

  const handleLogin = async (event) => {
    event.preventDefault();

    let authClient = await AuthClient.create();
    await new Promise((resolve) => {
      authClient.login({
        identityProvider:
          process.env.DFX_NETWORK === "ic"
            ? "https://identity.ic0.app"
            : `http://${process.env.CANISTER_ID_INTERNET_IDENTITY}.localhost:4943`,
        onSuccess: resolve,
      });
    });

    // Create an actor with the authenticated identity
    const identity = authClient.getIdentity();
    const agent = new HttpAgent({ identity });
    actor = createActor(process.env.CANISTER_ID_CONTACTS_BACKEND, {
      agent,
    });
    setActor(actor); 
  };

  return (
    <section>
      <form>
        <button id="login" onClick={handleLogin}>Login!</button>
      </form>
      <br />
      <form>
        <button id="whoAmI" onClick={handleWhoAmI}>Who Am I</button>
      </form>
      <section id="principal"></section>
      <section id="username">...</section>
    </section>
  );
}

export default UserCard;
