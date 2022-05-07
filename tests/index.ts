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
import { burnAll, mintTo } from "./token";

chai.use(chaiAsPromised);

const ctx = new Context();

const INITIAL_ISSUE = 10_000;
const INITIAL_PRICE = 100_000;

describe("setup", () => {
  it("setups", async () => {
    await ctx.setup();
  });
});

describe("instructions", () => {
  it("initialize", async () => {
    const roundTime = 2;

    await initialize(ctx, roundTime);

    const ido = await ctx.program.account.ido.fetch(ctx.ido);
    expect(ido.bump).to.be.above(200);
    expect(ido.authority).to.eql(ctx.idoAuthority.publicKey);
    expect(ido.state).to.eql({ notStarted: {} });
    expect(ido.acdmMint).to.eql(ctx.acdmMint);
    expect(ido.usdcMint).to.eql(ctx.usdcMint);
    expect(ido.usdcTraded.toNumber()).to.eql(INITIAL_ISSUE * INITIAL_PRICE);
    expect(ido.roundTime).to.eql(roundTime);
    expect(ido.currentStateStartTs).to.not.eql(0);
  });

  it("registerMember", async () => {
    await registerMember(ctx, ctx.user3, null);

    const member3 = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user3.publicKey)
    );
    expect(member3.bump).to.be.above(200);
    expect(member3.referer).to.eql(null);

    await registerMember(ctx, ctx.user2, ctx.user3.publicKey);

    const member2 = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user2.publicKey)
    );
    expect(member2.bump).to.be.above(200);
    expect(member2.referer).to.eql(ctx.user3.publicKey);

    await registerMember(ctx, ctx.user1, ctx.user2.publicKey);

    const member1 = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member1.bump).to.be.above(200);
    expect(member1.referer).to.eql(ctx.user2.publicKey);
  });

  it("startSaleRound", async () => {
    await startSaleRound(ctx);

    const ido = await ctx.program.account.ido.fetch(ctx.ido);
    expect(ido.state).to.eql({ saleRound: {} });
    expect(ido.currentStateStartTs).to.not.eql(0);
    expect(ido.acdmPrice.toNumber()).to.eql(INITIAL_PRICE);
    expect(ido.saleRoundsStarted).to.eql(1);
    expect(await ctx.idoAcdm.amount(ctx)).to.eql(INITIAL_ISSUE);
  });

  let buyAmount = 500;

  it("buyAcdm", async () => {
    await expect(
      buyAcdm(ctx, new BN(9_000_000_000_000_000), ctx.user1)
    ).to.be.rejectedWith("Overflow");

    await mintTo(
      ctx,
      await ctx.usdcATA(ctx.user1.publicKey),
      ctx.usdcMintAuthority,
      buyAmount * INITIAL_PRICE
    );
    await buyAcdm(ctx, new BN(buyAmount), ctx.user1);
    expect(await (await ctx.acdmATA(ctx.user1.publicKey)).amount(ctx)).to.eql(
      buyAmount
    );
    expect(await (await ctx.usdcATA(ctx.user1.publicKey)).amount(ctx)).to.eql(
      0
    );
    expect(await (await ctx.usdcATA(ctx.user2.publicKey)).amount(ctx)).to.eql(
      (buyAmount * INITIAL_PRICE * 5) / 100
    );
    await burnAll(ctx, await ctx.usdcATA(ctx.user2.publicKey), ctx.user2);
    expect(await (await ctx.usdcATA(ctx.user3.publicKey)).amount(ctx)).to.eql(
      (buyAmount * INITIAL_PRICE * 3) / 100
    );
    await burnAll(ctx, await ctx.usdcATA(ctx.user3.publicKey), ctx.user3);
    expect(await ctx.idoUsdc.amount(ctx)).to.eql(
      (buyAmount * INITIAL_PRICE * 92) / 100
    );

    await mintTo(
      ctx,
      await ctx.usdcATA(ctx.user2.publicKey),
      ctx.usdcMintAuthority,
      buyAmount * INITIAL_PRICE
    );
    await buyAcdm(ctx, new BN(buyAmount), ctx.user2);
    expect(await (await ctx.acdmATA(ctx.user2.publicKey)).amount(ctx)).to.eql(
      buyAmount
    );
    await burnAll(ctx, await ctx.acdmATA(ctx.user2.publicKey), ctx.user2);
    expect(await (await ctx.usdcATA(ctx.user2.publicKey)).amount(ctx)).to.eql(
      0
    );
    expect(await (await ctx.usdcATA(ctx.user3.publicKey)).amount(ctx)).to.eql(
      (buyAmount * INITIAL_PRICE * 5) / 100
    );
    await burnAll(ctx, await ctx.usdcATA(ctx.user3.publicKey), ctx.user3);
    expect(await ctx.idoUsdc.amount(ctx)).to.eql(
      (buyAmount * INITIAL_PRICE * 92) / 100 +
        (buyAmount * INITIAL_PRICE * 95) / 100
    );

    await mintTo(
      ctx,
      await ctx.usdcATA(ctx.user3.publicKey),
      ctx.usdcMintAuthority,
      buyAmount * INITIAL_PRICE
    );
    await buyAcdm(ctx, new BN(buyAmount), ctx.user3);
    expect(await (await ctx.acdmATA(ctx.user3.publicKey)).amount(ctx)).to.eql(
      buyAmount
    );
    expect(await (await ctx.usdcATA(ctx.user3.publicKey)).amount(ctx)).to.eql(
      0
    );
    expect(await ctx.idoUsdc.amount(ctx)).to.eql(
      (buyAmount * INITIAL_PRICE * 92) / 100 +
        (buyAmount * INITIAL_PRICE * 95) / 100 +
        buyAmount * INITIAL_PRICE
    );
  });

  it("startTradeRound", async () => {
    await startTradeRound(ctx);

    const ido = await ctx.program.account.ido.fetch(ctx.ido);
    expect(ido.state).to.eql({ tradeRound: {} });
    expect(ido.currentStateStartTs).to.not.eql(0);
    expect(ido.usdcTraded.toNumber()).to.eql(0);

    expect(await ctx.idoAcdm.amount(ctx)).to.eql(0);
  });

  let orderId: BN;
  let orderAmount = 100;
  let orderPrice = 130_000;

  it("addOrder", async () => {
    orderId = await addOrder(
      ctx,
      new BN(orderAmount),
      new BN(orderPrice),
      ctx.user1
    );

    expect(orderId.toNumber()).to.eql(0);
    expect(await (await ctx.acdmATA(ctx.user1.publicKey)).amount(ctx)).to.eql(
      400
    );
    expect(await (await ctx.orderAcdm(orderId)).amount(ctx)).to.eql(100);

    const order = await ctx.program.account.order.fetch(
      await ctx.order(orderId)
    );
    expect(order.bump).to.be.above(200);
    expect(order.authority).to.eql(ctx.user1.publicKey);
    expect(order.price.toNumber()).to.eql(orderPrice);

    const ido = await ctx.program.account.ido.fetch(ctx.ido);
    expect(ido.orders.toNumber()).to.eql(1);
  });

  const redeemAmount = 40;

  it("redeemOrder", async () => {
    await mintTo(
      ctx,
      await ctx.usdcATA(ctx.user2.publicKey),
      ctx.usdcMintAuthority,
      redeemAmount * orderPrice
    );
    await redeemOrder(ctx, orderId, new BN(redeemAmount), ctx.user2);

    expect(await (await ctx.acdmATA(ctx.user2.publicKey)).amount(ctx)).to.eql(
      redeemAmount
    );
    expect(await (await ctx.usdcATA(ctx.user1.publicKey)).amount(ctx)).to.eql(
      (redeemAmount * orderPrice * 95) / 100
    );
    expect(await (await ctx.usdcATA(ctx.user2.publicKey)).amount(ctx)).to.eql(
      (redeemAmount * orderPrice * 25) / 1000
    );
    expect(await (await ctx.usdcATA(ctx.user3.publicKey)).amount(ctx)).to.eql(
      (redeemAmount * orderPrice * 25) / 1000
    );
    expect(await (await ctx.orderAcdm(orderId)).amount(ctx)).to.eql(
      orderAmount - redeemAmount
    );

    const ido = await ctx.program.account.ido.fetch(ctx.ido);
    expect(ido.usdcTraded.toNumber()).to.eql(redeemAmount * orderPrice);
  });

  it("removeOrder", async () => {
    await removeOrder(ctx, orderId, ctx.user1);

    expect(await (await ctx.acdmATA(ctx.user1.publicKey)).amount(ctx)).to.eql(
      buyAmount - redeemAmount
    );
  });

  it("withdrawIdoUsdc", async () => {
    await withdrawIdoUsdc(ctx);

    expect(await ctx.idoUsdc.amount(ctx)).to.eql(0);
    expect(
      await (await ctx.usdcATA(ctx.idoAuthority.publicKey)).amount(ctx)
    ).to.not.eql(0);
  });

  it("endIdo", async () => {
    await endIdo(ctx);

    const ido = await ctx.program.account.ido.fetch(ctx.ido);
    expect(ido.state).to.eql({ over: {} });
  });
});
