import mongoose from 'mongoose';
import { Market, ObjectEvent } from 'zkwasm-ts-server';

(BigInt.prototype as any).toJSON = function () {
      return this.toString();
};

interface Card {
  duration: bigint;
  attributes: bigint;
}

class CardDecoder implements ObjectEvent.Decodable<Card> {
  constructor() {
  }
  fromData(u64data: bigint[]): Card {
    const duration: bigint = u64data.shift()!;
    const valueForAttributes: bigint = u64data.shift()!;
    return {
      duration: duration,
      attributes: valueForAttributes,
    }
  }
}

export function docToJSON(doc: mongoose.Document) {
    console.log("doc...", doc);
    const obj = doc.toObject({
        transform: (_, ret:any) => {
            delete ret._id;
            return ret;
        }
    });
    return obj;
}

export class IndexedObject {
    // token idx
    index: number;
    // 40-character hexadecimal Ethereum address
    data: bigint[];



    constructor(index: number, data: bigint[]) {
        this.index = index;
        this.data = data;
    }

    toObject() {
        let decoder = new CardDecoder();
        return Market.fromData(this.data, decoder);
    }

    toJSON() {
      return JSON.stringify(this.toObject());
    }

    static fromEvent(data: BigUint64Array): IndexedObject {
        return new IndexedObject(Number(data[0]),  Array.from(data.slice(1)))
    }

    async storeObject() {
        let obj = this.toObject() as any;
        console.log("object is:", obj);
        let doc = await MarketObjectModel.findOneAndUpdate({marketid: obj.marketid}, obj, {upsert: true});
        return doc;
    }
}

// Define the schema for the Token model
const CardObjectSchema = new mongoose.Schema({
  duration: {type: BigInt, required: true},
  attributes: {type: BigInt, required: true},
});

const MarketObjectSchema = Market.createMarketSchema(CardObjectSchema);

CardObjectSchema.pre('init', ObjectEvent.uint64FetchPlugin);

// Create the Token model
export const MarketObjectModel = mongoose.model('MarketObject', MarketObjectSchema);
export const CardObjectModel = mongoose.model('NuggetObject', CardObjectSchema);
