import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Ido } from "../target/types/ido";

import assert from "assert";
import * as spl from '@solana/spl-token';
import { Signer } from "@solana/web3.js";

interface PDAParameters {
  dealWalletKey: anchor.web3.PublicKey,
  stateKey: anchor.web3.PublicKey,
  dealBump: number,
  stateBump: number,
  idx: anchor.BN,
}


interface PDALpParameters {
  idx: anchor.BN,
  lpBump: number,
  lproviderKey: anchor.web3.PublicKey,
}


// Read the generated IDL.
const idl = JSON.parse(
  require("fs").readFileSync("./target/idl/ido.json", "utf8")
);

// Address of the deployed program.
const programId = new anchor.web3.PublicKey("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// Generate the program client from IDL.
const program = new anchor.Program(idl, programId);

const provider = anchor.getProvider() as anchor.AnchorProvider
anchor.setProvider(provider);

//const program = anchor.workspace.ido as Program<Ido>;

let mintAddress: anchor.web3.PublicKey;
let alice: anchor.web3.Keypair;
let aliceWallet: anchor.web3.PublicKey;

let bob: anchor.web3.Keypair;
let charlie: anchor.web3.Keypair;
let charlieWallet: anchor.web3.PublicKey;
let pda: PDAParameters;
let pdaLP: PDALpParameters;

const getPdaParameters = async (connection: anchor.web3.Connection, alice: anchor.web3.PublicKey, bob: anchor.web3.PublicKey, mint: anchor.web3.PublicKey) => {
  const uid = new anchor.BN(parseInt((Date.now() / 1000).toString()));
  const uidBuffer = uid.toBuffer('le', 8);

  // Find the dealState Program owned by Alice For Bob 
  let [statePubKey, stateBump] = await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from("state"), alice.toBuffer(), bob.toBuffer(), mint.toBuffer(), uidBuffer], program.programId,
  );


  let [walletPubKey, walletBump] = await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from("wallet"), alice.toBuffer(), bob.toBuffer(), mint.toBuffer(), uidBuffer], program.programId,
  );


  return {
    idx: uid,
    dealBump: walletBump,
    dealWalletKey: walletPubKey,
    stateBump,
    stateKey: statePubKey,
  }

}

// FIND an lprovider Account linked to the current deal                // pda.dealkey                 // charlie key
const getLpPdaParameters = async (connection: anchor.web3.Connection, deal: anchor.web3.PublicKey, lprovider: anchor.web3.PublicKey) => {
  const uid = new anchor.BN(parseInt((Date.now() / 1000).toString()));
  const uidBuffer = uid.toBuffer('le', 8);

  let [lproviderPubkey, lproviderBump] = await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from("lprovider"), deal.toBuffer(), lprovider.toBuffer(), uidBuffer], program.programId,
  );
  return {
    idx: uid,
    lpBump: lproviderBump,
    lproviderKey: lproviderPubkey,
  }

}


const createMint = async (connection: anchor.web3.Connection): Promise<anchor.web3.PublicKey> => {
  const tokenMint = new anchor.web3.Keypair();
  const lamportsForMint = await provider.connection.getMinimumBalanceForRentExemption(spl.MintLayout.span);
  let tx = new anchor.web3.Transaction();

  // Allocate mint
  tx.add(
    anchor.web3.SystemProgram.createAccount({
      programId: spl.TOKEN_PROGRAM_ID,
      space: spl.MintLayout.span,
      fromPubkey: provider.wallet.publicKey,
      newAccountPubkey: tokenMint.publicKey,
      lamports: lamportsForMint,
    })
  )
  // Allocate wallet account
  tx.add(
    spl.createInitializeMintInstruction(
      tokenMint.publicKey,
      6,
      provider.wallet.publicKey,
      provider.wallet.publicKey,
      spl.TOKEN_PROGRAM_ID
    )
  );
  const signer: Signer = tokenMint;
  const signers = [signer]
  const txwithsigner = { tx, signers }
  anchor.web3
  const signature = await provider.sendAll([txwithsigner]);

  console.log(`[${tokenMint.publicKey}] Created new mint account at ${signature}`);
  return tokenMint.publicKey;
}

