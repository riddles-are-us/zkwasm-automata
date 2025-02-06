import { createCommand, PlayerConvention, ZKWasmAppRpc } from "zkwasm-minirollup-rpc";

/* The modifier mush less than eight */
function encode_modifier(modifiers: Array<bigint>) {
  let c = 0n;
  for (const m of modifiers) {
    c = (c << 8n) + m;
  }
  return c;
}

const CMD_INSTALL_PLAYER = 1n;
const CMD_INSTALL_OBJECT = 2n;
const CMD_RESTART_OBJECT = 3n;
const CMD_UPGRADE_OBJECT = 4n;
const CMD_INSTALL_CARD = 5n;
const CMD_WITHDRAW= 6n;
const CMD_DEPOSIT = 7n;

export class Player extends PlayerConvention {
  constructor(key: string, rpc: ZKWasmAppRpc) {
    super(key, rpc, CMD_DEPOSIT, CMD_WITHDRAW);
    this.processingKey = key,
    this.rpc = rpc;
  }

  async installPlayer() {
    try {
      let result = await this.rpc.sendTransaction(
        createCommand(0n, CMD_INSTALL_PLAYER, []),
        this.processingKey
      );
      return result;
    } catch(e) {
      if(e instanceof Error) {
        console.log(e.message);
      }
      console.log("installPlayer error at processing key:", this.processingKey);
    }
  }

  async installObject(objid: bigint, modifiers: Array<bigint>) {
    let nonce = await this.getNonce();
    try {
      let result = await this.rpc.sendTransaction(
        createCommand(nonce, CMD_INSTALL_OBJECT, [objid, encode_modifier(modifiers)]),
        this.processingKey
      );
      return result
    } catch(e) {
      if(e instanceof Error) {
        console.log(e.message);
      }
      console.log("installObject error at modifiers:", modifiers, "processing key:", this.processingKey);
    }
  }

  async restartObject(objid: bigint, modifiers: Array<bigint>) {
    let nonce = await this.getNonce();
    try {
      let finished = await this.rpc.sendTransaction(
        createCommand(nonce, CMD_RESTART_OBJECT, [objid, encode_modifier(modifiers)]),
        this.processingKey
      );
      console.log("restartObject processed at:", finished);
    } catch(e) {
      if(e instanceof Error) {
        console.log(e);
        console.log(e.message);
      }
      console.log("restart object error", "processing key:", this.processingKey);
    }
  }

  async upgradeObject(objid: bigint, featureId: bigint) {
    let nonce = await this.getNonce();
    try {
      let finished = await this.rpc.sendTransaction(
        createCommand(nonce, CMD_UPGRADE_OBJECT, [objid, featureId]),
        this.processingKey
      );
      console.log("upgradeObject processed at:", finished);
    } catch(e) {
      if(e instanceof Error) {
        console.log(e.message);
      }
      console.log("upgrade object error", "processing key:", this.processingKey);
    }
  }


  async installCard() {
    let nonce = await this.getNonce();
    try {
      let finished = await this.rpc.sendTransaction(
        createCommand(nonce, CMD_INSTALL_CARD, []),
        this.processingKey
      );
      console.log("installCard processed at:", finished);
    } catch(e) {
      if(e instanceof Error) {
        console.log(e.message);
      }
      console.log("installCard error with processing key:", this.processingKey);
    }
  }
}
