import { BN } from "@project-serum/anchor";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { Context } from "./ctx";

export async function initialize(
  ctx: Context,
  roundTime: number,
): Promise<void> {
  await ctx.program.methods.initialize(roundTime)
    .accounts({
      ido: await ctx.ido(),
      idoAuthority: ctx.idoAuthority.publicKey,
      acdmMint: ctx.acdmMint,
      idoAcdm: await ctx.idoAcdm(),
      usdcMint: ctx.usdcMint,
      idoUsdc: await ctx.idoUsdc(),
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
    }).signers([ctx.idoAuthority]).rpc();
}

export async function registerMember(
  ctx: Context,
  user: Keypair,
  referer: PublicKey,
): Promise<PublicKey> {
  const member = await ctx.member(user.publicKey);
  const remainingAccounts = [];

  if (referer != null) {
    const refererMember = await ctx.member(referer);

    remainingAccounts.push({
      pubkey: refererMember,
      isWritable: false,
      isSigner: false,
    });
  }

  await ctx.program.methods.registerMember(referer).accounts({
    member: member,
    authority: user.publicKey,
    systemProgram: SystemProgram.programId,
  }).remainingAccounts(remainingAccounts).signers([user]).rpc();

  return member;
}

export async function startSaleRound(ctx: Context): Promise<void> {
  await ctx.program.methods.startSaleRound().accounts({
    ido: await ctx.ido(),
    idoAuthority: ctx.idoAuthority.publicKey,
    acdmMintAuthority: ctx.acdmMintAuthority.publicKey,
    acdmMint: ctx.acdmMint,
    idoAcdm: await ctx.idoAcdm(),
    tokenProgram: TOKEN_PROGRAM_ID,
  }).signers([ctx.idoAuthority, ctx.acdmMintAuthority]).rpc();
}

export async function buyAcdm(
  ctx: Context,
  amount: BN,
  buyer: Keypair,
): Promise<void> {
  const buyerMember = await ctx.member(buyer.publicKey);
  const buyerAcdm = await ctx.ata(buyer.publicKey, ctx.acdmMint);
  const buyerUsdc = await ctx.ata(buyer.publicKey, ctx.usdcMint);

  const remainingAccounts = [];

  const referer = (await ctx.program.account.member.fetch(buyerMember)).referer;

  if (referer) {
    const refererMember = await ctx.member(referer);
    const refererUsdc = await ctx.ata(referer, ctx.usdcMint);

    remainingAccounts.push({
      pubkey: refererMember,
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: refererUsdc,
      isWritable: true,
      isSigner: false,
    });

    const referer2 =
      (await ctx.program.account.member.fetch(refererMember)).referer;

    if (referer2) {
      const referer2Usdc = await ctx.ata(referer2, ctx.usdcMint);

      remainingAccounts.push({
        pubkey: referer2Usdc,
        isWritable: true,
        isSigner: false,
      });
    }
  }

  await ctx.program.methods.buyAcdm(amount).accounts({
    ido: await ctx.ido(),
    idoAcdm: await ctx.idoAcdm(),
    idoUsdc: await ctx.idoUsdc(),
    buyer: buyer.publicKey,
    buyerMember,
    buyerAcdm,
    buyerUsdc,
    tokenProgram: TOKEN_PROGRAM_ID,
  }).remainingAccounts(remainingAccounts).signers([buyer]).rpc();
}

export async function startTradeRound(ctx: Context): Promise<void> {
  await ctx.program.methods.startTradeRound().accounts({
    ido: await ctx.ido(),
    idoAuthority: ctx.idoAuthority.publicKey,
    acdmMint: ctx.acdmMint,
    idoAcdm: await ctx.idoAcdm(),
    tokenProgram: TOKEN_PROGRAM_ID,
  }).signers([ctx.idoAuthority]).rpc();
}

