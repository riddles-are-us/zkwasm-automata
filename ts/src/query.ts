import { Player } from "./api.js";
import {LeHexBN, ZKWasmAppRpc} from "zkwasm-ts-server";
import { PrivateKey, bnToHexLe } from "delphinus-curves/src/altjubjub";

const rpc = new ZKWasmAppRpc("http://127.0.0.1:3000");
let account = "1234";
let player = new Player(account, rpc);

async function main() {
  await player.installPlayer();
  let data = await player.getState();
  let pkey = PrivateKey.fromString(player.processingKey);
  let pubkey = pkey.publicKey.key.x.v;
  let leHexBN = new LeHexBN(bnToHexLe(pubkey));
  let pkeyArray = leHexBN.toU64Array();

  console.log('pid', pkeyArray[1], pkeyArray[2]);

  console.log("player info:");
  console.log(JSON.stringify(data));

  let config = await player.getConfig();
  console.log("config", config);
}

main();
