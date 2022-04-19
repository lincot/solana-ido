import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { getOrCreateAssociatedTokenAccount } from "@solana/spl-token";

export async function getATA(
  connection: Connection,
  user: Keypair,
  mint: PublicKey,
): Promise<PublicKey> {
  return (await getOrCreateAssociatedTokenAccount(
    connection,
    user,
    mint,
    user.publicKey,
  )).address;
}
