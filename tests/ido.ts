import * as anchor from "@project-serum/anchor";
import { BN } from "@project-serum/anchor";
import { Connection, Keypair } from "@solana/web3.js";
import { createMint, getAccount, mintTo } from "@solana/spl-token";
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

  ctx.program = anchor.workspace.Ido;
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

  it("mints", async () => {
    await Promise.all([
      mintTo(
        ctx.connection,
        user1,
        ctx.usdcMint,
        await ctx.ata(user1.publicKey, ctx.usdcMint),
        ctx.usdcMintAuthority,
        100_000_000,
      ),
      mintTo(
        ctx.connection,
        user2,
        ctx.usdcMint,
        await ctx.ata(user2.publicKey, ctx.usdcMint),
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

    const user1AcdmAccount = await getAccount(
      ctx.connection,
      await ctx.ata(user1.publicKey, ctx.acdmMint),
    );
    expect(user1AcdmAccount.amount).to.eql(BigInt(500));

    const user1UsdcAccount = await getAccount(
      ctx.connection,
      await ctx.ata(user1.publicKey, ctx.usdcMint),
    );
    expect(user1UsdcAccount.amount).to.eql(BigInt(50_000_000));

    const user2UsdcAccount = await getAccount(
      ctx.connection,
      await ctx.ata(user2.publicKey, ctx.usdcMint),
    );
    expect(user2UsdcAccount.amount).to.eql(BigInt(102_500_000));

    const user3UsdcAccount = await getAccount(
      ctx.connection,
      await ctx.ata(user3.publicKey, ctx.usdcMint),
    );
    expect(user3UsdcAccount.amount).to.eql(BigInt(1_500_000));

    const idoUsdcAccount = await getAccount(
      ctx.connection,
      await ctx.idoUsdc(),
    );
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

    const idoAcdmAccount = await getAccount(
      ctx.connection,
      await ctx.idoAcdm(),
    );
    expect(idoAcdmAccount.amount).to.eql(BigInt(0));
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

    const user1AcdmAccount = await getAccount(
      ctx.connection,
      await ctx.ata(user1.publicKey, ctx.acdmMint),
    );
    expect(user1AcdmAccount.amount).to.eql(BigInt(400));

    const orderAcdmAccount = await getAccount(
      ctx.connection,
      await ctx.orderAcdm(orderId),
    );
    expect(orderAcdmAccount.amount).to.eql(BigInt(100));
  });

  it("redeems order partly", async () => {
    await redeemOrder(
      ctx,
      orderId,
      new BN(40),
      user2,
    );

    const user1UsdcAccount = await getAccount(
      ctx.connection,
      await ctx.ata(user1.publicKey, ctx.usdcMint),
    );
    expect(user1UsdcAccount.amount).to.eql(BigInt(54_940_000));

    const user2AcdmAccount = await getAccount(
      ctx.connection,
      await ctx.ata(user2.publicKey, ctx.acdmMint),
    );
    expect(user2AcdmAccount.amount).to.eql(BigInt(40));

    const orderAcdmAccount = await getAccount(
      ctx.connection,
      await ctx.orderAcdm(orderId),
    );
    expect(orderAcdmAccount.amount).to.eql(BigInt(60));
  });

  it("removes order", async () => {
    await removeOrder(ctx, orderId, user1);

    const user1AcdmAccount = await getAccount(
      ctx.connection,
      await ctx.ata(user1.publicKey, ctx.acdmMint),
    );
    expect(user1AcdmAccount.amount).to.eql(BigInt(460));
  });

  it("withdraws ido usdc", async () => {
    await withdrawIdoUsdc(ctx);

    const idoUsdcAccount = await getAccount(
      ctx.connection,
      await ctx.idoUsdc(),
    );
    expect(idoUsdcAccount.amount).to.eql(BigInt(0));

    const idoAuthorityUsdcAccount = await getAccount(
      ctx.connection,
      await ctx.ata(ctx.idoAuthority.publicKey, ctx.usdcMint),
    );
    expect(idoAuthorityUsdcAccount.amount).to.eql(BigInt(46_000_000));
  });

  it("ends ido", async () => {
    await endIdo(ctx);
  });
});
