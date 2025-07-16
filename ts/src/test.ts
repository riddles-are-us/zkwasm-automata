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

  await player.installPlayer();
  let state = await player.getState();
  console.log("state", state);

  console.log("Start run LIST_NUGGET ...");
  await player.listCard(BigInt(0), BigInt(1000));

  console.log("Start run LIST_NUGGET ...");
  await player.listCard(BigInt(1), BigInt(1200));
}

main();
