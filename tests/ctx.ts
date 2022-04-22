import * as anchor from "@project-serum/anchor";
import { BN, Program } from "@project-serum/anchor";
import { getOrCreateAssociatedTokenAccount } from "@solana/spl-token";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { Ido } from "../target/types/ido";
import { createMint, mintTo, TokenAccount } from "./token";
import { airdrop } from "./utils";

export class Context {
  connection: Connection;

  program: Program<Ido>;

  payer: Keypair;

  acdmMintAuthority: Keypair;
  acdmMint: PublicKey;
  usdcMintAuthority: Keypair;
  usdcMint: PublicKey;

  idoAuthority: Keypair;

  user1: Keypair;
  user2: Keypair;
  user3: Keypair;

  private _ido: PublicKey;
  private _idoAcdm: PublicKey;
  private _idoUsdc: PublicKey;

  constructor() {
    this.program = anchor.workspace.Ido;
    this.connection = new Connection("http://localhost:8899", "recent");

    this.payer = new Keypair();
    this.acdmMintAuthority = new Keypair();
    this.usdcMintAuthority = new Keypair();
    this.idoAuthority = new Keypair();
    this.user1 = new Keypair();
    this.user2 = new Keypair();
    this.user3 = new Keypair();
  }

  async setup() {
    await airdrop(this, [
      this.idoAuthority.publicKey,
      this.user1.publicKey,
      this.user2.publicKey,
      this.user3.publicKey,
    ]);

    this.acdmMint = await createMint(this, this.acdmMintAuthority, 2);
    this.usdcMint = await createMint(this, this.usdcMintAuthority, 6);

    await mintTo(
      this,
      this.usdcMint,
      this.user1.publicKey,
      this.usdcMintAuthority,
      100_000_000
    );
    await mintTo(
      this,
      this.usdcMint,
      this.user2.publicKey,
      this.usdcMintAuthority,
      100_000_000
    );
  }

  public async ido(): Promise<PublicKey> {
    return (
      this._ido ??
      (
        await PublicKey.findProgramAddress(
          [Buffer.from("ido")],
          this.program.programId
        )
      )[0]
    );
  }

  public async idoAcdm(): Promise<TokenAccount> {
    const address =
      this._idoAcdm ??
      (
        await PublicKey.findProgramAddress(
          [Buffer.from("ido_acdm")],
          this.program.programId
        )
      )[0];

    return new TokenAccount(address, this.acdmMint);
  }

  public async idoUsdc(): Promise<TokenAccount> {
    const address =
      this._idoUsdc ??
      (
        await PublicKey.findProgramAddress(
          [Buffer.from("ido_usdc")],
          this.program.programId
        )
      )[0];

    return new TokenAccount(address, this.usdcMint);
  }

  async member(user: PublicKey): Promise<PublicKey> {
    return (
      await PublicKey.findProgramAddress(
        [Buffer.from("member"), user.toBuffer()],
        this.program.programId
      )
    )[0];
  }

  async order(id: BN): Promise<PublicKey> {
    return (
      await PublicKey.findProgramAddress(
        [Buffer.from("order"), id.toArrayLike(Buffer, "le", 8)],
        this.program.programId
      )
    )[0];
  }

  async orderAcdm(id: BN): Promise<TokenAccount> {
    const address = (
      await PublicKey.findProgramAddress(
        [Buffer.from("order_acdm"), id.toArrayLike(Buffer, "le", 8)],
        this.program.programId
      )
    )[0];

    return new TokenAccount(address, this.acdmMint);
  }

  async ata(user: PublicKey, mint: PublicKey): Promise<TokenAccount> {
    const address = (
      await getOrCreateAssociatedTokenAccount(
        this.connection,
        this.payer,
        mint,
        user
      )
    ).address;

    return new TokenAccount(address, mint);
  }
}
