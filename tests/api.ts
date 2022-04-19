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

const idoProgram = anchor.workspace.Ido as Program<
  Ido
>;

export async function initialize(
  acdmMint: PublicKey,
  usdcMint: PublicKey,
  roundTime: BN,
  idoAuthority: Keypair,
): Promise<[PublicKey, PublicKey, PublicKey]> {
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

  await idoProgram.methods.initialize(roundTime)
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

  return [ido, idoAcdm, idoUsdc];
}

export async function registerMember(
  user: Keypair,
  referer: PublicKey,
  refererMember: PublicKey,
): Promise<PublicKey> {
  const [member] = await PublicKey
    .findProgramAddress(
      [Buffer.from("member"), user.publicKey.toBuffer()],
      idoProgram.programId,
    );

  const remainingAccounts = [];

  if (refererMember != null) {
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

export async function startSaleRound(
  ido: PublicKey,
  idoAcdm: PublicKey,
  idoAuthority: Keypair,
  acdmMint: PublicKey,
  acdmMintAuthority: Keypair,
) {
  await idoProgram.methods.startSaleRound().accounts({
    ido,
    idoAuthority: idoAuthority.publicKey,
    acdmMintAuthority: acdmMintAuthority.publicKey,
    acdmMint,
    idoAcdm,
    tokenProgram: TOKEN_PROGRAM_ID,
  }).signers([idoAuthority, acdmMintAuthority]).rpc();
}

export async function buyAcdm(
  amount: BN,
  ido: PublicKey,
  idoAcdm: PublicKey,
  idoUsdc: PublicKey,
  buyer: Keypair,
  buyerAcdm: PublicKey,
  buyerUsdc: PublicKey,
  refererMember: PublicKey,
  refererUsdc: PublicKey,
  referer2Usdc: PublicKey,
): Promise<void> {
  const [buyerMember] = await PublicKey
    .findProgramAddress(
      [Buffer.from("member"), buyer.publicKey.toBuffer()],
      idoProgram.programId,
    );

  const remainingAccounts = [];

  if (refererMember) {
    remainingAccounts.push({
      pubkey: refererMember,
      isWritable: false,
      isSigner: false,
    });
  }
  if (refererUsdc) {
    remainingAccounts.push({
      pubkey: refererUsdc,
      isWritable: true,
      isSigner: false,
    });
  }
  if (referer2Usdc) {
    remainingAccounts.push({
      pubkey: referer2Usdc,
      isWritable: true,
      isSigner: false,
    });
  }

  await idoProgram.methods.buyAcdm(amount).accounts({
    ido,
    idoAcdm,
    idoUsdc,
    buyer: buyer.publicKey,
    buyerMember,
    buyerAcdm,
    buyerUsdc,
    tokenProgram: TOKEN_PROGRAM_ID,
  }).remainingAccounts(remainingAccounts).signers([buyer]).rpc();
}

export async function startTradeRound(
  ido: PublicKey,
  idoAcdm: PublicKey,
  idoAuthority: Keypair,
  acdmMint: PublicKey,
): Promise<void> {
  await idoProgram.methods.startTradeRound().accounts({
    ido,
    idoAuthority: idoAuthority.publicKey,
    acdmMint,
    idoAcdm,
    tokenProgram: TOKEN_PROGRAM_ID,
  }).signers([idoAuthority]).rpc();
}

export async function addOrder(
  amount: BN,
  price: BN,
  ido: PublicKey,
  acdmMint: PublicKey,
  seller: Keypair,
  sellerAcdm: PublicKey,
): Promise<[PublicKey, PublicKey, BN]> {
  const [order] = await PublicKey
    .findProgramAddress(
      [Buffer.from("order"), new BN(0).toArrayLike(Buffer, "le", 8)],
      idoProgram.programId,
    );

  const [orderAcdm] = await PublicKey
    .findProgramAddress(
      [Buffer.from("order_acdm"), new BN(0).toArrayLike(Buffer, "le", 8)],
      idoProgram.programId,
    );

  let listener: number;
  const [event, _] = await new Promise((resolve, _reject) => {
    listener = idoProgram.addEventListener("OrderEvent", (event, slot) => {
      resolve([event, slot]);
    });
    idoProgram.methods.addOrder(amount, price).accounts({
      ido,
      order,
      acdmMint,
      orderAcdm,
      seller: seller.publicKey,
      sellerAcdm,
      rent: SYSVAR_RENT_PUBKEY,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    }).signers([seller]).rpc();
  });
  await idoProgram.removeEventListener(listener);

  return [order, orderAcdm, event.id];
}

export async function redeemOrder(
  orderId: BN,
  amount: BN,
  ido: PublicKey,
  idoUsdc: PublicKey,
  order: PublicKey,
  orderAcdm: PublicKey,
  buyer: Keypair,
  buyerAcdm: PublicKey,
  buyerUsdc: PublicKey,
  seller: PublicKey,
  sellerMember: PublicKey,
  sellerUsdc: PublicKey,
  refererMember: PublicKey,
  refererUsdc: PublicKey,
  referer2Usdc: PublicKey,
): Promise<void> {
  const remainingAccounts = [];

  if (refererMember) {
    remainingAccounts.push({
      pubkey: refererMember,
      isWritable: false,
      isSigner: false,
    });
  }
  if (refererUsdc) {
    remainingAccounts.push({
      pubkey: refererUsdc,
      isWritable: true,
      isSigner: false,
    });
  }
  if (referer2Usdc) {
    remainingAccounts.push({
      pubkey: referer2Usdc,
      isWritable: true,
      isSigner: false,
    });
  }

  await idoProgram.methods.redeemOrder(orderId, amount).accounts({
    ido,
    idoUsdc,
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
  orderId: BN,
  order: PublicKey,
  orderAcdm: PublicKey,
  seller: Keypair,
  sellerAcdm: PublicKey,
): Promise<void> {
  await idoProgram.methods.removeOrder(orderId).accounts({
    order,
    orderAcdm,
    seller: seller.publicKey,
    sellerAcdm,
    tokenProgram: TOKEN_PROGRAM_ID,
  }).signers([seller]).rpc();
}

export async function withdrawIdoUsdc(
  ido: PublicKey,
  idoUsdc: PublicKey,
  idoAuthority: Keypair,
  idoAuthorityUsdc: PublicKey,
): Promise<void> {
  await idoProgram.methods.withdrawIdoUsdc().accounts({
    ido,
    idoAuthority: idoAuthority.publicKey,
    idoUsdc,
    to: idoAuthorityUsdc,
    tokenProgram: TOKEN_PROGRAM_ID,
  }).signers([idoAuthority]).rpc();
}

export async function endIdo(
  ido: PublicKey,
  idoAuthority: Keypair,
): Promise<void> {
  await idoProgram.methods.endIdo().accounts({
    ido,
    idoAuthority: idoAuthority.publicKey,
  }).signers([idoAuthority]).rpc();
}
