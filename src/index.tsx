import { Contract, WalletConnection } from "near-api-js";
import * as React from "react";
import ReactDOM from "react-dom";
import { App } from "./App";
import { initContract } from "./utils";

declare global {
  interface Window {
    nearInitPromise: any;
    walletConnection: WalletConnection;
    accountId: any;
    contract: Contract & {
      get_tokens: ({
        from_index,
        limit,
      }: {
        from_index: number;
        limit: number;
      }) => Promise<string[]>;
      set_token: (tokenAccountId: string) => Promise<boolean>;
      set_tokens: (tokenAccountIds: string[]) => Promise<number>;
    };
  }
}

window.nearInitPromise = initContract()
  .then(() => {
    ReactDOM.render(<App />, document.querySelector("#root"));
  })
  .catch(console.error);
