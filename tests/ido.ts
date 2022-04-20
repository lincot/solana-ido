import { BN } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { createMint, getAccount, mintTo } from "@solana/spl-token";
import { expect } from "chai";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { findATA } from "./utils";
import { Context } from "./ctx";
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
  const ctx = new Context();

  ctx.connection = new Connection("http://localhost:8899", "recent");

  ctx.payer = new Keypair();
  ctx.acdmMintAuthority = new Keypair();
  ctx.usdcMintAuthority = new Keypair();
  ctx.idoAuthority = new Keypair();
  const user1 = new Keypair();
  const user2 = new Keypair();
  const user3 = new Keypair();

  it("airdrops", async () => {
    await Promise.all(
      await Promise.all(
        [
          ctx.payer,
          ctx.idoAuthority,
          user1,
          user2,
          user3,
        ]
          .map(
            async (k) =>
              ctx.connection.confirmTransaction(
                await ctx.connection.requestAirdrop(
                  k.publicKey,
                  100_000_000,
                ),
              ),
          ),
      ),
    );
  });

  it("creates mints", async () => {
    ctx.acdmMint = await createMint(
      ctx.connection,
      ctx.payer,
      ctx.acdmMintAuthority.publicKey,
      undefined,
      2,
    );
    // fake usdc
    ctx.usdcMint = await createMint(
      ctx.connection,
      ctx.payer,
      ctx.usdcMintAuthority.publicKey,
      undefined,
      6,
    );
  });

  it("initializes", async () => {
    await initialize(
      ctx,
      2,
    );
  });

  it("fails to end ido right away", async () => {
    await expect(endIdo(ctx)).to.be.rejected;
  });

  it("starts sale round", async () => {
    await startSaleRound(ctx);
  });

  let user1Acdm: PublicKey;
  let user1Usdc: PublicKey;
  let user2Acdm: PublicKey;
  let user2Usdc: PublicKey;
  let user3Usdc: PublicKey;

  it("sets users' ATAs", async () => {
    [
      user1Acdm,
      user1Usdc,
      user2Acdm,
      user2Usdc,
      user3Usdc,
      ctx.idoAuthorityUsdc,
    ] = await Promise.all([
      findATA(ctx, user1.publicKey, ctx.acdmMint),
      findATA(ctx, user1.publicKey, ctx.usdcMint),
      findATA(ctx, user2.publicKey, ctx.acdmMint),
      findATA(ctx, user2.publicKey, ctx.usdcMint),
      findATA(ctx, user3.publicKey, ctx.usdcMint),
      findATA(ctx, ctx.idoAuthority.publicKey, ctx.usdcMint),
    ]);

    await Promise.all([
      mintTo(
        ctx.connection,
        user1,
        ctx.usdcMint,
        await findATA(ctx, user1.publicKey, ctx.usdcMint),
        ctx.usdcMintAuthority,
        100_000_000,
      ),
      mintTo(
        ctx.connection,
        user2,
        ctx.usdcMint,
        await findATA(ctx, user2.publicKey, ctx.usdcMint),
        ctx.usdcMintAuthority,
        100_000_000,
      ),
    ]);
  });

  it("registers users", async () => {
    await registerMember(ctx, user3, null);
    await registerMember(ctx, user2, user3.publicKey);
    await registerMember(ctx, user1, user2.publicKey);
  });

  it("buys ACDM", async () => {
    await buyAcdm(
      ctx,
      new BN(500),
      user1,
    );

    const user1AcdmAccount = await getAccount(ctx.connection, user1Acdm);
    expect(user1AcdmAccount.amount).to.eql(BigInt(500));

    const user1UsdcAccount = await getAccount(ctx.connection, user1Usdc);
    expect(user1UsdcAccount.amount).to.eql(BigInt(50_000_000));

    const user2UsdcAccount = await getAccount(ctx.connection, user2Usdc);
    expect(user2UsdcAccount.amount).to.eql(BigInt(102_500_000));

    const user3UsdcAccount = await getAccount(ctx.connection, user3Usdc);
    expect(user3UsdcAccount.amount).to.eql(BigInt(1_500_000));

    const idoUsdcAccount = await getAccount(ctx.connection, ctx.idoUsdc);
    expect(idoUsdcAccount.amount).to.eql(BigInt(46_000_000));
  });

  it("fails to buy too much", async () => {
    await expect(
      buyAcdm(
        ctx,
        new BN(9_000_000_000_000_000),
        user1,
      ),
    ).to.be.rejected;
  });

  it("starts trade round", async () => {
    await startTradeRound(ctx);

    const idoAcdmAccount = await getAccount(ctx.connection, ctx.idoAcdm);
    expect(idoAcdmAccount.amount).to.eql(BigInt(0));
  });

  it("adds order", async () => {
    [ctx.orderId, ctx.order, ctx.orderAcdm] = await addOrder(
      ctx,
      new BN(100),
      new BN(130_000),
      user1,
    );

    expect(ctx.orderId.toNumber()).to.eq(0);

    const user1AcdmAccount = await getAccount(ctx.connection, user1Acdm);
    expect(user1AcdmAccount.amount).to.eql(BigInt(400));

    const orderAcdmAccount = await getAccount(ctx.connection, ctx.orderAcdm);
    expect(orderAcdmAccount.amount).to.eql(BigInt(100));
  });

  it("redeems order partly", async () => {
    await redeemOrder(
      ctx,
      ctx.orderId,
      new BN(40),
      user2,
    );

    const user1UsdcAccount = await getAccount(ctx.connection, user1Usdc);
    expect(user1UsdcAccount.amount).to.eql(BigInt(54_940_000));

    const user2AcdmAccount = await getAccount(ctx.connection, user2Acdm);
    expect(user2AcdmAccount.amount).to.eql(BigInt(40));

    const orderAcdmAccount = await getAccount(ctx.connection, ctx.orderAcdm);
    expect(orderAcdmAccount.amount).to.eql(BigInt(60));
  });

  it("removes order", async () => {
    await removeOrder(ctx, ctx.orderId, user1);

    const user1AcdmAccount = await getAccount(ctx.connection, user1Acdm);
    expect(user1AcdmAccount.amount).to.eql(BigInt(460));
  });

  it("withdraws ido usdc", async () => {
    await withdrawIdoUsdc(ctx);

    const idoUsdcAccount = await getAccount(ctx.connection, ctx.idoUsdc);
    expect(idoUsdcAccount.amount).to.eql(BigInt(0));

    const idoAuthorityUsdcAccount = await getAccount(
      ctx.connection,
      ctx.idoAuthorityUsdc,
    );
    expect(idoAuthorityUsdcAccount.amount).to.eql(BigInt(46_000_000));
  });

  it("ends ido", async () => {
    await endIdo(ctx);
  });
});
