import * as anchor from "@project-serum/anchor";
import { BN } from "@project-serum/anchor";
import { Connection, Keypair } from "@solana/web3.js";
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
import { airdrop } from "./utils";
import { createMint, mintTo } from "./token";

chai.use(chaiAsPromised);

describe("ido", () => {
  const ctx = new Context();

  ctx.program = anchor.workspace.Ido;
  ctx.connection = new Connection("http://localhost:8899", "recent");

  ctx.payer = new Keypair();
  ctx.acdmMintAuthority = new Keypair();
  ctx.usdcMintAuthority = new Keypair();
  ctx.idoAuthority = new Keypair();
  const user1 = new Keypair();
  const user2 = new Keypair();
  const user3 = new Keypair();

  it("create state of the world", async () => {
    await airdrop(ctx, [
      ctx.idoAuthority.publicKey,
      user1.publicKey,
      user2.publicKey,
      user3.publicKey,
    ]);

    ctx.acdmMint = await createMint(
      ctx,
      ctx.acdmMintAuthority,
      2,
    );
    // fake usdc
    ctx.usdcMint = await createMint(
      ctx,
      ctx.usdcMintAuthority,
      6,
    );

    await mintTo(
      ctx,
      ctx.usdcMint,
      user1.publicKey,
      ctx.usdcMintAuthority,
      100_000_000,
    );
    await mintTo(
      ctx,
      ctx.usdcMint,
      user2.publicKey,
      ctx.usdcMintAuthority,
      100_000_000,
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

    await (await ctx.ata(user1.publicKey, ctx.acdmMint)).checkAmount(ctx, 500);
    await (await ctx.ata(user1.publicKey, ctx.usdcMint)).checkAmount(
      ctx,
      50_000_000,
    );
    await (await ctx.ata(user2.publicKey, ctx.usdcMint)).checkAmount(
      ctx,
      102_500_000,
    );
    await (await ctx.ata(user3.publicKey, ctx.usdcMint)).checkAmount(
      ctx,
      1_500_000,
    );
    await (await ctx.idoUsdc()).checkAmount(ctx, 46_000_000);
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

    await (await ctx.idoAcdm()).checkAmount(ctx, 0);
  });

  let orderId: BN;

  it("adds order", async () => {
    orderId = await addOrder(
      ctx,
      new BN(100),
      new BN(130_000),
      user1,
    );

    expect(orderId.toNumber()).to.eq(0);
    await (await ctx.ata(user1.publicKey, ctx.acdmMint)).checkAmount(ctx, 400);
    await (await ctx.orderAcdm(orderId)).checkAmount(ctx, 100);
  });

  it("redeems order partly", async () => {
    await redeemOrder(
      ctx,
      orderId,
      new BN(40),
      user2,
    );

    await (await ctx.ata(user1.publicKey, ctx.usdcMint)).checkAmount(
      ctx,
      54_940_000,
    );
    await (await ctx.ata(user2.publicKey, ctx.acdmMint)).checkAmount(ctx, 40);
    await (await ctx.orderAcdm(orderId)).checkAmount(ctx, 60);
  });

  it("removes order", async () => {
    await removeOrder(ctx, orderId, user1);

    await (await ctx.ata(user1.publicKey, ctx.acdmMint)).checkAmount(ctx, 460);
  });

  it("withdraws ido usdc", async () => {
    await withdrawIdoUsdc(ctx);

    await (await ctx.idoUsdc()).checkAmount(ctx, 0);
    await (await ctx.ata(ctx.idoAuthority.publicKey, ctx.usdcMint)).checkAmount(
      ctx,
      46_000_000,
    );
  });

  it("ends ido", async () => {
    await endIdo(ctx);
  });
});
