import { PublicKey } from "@solana/web3.js";
import { getOrCreateAssociatedTokenAccount } from "@solana/spl-token";
import { Context } from "./ctx";

export async function findATA(
  ctx: Context,
  user: PublicKey,
  mint: PublicKey,
): Promise<PublicKey> {
  return (await getOrCreateAssociatedTokenAccount(
    ctx.connection,
    ctx.payer,
    mint,
    user,
  )).address;
}
