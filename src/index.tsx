import { Contract, WalletConnection } from "near-api-js";
import * as React from "react";
import { createRoot } from "react-dom/client";
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
    const container = document.querySelector("#root");
    if (container) {
      const root = createRoot(container);
      root.render(<App />);
    } else {
      throw new Error("Unable to find query selector at #root");
    }
  })
  .catch(console.error);
