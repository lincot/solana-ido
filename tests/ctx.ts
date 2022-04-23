import * as anchor from "@project-serum/anchor";
import { BN, Program } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { Ido } from "../target/types/ido";
import { createMint, getATA, mintTo, TokenAccount } from "./token";
import { airdrop, findPDA } from "./utils";

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

  ido: PublicKey;
  idoAcdm: TokenAccount;
  idoUsdc: TokenAccount;

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

    this.ido = await this.getIdo();
    this.idoAcdm = await this.getIdoAcdm();
    this.idoUsdc = await this.getIdoUsdc();

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

  private async getIdo(): Promise<PublicKey> {
    return await findPDA(this, [Buffer.from("ido")]);
  }

  private async getIdoAcdm(): Promise<TokenAccount> {
    const address = await findPDA(this, [Buffer.from("ido_acdm")]);
    return new TokenAccount(address, this.acdmMint);
  }

  private async getIdoUsdc(): Promise<TokenAccount> {
    const address = await findPDA(this, [Buffer.from("ido_usdc")]);
    return new TokenAccount(address, this.usdcMint);
  }

  async member(user: PublicKey): Promise<PublicKey> {
    return await findPDA(this, [Buffer.from("member"), user.toBuffer()]);
  }

  async order(id: BN): Promise<PublicKey> {
    return await findPDA(this, [
      Buffer.from("order"),
      id.toArrayLike(Buffer, "le", 8),
    ]);
  }

  async orderAcdm(id: BN): Promise<TokenAccount> {
    const address = await findPDA(this, [
      Buffer.from("order_acdm"),
      id.toArrayLike(Buffer, "le", 8),
    ]);
    return new TokenAccount(address, this.acdmMint);
  }

  async acdmATA(owner: PublicKey): Promise<TokenAccount> {
    return getATA(this, owner, this.acdmMint);
  }

  async usdcATA(owner: PublicKey): Promise<TokenAccount> {
    return getATA(this, owner, this.usdcMint);
  }
}
