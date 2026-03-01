import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import type { AccountId, Hash } from '@polkadot/types/interfaces';
import type { Option, Bytes } from '@polkadot/types';

export interface IdentityKeyBundle {
  identityKey: Uint8Array;
  signedPrekey: Uint8Array;
  prekeySignature: Uint8Array;
  registrationBlock: number;
  updatedAt: number;
}

export interface OneTimePrekey {
  keyId: number;
  publicKey: Uint8Array;
}

export class SignalKeysClient {
  private api: ApiPromise;
  private keyring: Keyring;

  private constructor(api: ApiPromise) {
    this.api = api;
    this.keyring = new Keyring({ type: 'sr25519' });
  }

  static async connect(wsEndpoint: string = 'ws://localhost:9944'): Promise<SignalKeysClient> {
    const provider = new WsProvider(wsEndpoint);
    const api = await ApiPromise.create({ provider });
    return new SignalKeysClient(api);
  }

  async disconnect(): Promise<void> {
    await this.api.disconnect();
  }

  async registerIdentity(
    account: ReturnType<typeof this.keyring.addFromUri>,
    identityKey: Uint8Array,
    signedPrekey: Uint8Array,
    prekeySignature: Uint8Array
  ): Promise<Hash> {
    const tx = this.api.tx.signalKeys.registerIdentity(
      Array.from(identityKey),
      Array.from(signedPrekey),
      Array.from(prekeySignature)
    );

    return new Promise((resolve, reject) => {
      tx.signAndSend(account, ({ status, dispatchError }) => {
        if (dispatchError) {
          if (dispatchError.isModule) {
            const decoded = this.api.registry.findMetaError(dispatchError.asModule);
            reject(new Error(`${decoded.section}.${decoded.name}: ${decoded.docs.join(' ')}`));
          } else {
            reject(new Error(dispatchError.toString()));
          }
        } else if (status.isInBlock) {
          resolve(status.asInBlock);
        }
      });
    });
  }

  async addOneTimePrekeys(
    account: ReturnType<typeof this.keyring.addFromUri>,
    prekeys: Uint8Array[]
  ): Promise<Hash> {
    const prekeysArray = prekeys.map(pk => Array.from(pk));
    
    const tx = this.api.tx.signalKeys.addOneTimePrekeys(prekeysArray);

    return new Promise((resolve, reject) => {
      tx.signAndSend(account, ({ status, dispatchError }) => {
        if (dispatchError) {
          if (dispatchError.isModule) {
            const decoded = this.api.registry.findMetaError(dispatchError.asModule);
            reject(new Error(`${decoded.section}.${decoded.name}: ${decoded.docs.join(' ')}`));
          } else {
            reject(new Error(dispatchError.toString()));
          }
        } else if (status.isInBlock) {
          resolve(status.asInBlock);
        }
      });
    });
  }

  async removeIdentity(
    account: ReturnType<typeof this.keyring.addFromUri>
  ): Promise<Hash> {
    const tx = this.api.tx.signalKeys.removeIdentity();

    return new Promise((resolve, reject) => {
      tx.signAndSend(account, ({ status, dispatchError }) => {
        if (dispatchError) {
          if (dispatchError.isModule) {
            const decoded = this.api.registry.findMetaError(dispatchError.asModule);
            reject(new Error(`${decoded.section}.${decoded.name}: ${decoded.docs.join(' ')}`));
          } else {
            reject(new Error(dispatchError.toString()));
          }
        } else if (status.isInBlock) {
          resolve(status.asInBlock);
        }
      });
    });
  }

  async getIdentity(accountId: string | AccountId): Promise<IdentityKeyBundle | null> {
    const result = await this.api.query.signalKeys.identityKeys(accountId);
    
    if (result.isEmpty || result.toPrimitive() === null) {
      return null;
    }

    const bundle: any = result.toJSON();
    return {
      identityKey: new Uint8Array(bundle.identityKey),
      signedPrekey: new Uint8Array(bundle.signedPrekey),
      prekeySignature: new Uint8Array(bundle.prekeySignature),
      registrationBlock: bundle.registrationBlock,
      updatedAt: bundle.updatedAt,
    };
  }

  async getRemainingPrekeyCount(accountId: string | AccountId): Promise<number> {
    const result = await this.api.query.signalKeys.prekeyCounter(accountId);
    return result.isEmpty ? 0 : Number(result.toString());
  }

  async hasOneTimePrekeys(accountId: string | AccountId): Promise<boolean> {
    const counter = await this.getRemainingPrekeyCount(accountId);
    return counter > 0;
  }

  async getOneTimePrekey(accountId: string | AccountId): Promise<OneTimePrekey | null> {
    const counter = await this.getRemainingPrekeyCount(accountId);
    
    for (let keyId = 0; keyId < counter; keyId++) {
      const result = await this.api.query.signalKeys.oneTimePrekeys(accountId, keyId);
      
      if (!result.isEmpty && result.toPrimitive() !== null) {
        const prekey: any = result.toJSON();
        return {
          keyId: prekey.keyId,
          publicKey: new Uint8Array(prekey.publicKey),
        };
      }
    }
    
    return null;
  }

  createAccountFromUri(uri: string): ReturnType<typeof this.keyring.addFromUri> {
    return this.keyring.addFromUri(uri);
  }

  getApi(): ApiPromise {
    return this.api;
  }
}

export default SignalKeysClient;