export async function addOrder(
  ctx: Context,
  amount: BN,
  price: BN,
  seller: Keypair,
): Promise<BN> {
  const orderId = (await ctx.program.account.ido.fetch(await ctx.ido())).orders;

  const order = await ctx.order(orderId);
  const orderAcdm = await ctx.orderAcdm(orderId);
  const sellerAcdm = await ctx.ata(seller.publicKey, ctx.acdmMint);

  const ido = await ctx.ido();

  let listener: number;
  const [event, _] = await new Promise((resolve, _reject) => {
    listener = ctx.program.addEventListener("AddOrderEvent", (event, slot) => {
      resolve([event, slot]);
    });
    ctx.program.methods.addOrder(amount, price).accounts({
      ido,
      order,
      acdmMint: ctx.acdmMint,
      orderAcdm,
      seller: seller.publicKey,
      sellerAcdm,
      rent: SYSVAR_RENT_PUBKEY,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    }).signers([seller]).rpc();
  });
  await ctx.program.removeEventListener(listener);

  return event.id;
}

export async function redeemOrder(
  ctx: Context,
  orderId: BN,
  amount: BN,
  buyer: Keypair,
): Promise<void> {
  const order = await ctx.order(orderId);
  const orderAcdm = await ctx.orderAcdm(orderId);
  const buyerAcdm = await ctx.ata(buyer.publicKey, ctx.acdmMint);
  const buyerUsdc = await ctx.ata(buyer.publicKey, ctx.usdcMint);

  const seller = (await ctx.program.account.order.fetch(order)).authority;

  const sellerUsdc = await ctx.ata(seller, ctx.usdcMint);
  const sellerMember = await ctx.member(seller);

  const remainingAccounts = [];

  const referer =
    (await ctx.program.account.member.fetch(sellerMember)).referer;

  if (referer) {
    const refererMember = await ctx.member(referer);
    const refererUsdc = await ctx.ata(referer, ctx.usdcMint);

    remainingAccounts.push({
      pubkey: refererMember,
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: refererUsdc,
      isWritable: true,
      isSigner: false,
    });

    const referer2 =
      (await ctx.program.account.member.fetch(refererMember)).referer;

    if (referer2) {
      const referer2Usdc = await ctx.ata(referer2, ctx.usdcMint);

      remainingAccounts.push({
        pubkey: referer2Usdc,
        isWritable: true,
        isSigner: false,
      });
    }
  }

  await ctx.program.methods.redeemOrder(orderId, amount).accounts({
    ido: await ctx.ido(),
    idoUsdc: await ctx.idoUsdc(),
    order,
    orderAcdm,
    buyer: buyer.publicKey,
    buyerAcdm,
    buyerUsdc,
    seller,
    sellerMember,
    sellerUsdc,
    tokenProgram: TOKEN_PROGRAM_ID,
  }).remainingAccounts(remainingAccounts).signers([buyer]).rpc();
}

export async function removeOrder(
  ctx: Context,
  orderId: BN,
  seller: Keypair,
): Promise<void> {
  const order = await ctx.order(orderId);
  const orderAcdm = await ctx.orderAcdm(orderId);
  const sellerAcdm = await ctx.ata(seller.publicKey, ctx.acdmMint);

  await ctx.program.methods.removeOrder(orderId).accounts({
    order,
    orderAcdm,
    seller: seller.publicKey,
    sellerAcdm,
    tokenProgram: TOKEN_PROGRAM_ID,
  }).signers([seller]).rpc();
}

export async function withdrawIdoUsdc(
  ctx: Context,
): Promise<void> {
  await ctx.program.methods.withdrawIdoUsdc().accounts({
    ido: await ctx.ido(),
    idoAuthority: ctx.idoAuthority.publicKey,
    idoUsdc: await ctx.idoUsdc(),
    to: await ctx.ata(ctx.idoAuthority.publicKey, ctx.usdcMint),
    tokenProgram: TOKEN_PROGRAM_ID,
  }).signers([ctx.idoAuthority]).rpc();
}

export async function endIdo(ctx: Context): Promise<void> {
  await ctx.program.methods.endIdo().accounts({
    ido: await ctx.ido(),
    idoAuthority: ctx.idoAuthority.publicKey,
  }).signers([ctx.idoAuthority]).rpc();
}
