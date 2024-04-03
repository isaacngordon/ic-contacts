import React from 'react';
import { AuthClient } from "@dfinity/auth-client";
import { HttpAgent } from "@dfinity/agent";
import { createActor } from "@declarations/contacts_backend";

let actor;

function UserCard({ setActor }) {
  const handleWhoAmI = async () => {
    const principal = await actor.whoami();
    document.getElementById("principal").innerText = principal.toString();
  };

  const handleLogin = async (event) => {
    event.preventDefault();

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
    </section>
  );
}

export default UserCard;