const createUserAndAssociatedWallet = async (connection: anchor.web3.Connection, mint?: anchor.web3.PublicKey): Promise<[anchor.web3.Keypair, anchor.web3.PublicKey | undefined]> => {
  const user = new anchor.web3.Keypair();
  let userAssociatedTokenAccount: anchor.web3.PublicKey | undefined = undefined;

  // Fund user with some SOL
  let txFund = new anchor.web3.Transaction();
  txFund.add(anchor.web3.SystemProgram.transfer({
    fromPubkey: provider.wallet.publicKey,
    toPubkey: user.publicKey,
    lamports: 5 * anchor.web3.LAMPORTS_PER_SOL,
  }));

  const sigTxFund = await provider.sendAll([{ tx: txFund }]);
  console.log(`[${user.publicKey.toBase58()}] Funded new account with 5 SOL: ${sigTxFund}`);

  if (mint) {
    // Create a token account for the user and mint some tokens
    userAssociatedTokenAccount = await spl.getAssociatedTokenAddress(
      mint,
      user.publicKey,
      false,
      spl.TOKEN_PROGRAM_ID,
      spl.ASSOCIATED_TOKEN_PROGRAM_ID,

    )

    const txFundTokenAccount = new anchor.web3.Transaction();
    txFundTokenAccount.add(spl.createAssociatedTokenAccountInstruction(
      user.publicKey,
      userAssociatedTokenAccount,
      user.publicKey,
      mint,
      spl.TOKEN_PROGRAM_ID,
      spl.ASSOCIATED_TOKEN_PROGRAM_ID,

    ))
    txFundTokenAccount.add(spl.createMintToInstruction(
      mint,
      userAssociatedTokenAccount,
      provider.wallet.publicKey,
      1337000000,
      [],
      spl.TOKEN_PROGRAM_ID,

    ));
    const signer: Signer = user
    const signers = [user]
    const txandsigner = { tx: txFundTokenAccount, signers }
    const txFundTokenSig = await provider.sendAll([txandsigner]);
    console.log(`[${userAssociatedTokenAccount.toBase58()}] New associated account for mint ${mint.toBase58()}: ${txFundTokenSig}`);
  }
  return [user, userAssociatedTokenAccount];
}
const readAccount = async (accountPublicKey: anchor.web3.PublicKey, provider: anchor.Provider): Promise<[spl.RawAccount, string]> => {
  const tokenInfoLol = await provider.connection.getAccountInfo(accountPublicKey);
  const data = Buffer.from(tokenInfoLol.data);
  const accountInfo: spl.RawAccount = spl.AccountLayout.decode(data);

  const amount = (accountInfo.amount as any as Buffer);
  //console.log(amount.readBigInt64LE())
  return [accountInfo, amount.toString()];
}

const readMint = async (mintPublicKey: anchor.web3.PublicKey, provider: anchor.Provider): Promise<spl.RawMint> => {
  const tokenInfo = await provider.connection.getAccountInfo(mintPublicKey);
  const data = Buffer.from(tokenInfo.data);
  const accountInfo = spl.MintLayout.decode(data);
  return {
    ...accountInfo,
    mintAuthority: accountInfo.mintAuthority == null ? null : anchor.web3.PublicKey.decode(accountInfo.mintAuthority.toBuffer()),
    freezeAuthority: accountInfo.freezeAuthority == null ? null : anchor.web3.PublicKey.decode(accountInfo.freezeAuthority.toBuffer()),
  }
}


beforeEach(async () => {
  mintAddress = await createMint(provider.connection);
  [alice, aliceWallet] = await createUserAndAssociatedWallet(provider.connection, mintAddress);

  let _rest;
  [bob, ..._rest] = await createUserAndAssociatedWallet(provider.connection);

  [charlie, charlieWallet] = await createUserAndAssociatedWallet(provider.connection, mintAddress);
  pda = await getPdaParameters(provider.connection, alice.publicKey, bob.publicKey, mintAddress);
  pdaLP = await getLpPdaParameters(provider.connection, pda.stateKey, charlie.publicKey);
});

