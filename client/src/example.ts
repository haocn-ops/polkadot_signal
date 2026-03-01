import SignalKeysClient from './SignalKeysClient';

async function main() {
  console.log('🔗 Connecting to Polkadot Signal node...');
  const client = await SignalKeysClient.connect('ws://localhost:9944');

  try {
    const alice = client.createAccountFromUri('//Alice');
    const bob = client.createAccountFromUri('//Bob');

    console.log('\n📝 Registering Alice identity...');
    
    const aliceIdentityKey = generateRandomKey(32);
    const aliceSignedPrekey = generateRandomKey(32);
    const alicePrekeySignature = generateRandomKey(64);

    const aliceTxHash = await client.registerIdentity(
      alice,
      aliceIdentityKey,
      aliceSignedPrekey,
      alicePrekeySignature
    );
    console.log(`✅ Alice registered! Tx hash: ${aliceTxHash}`);

    console.log('\n📝 Registering Bob identity...');
    const bobIdentityKey = generateRandomKey(32);
    const bobSignedPrekey = generateRandomKey(32);
    const bobPrekeySignature = generateRandomKey(64);

    const bobTxHash = await client.registerIdentity(
      bob,
      bobIdentityKey,
      bobSignedPrekey,
      bobPrekeySignature
    );
    console.log(`✅ Bob registered! Tx hash: ${bobTxHash}`);

    console.log('\n📦 Adding one-time prekeys for Alice...');
    const alicePrekeys = Array.from({ length: 10 }, () => generateRandomKey(32));
    const prekeyTxHash = await client.addOneTimePrekeys(alice, alicePrekeys);
    console.log(`✅ Added 10 prekeys! Tx hash: ${prekeyTxHash}`);

    console.log('\n🔍 Querying Alice identity...');
    const aliceIdentity = await client.getIdentity(alice.address);
    if (aliceIdentity) {
      console.log('Alice identity found:');
      console.log(`  Identity Key: ${bufferToHex(aliceIdentity.identityKey)}`);
      console.log(`  Signed Prekey: ${bufferToHex(aliceIdentity.signedPrekey)}`);
      console.log(`  Registration Block: ${aliceIdentity.registrationBlock}`);
    }

    console.log('\n📊 Checking prekey status...');
    const remainingCount = await client.getRemainingPrekeyCount(alice.address);
    console.log(`Alice has ${remainingCount} remaining prekeys`);

    const hasPrekeys = await client.hasOneTimePrekeys(alice.address);
    console.log(`Alice has prekeys available: ${hasPrekeys}`);

    console.log('\n🔐 Simulating X3DH key exchange...');
    console.log('Bob wants to start a conversation with Alice...');
    console.log('Bob fetches Alice\'s key bundle from chain...');

    const bobFetchesAlice = await client.getIdentity(alice.address);
    const oneTimePrekey = await client.getOneTimePrekey(alice.address);
    
    if (bobFetchesAlice && oneTimePrekey) {
      console.log('Bob received:');
      console.log(`  - Alice\'s identity key: ${bufferToHex(bobFetchesAlice.identityKey)}`);
      console.log(`  - Alice\'s signed prekey: ${bufferToHex(bobFetchesAlice.signedPrekey)}`);
      console.log(`  - One-time prekey #${oneTimePrekey.keyId}: ${bufferToHex(oneTimePrekey.publicKey)}`);
      console.log('\n✅ Bob can now perform X3DH key agreement!');
      console.log('   1. DH1 = DH(IK_B, SPK_A)');
      console.log('   2. DH2 = DH(EK_B, IK_A)');
      console.log('   3. DH3 = DH(EK_B, SPK_A)');
      console.log('   4. DH4 = DH(EK_B, OPK_A)');
      console.log('   5. SK = KDF(DH1 || DH2 || DH3 || DH4)');
    }

    const newCount = await client.getRemainingPrekeyCount(alice.address);
    console.log(`\n📊 Alice now has ${newCount} remaining prekeys`);

  } finally {
    await client.disconnect();
    console.log('\n👋 Disconnected');
  }
}

function generateRandomKey(length: number): Uint8Array {
  const key = new Uint8Array(length);
  for (let i = 0; i < length; i++) {
    key[i] = Math.floor(Math.random() * 256);
  }
  return key;
}

function bufferToHex(buffer: Uint8Array): string {
  return Array.from(buffer)
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
}

main().catch(console.error);
