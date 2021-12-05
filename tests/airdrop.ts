import * as anchor from "@project-serum/anchor";
import { Program, BN, IdlAccounts } from "@project-serum/anchor";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, Token } from "@solana/spl-token";
import { assert } from "chai";
// @ts-ignore
import { Airdrop } from "../target/types/airdrop";

type AirdropAccount = IdlAccounts<Airdrop>["airdropAccount"];

describe("airdrop", () => {
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Airdrop as Program<Airdrop>;

  let mint: Token = null;
  let initializerTokenAccount: PublicKey = null;
  let takerTokenAccount: PublicKey = null;
  let airdropTokenAccount: PublicKey = null;
  let pda: PublicKey = null;
  const airdropAmount = 500;
  const withdrawAmount = 3;

  const airdropAccount = Keypair.generate();
  const payer = Keypair.generate();
  const mintAuthority = Keypair.generate();

  it("Initialise airdrop state", async () => {
    // Airdropping tokens to a payer.
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, 10000000000),
      "confirmed"
    );

    mint = await Token.createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0,
      TOKEN_PROGRAM_ID
    );

    initializerTokenAccount = await mint.createAccount(
      provider.wallet.publicKey
    );
    takerTokenAccount = await mint.createAccount(provider.wallet.publicKey);

    await mint.mintTo(
      initializerTokenAccount,
      mintAuthority.publicKey,
      [mintAuthority],
      airdropAmount
    );

    let _initializerTokenAccountA = await mint.getAccountInfo(
      initializerTokenAccount
    );

    assert.ok(_initializerTokenAccountA.amount.toNumber() == airdropAmount);



    airdropTokenAccount = await mint.createAccount(
        airdropAccount.publicKey
    );

    await mint.mintTo(
        airdropTokenAccount,
        mintAuthority.publicKey,
        [mintAuthority],
        0
    );

    let _airdropTokenAccountA = await mint.getAccountInfo(
        airdropTokenAccount
    );

    assert.ok(_airdropTokenAccountA.amount.toNumber() == 0);

  });

  it("Initialize airdrop", async () => {
    await program.rpc.initializeAirdrop(
      new BN(airdropAmount),
      new BN(withdrawAmount),
      {
        accounts: {
          initializer: provider.wallet.publicKey,
          initializerDepositTokenAccount: initializerTokenAccount,
          airdropAccount: airdropAccount.publicKey,
          airdropTokenAccount: airdropTokenAccount,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID
        },
        signers: [airdropAccount],
      }
    );

    // Get the PDA that is assigned authority to token account.
    const [_pda, _nonce] = await PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode("streamflow-airdrop"))],
      program.programId
    );

    pda = _pda;

    let _initializerTokenAccount = await mint.getAccountInfo(
      initializerTokenAccount
    );

    let _airdropTokenAccount = await mint.getAccountInfo(
        airdropTokenAccount
    );

    let _airdropAccount: AirdropAccount =
      await program.account.airdropAccount.fetch(airdropAccount.publicKey);

    // Check that the new owner is the PDA.
    //assert.ok(_initializerTokenAccount.owner.equals(pda));

    // Check that the values in the airdrop account match what we expect.
    assert.ok(_airdropAccount.initializerKey.equals(provider.wallet.publicKey));
    //assert.ok(_airdropAccount.airdropAmount.toNumber() == airdropAmount);

    assert.ok(_airdropTokenAccount.amount.toNumber() == airdropAmount);
    // assert.ok(
    //   _airdropAccount.initializerDepositTokenAccount.equals(
    //     initializerTokenAccount
    //   )
    // );

  });

  it("Get airdrop", async () => {
    await program.rpc.getAirdrop({
      accounts: {
        taker: provider.wallet.publicKey,
        takerReceiveTokenAccount: takerTokenAccount,
        pdaDepositTokenAccount: initializerTokenAccount,
        initializerMainAccount: provider.wallet.publicKey,
        airdropAccount: airdropAccount.publicKey,
        airdropTokenAccount: airdropTokenAccount,
        pdaAccount: pda,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [airdropAccount]
    });

    let _takerTokenAccount = await mint.getAccountInfo(takerTokenAccount);

    let _airdropTokenAccount = await mint.getAccountInfo(
        airdropTokenAccount
    );

    assert.ok(_takerTokenAccount.amount.toNumber() == withdrawAmount);
    assert.ok(_airdropTokenAccount.amount.toNumber() == airdropAmount - withdrawAmount);

    // Check that the owner is still the PDA.
    assert.ok(_airdropTokenAccount.owner.equals(pda));
  });

  //todo test multiple get airdrop calls

  // it("Cancel airdrop", async () => {
  //   let newAirdrop = Keypair.generate();
  //
  //   //reset test and then cancel airdrop
  //   let newInitializerTokenAccount = await mint.createAccount(
  //       provider.wallet.publicKey
  //   );
  //
  //   await mint.mintTo(
  //       newInitializerTokenAccount,
  //       mintAuthority.publicKey,
  //       [mintAuthority],
  //       airdropAmount
  //   );
  //
  //   //initialize new account
  //   await program.rpc.initializeAirdrop(
  //       new BN(airdropAmount),
  //       new BN(withdrawAmount),
  //       {
  //         accounts: {
  //           initializer: provider.wallet.publicKey,
  //           initializerDepositTokenAccount: newInitializerTokenAccount,
  //           airdropAccount: newAirdrop.publicKey,
  //           systemProgram: SystemProgram.programId,
  //           tokenProgram: TOKEN_PROGRAM_ID,
  //         },
  //         signers: [newAirdrop],
  //       }
  //   );
  //
  //   let _initializerTokenAccount = await mint.getAccountInfo(
  //       newInitializerTokenAccount
  //   );
  //
  //   // Check that the new owner is the PDA.
  //   assert.ok(_initializerTokenAccount.owner.equals(pda));
  //
  //   // call the cancel
  //   await program.rpc.cancelAirdrop({
  //     accounts: {
  //       initializer: provider.wallet.publicKey,
  //       pdaDepositTokenAccount: newInitializerTokenAccount,
  //       pdaAccount: pda,
  //       airdropAccount: newAirdrop.publicKey,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //     },
  //   });
  //
  //   _initializerTokenAccount = await mint.getAccountInfo(
  //       newInitializerTokenAccount
  //   );
  //
  //   // Check the final owner should be the initializer (provider public key).
  //   assert.ok(
  //       _initializerTokenAccount.owner.equals(provider.wallet.publicKey)
  //   );
  //
  //   // Check all the funds are still there.
  //   assert.ok(_initializerTokenAccount.amount.toNumber() == airdropAmount);
  // });

});
