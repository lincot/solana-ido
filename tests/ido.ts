import { BN } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { createMint, getAccount, mintTo } from "@solana/spl-token";
import { expect } from "chai";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { getAta } from "./utils";
import {
  addOrder,
  buyAcdm,
  endIdo,
  initialize,
  redeemOrder,
  registerMember,
  removeOrder,
  startSaleRound,
  startTradeRound,
  withdrawIdoUsdc,
} from "./api";

chai.use(chaiAsPromised);

describe("ido", () => {
  const connection = new Connection("http://localhost:8899", "recent");

  const acdmMintAuthority = new Keypair();
  const usdcMintAuthority = new Keypair();
  const idoAuthority = new Keypair();
  const user1 = new Keypair();
  const user2 = new Keypair();
  const user3 = new Keypair();

  it("airdrops", async () => {
    await Promise.all(
      await Promise.all(
        [
          acdmMintAuthority,
          usdcMintAuthority,
          idoAuthority,
          user1,
          user2,
          user3,
        ]
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
    [ido, idoAcdm, idoUsdc] = await initialize(
      acdmMint,
      usdcMint,
      idoAuthority,
    );
  });

  it("fails to end ido right away", async () => {
    await expect(endIdo(ido, idoAuthority)).to.be.rejected;
  });

  it("starts sale round", async () => {
    await startSaleRound(
      ido,
      idoAcdm,
      idoAuthority,
      acdmMint,
      acdmMintAuthority,
    );
  });

  let user1Acdm: PublicKey;
  let user1Usdc: PublicKey;
  let user2Acdm: PublicKey;
  let user2Usdc: PublicKey;
  let user3Usdc: PublicKey;
  let idoAuthorityUsdc: PublicKey;

  it("sets users' ATAs", async () => {
    [
      user1Acdm,
      user1Usdc,
      user2Acdm,
      user2Usdc,
      user3Usdc,
      idoAuthorityUsdc,
    ] = await Promise.all([
      getAta(connection, user1, acdmMint),
      getAta(connection, user1, usdcMint),
      getAta(connection, user2, acdmMint),
      getAta(connection, user2, usdcMint),
      getAta(connection, user3, usdcMint),
      getAta(connection, idoAuthority, usdcMint),
    ]);

    await Promise.all([
      mintTo(
        connection,
        user1,
        usdcMint,
        user1Usdc,
        usdcMintAuthority,
        100_000_000,
      ),
      mintTo(
        connection,
        user2,
        usdcMint,
        user2Usdc,
        usdcMintAuthority,
        100_000_000,
      ),
    ]);
  });

  let member1: PublicKey;
  let member2: PublicKey;
  let member3: PublicKey;

  it("registers users", async () => {
    member3 = await registerMember(user3, null, null);
    member2 = await registerMember(user2, user3.publicKey, member3);
    member1 = await registerMember(user1, user2.publicKey, member2);
  });

  it("buys ACDM", async () => {
    await buyAcdm(
      new BN(500),
      ido,
      idoAcdm,
      idoUsdc,
      user1,
      user1Acdm,
      user1Usdc,
      member2,
      user2Usdc,
      user3Usdc,
    );

    const user1AcdmAccount = await getAccount(connection, user1Acdm);
    expect(user1AcdmAccount.amount).to.eql(BigInt(500));

    const user1UsdcAccount = await getAccount(connection, user1Usdc);
    expect(user1UsdcAccount.amount).to.eql(BigInt(50_000_000));

    const user2UsdcAccount = await getAccount(connection, user2Usdc);
    expect(user2UsdcAccount.amount).to.eql(BigInt(102_500_000));

    const user3UsdcAccount = await getAccount(connection, user3Usdc);
    expect(user3UsdcAccount.amount).to.eql(BigInt(1_500_000));

    const idoUsdcAccount = await getAccount(connection, idoUsdc);
    expect(idoUsdcAccount.amount).to.eql(BigInt(46_000_000));
  });

  it("fails to buy too much or 0", async () => {
    await expect(buyAcdm(
      new BN(9_000_000_000_000_000),
      ido,
      idoAcdm,
      idoUsdc,
      user1,
      user1Acdm,
      user1Usdc,
      member2,
      user2Usdc,
      user3Usdc,
    )).to.be.rejected;
    await expect(buyAcdm(
      new BN(0),
      ido,
      idoAcdm,
      idoUsdc,
      user1,
      user1Acdm,
      user1Usdc,
      member2,
      user2Usdc,
      user3Usdc,
    )).to.be.rejected;
  });

  it("starts trade round", async () => {
    await startTradeRound(ido, idoAcdm, idoAuthority, acdmMint);

    const idoAcdmAccount = await getAccount(connection, idoAcdm);
    expect(idoAcdmAccount.amount).to.eql(BigInt(0));
  });

  let order: PublicKey;
  let orderAcdm: PublicKey;
  let orderId: BN;

  it("adds order", async () => {
    [order, orderAcdm, orderId] = await addOrder(
      new BN(100),
      new BN(130_000),
      ido,
      acdmMint,
      user1,
      user1Acdm,
    );

    expect(orderId.toNumber()).to.eq(0);

    const user1AcdmAccount = await getAccount(connection, user1Acdm);
    expect(user1AcdmAccount.amount).to.eql(BigInt(400));

    const orderAcdmAccount = await getAccount(connection, orderAcdm);
    expect(orderAcdmAccount.amount).to.eql(BigInt(100));
  });

  it("redeems order partly", async () => {
    await redeemOrder(
      orderId,
      new BN(40),
      ido,
      idoUsdc,
      order,
      orderAcdm,
      user2,
      user2Acdm,
      user2Usdc,
      user1.publicKey,
      member1,
      user1Usdc,
      member2,
      user2Usdc,
      user3Usdc,
    );

    const user1UsdcAccount = await getAccount(connection, user1Usdc);
    expect(user1UsdcAccount.amount).to.eql(BigInt(54_940_000));

    const user2AcdmAccount = await getAccount(connection, user2Acdm);
    expect(user2AcdmAccount.amount).to.eql(BigInt(40));

    const orderAcdmAccount = await getAccount(connection, orderAcdm);
    expect(orderAcdmAccount.amount).to.eql(BigInt(60));
  });

  it("removes order", async () => {
    await removeOrder(orderId, order, orderAcdm, user1, user1Acdm);

    const user1AcdmAccount = await getAccount(connection, user1Acdm);
    expect(user1AcdmAccount.amount).to.eql(BigInt(460));
  });

  it("withdraws ido usdc", async () => {
    await withdrawIdoUsdc(ido, idoUsdc, idoAuthority, idoAuthorityUsdc);

    const idoUsdcAccount = await getAccount(connection, idoUsdc);
    expect(idoUsdcAccount.amount).to.eql(BigInt(0));

    const idoAuthorityUsdcAccount = await getAccount(
      connection,
      idoAuthorityUsdc,
    );
    expect(idoAuthorityUsdcAccount.amount).to.eql(BigInt(46_000_000));
  });

  it("ends ido", async () => {
    await endIdo(ido, idoAuthority);
  });
});
