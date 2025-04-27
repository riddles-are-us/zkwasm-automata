import {PrivateKey, bnToHexLe} from "delphinus-curves/src/altjubjub";
import { Player } from "./api.js";
import dotenv from 'dotenv';
import {LeHexBN, ZKWasmAppRpc} from "zkwasm-ts-server";
dotenv.config();

const rpc = new ZKWasmAppRpc("http://127.0.0.1:3000");

let account = "1234";
let player = new Player(account, rpc);
let bidderAccount = "1235";
let bidder = new Player(bidderAccount, rpc);

let admin = new Player(process.env.SERVER_ADMIN_KEY!, rpc);

let pkey = PrivateKey.fromString(player.processingKey);
let pubkey = pkey.publicKey.key.x.v;
let leHexBN = new LeHexBN(bnToHexLe(pubkey));
let pkeyArray = leHexBN.toU64Array();

async function main() {
  let config = await player.getConfig();
  console.log("config", config);

  console.log("install player ...\n");
  await player.installPlayer();

  console.log("install player ...\n");
  await bidder.installPlayer();

  console.log("install admin ...\n");
  await admin.installPlayer();

  /*
  console.log("deposit ...\n", pkeyArray[1], pkeyArray[2], admin.processingKey);
  await admin.deposit(pkeyArray[1], pkeyArray[2], 0n, 10n);
  */

  /*
  console.log("install object ...\n");
  await player.installObject(0n, [0n, 0n, 0n, 0n, 0n, 0n, 0n, 0n]);
  */

  console.log("install card...\n");
  await player.installCard();

  console.log("list card...\n");
  await player.listCard(0n, 10n);

  let markets = await player.rpc.queryData("markets");
  console.log("query markets ...", markets);

  console.log("list card...\n");
  await bidder.bidCard(1n, 8n);

  markets = await player.rpc.queryData("markets");
  console.log("query markets ...", markets);

  console.log("sell card...\n");
  await player.sellCard(0n);

  markets = await player.rpc.queryData("markets");
  console.log("query markets ...", markets);

  /*
  console.log("sell card...\n");
  await player.sellCard(0n);

  markets = await player.rpc.queryData("markets");
  console.log("query markets ...", markets);

  console.log("sell card...\n");
  await player.sellCard(0n);

  markets = await player.rpc.queryData("markets");
  console.log("query markets ...", markets);



/*
  console.log("restart object ...\n");
  await player.restartObject(0n, [0n, 0n, 0n, 0n, 0n, 0n, 0n, 4n]);

  console.log("upgrade object ...\n");
  await player.upgradeObject(0n);

  let state = await player.getState();
  console.log("query state:", state);

  console.log("withdraw:\n");
  await player.withdrawRewards("c177d1d314C8FFe1Ea93Ca1e147ea3BE0ee3E470", 1n);
  */
}

main();
