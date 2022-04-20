import { BN } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";

export class Context {
  connection: Connection;

  payer: Keypair;

  acdmMintAuthority: Keypair;
  acdmMint: PublicKey;
  usdcMintAuthority: Keypair;
  usdcMint: PublicKey;

  idoAuthority: Keypair;
  idoAuthorityUsdc: PublicKey;

  ido: PublicKey;
  idoAcdm: PublicKey;
  idoUsdc: PublicKey;

  orderId: BN;
  order: PublicKey;
  orderAcdm: PublicKey;
}
