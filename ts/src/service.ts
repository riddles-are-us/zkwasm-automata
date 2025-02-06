import { Service } from "zkwasm-ts-server";
import {TxWitness} from "zkwasm-ts-server/src/prover";
import { Express } from "express";
import {merkleRootToBeHexString} from "zkwasm-ts-server/src/lib.js";

const service = new Service(eventCallback, batchedCallback, extra, bootstrap);
await service.initialize();


let currentUncommitMerkleRoot: string = merkleRootToBeHexString(service.merkleRoot);

function extra (app: Express) {
  
}

service.serve();

async function bootstrap(merkleRoot: string): Promise<TxWitness[]> {
  return [];
}

async function batchedCallback(arg: TxWitness[], preMerkle: string, postMerkle: string) {
  return;
}

async function eventCallback(arg: TxWitness, data: BigUint64Array) {
}
