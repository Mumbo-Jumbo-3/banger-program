import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BangerProgram } from "../target/types/banger_program";
import { randomBytes } from "crypto";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
  SYSVAR_INSTRUCTIONS_PUBKEY
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount
} from "@solana/spl-token";
import Irys from "@irys/sdk";
import { CreateAndUploadOptions } from "@irys/sdk/build/cjs/common/types";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";

describe("banger-program", () => {
    // Configure the client.
    anchor.setProvider(anchor.AnchorProvider.env());
    const provider = anchor.getProvider();
    const connection = provider.connection;
    const program = anchor.workspace.BangerProgram as Program<BangerProgram>;

    const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

    // Function for confirming transactions
    const confirm = async (signature: string): Promise<string> => {
        const block = await connection.getLatestBlockhash();
        await connection.confirmTransaction({
        signature,
        ...block,
        });
        return signature;
    };

    // Function for logging confirmed transactions
    const log = async (signature: string): Promise<string> => {
        console.log(
        `Your transaction signature: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`
        );
        return signature;
    };

    // Generate random seed
    let seed = new anchor.BN(randomBytes(8));

    // Generate keypairs for maker, taker, token x, and token y
    const [admin, trader, mintX, treasury] = Array.from({ length: 5 }, () =>
        Keypair.generate()
    );

    // Generate maker and taker ATAs for token x and token y
    /*
    const [makerAtaX, makerAtaY, takerAtaX, takerAtaY] = [maker, taker]
    .map((a) =>
        [mintX, mintY].map((m) =>
        getAssociatedTokenAddressSync(m.publicKey, a.publicKey)
        )
    )
    .flat();
    */

    // Generate PDA accounts
    const curve = PublicKey.findProgramAddressSync(
        [Buffer.from("curve")],
        program.programId
    )[0];

    const pool = PublicKey.findProgramAddressSync(
        [Buffer.from("pool"), mintX.publicKey.toBuffer()],
        program.programId
    )[0];

    const authority = PublicKey.findProgramAddressSync([Buffer.from("authority")], program.programId)[0];

    const creatorVault = PublicKey.findProgramAddressSync([Buffer.from("creator_vault"), Buffer.from("12345")], program.programId)[0];

    //const traderAtaX = getAssociatedTokenAddressSync(mintX.publicKey, trader.publicKey);
    // Request SOL to trader
    /*
    it("Airdrop", async () => {
        await Promise.all([
            await connection
                .requestAirdrop(admin.publicKey, LAMPORTS_PER_SOL * 10)
                .then(confirm),
            await connection
                .requestAirdrop(trader.publicKey, LAMPORTS_PER_SOL * 10)
                .then(confirm),
        ]);
    });
    */
    const provider2 = anchor.AnchorProvider.env();
    const wallet = provider2.wallet as NodeWallet
    it("Load SOL", async () => {
        // Add transfer instruction to transaction
        var transaction = new anchor.web3.Transaction().add(
            anchor.web3.SystemProgram.transfer({
            fromPubkey: provider.publicKey,
            toPubkey: admin.publicKey,
            lamports: LAMPORTS_PER_SOL * 1, // number of SOL to send
            }),
        );
        
        // Sign transaction, broadcast, and confirm
        var signature = await anchor.web3.sendAndConfirmTransaction(connection, transaction, [
            wallet.payer,
        ]);
        console.log('SIGNATURE', signature);
        
         // Add transfer instruction to transaction
         var transaction = new anchor.web3.Transaction().add(
            anchor.web3.SystemProgram.transfer({
            fromPubkey: provider.publicKey,
            toPubkey: trader.publicKey,
            lamports: LAMPORTS_PER_SOL * 1, // number of SOL to send
            }),
        );
        
        // Sign transaction, broadcast, and confirm
        var signature = await anchor.web3.sendAndConfirmTransaction(connection, transaction, [
            wallet.payer,
        ]);
        console.log('SIGNATURE', signature);
        
    });
    /*
    it("Setup tokens", async () => {
        const lamports = await getMinimumBalanceForRentExemptMint(connection);
        let tx = new Transaction();
        tx.instructions = [
            // create token x account
            anchor.web3.SystemProgram.createAccount({
                fromPubkey: provider.publicKey,
                newAccountPubkey: mintX.publicKey,
                lamports,
                space: MINT_SIZE,
                programId: TOKEN_PROGRAM_ID,
            }),
            // initialize token x
            createInitializeMint2Instruction(
                mintX.publicKey,
                0,
                admin.publicKey,
                null
            ),
        ];

        try {
            await provider.sendAndConfirm(tx, [mintX, admin]).then(log);
        } catch (error) {
            console.log(error);
        }
    });
    */
    
    xit("Init curve", async () => {
        try {
        const tx = await program.methods.initCurve(new anchor.BN(2), new anchor.BN(32000))
            .accounts({
                admin: admin.publicKey,
                curve,
                systemProgram: SystemProgram.programId
            })
            .signers([admin])
            .rpc({skipPreflight: true})
            .then(confirm)
            .then(log);
        } catch(e) {
            console.error(e);
        }
    });

    // Upload metadata
    const getIrys = async () => {
        const url = "https://devnet.irys.xyz";
        // Devnet RPC URLs change often, use a recent one from https://chainlist.org/chain/80001
        const providerUrl = "https://api.devnet.solana.com";
        const token = "solana";
     
        const irys = new Irys({
            url, // URL of the node you want to connect to
            token, // Token used for payment
            key: admin.secretKey, // ETH or SOL private key
            config: { providerUrl }, // Optional provider URL, only required when using Devnet
        });
        return irys;
    };
    
    const fundNode = async () => {
        const irys = await getIrys();
        try {
            const fundTx = await irys.fund(irys.utils.toAtomic(0.05));
            console.log(`Successfully funded ${irys.utils.fromAtomic(fundTx.quantity)} ${irys.token}`);
        } catch (e) {
            console.log("Error uploading data ", e);
        }
    };

    const uploadFile = async (file) => {
        const irys = await getIrys();
        try {
            const receipt = await irys.uploadFile(file);
            console.log(`Data uploaded ==> https://gateway.irys.xyz/${receipt.id}`);
            return `https://gateway.irys.xyz/${receipt.id}`;
        } catch (e) {
            console.log("Error uploading data ", e);
        }
    };

    const uploadData = async (data, type) => {
        const irys = await getIrys();
        try {
            const opts: CreateAndUploadOptions = {
                tags: [{ name: 'Content-Type', value: type}]
            }
            const receipt = await irys.upload(data, opts);
            console.log(`Data uploaded ==> https://gateway.irys.xyz/${receipt.id}`);
            return `https://gateway.irys.xyz/${receipt.id}`;
        } catch (e) {
            console.log("Error uploading data ", e);
        }
    };

    const imageUrl = async() => await uploadFile('banger4.jpg');
    
    const metadataObj = JSON.stringify({
      name: "Test",
      symbol: "BNGR",
      description: "Cultural artifact verified by Banger and curated by the owner.",
      image: imageUrl,
      external_url: "https://banger.lol",
      attributes: [
        {
          trait_type: "Platform",
          value: "twitter",
        },
        {
          trait_type: "Creator",
          value: "12345",
        },
        {
          trait_type: "Created At",
          value: "2024-03-27",
        },
        {
          trait_type: "Source",
          value: "https://x.com/12345"
        }
      ],
    });

    const getMetadata = async (mint: anchor.web3.PublicKey): Promise<anchor.web3.PublicKey> => {
        return (
          anchor.web3.PublicKey.findProgramAddressSync(
            [
              Buffer.from("metadata"),
              TOKEN_METADATA_PROGRAM_ID.toBuffer(),
              mint.toBuffer(),
            ],
            TOKEN_METADATA_PROGRAM_ID
          )
        )[0];
    };

    // Test init pool
    it("Init Pool", async () => {
        try {
        const metadata = await getMetadata(mintX.publicKey);
        const metadataUrl = await uploadData(metadataObj, 'application/json');
        await program.methods
            .initPool("12345", 500, 500, "Test", metadataUrl)
            .accounts({
            admin: admin.publicKey,
            mint: mintX.publicKey,
            authority: authority,
            metadata: metadata,
            curve: curve,
            pool,
            treasury: treasury.publicKey,
            creatorVault: creatorVault,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            metadataProgram: TOKEN_METADATA_PROGRAM_ID,
            sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY
            })
            .signers([admin, mintX])
            .rpc({skipPreflight: true})
            .then(confirm)
            .then(log);
        } catch(e) {
            console.error(e);
        }
    });

    // Test buy
    it("Buy", async () => {
        try {
        const metadata = await getMetadata(mintX.publicKey);
        const traderAtaX = await getOrCreateAssociatedTokenAccount(connection, admin, mintX.publicKey, trader.publicKey);
        await program.methods
            .buy(new anchor.BN(0), new anchor.BN(1))
            .accounts({
            buyer: trader.publicKey,
            mint: mintX.publicKey,
            buyerAta: traderAtaX.address,
            authority: authority,
            metadata: metadata,
            curve: curve,
            treasury: treasury.publicKey,
            creatorVault: creatorVault,
            pool,
            systemProgram: SystemProgram.programId,
            metadataProgram: TOKEN_METADATA_PROGRAM_ID,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
            sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY
            })
            .signers([trader])
            .rpc({skipPreflight: true})
            .then(confirm)
            .then(log);
        } catch(e) {
            console.error(e);
        }
    });

    // Test sell
    it("Sell", async () => {
        try {
        const metadata = await getMetadata(mintX.publicKey);
        const traderAtaX = await getOrCreateAssociatedTokenAccount(connection, admin, mintX.publicKey, trader.publicKey);
        await program.methods
            .sell(new anchor.BN(1), new anchor.BN(0))
            .accounts({
            seller: trader.publicKey,
            mint: mintX.publicKey,
            sellerAta: traderAtaX.address,
            authority: authority,
            metadata: metadata,
            curve: curve,
            treasury: treasury.publicKey,
            creatorVault: creatorVault,
            pool,
            systemProgram: SystemProgram.programId,
            metadataProgram: TOKEN_METADATA_PROGRAM_ID,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
            sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY
            })
            .signers([trader])
            .rpc({skipPreflight: true})
            .then(confirm)
            .then(log);
        } catch(e) {
            console.error(e);
        }
    });
})


