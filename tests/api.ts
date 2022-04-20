import * as anchor from "@project-serum/anchor";
import { BN, Program } from "@project-serum/anchor";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { Ido } from "../target/types/ido";
import { Context } from "./ctx";
import { findATA } from "./utils";

const idoProgram = anchor.workspace.Ido as Program<
  Ido
>;

export async function initialize(
  ctx: Context,
  roundTime: number,
): Promise<void> {
  const [ido] = await PublicKey
    .findProgramAddress(
      [Buffer.from("ido")],
      idoProgram.programId,
    );
  const [idoAcdm] = await PublicKey.findProgramAddress(
    [
      Buffer.from("ido_acdm"),
    ],
    idoProgram.programId,
  );
  const [idoUsdc] = await PublicKey.findProgramAddress(
    [
      Buffer.from("ido_usdc"),
    ],
    idoProgram.programId,
  );

  ctx.ido = ido;
  ctx.idoAcdm = idoAcdm;
  ctx.idoUsdc = idoUsdc;

  await idoProgram.methods.initialize(roundTime)
    .accounts({
      ido,
      idoAuthority: ctx.idoAuthority.publicKey,
      acdmMint: ctx.acdmMint,
      idoAcdm,
      usdcMint: ctx.usdcMint,
      idoUsdc,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
    }).signers([ctx.idoAuthority]).rpc();
}

export async function registerMember(
  _ctx: Context,
  user: Keypair,
  referer: PublicKey,
): Promise<PublicKey> {
  const [member] = await PublicKey
    .findProgramAddress(
      [Buffer.from("member"), user.publicKey.toBuffer()],
      idoProgram.programId,
    );

  const remainingAccounts = [];

  if (referer != null) {
    const [refererMember] = await PublicKey
      .findProgramAddress(
        [Buffer.from("member"), referer.toBuffer()],
        idoProgram.programId,
      );

    remainingAccounts.push({
      pubkey: refererMember,
      isWritable: false,
      isSigner: false,
    });
  }

  await idoProgram.methods.registerMember(referer).accounts({
    member: member,
    authority: user.publicKey,
    systemProgram: SystemProgram.programId,
  }).remainingAccounts(remainingAccounts).signers([user]).rpc();

  return member;
}

export async function startSaleRound(ctx: Context): Promise<void> {
  await idoProgram.methods.startSaleRound().accounts({
    ido: ctx.ido,
    idoAuthority: ctx.idoAuthority.publicKey,
    acdmMintAuthority: ctx.acdmMintAuthority.publicKey,
    acdmMint: ctx.acdmMint,
    idoAcdm: ctx.idoAcdm,
    tokenProgram: TOKEN_PROGRAM_ID,
  }).signers([ctx.idoAuthority, ctx.acdmMintAuthority]).rpc();
}

export async function buyAcdm(
  ctx: Context,
  amount: BN,
  buyer: Keypair,
): Promise<void> {
  const [buyerMember] = await PublicKey
    .findProgramAddress(
      [Buffer.from("member"), buyer.publicKey.toBuffer()],
      idoProgram.programId,
    );
  const buyerAcdm = await findATA(ctx, buyer.publicKey, ctx.acdmMint);
  const buyerUsdc = await findATA(ctx, buyer.publicKey, ctx.usdcMint);

  const remainingAccounts = [];

  const referer = (await idoProgram.account.member.fetch(buyerMember)).referer;

  if (referer) {
    const [refererMember] = await PublicKey
      .findProgramAddress(
        [Buffer.from("member"), referer.toBuffer()],
        idoProgram.programId,
      );
    const refererUsdc = await findATA(ctx, referer, ctx.usdcMint);

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
      (await idoProgram.account.member.fetch(refererMember)).referer;

    if (referer2) {
      const referer2Usdc = await findATA(ctx, referer2, ctx.usdcMint);

      remainingAccounts.push({
        pubkey: referer2Usdc,
        isWritable: true,
        isSigner: false,
      });
    }
  }

  await idoProgram.methods.buyAcdm(amount).accounts({
    ido: ctx.ido,
    idoAcdm: ctx.idoAcdm,
    idoUsdc: ctx.idoUsdc,
    buyer: buyer.publicKey,
    buyerMember,
    buyerAcdm,
    buyerUsdc,
    tokenProgram: TOKEN_PROGRAM_ID,
  }).remainingAccounts(remainingAccounts).signers([buyer]).rpc();
}

