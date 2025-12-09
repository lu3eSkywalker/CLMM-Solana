import * as anchor from "@coral-xyz/anchor";
import * as web3 from "@solana/web3.js";
import { CpiGuardLayout, getAccount, getAssociatedTokenAddress } from "@solana/spl-token";
import { SimpleTokenSwap } from "../target/types/Simple_Token_Swap";
import { BN } from "bn.js";

describe("Test", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.SimpleTokenSwap as anchor.Program<SimpleTokenSwap>;

  const tokenA_mint_address = new web3.PublicKey("6brEek47QhmqAxAuqBBnRMjshVM4XphbxFLjdVNE3uiM");
  const tokenB_mint_address = new web3.PublicKey("3vwLsA3XrM6Kqg1v6ACF4qYoZUQTM54i1atCFTPEKZG5");

  it("initializes a Vault Account For Token A", async () => {
    const [vault_token_account, bump1] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vaultTokenA"), tokenA_mint_address.toBuffer()],
      program.programId
    )

    const [vaultPDA, bump2] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault_auth_a"), tokenA_mint_address.toBuffer()],
      program.programId
    );

    console.log("This is the Token vault_token_account for Token A: ", vault_token_account.toString());
    console.log("This is the vaultPDA for Token A: ", vaultPDA.toString());

    // Send Transaction
    const txHash = await program.methods
    .initializeVaultTokenA()
    .accounts({
      vaultTokenAccount: vault_token_account,
      vault_auth: vaultPDA,
      payer: program.provider.publicKey,
      mint: tokenA_mint_address,
      systemProgram: web3.SystemProgram.programId,
      tokenProgram: new web3.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
      rent: web3.SYSVAR_RENT_PUBKEY,      
    })
    .rpc();

    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

    // Confirm Transaction
    await program.provider.connection.confirmTransaction(txHash);
  });

  it("initializes a Vault Account For Token B", async() => {
    const [vault_token_account, bump1] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vaultTokenB"), tokenB_mint_address.toBuffer()],
      program.programId
    )

    const [vaultPDA, bump2] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault_auth_b"), tokenB_mint_address.toBuffer()],
      program.programId
    );

    console.log("This is the Token vault_token_account for Token B: ", vault_token_account.toString());
    console.log("This is the vaultPDA for Token B: ", vaultPDA.toString());

    // Send Transaction
    const txHash = await program.methods
    .initializeVaultTokenB()
    .accounts({
      vaultTokenAccount: vault_token_account,
      vault_auth: vaultPDA,
      payer: program.provider.publicKey,
      mint: tokenB_mint_address,
      systemProgram: web3.SystemProgram.programId,
      tokenProgram: new web3.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
      rent: web3.SYSVAR_RENT_PUBKEY,    
    })
    .rpc();

    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

    // Confirm Transaction
    await program.provider.connection.confirmTransaction(txHash);

    const pda_token_value = await getAccount(program.provider.connection, vault_token_account);
    console.log("Vault Token B Account Balance: ", pda_token_value.amount.toString());
  });

  it("deposit Token A to vault", async () => {

    const [vault_token_account, bump1] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vaultTokenA"), tokenA_mint_address.toBuffer()],
      program.programId
    )

    const amount_to_deposit = new BN(5_000_000_000);

    const userATA = new web3.PublicKey("Bd1Ho7Y9PsZ1zK6YpXnHGmzSoygPVM9FhYMuj85ytEUg");

    // Send Transaction
    const txHash = await program.methods
    .tokenADepositInPdaVault(amount_to_deposit)
    .accounts({
      user: program.provider.publicKey,
      userTokenAccount: userATA,
      vaultTokenAccount: vault_token_account,
      mint: tokenA_mint_address,
      tokenProgram: new web3.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),      
    })
    .rpc();

    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

    // Confirm Transaction
    await program.provider.connection.confirmTransaction(txHash);
  });

  it("deposit Token B to vault", async () => {
    const [vault_token_account, bump1] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vaultTokenB"), tokenB_mint_address.toBuffer()],
      program.programId
    )

    const amount_to_deposit = new BN(5_000_000_000);

    const userATA = new web3.PublicKey("7TWc3HMxNi2FH33BWzdMeexN93C8DLXdDBtfKgXuXEQc");

    // Send Transaction
    const txHash = await program.methods
    .tokenBDepositInPdaVault(amount_to_deposit)
    .accounts({
      user: program.provider.publicKey,
      userTokenAccount: userATA,
      vaultTokenAccount: vault_token_account,
      mint: tokenB_mint_address,
      tokenProgram: new web3.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
    })
    .rpc();

    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

    // Confirm Transaction
    await program.provider.connection.confirmTransaction(txHash);
  });

  it("Swap Token B for Token A", async () => {

    const userATAforTokenA = new web3.PublicKey("Bd1Ho7Y9PsZ1zK6YpXnHGmzSoygPVM9FhYMuj85ytEUg");
    const userATAforTokenB = new web3.PublicKey("7TWc3HMxNi2FH33BWzdMeexN93C8DLXdDBtfKgXuXEQc");

    const [vault_token_account_a, bump1] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vaultTokenA"), tokenA_mint_address.toBuffer()],
      program.programId
    );

    const [vault_token_account_b, bump2] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vaultTokenB"), tokenB_mint_address.toBuffer()],
      program.programId
    );

    const [vault_auth_a, bump3] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault_auth_a"), tokenA_mint_address.toBuffer()],
      program.programId
    );

    const [vault_auth_b, bump4] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault_auth_b"), tokenB_mint_address.toBuffer()],
      program.programId
    );

    const vaultTokenAVault = await getAccount(program.provider.connection, vault_token_account_a);
    const vaultTokenBVault = await getAccount(program.provider.connection, vault_token_account_b);

    console.log("This is the value of vault_token_account_a: ", vaultTokenAVault.amount.toString());
    console.log("This is the value of vault_token_account_b: ", vaultTokenBVault.amount.toString());

    const amount = new BN(2_000_000_000);

    const txHash = await program.methods
      .swapBForA(amount)
      .accounts({
        user: program.provider.publicKey,
        userTokenAccountForTokenA: userATAforTokenA,
        userTokenAccountForTokenB: userATAforTokenB,
        vaultTokenAAccount: vault_token_account_a,
        vaultTokenBAccount: vault_token_account_b,
        vaultAuthA: vault_auth_a,
        vaultAuthB: vault_auth_b,
        mintA: tokenA_mint_address,
        mintB: tokenB_mint_address,
        tokenProgram: new web3.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
      })
      .rpc();

      console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

      // Confirm Transaction
      await program.provider.connection.confirmTransaction(txHash);
  })

    it("Swap Token A for Token B", async () => {

    const userATAforTokenA = new web3.PublicKey("Bd1Ho7Y9PsZ1zK6YpXnHGmzSoygPVM9FhYMuj85ytEUg");
    const userATAforTokenB = new web3.PublicKey("7TWc3HMxNi2FH33BWzdMeexN93C8DLXdDBtfKgXuXEQc");

    const [vault_token_account_a, bump1] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vaultTokenA"), tokenA_mint_address.toBuffer()],
      program.programId
    );

    const [vault_token_account_b, bump2] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vaultTokenB"), tokenB_mint_address.toBuffer()],
      program.programId
    );

    const [vault_auth_a, bump3] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault_auth_a"), tokenA_mint_address.toBuffer()],
      program.programId
    );

    const [vault_auth_b, bump4] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault_auth_b"), tokenB_mint_address.toBuffer()],
      program.programId
    );

    const vaultTokenAVault = await getAccount(program.provider.connection, vault_token_account_a);
    const vaultTokenBVault = await getAccount(program.provider.connection, vault_token_account_b);

    console.log("This is the value of vault_token_account_a: ", vaultTokenAVault.amount.toString());
    console.log("This is the value of vault_token_account_b: ", vaultTokenBVault.amount.toString());

    const amount = new BN(1_000_000_000);

    const txHash = await program.methods
      .swapAForB(amount)
      .accounts({
        user: program.provider.publicKey,
        userTokenAccountForTokenA: userATAforTokenA,
        userTokenAccountForTokenB: userATAforTokenB,
        vaultTokenAAccount: vault_token_account_a,
        vaultTokenBAccount: vault_token_account_b,
        vaultAuthA: vault_auth_a,
        vaultAuthB: vault_auth_b,
        mintA: tokenA_mint_address,
        mintB: tokenB_mint_address,
        tokenProgram: new web3.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
      })
      .rpc();

      console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

      // Confirm Transaction
      await program.provider.connection.confirmTransaction(txHash);
  })
});