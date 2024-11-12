import { expect } from 'chai';
import * as anchor from '@coral-xyz/anchor';
import { PublicKey } from "@solana/web3.js";
import { Presale } from '../target/types/presale';
import * as ed from '@noble/ed25519';
import { AnchorProvider } from '@coral-xyz/anchor';
import { createMint, getOrCreateAssociatedTokenAccount, mintTo, getAccount } from '@solana/spl-token';

interface StableInfo {
  mint: PublicKey;
  payerAta: PublicKey;
  storeAta: PublicKey;
}; 

interface StablesInfo {
  usdc: StableInfo; 
  usdt: StableInfo;
}

function i16ToBytesLE(value: number): Uint8Array {
  const buffer = new ArrayBuffer(2); // Allocate 2 bytes for the i16 value
  const view = new DataView(buffer);
  view.setInt16(0, value, true); // Set the value in little-endian format
  return new Uint8Array(buffer);
}

const ROUND_TAG = Buffer.from('ITERATION');
const USER_TAG = Buffer.from('BUYER');
const REF_TAG = Buffer.from('ADVISER');

const prepareStable = async (provider: AnchorProvider, payer: anchor.web3.Keypair, store: PublicKey, keypair: anchor.web3.Keypair): Promise<StableInfo> => {
  try {
    const mint = await createMint(
      provider.connection,
      payer,
      payer.publicKey,
      payer.publicKey,
      6,
      keypair,
    ); 

    const payerAta = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      payer,
      mint,
      payer.publicKey,
      false,
    );

    const storeAta = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      payer,
      mint,
      store,
      false
    );
    
    await mintTo(
      provider.connection,
      payer,
      mint,
      payerAta.address,
      payer,
      //@ts-ignore
      1000000000000000000n,
    );
    return { mint, payerAta: payerAta.address, storeAta: storeAta.address };
  } catch (e) {
    console.log(e);
    return undefined;
  }
};

const prepareTokens = async (provider: AnchorProvider, payer: anchor.web3.Keypair, store: PublicKey): Promise<StablesInfo> => {
  // usdc GQ68QjtN1FTMUQp6W5rmYNHvFwpayfyysYWoX8zyU8tZ
  const usdcKeypair = anchor.web3.Keypair.fromSecretKey(Buffer.from(
    [225,222,156,122,107,181,201,154,182,33,123,95,37,133,115,191,20,18,122,37,246,185,228,170,34,57,146,226,140,142,160,64,228,201,117,225,17,141,247,86,50,78,246,212,163,41,5,81,184,144,159,177,193,236,73,174,131,21,90,159,240,169,8,194]
  ));

  // usdt 3xuWgjVWoaG3S5xHoJL8Ks1wyUszfD4SyFja3RM4g6x5
  const usdtKeypair = anchor.web3.Keypair.fromSecretKey(Buffer.from(
    [167,21,230,127,187,156,129,244,40,35,253,186,217,253,235,60,203,75,86,161,46,96,110,232,7,119,55,59,179,191,49,53,44,9,177,146,200,21,68,53,55,119,36,83,54,64,6,213,9,192,3,60,129,150,208,102,160,218,128,119,140,114,121,246]
  ));

  return { 
    usdc: await prepareStable(provider, payer, store, usdcKeypair), 
    usdt: await prepareStable(provider, payer, store, usdtKeypair),
  };
};

