import { BN, Program } from "@project-serum/anchor";
import { getOrCreateAssociatedTokenAccount } from "@solana/spl-token";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { Ido } from "../target/types/ido";

export class Context {
  connection: Connection;

  program: Program<Ido>;

  payer: Keypair;

  acdmMintAuthority: Keypair;
  acdmMint: PublicKey;
  usdcMintAuthority: Keypair;
  usdcMint: PublicKey;

  idoAuthority: Keypair;

  private _ido: PublicKey;
  private _idoAcdm: PublicKey;
  private _idoUsdc: PublicKey;

  public async ido(): Promise<PublicKey> {
    return this._ido ?? (await PublicKey
      .findProgramAddress(
        [Buffer.from("ido")],
        this.program.programId,
      ))[0];
  }

  public async idoAcdm(): Promise<PublicKey> {
    return this._idoAcdm ?? (await PublicKey
      .findProgramAddress(
        [Buffer.from("ido_acdm")],
        this.program.programId,
      ))[0];
  }

  public async idoUsdc(): Promise<PublicKey> {
    return this._idoUsdc ?? (await PublicKey
      .findProgramAddress(
        [Buffer.from("ido_usdc")],
        this.program.programId,
      ))[0];
  }

  async member(user: PublicKey): Promise<PublicKey> {
    return (await PublicKey
      .findProgramAddress(
        [Buffer.from("member"), user.toBuffer()],
        this.program.programId,
      ))[0];
  }

  async order(id: BN): Promise<PublicKey> {
    return (await PublicKey
      .findProgramAddress(
        [Buffer.from("order"), id.toArrayLike(Buffer, "le", 8)],
        this.program.programId,
      ))[0];
  }

  async orderAcdm(id: BN): Promise<PublicKey> {
    return (await PublicKey
      .findProgramAddress(
        [Buffer.from("order_acdm"), id.toArrayLike(Buffer, "le", 8)],
        this.program.programId,
      ))[0];
  }

  async ata(
    user: PublicKey,
    mint: PublicKey,
  ): Promise<PublicKey> {
    return (await getOrCreateAssociatedTokenAccount(
      this.connection,
      this.payer,
      mint,
      user,
    )).address;
  }
}
