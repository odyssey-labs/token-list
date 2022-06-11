import "regenerator-runtime/runtime";
import * as React from "react";
import { useState, useEffect } from "react";
import { login, logout } from "./utils";
import "./global.css";

import { getConfig } from "./config";
import { FungibleTokenMetadata, Token } from "./Token";
import pMap from "p-map";
const { networkId } = getConfig(process.env.NODE_ENV || "development");

export type TokenData = {
  accountId: string;
  balance: number | null;
  metadata: FungibleTokenMetadata;
};

const getTokenData = async (
  accountId: string | undefined
): Promise<TokenData[]> => {
  // TODO: Handle pagination
  const tokenAccountIds = await window.contract.get_tokens({
    from_index: 0,
    limit: 100,
  });
  return pMap(tokenAccountIds, async (tokenAccountId) => {
    const tokenAccount = await window.walletConnection._near.account(
      tokenAccountId
    );
    const balance: number | null = accountId
      ? await tokenAccount.viewFunction(tokenAccountId, "ft_balance_of", {
          account_id: window.accountId,
        })
      : null;
    const metadata: FungibleTokenMetadata = await tokenAccount.viewFunction(
      tokenAccountId,
      "ft_metadata"
    );
    return {
      accountId: tokenAccountId,
      balance,
      metadata,
    };
  });
};

export function App() {
  const [tokens, setTokens] = useState<TokenData[]>([]);

  const [buttonDisabled, setButtonDisabled] = useState(true);

  const [showNotification, setShowNotification] = useState(false);
  const [isFailureNotification, setIsFailureNotification] = useState(false);
  const [notificationMessage, setNotificationMessage] = useState("");

  useEffect(() => {
    getTokenData(window.accountId).then(setTokens);
  }, []);

  return (
    <>
      <div style={{ display: "flex", float: "right" }}>
        <p>{window.accountId}</p>
        <button
          className="link"
          style={{
            color: window.walletConnection.isSignedIn()
              ? "var(--primary)"
              : "var(--secondary)",
          }}
          onClick={window.walletConnection.isSignedIn() ? logout : login}
        >
          Sign {window.walletConnection.isSignedIn() ? "out" : "in"}
        </button>
      </div>
      <main>
        <h1>Token List</h1>
        <form
          onSubmit={async (event) => {
            event.preventDefault();

            // get elements from the form using their id attribute
            const { fieldset, token } = (event.target as any).elements;

            // disable the form while the value gets updated on-chain
            fieldset.disabled = true;

            try {
              await window.contract.add_token({
                token: token.value,
              });
              setNotificationMessage(`Added ${token.value} to the token list`);
              setShowNotification(true);

              // remove Notification again after css animation completes
              // this allows it to be shown again next time the form is submitted
              setTimeout(() => {
                setNotificationMessage("");
                setIsFailureNotification(false);
                setShowNotification(false);
              }, 11000);
            } catch (e) {
              try {
                const jsonError = JSON.parse((e as Error).message);
                const isNotTokenAccount =
                  jsonError &&
                  jsonError?.kind?.ExecutionError &&
                  jsonError.kind.ExecutionError.includes(
                    "Unable to get result of token account verification"
                  );
                if (isNotTokenAccount) {
                  setNotificationMessage(
                    "The provided account ID does not contain a fungible token contract"
                  );
                  setIsFailureNotification(true);
                  setShowNotification(true);

                  // remove Notification again after css animation completes
                  // this allows it to be shown again next time the form is submitted
                  setTimeout(() => {
                    setNotificationMessage("");
                    setIsFailureNotification(false);
                    setShowNotification(false);
                  }, 11000);
                }
              } catch (handleError) {
                alert(
                  "Something went wrong! " +
                    "Maybe you need to sign out and back in? " +
                    "Check your browser console for more info."
                );
                throw e;
              }
            } finally {
              // re-enable the form, whether the call succeeded or failed
              fieldset.disabled = false;
              getTokenData(window.accountId).then(setTokens);
            }
          }}
        >
          <fieldset id="fieldset">
            <label
              htmlFor="greeting"
              style={{
                display: "block",
                color: "var(--gray)",
                marginBottom: "0.5em",
              }}
            >
              Add New Token
            </label>
            <div style={{ display: "flex" }}>
              <input
                autoComplete="off"
                id="token"
                onChange={(e) => setButtonDisabled(e.target.value.length === 0)}
                style={{ flex: 1 }}
              />
              <button
                disabled={buttonDisabled}
                style={{ borderRadius: "0 5px 5px 0" }}
              >
                Add
              </button>
            </div>
          </fieldset>
        </form>
        {tokens.map((data) => (
          <Token key={data.accountId} {...data} />
        ))}
      </main>
      {showNotification && (
        <Notification
          failure={isFailureNotification}
          message={notificationMessage}
        />
      )}
    </>
  );
}

// this component gets rendered by App after the form is submitted
function Notification({
  failure,
  message,
}: {
  failure: boolean;
  message: string;
}) {
  const urlPrefix = `https://explorer.${networkId}.near.org/accounts`;
  return (
    <aside>
      <a
        target="_blank"
        rel="noreferrer"
        href={`${urlPrefix}/${window.accountId}`}
      >
        {window.accountId}
      </a>
      {
        " " /* React trims whitespace around tags; insert literal space character when needed */
      }
      called method: 'add_token' in contract:{" "}
      <a
        target="_blank"
        rel="noreferrer"
        href={`${urlPrefix}/${window.contract.contractId}`}
      >
        {window.contract.contractId}
      </a>
      <br />
      {message}
      <footer>
        <div style={{ color: failure ? "var(--primary)" : "var(--success)" }}>
          {failure ? "✘ Failed" : "✔ Succeeded"}
        </div>
        <div>Just now</div>
      </footer>
    </aside>
  );
}
