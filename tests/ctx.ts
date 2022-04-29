import * as anchor from "@project-serum/anchor";
import { BN, Program } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { Ido } from "../target/types/ido";
import { createMint, findATA, mintTo, TokenAccount } from "./token";
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
  ido: PublicKey;
  idoAcdm: TokenAccount;
  idoUsdc: TokenAccount;

  user1: Keypair;
  user2: Keypair;
  user3: Keypair;

  constructor() {
    this.connection = new Connection("http://localhost:8899", "recent");
    this.program = anchor.workspace.Ido;
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

    this.ido = await findPDA(this, [Buffer.from("ido")]);
    this.idoAcdm = await this.acdmATA(this.ido);
    this.idoUsdc = await this.usdcATA(this.ido);

    await mintTo(
      this,
      await this.usdcATA(this.user1.publicKey),
      this.usdcMintAuthority,
      100_000_000
    );
    await mintTo(
      this,
      await this.usdcATA(this.user2.publicKey),
      this.usdcMintAuthority,
      100_000_000
    );
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
    return this.acdmATA(await this.order(id));
  }

  async acdmATA(owner: PublicKey): Promise<TokenAccount> {
    return await findATA(this, owner, this.acdmMint);
  }

  async usdcATA(owner: PublicKey): Promise<TokenAccount> {
    return await findATA(this, owner, this.usdcMint);
  }
}
