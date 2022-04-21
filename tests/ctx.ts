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

  members: Map<PublicKey, PublicKey>;

  async member(user: PublicKey) {
    const in_map = this.members.get(user);

    if (in_map) {
      return in_map;
    }

    const [member] = await PublicKey
      .findProgramAddress(
        [Buffer.from("member"), user.toBuffer()],
        this.program.programId,
      );

    this.members.set(user, member);

    return member;
  }

  private orders: Map<BN, PublicKey>;

  async order(id: BN) {
    const in_map = this.orders.get(id);

    if (in_map) {
      return in_map;
    }

    const [order] = await PublicKey
      .findProgramAddress(
        [Buffer.from("order"), id.toArrayLike(Buffer, "le", 8)],
        this.program.programId,
      );

    this.orders.set(id, order);

    return order;
  }

  private orderAcdms: Map<BN, PublicKey>;

  async orderAcdm(id: BN) {
    const in_map = this.orderAcdms.get(id);

    if (in_map) {
      return in_map;
    }

    const [orderAcdm] = await PublicKey
      .findProgramAddress(
        [Buffer.from("order_acdm"), id.toArrayLike(Buffer, "le", 8)],
        this.program.programId,
      );

    this.orderAcdms.set(id, orderAcdm);

    return orderAcdm;
  }

  private atas: Map<[PublicKey, PublicKey], PublicKey>;

  async ata(
    user: PublicKey,
    mint: PublicKey,
  ): Promise<PublicKey> {
    const in_map = this.atas.get([user, mint]);

    if (in_map) {
      return in_map;
    }

    const ata = (await getOrCreateAssociatedTokenAccount(
      this.connection,
      this.payer,
      mint,
      user,
    )).address;

    this.atas.set([user, mint], ata);

    return ata;
  }
  constructor() {
    this.members = new Map();
    this.orders = new Map();
    this.orderAcdms = new Map();
    this.atas = new Map();
  }
}
