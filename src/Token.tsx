import * as React from "react";
import { FC, useState } from "react";
import { Collapse } from "react-collapse";
import { TokenData } from "./App";
import { ChevronDown } from "./ChevronDown";

export type FungibleTokenMetadata = {
  spec: string;
  name: string;
  symbol: string;
  icon: string | null;
  reference: string | null;
  reference_hash: string | null;
  decimals: number;
};

export const Token: FC<TokenData> = ({ accountId, metadata, balance }) => {
  const [isOpen, setIsOpen] = useState(false);
  const icon =
    accountId === "wrap.testnet"
      ? "https://assets.coingecko.com/coins/images/18280/small/EX4mrWMW_400x400.jpg?1631244046"
      : metadata.icon;
  return (
    <div
      style={{
        border: "2px solid var(--fg)",
        borderRadius: "5px",
        padding: "5px",
        cursor: "pointer",
        userSelect: isOpen ? "inherit" : "none",
        marginBottom: "10px",
      }}
      onClick={() => setIsOpen((state) => !state)}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
        }}
      >
        <div
          style={{ display: "flex", alignItems: "center", userSelect: "none" }}
        >
          <img
            src={icon ?? undefined}
            height={30}
            style={{ paddingRight: "5px" }}
          />
          <p>{metadata.name}</p>
        </div>
        <div style={{ display: "flex", alignItems: "center" }}>
          <p
            style={{
              paddingRight: "5px",
              userSelect: isOpen ? "none" : "initial",
            }}
          >
            {balance ? (balance ?? 0) / 10 ** metadata.decimals : "-"}
          </p>
          <ChevronDown />
        </div>
      </div>
      <Collapse isOpened={isOpen}>
        <div style={{ padding: "5px" }}>
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
            }}
          >
            <p>Symbol</p>
            <p>{metadata.symbol}</p>
          </div>
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
            }}
          >
            <p>Decimals</p>
            <p>{metadata.decimals}</p>
          </div>
          {metadata.reference?.startsWith("http") && (
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
              }}
            >
              <p>Website</p>
              <a
                href={metadata.reference}
                style={{ color: "var(--secondary)" }}
              >
                Link
              </a>
            </div>
          )}
        </div>
      </Collapse>
    </div>
  );
};
