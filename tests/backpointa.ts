import fs from 'fs';

import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  Token,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import {
  Keypair,
  PublicKey,
  SystemProgram,
} from '@solana/web3.js';

import { Backpointa } from '../target/types/backpointa';

describe("backpointa", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Backpointa as Program<Backpointa>;
  const provider = program.provider;
  const funder = Keypair.fromSecretKey(new Uint8Array(
    JSON.parse(
      fs.readFileSync('/Users/jd/7i.json').toString()
      
    )

  )  )
  const recipient = Keypair.generate();
  let unwrappedMint, funderTokenAccount, recipientWrappedTokenAccount, escrow, wrappedMint
  // Service account that collects fees
  const serviceAccount = new PublicKey("7ihN8QaTfNoDTRTQGULCzbUT3PHwPDTu5Brcu4iT2paP");
  before(async () => {
 
  // Creating a new mint for the unwrapped token
   unwrappedMint = await Token.createMint(
    provider.connection,
    funder,
    funder.publicKey, // Owner of the mint
    null, // Freeze authority
    9, // Decimals
    TOKEN_PROGRAM_ID
  );
  var [backpointa, _] = PublicKey.findProgramAddressSync(
    [
      anchor.utils.bytes.utf8.encode("backpointa"),unwrappedMint.publicKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer()],
    program.programId
  );
  wrappedMint = Keypair.generate();
  
  // Creating a token account for the funder
   funderTokenAccount = await unwrappedMint.createAccount(funder.publicKey);
   // Recipient's account for wrapped tokens
    recipientWrappedTokenAccount = await Token.getAssociatedTokenAddress(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      wrappedMint.publicKey,
      funder.publicKey,
      true);
  
  // Airdrop some tokens to the funder's token account
  await unwrappedMint.mintTo(
    funderTokenAccount,
    funder.publicKey,
    [],
    1000000000 // Adjust the amount based on your needs
  );
  
  // Creating a token account for the service fees
  await unwrappedMint.createAccount(serviceAccount);
  })
  it('Create Wrapped Mint', async () => {
    var [backpointa, _] = await PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode("backpointa"),unwrappedMint.publicKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer()],
      program.programId
    );
  
    let create = await program.methods.createMint(true)
      .accounts({
        funder: funder.publicKey,
        unwrappedMint: unwrappedMint.publicKey,
        wrappedMint: wrappedMint.publicKey,
        wrappedMintBackpointer: backpointa,
        tokenProgram: TOKEN_PROGRAM_ID,
        tokenProgramWrapped: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
        toServiceAccount: serviceAccount,
      })
      .signers([funder, wrappedMint])
      .rpc({skipPreflight:false});
      console.log('create https://solscan.io/tx/'+create+'?cluster=devnet')
  });

  it('Wrap Tokens', async () => {
    // Escrow account to hold unwrapped tokens
    var [backpointa, _] = await PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode("backpointa"),unwrappedMint.publicKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer()],
      program.programId
    );
  
    escrow = await unwrappedMint.createAccount(backpointa);
  
    let wrap = await program.methods.wrap(new anchor.BN(500000000)) // Adjust amount as needed
      .accounts({
        funder: funder.publicKey,
        unwrappedTokenAccount: funderTokenAccount,
        escrow: escrow,
        unwrappedMint: unwrappedMint.publicKey,
        wrappedMint: wrappedMint.publicKey, // From createMint test
        recipientWrappedTokenAccount: recipientWrappedTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        wrappedMintBackpointer: backpointa,
        tokenProgramWrapped: TOKEN_PROGRAM_ID,
        transferAuthority: funder.publicKey,
        toServiceAccount: serviceAccount,
      })
      .signers([funder])
      .rpc({skipPreflight: true});
      console.log('wrap https://solscan.io/tx/'+wrap+'?cluster=devnet')
  });
  it('Unwrap Tokens', async () => {
    // Assuming the recipientWrappedTokenAccount has wrapped tokens from the previous wrap test
    var [backpointa, _] = await PublicKey.findProgramAddress(
      [
      anchor.utils.bytes.utf8.encode("backpointa"),
        unwrappedMint.publicKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer()],
      program.programId
    );
  
    let unwrap = await program.methods.unwrap(new anchor.BN(500000000)) // Adjust amount as needed
      .accounts({
        wrappedTokenAccount: recipientWrappedTokenAccount, // From wrap test
        wrappedMint: wrappedMint.publicKey, // From createMint test
        escrow: escrow, // From wrap test
        recipientUnwrappedTokenAccount: funderTokenAccount,
        unwrappedMint: unwrappedMint.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        tokenProgramWrapped: TOKEN_PROGRAM_ID,
        wrappedMintBackpointer: backpointa,
        transferAuthority: funder.publicKey,
        toServiceAccount: serviceAccount,
      })
      .signers([funder])
      .rpc();
      console.log('unwrap https://solscan.io/tx/'+unwrap+'?cluster=devnet')
  });
    
});
