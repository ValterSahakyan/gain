import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TokenSale } from "../target/types/token_sale";
import { expect } from "chai";

describe("token_sale", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.TokenSale as Program<TokenSale>;
  const provider = anchor.getProvider();

  it("Initializes the token sale", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
