import {PrivateKey, bnToHexLe} from "delphinus-curves/src/altjubjub";
import { Player } from "./api.js";
import dotenv from 'dotenv';
import {LeHexBN, ZKWasmAppRpc} from "zkwasm-ts-server";
dotenv.config();

const rpc = new ZKWasmAppRpc("http://127.0.0.1:3000");


async function main() {
  let account = "1234";
  let player = new Player(account, rpc);
  await player.installPlayer();
  await player.installCard();
  let state = await player.getState();
  let card_index = state.player.data.cards.findIndex((card: { marketid: number; }) => card.marketid == 0);
  console.log("card index", card_index);

  console.log("Start run LIST_NUGGET ...");
  await player.listCard(BigInt(card_index), BigInt(500));

  state = await player.getState();
  let market_index = state.player.data.cards[card_index].marketid;
  console.log("market index", market_index);

  let bid_account = "5678";
  let bid_player = new Player(bid_account, rpc);
  await bid_player.installPlayer();

  state = await bid_player.getState();
  let money_before = state.player.data.local[7];

  console.log("Start run 1st BID_NUGGET ...");
  await bid_player.bidCard(BigInt(market_index), BigInt(200));

  console.log("Start run 2nd BID_NUGGET ...");
  await bid_player.bidCard(BigInt(market_index), BigInt(300));

  state = await bid_player.getState();
  let money_after = state.player.data.local[7];
  let money_change = money_after - money_before;
  console.log("money change", money_change);
  if (money_change != -300) {
    throw new Error("Money change incorrect");
  }
}

main();