describe("Presale", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  async function fund(publicKey: anchor.web3.PublicKey) {
    const lamports = 2 * anchor.web3.LAMPORTS_PER_SOL;
    await provider.connection.requestAirdrop(publicKey, lamports);
    await new Promise( resolve => setTimeout(resolve, 3 * 1000) ); // Sleep 3s
  }

  async function generateKeypair() {
    const keypair = anchor.web3.Keypair.generate();
    await fund(keypair.publicKey);
    return keypair;
  }

  describe("Presale program", () => {
    // Configure the client to use the local cluster.
    const program: anchor.Program<Presale> = anchor.workspace.presale;
    const payer = anchor.web3.Keypair.fromSecretKey(Buffer.from(LocalAccountPrivateKeyBase58));
    const store = new PublicKey('4KcKQ6yTZydKhKuCi4THKnX9oFt1U3asBoqkMYDvbLvx');

    let joe_adviser: anchor.web3.Keypair;
    let joe_adviser_code = 'XYJ-XYJ';

    let bob_adviser: anchor.web3.Keypair;
    let bob_adviser_code = 'XYB-XYB';

    let stables: StablesInfo;

    before(async function () {
      joe_adviser = await generateKeypair();
      bob_adviser = await generateKeypair();
      await fund(payer.publicKey);

      stables = await prepareTokens(provider, payer, store);
    });
    
    it('should be able to init', async () => {
      const accounts = { payer: payer.publicKey };
      await program.methods.init().accounts(accounts).signers([payer]).rpc();

      const minCap = new anchor.BN('1000000000');
      const firstInterest = new anchor.BN('50000000');
      const secondInterest = new anchor.BN('50000000');

      let [presalePda, _] = anchor.web3.PublicKey.findProgramAddressSync([], program.programId);
      const presale = await program.account.presale.fetch(presalePda);
      expect(presale.minBuy.toString()).to.equal(minCap.toString());
      expect(presale.cPercent.toString()).to.equal(firstInterest.toString());
      expect(presale.tPercent.toString()).to.equal(secondInterest.toString());
      expect('none' in presale.status).to.equal(true);
      expect(presale.iteration).to.equal(-1);
    });
    
    it('should not be able to init twice', async () => {
      const payer = await generateKeypair();
      let [presalePda, _] = anchor.web3.PublicKey.findProgramAddressSync([], program.programId);
      const accounts = { payer: payer.publicKey, presale: presalePda };

      try {
        await program.methods.init().accounts(accounts).signers([payer]).rpc();
        expect.fail('Expected action to throw an error');
      } catch (err) {
        expect(err.message).includes('Transaction simulation failed: Error processing Instruction 0: custom program error: 0x0. ');
      }
    });
    
    it('should be able to set new Caps', async () => {
      const payer = anchor.web3.Keypair.fromSecretKey(Buffer.from(LocalAccountPrivateKeyBase58));
      let [presalePda, _] = anchor.web3.PublicKey.findProgramAddressSync([], program.programId);
      const accounts = { payer: payer.publicKey, presale: presalePda };
      const min = new anchor.BN(1000000000);

      await program.methods.setPresaleMinBuy(min).accounts(accounts).signers([payer]).rpc();
      const presale = await program.account.presale.fetch(presalePda);
      expect(presale.minBuy.toString()).to.equal(min.toString());
    });
    
    it('should not be able to set new Caps if Unauthorized Signer', async () => {
      const payer = await generateKeypair();
      let [presalePda, _] = anchor.web3.PublicKey.findProgramAddressSync([], program.programId);
      const accounts = { payer: payer.publicKey, presale: presalePda };
      const min = new anchor.BN(1000000000);

      try {
        await program.methods.setPresaleMinBuy(min).accounts(accounts).signers([payer]).rpc();
        expect.fail('Expected action to throw an error');
      } catch (err) {
        expect(err.error.errorMessage).to.equal('Unauthorized Signer');
      }
    });

    it('should be able to set new adviser interests', async () => {
      const payer = anchor.web3.Keypair.fromSecretKey(Buffer.from(LocalAccountPrivateKeyBase58));
      let [presalePda, _] = anchor.web3.PublicKey.findProgramAddressSync([], program.programId);
      const accounts = { payer: payer.publicKey, presale: presalePda };
      const first = new anchor.BN(100000000);
      const secondary = new anchor.BN(150000000);
      await program.methods.setPresaleAdviserInterest(first, secondary).accounts(accounts).signers([payer]).rpc();

      const presale = await program.account.presale.fetch(presalePda);
      expect(presale.cPercent.toString()).to.equal(first.toString());
      expect(presale.tPercent.toString()).to.equal(secondary.toString());
    });

    it('should not be able to set new adviser interests if Unauthorized Signer', async () => {
      const payer = await generateKeypair();
      let [presalePda, _] = anchor.web3.PublicKey.findProgramAddressSync([], program.programId);
      const accounts = { payer: payer.publicKey, presale: presalePda };
      try {
        const first = new anchor.BN(100000000);
        const secondary = new anchor.BN(150000000);
        await program.methods.setPresaleAdviserInterest(first, secondary).accounts(accounts).signers([payer]).rpc();
        expect.fail('Expected action to throw an error');
      } catch (err) {
        expect(err.error.errorMessage).to.equal('Unauthorized Signer');
      }
    });

    it('should not be able to enable presale if Unauthorized Signer', async () => {
      const payer = await generateKeypair();
      let [presalePda, _] = anchor.web3.PublicKey.findProgramAddressSync([], program.programId);
      const accounts = { payer: payer.publicKey, presale: presalePda };
      try {
        await program.methods.openPresale().accounts(accounts).signers([payer]).rpc();
        expect.fail('Expected action to throw an error');
      } catch (err) {
        expect(err.error.errorMessage).to.equal('Unauthorized Signer');
      }
    });

    it('should be able to enable presale', async () => {
      const payer = anchor.web3.Keypair.fromSecretKey(Buffer.from(LocalAccountPrivateKeyBase58));
      let [presalePda, _] = anchor.web3.PublicKey.findProgramAddressSync([], program.programId);
      const accounts = { payer: payer.publicKey, presale: presalePda };
      await program.methods.openPresale().accounts(accounts).signers([payer]).rpc();

      const presale = await program.account.presale.fetch(presalePda);
      expect('open' in presale.status).to.equal(true);
    });

    it('should not be able to enable presale twice', async () => {
      const payer = anchor.web3.Keypair.fromSecretKey(Buffer.from(LocalAccountPrivateKeyBase58));
      let [presalePda, _] = anchor.web3.PublicKey.findProgramAddressSync([], program.programId);
      const accounts = { payer: payer.publicKey, presale: presalePda };

      try {
        await program.methods.openPresale().accounts(accounts).signers([payer]).rpc();
        expect.fail('Expected action to throw an error');
      } catch (err) {
        expect(err.error.errorMessage).to.equal('Presale already open');
      }
    });

    it('should not be able to set new iteration if Unauthorized Signer', async () => {
      const payer = await generateKeypair();
      const accounts = { payer: payer.publicKey };

      try {
        let iteration1id = 1;
        let iteration1price = new anchor.BN(320000000);
        let iteration1totalSupply = new anchor.BN(1000000);
        await program.methods
          .createIteration(iteration1id, iteration1price, iteration1totalSupply)
          .accounts(accounts)
          .signers([payer])
          .rpc();

        expect.fail('Expected action to throw an error');
      } catch (err) {
        expect(err.error.errorMessage).to.equal('Unauthorized Signer');
      }
    });

    it('should be able to set new iterations', async () => {
      const payer = anchor.web3.Keypair.fromSecretKey(Buffer.from(LocalAccountPrivateKeyBase58));
      const accounts = { payer: payer.publicKey };

      let iteration1id = 1;
      let iteration1price = new anchor.BN(320000000);
      let iteration1totalSupply = new anchor.BN(1000000000000000);
      await program.methods
        .createIteration(iteration1id, iteration1price, iteration1totalSupply)
        .accounts(accounts)
        .signers([payer])
        .rpc();

      let iteration2id = 2;
      let iteration2price = new anchor.BN(340000000);
      let iteration2totalSupply = new anchor.BN(1000000000000000);
      await program.methods
          .createIteration(iteration2id, iteration2price, iteration2totalSupply)
          .accounts(accounts)
          .signers([payer])
          .rpc();

      let [iteration1Pda,] = anchor.web3.PublicKey.findProgramAddressSync([
        ROUND_TAG, Buffer.from('_'), i16ToBytesLE(iteration1id)
      ], program.programId);
      const iteration1 = await program.account.iteration.fetch(iteration1Pda);
      expect(iteration1.price.toString()).to.equal(iteration1price.toString());
      expect(iteration1.total.toString()).to.equal(iteration1totalSupply.toString());
      expect(iteration1.id).to.equal(iteration1id);

      let [iteration2Pda,] = anchor.web3.PublicKey.findProgramAddressSync([
        ROUND_TAG, Buffer.from('_'), i16ToBytesLE(iteration2id)
      ], program.programId);
      const iteration2 = await program.account.iteration.fetch(iteration2Pda);
      expect(iteration2.price.toString()).to.equal(iteration2price.toString());
      expect(iteration2.total.toString()).to.equal(iteration2totalSupply.toString());
      expect(iteration2.id).to.equal(iteration2id);
    });

    it('should be able to set new iteration price', async () => {
      const payer = anchor.web3.Keypair.fromSecretKey(Buffer.from(LocalAccountPrivateKeyBase58));
      let iteration1id = 1;
      let [iteration1Pda,] = anchor.web3.PublicKey.findProgramAddressSync([
        ROUND_TAG, Buffer.from('_'), i16ToBytesLE(iteration1id)
      ], program.programId);
      const accounts = { payer: payer.publicKey, iteration: iteration1Pda };

      let iteration1price = new anchor.BN(340000000);
      await program.methods
        .setIterationPrice(iteration1price)
        .accounts(accounts)
        .signers([payer])
        .rpc();

      const iteration1 = await program.account.iteration.fetch(iteration1Pda);
      expect(iteration1.price.toString()).to.equal(iteration1price.toString());
    });

    it('should not be able to set new iteration price if Unauthorized Signer', async () => {
      const payer = await generateKeypair()
      let iteration1id = 1;
      let [iteration1Pda,] = anchor.web3.PublicKey.findProgramAddressSync([
        ROUND_TAG, Buffer.from('_'), i16ToBytesLE(iteration1id)
      ], program.programId);
      const accounts = { payer: payer.publicKey, iteration: iteration1Pda };

      let iteration1price = new anchor.BN(320000000);
      try {
        await program.methods.setIterationPrice(iteration1price).accounts(accounts).signers([payer]).rpc();
        expect.fail('Expected action to throw an error');
      } catch (err) {
        expect(err.error.errorMessage).to.equal('Unauthorized Signer');
      }
    });

    it('should be able to set new iteration supply', async () => {
      const payer = anchor.web3.Keypair.fromSecretKey(Buffer.from(LocalAccountPrivateKeyBase58));
      let iteration1id = 1;
      let [iteration1Pda,] = anchor.web3.PublicKey.findProgramAddressSync([
        ROUND_TAG, Buffer.from('_'), i16ToBytesLE(iteration1id)
      ], program.programId);
      const accounts = { payer: payer.publicKey, iteration: iteration1Pda };

      let iteration1totalSupply = new anchor.BN(1000000000000001);
      await program.methods.setIterationTotal(iteration1totalSupply).accounts(accounts).signers([payer]).rpc();

      const iteration1 = await program.account.iteration.fetch(iteration1Pda);
      expect(iteration1.total.toString()).to.equal(iteration1totalSupply.toString());
    });

    it('should not be able to set new iteration supply if Unauthorized Signer', async () => {
      const payer = await generateKeypair();
      let iteration1id = 1;
      let [iteration1Pda,] = anchor.web3.PublicKey.findProgramAddressSync([
        ROUND_TAG, Buffer.from('_'), i16ToBytesLE(iteration1id)
      ], program.programId);
      const accounts = { payer: payer.publicKey, iteration: iteration1Pda };

      let iteration1totalSupply = new anchor.BN(1000000000000001);
      try {
        await program.methods.setIterationTotal(iteration1totalSupply).accounts(accounts).signers([payer]).rpc();
        expect.fail('Expected action to throw an error');
      } catch (err) {
        expect(err.error.errorMessage).to.equal('Unauthorized Signer');
      }
    });

    it('should be able to set new adviser', async () => {
      const payer = anchor.web3.Keypair.fromSecretKey(Buffer.from(LocalAccountPrivateKeyBase58));
      const accounts = { payer: payer.publicKey };

      let firstInterest = new anchor.BN(100000000);
      let secondInterest = new anchor.BN(100000000);
      await program.methods
        .initAdviser(joe_adviser_code, firstInterest, secondInterest)
        .accounts(accounts)
        .signers([payer])
        .rpc();

      let [adviserPda,] = anchor.web3.PublicKey.findProgramAddressSync([
        REF_TAG, Buffer.from('_'), Buffer.from(joe_adviser_code)
      ], program.programId);
      const adviser = await program.account.adviser.fetch(adviserPda);
      expect(adviser.cPercent.toString()).to.equal(firstInterest.toString());
      expect(adviser.tPercent.toString()).to.equal(secondInterest.toString());
    });

    it('should not be able to set new adviser if Unauthorized Signer', async () => {
      const payer = await generateKeypair();
      const accounts = { payer: payer.publicKey };

      let firstadviserReward = new anchor.BN(100000000);
      let secondaryadviserReward = new anchor.BN(100000000);
      try {
        await program.methods
          .initAdviser(bob_adviser_code, firstadviserReward, secondaryadviserReward)
          .accounts(accounts)
          .signers([payer])
          .rpc();
        expect.fail('Expected action to throw an error');
      } catch (err) {
        expect(err.error.errorMessage).to.equal('Unauthorized Signer');
      }
    });

    it('should not be able to disable if Unauthorized Signer', async () => {
      const payer = await generateKeypair();
      let [adviserPda,] = anchor.web3.PublicKey.findProgramAddressSync([
        REF_TAG, Buffer.from('_'), Buffer.from(joe_adviser_code)
      ], program.programId);
      const accounts = { payer: payer.publicKey, adviser: adviserPda };

      try {
        await program.methods.disableAdviser().accounts(accounts).signers([payer]).rpc();
        expect.fail('Expected action to throw an error');
      } catch (err) {
        expect(err.error.errorMessage).to.equal('Unauthorized Signer');
      }
    });

    it('should be able to disable adviser', async () => {
      const payer = anchor.web3.Keypair.fromSecretKey(Buffer.from(LocalAccountPrivateKeyBase58));
      let [adviserPda,] = anchor.web3.PublicKey.findProgramAddressSync([
        REF_TAG, Buffer.from('_'), Buffer.from(joe_adviser_code)
      ], program.programId);
      const accounts = { payer: payer.publicKey, adviser: adviserPda };

      await program.methods.disableAdviser().accounts(accounts).signers([payer]).rpc();

      const adviser = await program.account.adviser.fetch(adviserPda);
      expect(adviser.enabled).to.equal(false);
    });

    it('should not be able to enable if Unauthorized Signer', async () => {
      const payer = await generateKeypair();
      let [adviserPda,] = anchor.web3.PublicKey.findProgramAddressSync([
        REF_TAG, Buffer.from('_'), Buffer.from(joe_adviser_code)
      ], program.programId);
      const accounts = { payer: payer.publicKey, adviser: adviserPda };

      try {
        await program.methods.enableAdviser().accounts(accounts).signers([payer]).rpc();
        expect.fail('Expected action to throw an error');
      } catch (err) {
        expect(err.error.errorMessage).to.equal('Unauthorized Signer');
      }
    });

    it('should be able to enable adviser', async () => {
      const payer = anchor.web3.Keypair.fromSecretKey(Buffer.from(LocalAccountPrivateKeyBase58));
      let [adviserPda,] = anchor.web3.PublicKey.findProgramAddressSync([
        REF_TAG, Buffer.from('_'), Buffer.from(joe_adviser_code)
      ], program.programId);
      const accounts = { payer: payer.publicKey, adviser: adviserPda };

      await program.methods.enableAdviser().accounts(accounts).signers([payer]).rpc();

      const adviser = await program.account.adviser.fetch(adviserPda);
      expect(adviser.enabled).to.equal(true);
    });

    it('should be able to set new adviser interest', async () => {
      const payer = anchor.web3.Keypair.fromSecretKey(Buffer.from(LocalAccountPrivateKeyBase58));
      let [adviserPda,] = anchor.web3.PublicKey.findProgramAddressSync([
        REF_TAG, Buffer.from('_'), Buffer.from(joe_adviser_code)
      ], program.programId);
      const accounts = { payer: payer.publicKey, adviser: adviserPda };

      let firstadviserInterest = new anchor.BN(150000000);
      let secondaryadviserInterest = new anchor.BN(150000000);
      await program.methods
        .setAdviserInterest(firstadviserInterest, secondaryadviserInterest)
        .accounts(accounts)
        .signers([payer])
        .rpc();

      const adviser = await program.account.adviser.fetch(adviserPda);
      expect(adviser.cPercent.toString()).to.equal(firstadviserInterest.toString());
      expect(adviser.tPercent.toString()).to.equal(secondaryadviserInterest.toString());
    });

    it('should not be able to set new adviser interest if Unauthorized Signer', async () => {
      const payer = await generateKeypair();
      let [adviserPda,] = anchor.web3.PublicKey.findProgramAddressSync([
        REF_TAG, Buffer.from('_'), Buffer.from(joe_adviser_code)
      ], program.programId);
      const accounts = { payer: payer.publicKey, adviser: adviserPda };

      try {
        let firstadviserInterest = new anchor.BN(150000000);
        let secondaryadviserInterest = new anchor.BN(150000000);
        await program.methods
          .setAdviserInterest(firstadviserInterest, secondaryadviserInterest)
          .accounts(accounts)
          .signers([payer])
          .rpc();
        expect.fail('Expected action to throw an error');
      } catch (err) {
        expect(err.error.errorMessage).to.equal('Unauthorized Signer');
      }
    });

    it('should be able to enable iteration', async () => {
      const payer = anchor.web3.Keypair.fromSecretKey(Buffer.from(LocalAccountPrivateKeyBase58));
      let iteration1id = 1;
      let [iteration1Pda,] = anchor.web3.PublicKey.findProgramAddressSync([
        ROUND_TAG, Buffer.from('_'), i16ToBytesLE(iteration1id)
      ], program.programId);
      let [presalePda, _] = anchor.web3.PublicKey.findProgramAddressSync([
      ], program.programId);

      const accounts = {
        payer: payer.publicKey,
        iteration: iteration1Pda,
        presale: presalePda
      };

      await program.methods.openIteration().accounts(accounts).signers([payer]).rpc();

      const iteration1 = await program.account.iteration.fetch(iteration1Pda);
      expect('open' in iteration1.status).to.equal(true);
    });

    it('should be able to enable/close iteration', async () => {
      const payer = anchor.web3.Keypair.fromSecretKey(Buffer.from(LocalAccountPrivateKeyBase58));

      let iteration2id = 2;
      let [iteration2Pda,] = anchor.web3.PublicKey.findProgramAddressSync([
        ROUND_TAG, Buffer.from('_'), i16ToBytesLE(iteration2id)
      ], program.programId);

      let [presalePda, _] = anchor.web3.PublicKey.findProgramAddressSync([
      ], program.programId);

      const accounts = {
        payer: payer.publicKey,
        iteration: iteration2Pda,
        presale: presalePda,
      };

      await program.methods.openIteration().accounts(accounts).signers([payer]).rpc();

      const iteration2 = await program.account.iteration.fetch(iteration2Pda);
      expect('open' in iteration2.status).to.equal(true);
    });

    it('should be able to deposit_sol to iteration with joe adviser', async () => {
      let iteration2id = 2;

      let [presalePda, _] = anchor.web3.PublicKey.findProgramAddressSync([
      ], program.programId);

      let [iteration2Pda,] = anchor.web3.PublicKey.findProgramAddressSync([
        ROUND_TAG, Buffer.from('_'), i16ToBytesLE(iteration2id)
      ], program.programId);

      let [userPda,] = anchor.web3.PublicKey.findProgramAddressSync([
        USER_TAG, Buffer.from('_'), payer.publicKey.toBuffer()
      ], program.programId);

      let [adviserPda,] = anchor.web3.PublicKey.findProgramAddressSync([
        REF_TAG, Buffer.from('_'), Buffer.from(joe_adviser_code)
      ], program.programId);

      const priceUpdate = new PublicKey('7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE');
      const accounts = {
        payer: payer.publicKey,
        iteration: iteration2Pda,
        presale: presalePda,
        storeInfo: store,
        priceUpdate: priceUpdate,
        buyer: userPda,
        adviser: adviserPda,
      };

      const amount = new anchor.BN(500000000);
      await program.methods
        .buySol(joe_adviser_code, amount)
        .accounts(accounts)
        .signers([payer])
        .rpc();

      const presale = await program.account.presale.fetch(presalePda);
      const iteration2 = await program.account.iteration.fetch(iteration2Pda);
      const buyer = await program.account.buyer.fetch(userPda);
      const adviser = await program.account.adviser.fetch(adviserPda);
      
      const precision = new anchor.BN(1000000000);
      const usd = new anchor.BN(72);
      const firstRew = new anchor.BN(150000000);
      const secondRew = new anchor.BN(150000000);
      const tokenAmount = usd.mul(precision).mul(precision).div(iteration2.price);
      const storeBalance = await provider.connection.getBalance(store);
      const adviserBalance = await provider.connection.getBalance(adviserPda);

      expect('open' in iteration2.status).to.equal(true);

      expect(tokenAmount.toString()).to.equal(presale.totalReleased.toString());
      expect(tokenAmount.toString()).to.equal(iteration2.sold.toString());
      expect(tokenAmount.toString()).to.equal(buyer.balance.toString());
      
      expect(adviser.solReward.toString()).to.equal((amount.mul(firstRew).div(precision)).toString());
      expect(adviser.tokenReward.toString()).to.equal((tokenAmount.mul(secondRew).div(precision)).toString());
      expect(storeBalance.toString()).to.equal((amount.sub(adviser.solReward)).toString());
      expect(adviserBalance).to.be.greaterThanOrEqual(adviser.solReward.toNumber());
    });

    it('should be able to deposit_usdc to iteration with bob adviser', async () => {
      let iteration2id = 2;

      let [presalePda, _] = anchor.web3.PublicKey.findProgramAddressSync([
      ], program.programId);

      let [iteration2Pda,] = anchor.web3.PublicKey.findProgramAddressSync([
        ROUND_TAG, Buffer.from('_'), i16ToBytesLE(iteration2id)
      ], program.programId);

      let [userPda,] = anchor.web3.PublicKey.findProgramAddressSync([
        USER_TAG, Buffer.from('_'), payer.publicKey.toBuffer()
      ], program.programId);

      let [adviserPda,] = anchor.web3.PublicKey.findProgramAddressSync([
        REF_TAG, Buffer.from('_'), Buffer.from(bob_adviser_code)
      ], program.programId);

      const adviserPdaAta = await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        stables.usdc.mint,
        adviserPda,
        true,
      );

      const amount = new anchor.BN(50000000); // $50
      await program.methods
        .buyUsdc(bob_adviser_code, amount)
        .accounts({
          payer: payer.publicKey,
          iteration: iteration2Pda,
          presale: presalePda,
          buyer: userPda,
          adviser: adviserPda,
          buyerAta: stables.usdc.payerAta,
          storeAta: stables.usdc.storeAta,
          adviserPdaAta: adviserPdaAta.address,
        })
        .signers([payer])
        .rpc();

      const adviser = await program.account.adviser.fetch(adviserPda);
      const iteration2 = await program.account.iteration.fetch(iteration2Pda);

      const precision = new anchor.BN(1000000000);
      const stablePresicion = new anchor.BN(1000000);
      const firstRew = new anchor.BN(100000000);
      const secondRew = new anchor.BN(150000000);

      const tokenAmount = amount.mul(precision).mul(precision).div(iteration2.price).div(stablePresicion);

      expect(adviser.usdcReward.toString()).to.equal((amount.mul(firstRew).div(precision)).toString());
      expect(adviser.tokenReward.toString()).to.equal((tokenAmount.mul(secondRew).div(precision)).toString());
    });

    it('should be able to deposit_usdt to iteration with bob adviser', async () => {
      let iteration2id = 2;

      let [presalePda, _] = anchor.web3.PublicKey.findProgramAddressSync([
      ], program.programId);

      let [iteration2Pda,] = anchor.web3.PublicKey.findProgramAddressSync([
        ROUND_TAG, Buffer.from('_'), i16ToBytesLE(iteration2id)
      ], program.programId);

      let [userPda,] = anchor.web3.PublicKey.findProgramAddressSync([
        USER_TAG, Buffer.from('_'), payer.publicKey.toBuffer()
      ], program.programId);

      let [adviserPda,] = anchor.web3.PublicKey.findProgramAddressSync([
        REF_TAG, Buffer.from('_'), Buffer.from(bob_adviser_code)
      ], program.programId);

      const adviserPdaAta = await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        stables.usdt.mint,
        adviserPda,
        true,
      );

      const amount = new anchor.BN(50000000); // $50
      await program.methods
        .buyUsdt(bob_adviser_code, amount)
        .accounts({
          payer: payer.publicKey,
          iteration: iteration2Pda,
          presale: presalePda,
          buyer: userPda,
          adviser: adviserPda,
          buyerAta: stables.usdt.payerAta,
          storeAta: stables.usdt.storeAta,
          adviserPdaAta: adviserPdaAta.address,
        })
        .signers([payer])
        .rpc();

      const adviser = await program.account.adviser.fetch(adviserPda);
      const iteration2 = await program.account.iteration.fetch(iteration2Pda);

      const precision = new anchor.BN(1000000000);
      const stablePresicion = new anchor.BN(1000000);
      const firstRew = new anchor.BN(100000000);
      const secondRew = new anchor.BN(150000000);

      const tokenAmount = amount.mul(precision).mul(precision).div(iteration2.price).div(stablePresicion);

      expect(adviser.usdtReward.toString()).to.equal((amount.mul(firstRew).div(precision)).toString());
      expect(adviser.tokenReward.toString()).to.equal((tokenAmount.mul(secondRew).div(precision).mul(new anchor.BN(2))).toString());
    });

    it('should be able to deposit_sol to iteration', async () => {
      let iteration2id = 2;
      let [presalePda, _] = anchor.web3.PublicKey.findProgramAddressSync([
      ], program.programId);

      let [iteration2Pda,] = anchor.web3.PublicKey.findProgramAddressSync([
        ROUND_TAG, Buffer.from('_'), i16ToBytesLE(iteration2id)
      ], program.programId);

      let [userPda,] = anchor.web3.PublicKey.findProgramAddressSync([
        USER_TAG, Buffer.from('_'), bob_adviser.publicKey.toBuffer()
      ], program.programId);

      let [adviserPda,] = anchor.web3.PublicKey.findProgramAddressSync([
        REF_TAG, Buffer.from('_'), Buffer.from("")
      ], program.programId);

      const priceUpdate = new PublicKey('7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE');

      const accounts = {
        payer: bob_adviser.publicKey,
        iteration: iteration2Pda,
        presale: presalePda,
        storeInfo: store,
        priceUpdate: priceUpdate,
        buyer: userPda,
        adviser: adviserPda,
      };

      const amount = new anchor.BN(1000000000);
      await program.methods
        .buySol("", amount)
        .accounts(accounts)
        .signers([bob_adviser])
        .rpc();

        const iteration2 = await program.account.iteration.fetch(iteration2Pda);
        const buyer = await program.account.buyer.fetch(userPda);
        
        const precision = new anchor.BN(1000000000);
        const usd = new anchor.BN(144);
        const tokenAmount = usd.mul(precision).mul(precision).div(iteration2.price);
  
        expect(tokenAmount.toString()).to.equal(buyer.balance.toString());        
    });
    
    it('should not be able to claim_sol adviser interest with invalid sign', async () => {
      const payer = anchor.web3.Keypair.fromSecretKey(Buffer.from(LocalAccountPrivateKeyBase58));
      const deadline = Math.floor(new Date().getTime() / 1000) - 50; // 10m from now

      const message = Uint8Array.from(Buffer.from(`${joe_adviser_code}${joe_adviser.publicKey}${deadline}`));
      const signature: Uint8Array = await ed.sign(message, payer.secretKey.slice(0, 32));

      let [adviserPda,] = anchor.web3.PublicKey.findProgramAddressSync([
        REF_TAG, Buffer.from('_'), Buffer.from(joe_adviser_code)
      ], program.programId);

      const tx = new anchor.web3.Transaction()
        .add(
          // Ed25519 instruction
          anchor.web3.Ed25519Program.createInstructionWithPublicKey({
            publicKey: payer.publicKey.toBytes(),
            message: message,
            signature: signature,
          })
        )
        .add(
          await program.methods
            .claimSol(joe_adviser_code, new anchor.BN(deadline), Array.from(signature), 0)
            .accounts({ payer: joe_adviser.publicKey, adviser: adviserPda })
            .signers([joe_adviser])
            .instruction()
        );

      const { lastValidBlockHeight, blockhash } = await provider.connection.getLatestBlockhash();
      tx.lastValidBlockHeight = lastValidBlockHeight;
      tx.recentBlockhash = blockhash;
      tx.feePayer = joe_adviser.publicKey;

      try {
        tx.sign(joe_adviser);
        const hash = await provider.connection.sendRawTransaction(tx.serialize());
        const confirmation = await provider.connection.confirmTransaction(hash, 'confirmed');
        if (confirmation.value.err) {
          throw confirmation.value.err;
        }
        expect.fail('Expected action to throw an error');
      } catch (err) {
        expect(err.message).to.contain('ExpiredSignature'); 
      }
    });

    it('should be able to claim_sol adviser interest', async () => {
      const deadline = Math.floor(new Date().getTime() / 1000) + 600; // 10m from now

      const message = Uint8Array.from(Buffer.from(`${joe_adviser_code}${joe_adviser.publicKey}${deadline}`));
      const signature: Uint8Array = await ed.sign(message, payer.secretKey.slice(0, 32));
      const adviserBalance1 = await provider.connection.getBalance(joe_adviser.publicKey);

      let [adviserPda,] = anchor.web3.PublicKey.findProgramAddressSync([
        REF_TAG, Buffer.from('_'), Buffer.from(joe_adviser_code)
      ], program.programId);

      const tx = new anchor.web3.Transaction()
        .add(
          // Ed25519 instruction
          anchor.web3.Ed25519Program.createInstructionWithPublicKey({
            publicKey: payer.publicKey.toBytes(),
            message: message,
            signature: signature,
          })
        )
        .add(
          await program.methods
            .claimSol(joe_adviser_code, new anchor.BN(deadline), Array.from(signature), 0)
            .accounts({ payer: joe_adviser.publicKey, adviser: adviserPda })
            .instruction()
        );

      const { lastValidBlockHeight, blockhash } = await provider.connection.getLatestBlockhash();
      tx.lastValidBlockHeight = lastValidBlockHeight;
      tx.recentBlockhash = blockhash;
      tx.feePayer = joe_adviser.publicKey;

      tx.sign(joe_adviser);
      const hash = await provider.connection.sendRawTransaction(tx.serialize());
      const confirmation = await provider.connection.confirmTransaction(hash, 'confirmed');
      if (confirmation.value.err) {
        throw confirmation.value.err;
      }
      
      const adviserBalance2 = await provider.connection.getBalance(joe_adviser.publicKey);
      const adviser = await program.account.adviser.fetch(adviserPda);
      expect(adviser.solReward.toString()).to.equal('0');
      expect((adviserBalance2 - adviserBalance1) / 1000000000).to.approximately(75000000 / 1000000000, 0.0001);
    });

    it('should be able to claim_usdc adviser interest', async () => {
      const deadline = Math.floor(new Date().getTime() / 1000) + 600; // 10m from now

      const message = Uint8Array.from(Buffer.from(`${bob_adviser_code}${bob_adviser.publicKey}${deadline}`));
      const signature: Uint8Array = await ed.sign(message, payer.secretKey.slice(0, 32));

      let [adviserPda,] = anchor.web3.PublicKey.findProgramAddressSync([
        REF_TAG, Buffer.from('_'), Buffer.from(bob_adviser_code)
      ], program.programId);

      const adviserAta = await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        stables.usdc.mint,
        bob_adviser.publicKey,
        false,
      );

      const adviserPdaAta = await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        stables.usdc.mint,
        adviserPda,
        true,
      );

      const tx = new anchor.web3.Transaction()
        .add(
          // Ed25519 instruction
          anchor.web3.Ed25519Program.createInstructionWithPublicKey({
            publicKey: payer.publicKey.toBytes(),
            message: message,
            signature: signature,
          })
        )
        .add(
          await program.methods
            .claimUsdc(bob_adviser_code, new anchor.BN(deadline), Array.from(signature), 0)
            .accounts({ 
              payer: bob_adviser.publicKey,
              adviser: adviserPda,
              adviserAta: adviserAta.address,
              adviserPdaAta: adviserPdaAta.address,
            })
            .instruction()
        );

      const { lastValidBlockHeight, blockhash } = await provider.connection.getLatestBlockhash();
      tx.lastValidBlockHeight = lastValidBlockHeight;
      tx.recentBlockhash = blockhash;
      tx.feePayer = bob_adviser.publicKey;

      tx.sign(bob_adviser);
      const hash = await provider.connection.sendRawTransaction(tx.serialize());
      const confirmation = await provider.connection.confirmTransaction(hash, 'confirmed');
      if (confirmation.value.err) {
        throw confirmation.value.err;
      }
      
      const adviser = await program.account.adviser.fetch(adviserPda);
      const usdcAccount = await getAccount(provider.connection, adviserAta.address);
      expect(usdcAccount.amount.toString()).to.equal('5000000');
      expect(adviser.usdcReward.toString()).to.equal('0');
    });

    it('should be able to claim_usdt adviser interest', async () => {
      const deadline = Math.floor(new Date().getTime() / 1000) + 600; // 10m from now

      const message = Uint8Array.from(Buffer.from(`${bob_adviser_code}${bob_adviser.publicKey}${deadline}`));
      const signature: Uint8Array = await ed.sign(message, payer.secretKey.slice(0, 32));

      let [adviserPda,] = anchor.web3.PublicKey.findProgramAddressSync([
        REF_TAG, Buffer.from('_'), Buffer.from(bob_adviser_code)
      ], program.programId);

      const adviserAta = await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        stables.usdt.mint,
        bob_adviser.publicKey,
        false,
      );

      const adviserPdaAta = await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        stables.usdt.mint,
        adviserPda,
        true,
      );

      const tx = new anchor.web3.Transaction()
        .add(
          // Ed25519 instruction
          anchor.web3.Ed25519Program.createInstructionWithPublicKey({
            publicKey: payer.publicKey.toBytes(),
            message: message,
            signature: signature,
          })
        )
        .add(
          await program.methods
            .claimUsdt(bob_adviser_code, new anchor.BN(deadline), Array.from(signature), 0)
            .accounts({ 
              payer: bob_adviser.publicKey,
              adviser: adviserPda,
              adviserAta: adviserAta.address,
              adviserPdaAta: adviserPdaAta.address,
            })
            .instruction()
        );

      const { lastValidBlockHeight, blockhash } = await provider.connection.getLatestBlockhash();
      tx.lastValidBlockHeight = lastValidBlockHeight;
      tx.recentBlockhash = blockhash;
      tx.feePayer = bob_adviser.publicKey;

      tx.sign(bob_adviser);
      const hash = await provider.connection.sendRawTransaction(tx.serialize());
      const confirmation = await provider.connection.confirmTransaction(hash, 'confirmed');
      if (confirmation.value.err) {
        throw confirmation.value.err;
      }
      
      const adviser = await program.account.adviser.fetch(adviserPda);
      const usdtAccount = await getAccount(provider.connection, adviserAta.address);
      expect(usdtAccount.amount.toString()).to.equal('5000000');
      expect(adviser.usdtReward.toString()).to.equal('0');
    });
    
    it('should not be able to claim_sol bob adviser interest by joe signer', async () => {
      const payer = anchor.web3.Keypair.fromSecretKey(Buffer.from(LocalAccountPrivateKeyBase58));
      const message = Uint8Array.from(Buffer.from(`${bob_adviser_code}${bob_adviser.publicKey}`));
      const signature: Uint8Array = await ed.sign(message, payer.secretKey.slice(0, 32));
      const deadline = Math.floor(new Date().getTime() / 1000) + 600; // 10m from now

      let [adviserPda,] = anchor.web3.PublicKey.findProgramAddressSync([
        REF_TAG, Buffer.from('_'), Buffer.from(joe_adviser_code)
      ], program.programId);

      const tx = new anchor.web3.Transaction()
        .add(
          // Ed25519 instruction
          anchor.web3.Ed25519Program.createInstructionWithPublicKey({
            publicKey: payer.publicKey.toBytes(),
            message: message,
            signature: signature,
          })
        )
        .add(
          await program.methods
            .claimSol(bob_adviser_code, new anchor.BN(deadline), Array.from(signature), 0)
            .accounts({ payer: bob_adviser.publicKey, adviser: adviserPda })
            .signers([joe_adviser])
            .instruction()
        );

      const { lastValidBlockHeight, blockhash } = await provider.connection.getLatestBlockhash();
      tx.lastValidBlockHeight = lastValidBlockHeight;
      tx.recentBlockhash = blockhash;
      tx.feePayer = joe_adviser.publicKey;

      try {
        tx.sign(joe_adviser);
        const hash = await provider.connection.sendRawTransaction(tx.serialize());
        await provider.connection.confirmTransaction(hash, 'confirmed');
        expect.fail('Expected action to throw an error');
      } catch(err) {
        expect(err.message).to.contain('Signature verification failed.'); 
      }
    });
    
    it('should not be able to close presale if Unauthorized Signer', async () => {
      const payer = await generateKeypair();
      let [presalePda, _] = anchor.web3.PublicKey.findProgramAddressSync([], program.programId);
      const accounts = { payer: payer.publicKey, presale: presalePda };
      try {
        await program.methods.closePresale().accounts(accounts).signers([payer]).rpc();
        expect.fail('Expected action to throw an error');
      } catch (err) {
        expect(err.error.errorMessage).to.equal('Unauthorized Signer');
      }
    });

    it('should be able to close presale', async () => {
      const payer = anchor.web3.Keypair.fromSecretKey(Buffer.from(LocalAccountPrivateKeyBase58));
      let [presalePda, _] = anchor.web3.PublicKey.findProgramAddressSync([], program.programId);
      const accounts = { payer: payer.publicKey, presale: presalePda };
      await program.methods.closePresale().accounts(accounts).signers([payer]).rpc();

      const presale = await program.account.presale.fetch(presalePda);
      expect('closed' in presale.status).to.equal(true);
    });
  });
});