it('can initialize a safe payment by Alice', async () => {
  const [, aliceBalancePre] = await readAccount(aliceWallet, provider);
  assert.equal(aliceBalancePre, '1337000000');

  const amount = new anchor.BN(20000000);
  const senior_tranche = new anchor.BN(60000000);
  // Initialize mint account and fund the account
  const tx1 = await program.methods.initializeDeal(pda.idx, pda.stateBump, pda.dealBump, amount, senior_tranche)

  await tx1.accounts({
    dealState: pda.stateKey,
    dealWallet: pda.dealWalletKey,
    mintOfTokenBeingSent: mintAddress,
    underwriter: alice.publicKey,
    borrower: bob.publicKey,
    walletToWithdrawFrom: aliceWallet,
    systemProgram: anchor.web3.SystemProgram.programId,
    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    tokenProgram: spl.TOKEN_PROGRAM_ID
  }).signers([alice]).rpc();

  console.log(`Initialized a new Deal instance. Alice will pay bob 20 tokens`);

  // Assert that 20 tokens were moved from Alice's account to the deal.
  const [, aliceBalancePost] = await readAccount(aliceWallet, provider);
  assert.equal(aliceBalancePost, '1317000000');
  const [, DealBalancePost] = await readAccount(pda.dealWalletKey, provider);
  assert.equal(DealBalancePost, '20000000');

  const state = await program.account.dealState.fetch(pda.stateKey);
  console.log(state);
  assert.equal(state.amountUnderwritten.toString(), '20000000');
  assert.equal(state.epoch.toString(), '1');

})



it("Creates a member and stake to the escrow wallet", async () => {

  const amount = new anchor.BN(20000000);
  const senior_tranche = new anchor.BN(60000000);
  // Initialize mint account and fund the account
  const tx1 = await program.methods.initializeDeal(pda.idx, pda.stateBump, pda.dealBump, amount, senior_tranche)

  await tx1.accounts({
    dealState: pda.stateKey,
    dealWallet: pda.dealWalletKey,
    mintOfTokenBeingSent: mintAddress,
    underwriter: alice.publicKey,
    borrower: bob.publicKey,
    walletToWithdrawFrom: aliceWallet,
    systemProgram: anchor.web3.SystemProgram.programId,
    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    tokenProgram: spl.TOKEN_PROGRAM_ID
  }).signers([alice]).rpc();

  console.log(`Initialized a new Deal instance. Alice will pay bob 20 tokens`);

  const tx = await program.methods.createLp(pda.idx, pdaLP.lpBump, pda.stateBump, pda.dealBump).accounts({
    dealState: pda.stateKey,
    underwriter: alice.publicKey,
    borrower: bob.publicKey,
    dealWallet: pda.dealWalletKey,
    mintOfTokenBeingSent: mintAddress,
    lprovider: pdaLP.lproviderKey,
    staker: charlie.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
    tokenProgram: spl.TOKEN_PROGRAM_ID,
    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  }).signers([charlie]).rpc();

  const statedeal = await program.account.dealState.fetch(pda.stateKey);
  //console.log(statedeal);
  const state = await program.account.liqProvider.fetch(pdaLP.lproviderKey);
  // Deal State is associated to liqProvider
  assert.equal(state.dealWallet._bn.toString(), statedeal.dealWallet._bn.toString());
  console.log(`Member added to the deal`);
  const tx2 = await program.methods.stake(pdaLP.idx, pdaLP.lpBump, pda.stateBump, pda.dealBump, amount).accounts({
    dealState: pda.stateKey,
    underwriter: alice.publicKey,
    borrower: bob.publicKey,
    dealWallet: pda.dealWalletKey,
    mintOfTokenBeingSent: mintAddress,
    lprovider: pdaLP.lproviderKey,
    staker: charlie.publicKey,
    stakerWallet: charlieWallet,
    systemProgram: anchor.web3.SystemProgram.programId,
    tokenProgram: spl.TOKEN_PROGRAM_ID,
    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  }).signers([charlie]).rpc();

  console.log(`Charlie is staking 20 tokens in alice escrow wallet`);
  const [, charlieBalancePost] = await readAccount(charlieWallet, provider);
  assert.equal(charlieBalancePost, '1317000000');
  const [, DealBalancePost] = await readAccount(pda.dealWalletKey, provider);
  assert.equal(DealBalancePost, '40000000');

});