export async function startTradeRound(ctx: Context): Promise<void> {
  await idoProgram.methods.startTradeRound().accounts({
    ido: ctx.ido,
    idoAuthority: ctx.idoAuthority.publicKey,
    acdmMint: ctx.acdmMint,
    idoAcdm: ctx.idoAcdm,
    tokenProgram: TOKEN_PROGRAM_ID,
  }).signers([ctx.idoAuthority]).rpc();
}

export async function addOrder(
  ctx: Context,
  amount: BN,
  price: BN,
  seller: Keypair,
): Promise<[BN, PublicKey, PublicKey]> {
  const ordersCount = (await idoProgram.account.ido.fetch(ctx.ido)).orders;

  const [order] = await PublicKey
    .findProgramAddress(
      [Buffer.from("order"), ordersCount.toArrayLike(Buffer, "le", 8)],
      idoProgram.programId,
    );
  const [orderAcdm] = await PublicKey
    .findProgramAddress(
      [Buffer.from("order_acdm"), ordersCount.toArrayLike(Buffer, "le", 8)],
      idoProgram.programId,
    );
  const sellerAcdm = await findATA(ctx, seller.publicKey, ctx.acdmMint);

  let listener: number;
  const [event, _] = await new Promise((resolve, _reject) => {
    listener = idoProgram.addEventListener("OrderEvent", (event, slot) => {
      resolve([event, slot]);
    });
    idoProgram.methods.addOrder(amount, price).accounts({
      ido: ctx.ido,
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
  await idoProgram.removeEventListener(listener);

  return [event.id, order, orderAcdm];
}

export async function redeemOrder(
  ctx: Context,
  orderId: BN,
  amount: BN,
  buyer: Keypair,
): Promise<void> {
  const [order] = await PublicKey
    .findProgramAddress(
      [Buffer.from("order"), orderId.toArrayLike(Buffer, "le", 8)],
      idoProgram.programId,
    );
  const [orderAcdm] = await PublicKey
    .findProgramAddress(
      [Buffer.from("order_acdm"), orderId.toArrayLike(Buffer, "le", 8)],
      idoProgram.programId,
    );
  const buyerAcdm = await findATA(ctx, buyer.publicKey, ctx.acdmMint);
  const buyerUsdc = await findATA(ctx, buyer.publicKey, ctx.usdcMint);

  const seller = (await idoProgram.account.order.fetch(order)).authority;

  const sellerUsdc = await findATA(ctx, seller, ctx.usdcMint);
  const [sellerMember] = await PublicKey
    .findProgramAddress(
      [Buffer.from("member"), seller.toBuffer()],
      idoProgram.programId,
    );

  const remainingAccounts = [];

  const referer = (await idoProgram.account.member.fetch(sellerMember)).referer;

  if (referer) {
    const [refererMember] = await PublicKey
      .findProgramAddress(
        [Buffer.from("member"), referer.toBuffer()],
        idoProgram.programId,
      );
    const refererUsdc = await findATA(ctx, referer, ctx.usdcMint);

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
      (await idoProgram.account.member.fetch(refererMember)).referer;

    if (referer2) {
      const referer2Usdc = await findATA(ctx, referer2, ctx.usdcMint);

      remainingAccounts.push({
        pubkey: referer2Usdc,
        isWritable: true,
        isSigner: false,
      });
    }
  }

  await idoProgram.methods.redeemOrder(orderId, amount).accounts({
    ido: ctx.ido,
    idoUsdc: ctx.idoUsdc,
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
  const [order] = await PublicKey
    .findProgramAddress(
      [Buffer.from("order"), orderId.toArrayLike(Buffer, "le", 8)],
      idoProgram.programId,
    );
  const [orderAcdm] = await PublicKey
    .findProgramAddress(
      [Buffer.from("order_acdm"), orderId.toArrayLike(Buffer, "le", 8)],
      idoProgram.programId,
    );
  const sellerAcdm = await findATA(ctx, seller.publicKey, ctx.acdmMint);

  await idoProgram.methods.removeOrder(orderId).accounts({
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
  await idoProgram.methods.withdrawIdoUsdc().accounts({
    ido: ctx.ido,
    idoAuthority: ctx.idoAuthority.publicKey,
    idoUsdc: ctx.idoUsdc,
    to: ctx.idoAuthorityUsdc,
    tokenProgram: TOKEN_PROGRAM_ID,
  }).signers([ctx.idoAuthority]).rpc();
}

export async function endIdo(ctx: Context): Promise<void> {
  await idoProgram.methods.endIdo().accounts({
    ido: ctx.ido,
    idoAuthority: ctx.idoAuthority.publicKey,
  }).signers([ctx.idoAuthority]).rpc();
}
