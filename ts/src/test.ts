import {PrivateKey, bnToHexLe} from "delphinus-curves/src/altjubjub";
import { Player } from "./api.js";
import dotenv from 'dotenv';
import {LeHexBN, ZKWasmAppRpc} from "zkwasm-ts-server";
dotenv.config();

const rpc = new ZKWasmAppRpc("http://127.0.0.1:3000");


async function main() {
  let account = "1234";
  let player = new Player(account, rpc);
  let config = await player.getConfig();
  console.log("config", config);

  let pid = ["3563625733937835073","2929573097060223531"];
  console.log("query bids...");

  let bids = await player.rpc.queryData(`bid/${BigInt(pid[0])}/${BigInt(pid[1])}`);
  console.log("query bid...", bids);
}

main();