it('can send escrow funds to Bob', async () => {
  // Create a token account for Bob.
  const amount = new anchor.BN(20000000);
  const stakeramount = new anchor.BN(40000000);
  const senior_tranche = new anchor.BN(40000000);
  // Initialize mint account and fund the account
  const tx1 = await program.methods.initializeDeal(pda.idx, pda.stateBump, pda.dealBump, amount, senior_tranche)

  await tx1.accounts({
    dealState: pda.stateKey,
    dealWallet: pda.dealWalletKey,
    mintOfTokenBeingSent: mintAddress,
    underwriter: alice.publicKey,
    borrower: bob.publicKey,
    walletToWithdrawFrom: aliceWallet,
    systemProgram: anchor.web3.SystemProgram.programId,
    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    tokenProgram: spl.TOKEN_PROGRAM_ID
  }).signers([alice]).rpc();

  console.log(`Initialized a new Deal instance. Alice will pay bob 20 tokens`);

  // DEBUG NOTE: The owner of  

  const tx = await program.methods.createLp(pdaLP.idx, pdaLP.lpBump, pda.stateBump, pda.dealBump).accounts({
    dealState: pda.stateKey,
    underwriter: alice.publicKey,
    borrower: bob.publicKey,
    dealWallet: pda.dealWalletKey,
    mintOfTokenBeingSent: mintAddress,
    lprovider: pdaLP.lproviderKey,
    staker: charlie.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
    tokenProgram: spl.TOKEN_PROGRAM_ID,
    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  }).signers([charlie]).rpc();
  const tx2 = await program.methods.stake(pdaLP.idx, pdaLP.lpBump, pda.stateBump, pda.dealBump, stakeramount).accounts({
    dealState: pda.stateKey,
    underwriter: alice.publicKey,
    borrower: bob.publicKey,
    dealWallet: pda.dealWalletKey,
    mintOfTokenBeingSent: mintAddress,
    lprovider: pdaLP.lproviderKey,
    staker: charlie.publicKey,
    stakerWallet: charlieWallet,
    systemProgram: anchor.web3.SystemProgram.programId,
    tokenProgram: spl.TOKEN_PROGRAM_ID,
    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  })
  //console.log(tx2)
  await tx2.signers([charlie]).rpc();

  const bobTokenAccount = await spl.getAssociatedTokenAddress(
    mintAddress,
    bob.publicKey,
    false,
    spl.TOKEN_PROGRAM_ID,
    spl.ASSOCIATED_TOKEN_PROGRAM_ID,

  )
  //console.log(bobTokenAccount)  
  const tx3 = await program.methods.borrow(pda.idx, pda.stateBump, pda.dealBump).accounts({
      dealState: pda.stateKey,
      underwriter: alice.publicKey,
      borrower: bob.publicKey,
      dealWallet: pda.dealWalletKey,
      mintOfTokenBeingSent: mintAddress,
      walletToDepositTo: bobTokenAccount,
      systemProgram: anchor.web3.SystemProgram.programId,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      tokenProgram: spl.TOKEN_PROGRAM_ID,
      associatedTokenProgram: spl.ASSOCIATED_TOKEN_PROGRAM_ID,
  })
  //console.log(tx3)
  await tx3.signers([bob]).rpc();
  //console.log(await provider.connection.getAccountInfo(aliceWallet));
  // Assert that 20 tokens were sent back.
  const [, bobBalance] = await readAccount(bobTokenAccount, provider);
  //console.log(bobBalance)
  assert.equal(bobBalance, '60000000');

});