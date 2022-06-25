import * as anchor from "@project-serum/anchor";
import { BN, Program } from "@project-serum/anchor";
import {
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
  Transaction,
} from "@solana/web3.js";
import { Ido } from "../target/types/ido";
import { createMint, findATA, getTokenMetadata, TokenAccount } from "./token";
import { airdrop, findPDA } from "./utils";
import { createCreateMetadataAccountV2Instruction } from "@metaplex-foundation/mpl-token-metadata";

export class Context {
  connection: Connection;
  program: Program<Ido>;
  payer: Keypair;

  acdmMint: PublicKey;
  acdmMintAuthority: Keypair;
  usdcMint: PublicKey;
  usdcMintAuthority: Keypair;

  ido: PublicKey;
  idoAuthority: Keypair;
  idoAcdm: TokenAccount;
  idoUsdc: TokenAccount;

  user1: Keypair;
  user2: Keypair;
  user3: Keypair;

  constructor() {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    this.connection = provider.connection;
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
      this.acdmMintAuthority.publicKey,
      this.user1.publicKey,
      this.user2.publicKey,
      this.user3.publicKey,
    ]);

    this.acdmMint = await createMint(this, this.acdmMintAuthority, 2);
    this.usdcMint = await createMint(this, this.usdcMintAuthority, 6);

    const acdmMetadata = await getTokenMetadata(this.acdmMint);

    let ix = createCreateMetadataAccountV2Instruction(
      {
        metadata: acdmMetadata,
        mint: this.acdmMint,
        mintAuthority: this.acdmMintAuthority.publicKey,
        payer: this.acdmMintAuthority.publicKey,
        updateAuthority: this.acdmMintAuthority.publicKey,
      },
      {
        createMetadataAccountArgsV2: {
          data: {
            name: "Academy Token",
            symbol: "ACDM",
            uri: "https://academy.com/token-metadata",
            sellerFeeBasisPoints: 10,
            creators: null,
            collection: null,
            uses: null,
          },
          isMutable: true,
        },
      }
    );
    let tx = new Transaction().add(ix);
    await sendAndConfirmTransaction(this.connection, tx, [
      this.acdmMintAuthority,
    ]);

    this.ido = await findPDA(this, [Buffer.from("ido")]);
    this.idoAcdm = await this.acdmATA(this.ido);
    this.idoUsdc = await this.usdcATA(this.ido);
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
