import mongoose from 'mongoose';
import { Decodable, Bidder, fromData, MarketInfo, marketObjectSchema } from './market.js';

(BigInt.prototype as any).toJSON = function () {
      return this.toString();
};

interface Card {
  duration: bigint;
  attributes: bigint;
}

class CardDecoder implements Decodable<Card> {
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

export class IndexedObject {
    // token idx
    marketid: bigint;
    askprice: bigint;
    settleinfo: bigint;
    bidder: Bidder | null;
    object: Card;

    constructor(m: MarketInfo<Card>) {
        this.marketid = m.marketid;
        this.object = m.object;
        this.askprice = m.askprice;
        this.settleinfo = m.settleinfo;
        this.bidder = m.bidder;
    }

    static fromMongooseDoc(doc: mongoose.Document): IndexedObject {
        const obj = doc.toObject({
            transform: (_doc, ret) => {
                delete ret._id;
                return ret;
            }
        });

        // Convert the second value into its 8 little-endian bytes.
        // const leBytes = toLEBytes(obj.attributes);

        return new IndexedObject(obj);
    }

    toMongooseDoc(): mongoose.Document {
        return new IndexedObjectModel(this.toObject());
    }

    toObject() {
        return {
            marketid: this.marketid,
            askprice: this.askprice,
            settleinfo: this.settleinfo,
            object: this.object,
            bidder: this.bidder,
        };
    }

    toJSON() {
      return JSON.stringify(this.toObject());
    }

    static fromEvent(data: BigUint64Array): IndexedObject {
        let marketinfo = fromData(Array.from(data.slice(1)), new CardDecoder());
        return new IndexedObject(marketinfo)
    }
}

// Create the Token model
export const IndexedObjectModel = mongoose.model('IndexedObject', marketObjectSchema);
