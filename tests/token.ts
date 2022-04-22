import { getAccount } from "@solana/spl-token";
import { Keypair, PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import { Context } from "./ctx";
import * as token from "@solana/spl-token";

export class TokenAccount {
  address: PublicKey;
  mint: PublicKey;

  constructor(address: PublicKey, mint: PublicKey) {
    this.address = address;
    this.mint = mint;
  }

  async amount(ctx: Context): Promise<BigInt> {
    return (await getAccount(
      ctx.connection,
      this.address,
    )).amount;
  }

  async checkAmount(ctx: Context, amount: number): Promise<void> {
    expect(await this.amount(ctx)).to
      .eql(
        BigInt(amount),
      );
  }
}

export async function createMint(
  ctx: Context,
  authority: Keypair,
  decimals: number,
) {
  return await token.createMint(
    ctx.connection,
    ctx.payer,
    authority.publicKey,
    undefined,
    decimals,
  );
}

export async function mintTo(
  ctx: Context,
  mint: PublicKey,
  user: PublicKey,
  mintAuthority: Keypair,
  amount: number | bigint,
) {
  token.mintTo(
    ctx.connection,
    ctx.payer,
    mint,
    (await ctx.ata(user, mint)).address,
    mintAuthority,
    amount,
  );
}
