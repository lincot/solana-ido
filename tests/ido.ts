import { BN } from "@project-serum/anchor";
import { expect } from "chai";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
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

  it("setups state of the world", async () => {
    await ctx.setup();
  });

  it("initializes", async () => {
    await initialize(ctx, 2);
  });

  it("fails to end ido right away", async () => {
    await expect(endIdo(ctx)).to.be.rejected;
  });

  it("starts sale round", async () => {
    await startSaleRound(ctx);
  });

  it("registers users", async () => {
    await registerMember(ctx, ctx.user3, null);
    await registerMember(ctx, ctx.user2, ctx.user3.publicKey);
    await registerMember(ctx, ctx.user1, ctx.user2.publicKey);
  });

  it("buys ACDM", async () => {
    await buyAcdm(ctx, new BN(500), ctx.user1);

    expect(
      await (await ctx.ata(ctx.user1.publicKey, ctx.acdmMint)).amount(ctx)
    ).to.eql(BigInt(500));
    expect(
      await (await ctx.ata(ctx.user1.publicKey, ctx.acdmMint)).amount(ctx)
    ).to.eql(BigInt(500));
    expect(
      await (await ctx.ata(ctx.user1.publicKey, ctx.usdcMint)).amount(ctx)
    ).to.eql(BigInt(50_000_000));
    expect(
      await (await ctx.ata(ctx.user2.publicKey, ctx.usdcMint)).amount(ctx)
    ).to.eql(BigInt(102_500_000));
    expect(
      await (await ctx.ata(ctx.user3.publicKey, ctx.usdcMint)).amount(ctx)
    ).to.eql(BigInt(1_500_000));
    expect(await (await ctx.idoUsdc()).amount(ctx)).to.eql(BigInt(46_000_000));
  });

  it("fails to buy too much", async () => {
    await expect(buyAcdm(ctx, new BN(9_000_000_000_000_000), ctx.user1)).to.be
      .rejected;
  });

  it("starts trade round", async () => {
    await startTradeRound(ctx);

    expect(await (await ctx.idoAcdm()).amount(ctx)).to.eql(BigInt(0));
  });

  let orderId: BN;

  it("adds order", async () => {
    orderId = await addOrder(ctx, new BN(100), new BN(130_000), ctx.user1);

    expect(orderId.toNumber()).to.eq(0);
    expect(
      await (await ctx.ata(ctx.user1.publicKey, ctx.acdmMint)).amount(ctx)
    ).to.eql(BigInt(400));
    expect(await (await ctx.orderAcdm(orderId)).amount(ctx)).to.eql(
      BigInt(100)
    );
  });

  it("redeems order partly", async () => {
    await redeemOrder(ctx, orderId, new BN(40), ctx.user2);

    expect(
      await (await ctx.ata(ctx.user1.publicKey, ctx.usdcMint)).amount(ctx)
    ).to.eql(BigInt(54_940_000));
    expect(
      await (await ctx.ata(ctx.user2.publicKey, ctx.acdmMint)).amount(ctx)
    ).to.eql(BigInt(40));
    expect(await (await ctx.orderAcdm(orderId)).amount(ctx)).to.eql(BigInt(60));
  });

  it("removes order", async () => {
    await removeOrder(ctx, orderId, ctx.user1);

    expect(
      await (await ctx.ata(ctx.user1.publicKey, ctx.acdmMint)).amount(ctx)
    ).to.eql(BigInt(460));
  });

  it("withdraws ido usdc", async () => {
    await withdrawIdoUsdc(ctx);

    expect(await (await ctx.idoUsdc()).amount(ctx)).to.eql(BigInt(0));
    expect(
      await (
        await ctx.ata(ctx.idoAuthority.publicKey, ctx.usdcMint)
      ).amount(ctx)
    ).to.eql(BigInt(46_000_000));
  });

  it("ends ido", async () => {
    await endIdo(ctx);
  });
});
