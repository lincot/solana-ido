import * as anchor from "@project-serum/anchor";
import { BN, Program } from "@project-serum/anchor";
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  createMint,
  getAccount,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { Ido } from "../target/types/ido";
import { expect } from "chai";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";

chai.use(chaiAsPromised);

describe("ido", () => {
  const connection = new Connection("http://localhost:8899", "recent");
  const idoProgram = anchor.workspace.Ido as Program<
    Ido
  >;

  const acdmMintAuthority = new Keypair();
  const usdcMintAuthority = new Keypair();
  const idoAuthority = new Keypair();
  const user = new Keypair();
  const user2 = new Keypair();
  const user3 = new Keypair();

  it("airdrops", async () => {
    await Promise.all(
      await Promise.all(
        [acdmMintAuthority, usdcMintAuthority, idoAuthority, user, user2, user3]
          .map(
            async (k) =>
              connection.confirmTransaction(
                await connection.requestAirdrop(
                  k.publicKey,
                  100_000_000,
                ),
              ),
          ),
      ),
    );
  });

  let acdmMint: PublicKey;
  let usdcMint: PublicKey;

  it("creates mints", async () => {
    acdmMint = await createMint(
      connection,
      acdmMintAuthority,
      acdmMintAuthority.publicKey,
      undefined,
      2,
    );
    // fake usdc
    usdcMint = await createMint(
      connection,
      usdcMintAuthority,
      usdcMintAuthority.publicKey,
      undefined,
      6,
    );
  });

  let ido: PublicKey;
  let idoAcdm: PublicKey;
  let idoUsdc: PublicKey;

  it("initializes", async () => {
    [ido] = await PublicKey
      .findProgramAddress(
        [Buffer.from("ido")],
        idoProgram.programId,
      );

    [idoAcdm] = await PublicKey.findProgramAddress(
      [
        Buffer.from("ido_acdm"),
      ],
      idoProgram.programId,
    );

    [idoUsdc] = await PublicKey.findProgramAddress(
      [
        Buffer.from("ido_usdc"),
      ],
      idoProgram.programId,
    );

    await idoProgram.methods.initialize(new BN(2))
      .accounts({
        ido,
        idoAuthority: idoAuthority.publicKey,
        acdmMint,
        idoAcdm,
        usdcMint,
        idoUsdc,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
      }).signers([idoAuthority]).rpc();
  });

  const endIdo = async () => {
    await idoProgram.methods.endIdo().accounts({
      ido,
      idoAuthority: idoAuthority.publicKey,
    }).signers([idoAuthority]).rpc();
  };

  it("fails to end ido right away", async () => {
    await expect(endIdo()).to.be.rejected;
  });

  it("starts sale round", async () => {
    await idoProgram.methods.startSaleRound().accounts({
      ido,
      idoAuthority: idoAuthority.publicKey,
      acdmMintAuthority: acdmMintAuthority.publicKey,
      acdmMint,
      idoAcdm,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([idoAuthority, acdmMintAuthority]).rpc();
  });

  let userAcdm: PublicKey;
  let userUsdc: PublicKey;
  let user2Acdm: PublicKey;
  let user2Usdc: PublicKey;
  let user3Usdc: PublicKey;
  let idoAuthorityUsdc: PublicKey;

  const getAta = async (user: Keypair, mint: PublicKey) =>
    (await getOrCreateAssociatedTokenAccount(
      connection,
      user,
      mint,
      user.publicKey,
    )).address;

  it("sets users' ATAs", async () => {
    [userAcdm, userUsdc, user2Acdm, user2Usdc, user3Usdc, idoAuthorityUsdc] =
      await Promise.all([
        getAta(user, acdmMint),
        getAta(user, usdcMint),
        getAta(user2, acdmMint),
        getAta(user2, usdcMint),
        getAta(user3, usdcMint),
        getAta(idoAuthority, usdcMint),
      ]);

    await mintTo(
      connection,
      user,
      usdcMint,
      userUsdc,
      usdcMintAuthority,
      100_000_000,
    );

    await mintTo(
      connection,
      user2,
      usdcMint,
      user2Usdc,
      usdcMintAuthority,
      100_000_000,
    );
  });

  let member: PublicKey;
  let member2: PublicKey;
  let member3: PublicKey;

  it("registers users", async () => {
    [member3] = await PublicKey
      .findProgramAddress(
        [Buffer.from("member"), user3.publicKey.toBuffer()],
        idoProgram.programId,
      );

    await idoProgram.methods.registerMember(null).accounts({
      member: member3,
      authority: user3.publicKey,
      systemProgram: SystemProgram.programId,
    }).signers([user3]).rpc();

    [member2] = await PublicKey
      .findProgramAddress(
        [Buffer.from("member"), user2.publicKey.toBuffer()],
        idoProgram.programId,
      );

    await idoProgram.methods.registerMember(user3.publicKey).accounts({
      member: member2,
      authority: user2.publicKey,
      systemProgram: SystemProgram.programId,
    }).remainingAccounts([{
      pubkey: member3,
      isWritable: false,
      isSigner: false,
    }]).signers([user2]).rpc();

    [member] = await PublicKey
      .findProgramAddress(
        [Buffer.from("member"), user.publicKey.toBuffer()],
        idoProgram.programId,
      );

    await idoProgram.methods.registerMember(user2.publicKey).accounts({
      member: member,
      authority: user.publicKey,
      systemProgram: SystemProgram.programId,
    }).remainingAccounts([{
      pubkey: member2,
      isWritable: false,
      isSigner: false,
    }]).signers([user]).rpc();
  });

  it("buys ACDM", async () => {
    await idoProgram.methods.buyAcdm(new BN(500)).accounts({
      ido,
      idoAcdm,
      idoUsdc,
      buyer: user.publicKey,
      buyerMember: member,
      buyerAcdm: userAcdm,
      buyerUsdc: userUsdc,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).remainingAccounts([{
      pubkey: member2,
      isWritable: false,
      isSigner: false,
    }, {
      pubkey: user2Usdc,
      isWritable: true,
      isSigner: false,
    }, {
      pubkey: user3Usdc,
      isWritable: true,
      isSigner: false,
    }]).signers([user]).rpc();

    const userAcdmAccount = await getAccount(connection, userAcdm);
    expect(userAcdmAccount.amount).to.eql(BigInt(500));

    const userUsdcAccount = await getAccount(connection, userUsdc);
    expect(userUsdcAccount.amount).to.eql(BigInt(50_000_000));

    const user2UsdcAccount = await getAccount(connection, user2Usdc);
    expect(user2UsdcAccount.amount).to.eql(BigInt(102_500_000));

    const user3UsdcAccount = await getAccount(connection, user3Usdc);
    expect(user3UsdcAccount.amount).to.eql(BigInt(1_425_000));

    const idoUsdcAccount = await getAccount(connection, idoUsdc);
    expect(idoUsdcAccount.amount).to.eql(BigInt(46_075_000));
  });

  it("starts trade round", async () => {
    await idoProgram.methods.startTradeRound().accounts({
      ido,
      idoAuthority: idoAuthority.publicKey,
      acdmMint,
      idoAcdm,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([idoAuthority]).rpc();

    const idoAcdmAccount = await getAccount(connection, idoAcdm);
    expect(idoAcdmAccount.amount).to.eql(BigInt(0));
  });

  let order: PublicKey;
  let orderAcdm: PublicKey;

  it("adds order", async () => {
    [order] = await PublicKey
      .findProgramAddress(
        [Buffer.from("order"), new BN(0).toArrayLike(Buffer, "le", 8)],
        idoProgram.programId,
      );

    [orderAcdm] = await PublicKey
      .findProgramAddress(
        [Buffer.from("order_acdm"), new BN(0).toArrayLike(Buffer, "le", 8)],
        idoProgram.programId,
      );

    await idoProgram.methods.addOrder(new BN(100), new BN(130_000)).accounts({
      ido,
      order,
      acdmMint,
      orderAcdm,
      user: user.publicKey,
      userAcdm,
      rent: SYSVAR_RENT_PUBKEY,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    }).signers([user]).rpc();

    const userAcdmAccount = await getAccount(connection, userAcdm);
    expect(userAcdmAccount.amount).to.eql(BigInt(400));

    const orderAcdmAccount = await getAccount(connection, orderAcdm);
    expect(orderAcdmAccount.amount).to.eql(BigInt(100));
  });

  it("redeems order partly", async () => {
    await idoProgram.methods.redeemOrder(new BN(0), new BN(40)).accounts({
      ido,
      usdcMint,
      idoUsdc,
      order,
      orderAcdm,
      buyer: user2.publicKey,
      buyerAcdm: user2Acdm,
      buyerUsdc: user2Usdc,
      seller: user.publicKey,
      sellerMember: member,
      sellerUsdc: userUsdc,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).remainingAccounts([{
      pubkey: member2,
      isWritable: false,
      isSigner: false,
    }, {
      pubkey: user2Usdc,
      isWritable: true,
      isSigner: false,
    }, {
      pubkey: user3Usdc,
      isWritable: true,
      isSigner: false,
    }]).signers([user2]).rpc();

    const userUsdcAccount = await getAccount(connection, userUsdc);
    expect(userUsdcAccount.amount).to.eql(BigInt(54_940_000));

    const user2AcdmAccount = await getAccount(connection, user2Acdm);
    expect(user2AcdmAccount.amount).to.eql(BigInt(40));

    const orderAcdmAccount = await getAccount(connection, orderAcdm);
    expect(orderAcdmAccount.amount).to.eql(BigInt(60));
  });

  it("closes order", async () => {
    await idoProgram.methods.removeOrder(new BN(0)).accounts({
      order,
      orderAcdm,
      user: user.publicKey,
      userAcdm,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([user]).rpc();

    const userAcdmAccount = await getAccount(connection, userAcdm);
    expect(userAcdmAccount.amount).to.eql(BigInt(460));
  });

  it("withdraws ido usdc", async () => {
    await idoProgram.methods.withdrawIdoUsdc().accounts({
      ido,
      idoAuthority: idoAuthority.publicKey,
      idoUsdc,
      to: idoAuthorityUsdc,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([idoAuthority]).rpc();

    const idoUsdcAccount = await getAccount(connection, idoUsdc);
    expect(idoUsdcAccount.amount).to.eql(BigInt(0));

    const idoAuthorityUsdcAccount = await getAccount(
      connection,
      idoAuthorityUsdc,
    );
    expect(idoAuthorityUsdcAccount.amount).to.eql(BigInt(46_075_000));
  });

  it("ends ido", async () => {
    await endIdo();
  });
});
