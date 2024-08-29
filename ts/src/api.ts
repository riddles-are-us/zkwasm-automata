import { query, LeHexBN, ZKWasmAppRpc } from "zkwasm-ts-server";
import BN from "bn.js";

/* The modifier mush less than eight */
function encode_modifier(modifiers: Array<bigint>) {
  let c = 0n;
  for (const m of modifiers) {
    c = (c << 8n) + m;
  }
  return c;
}

function addrToParams(bn: BN): Array<bigint> {
  // address is encoded in BigEndian
  const mask = new BN('ffffffffffffffff', 16);
  let a = bn.and(mask).toArray('le', 8);
  let b = bn.shrn(64).and(mask).toArray('le', 8);
  let c = bn.shrn(128).and(mask).toArray('le', 8);
  let aHex = a.map(byte => byte.toString(16).padStart(2, '0')).join('');
  let bHex = b.map(byte => byte.toString(16).padStart(2, '0')).join('');
  let cHex = c.map(byte => byte.toString(16).padStart(2, '0')).join('');
  return [BigInt(`0x${cHex}`), BigInt(`0x${bHex}`), BigInt(`0x${aHex}`)];
}

const CMD_INSTALL_PLAYER = 1n;
const CMD_INSTALL_OBJECT = 2n;
const CMD_RESTART_OBJECT = 3n;
const CMD_WITHDRAW= 4n;
const CMD_DEPOSIT = 4n;

function createCommand(nonce: bigint, command: bigint, objindex: bigint) {
  return (nonce << 16n) + (objindex << 8n) + command;
}

const rpc = new ZKWasmAppRpc("http://localhost:3000");

export class Player {
  processingKey: string;
  constructor(key: string) {
    this.processingKey = key
  }

  async getConfig(): Promise<any> {
    let config = await rpc.query_config();
    return config;
  }

  async getState(): Promise<any> {
    // Get the state response
    let state = await rpc.queryState(this.processingKey);

    // Parse the response to ensure it is a plain JSON object
    const parsedState = JSON.parse(JSON.stringify(state));

    // Extract the data from the parsed response
    const data = JSON.parse(parsedState.data);

    return data;
  }

  async getNonce(): Promise<bigint> {
    const data = await this.getState();
    let nonce = BigInt(data[0].nonce);
    return nonce;
  }

  async installPlayer() {
    try {
      let finished = await rpc.sendTransaction(
        [createCommand(0n, CMD_INSTALL_PLAYER, 0n), 0n, 0n, 0n],
        this.processingKey
      );
      console.log("installPlayer processed at:", finished);
    } catch(e) {
      if(e instanceof Error) {
        console.log(e.message);
      }
      console.log("installPlayer error at processing key:", this.processingKey);
    }
  }

  async installObject(modifiers: Array<bigint>) {
    let nonce = await this.getNonce();
    try {
      let finished = await rpc.sendTransaction(
        [createCommand(nonce, CMD_INSTALL_OBJECT, 0n), encode_modifier(modifiers), 0n, 0n],
        this.processingKey
      );
      console.log("installObject processed at:", finished);
    } catch(e) {
      if(e instanceof Error) {
        console.log(e.message);
      }
      console.log("installObject error at modifiers:", modifiers, "processing key:", this.processingKey);
    }
  }

  async deposit() {
    let nonce = await this.getNonce();
    let accountInfo = new LeHexBN(query(this.processingKey).pkx).toU64Array();
    try {
      let finished = await rpc.sendTransaction(
        [createCommand(nonce, CMD_DEPOSIT, 0n), accountInfo[1], accountInfo[2], 1000n],
        this.processingKey
      );
      console.log("deposit processed at:", finished);
    } catch(e) {
      if(e instanceof Error) {
        console.log(e.message);
      }
      console.log("deposit error at processing key:", this.processingKey);
    }
  }

  async withdraw(address: string) {
    let nonce = await this.getNonce();
    let addParams = addrToParams(new BN(address, 16));
    try {
      let finished = await rpc.sendTransaction(
        [createCommand(nonce, CMD_WITHDRAW, 0n), addParams[0], addParams[1], addParams[2]],
        this.processingKey
      );
      console.log("withdraw processed at:", finished);
    } catch(e) {
      if(e instanceof Error) {
        console.log(e.message);
      }
      console.log("withdraw error at address:", address, "processing key:", this.processingKey);
    }
  }
}