import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BangerProgram } from "../target/types/banger_program";

describe("banger-program", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());

    const provider = anchor.getProvider();
    const connection = provider.connection;
    const program = anchor.workspace.BangerProgram as Program<BangerProgram>;



    it("Is initialized!", async () => {
        // Add your test here.
        const tx = await program.methods.initialize().rpc();
        console.log("Your transaction signature", tx);
    });
});
