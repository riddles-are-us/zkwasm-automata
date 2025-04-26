import mongoose, { Schema } from 'mongoose';

// recursive masker that applies to bigints or arrays of bigints
function maskUint64(v: any): any {
    if (typeof v === 'bigint') {
        return BigInt.asUintN(64, v);
    }
    if (Array.isArray(v)) {
        return v.map(maskUint64);
    }
    // if you have nested plain objects you also want to walk, you could:
    if (v !== null && typeof v === 'object') {
        for (const k of Object.keys(v)) {
            v[k] = maskUint64(v[k]);
        }
    }
    return v;
}

function uint64FetchPlugin(next: any, rawDoc: any) {
    return maskUint64(rawDoc);
}

interface Bidder {
  bidprice: bigint;
  bidder: bigint[];
}
interface Card {
  duration: bigint;
  attributes: bigint;
}

interface MarketInfo {
    marketid: bigint;
    askprice: bigint;
    settleinfo: bigint;
    bidder: Bidder | null;
    card: Card;
}

const BidderSchema = new mongoose.Schema<Bidder>({
  bidprice:  { type: BigInt, required: true },
  bidder:    { type: [BigInt], required: true }
});

const CardSchema = new mongoose.Schema<Card>({
  duration: { type: BigInt, require: true},
  attributes: { type: BigInt, require: true},
});




// Define the schema for the Token model
const indexedObjectSchema = new mongoose.Schema({
    marketid: { type: BigInt, required: true, unique: true},
    askprice: { type: BigInt, require: true},
    settleinfo: { type: BigInt, require: true},
    bidder: { type: BidderSchema, require: false},
    card: {type: CardSchema, require: true},
});

indexedObjectSchema.pre('init', uint64FetchPlugin);

(BigInt.prototype as any).toJSON = function () {
      return this.toString();
};


// Utility function to convert a bigint to an array of 8 bytes in little-endian order.
function toLEBytes(num: bigint): number[] {
  const bytes: number[] = [];
  const mask = 0xffn;
  for (let i = 0; i < 8; i++) {
    bytes.push(Number(num & mask));
    num = num >> 8n;
  }
  return bytes;
}

function fromData(u64datasource: bigint[]): MarketInfo {
  const u64data = u64datasource.slice();
  // Ensure there are at least three elements.
  if (u64data.length < 3) {
    throw new Error("Not enough data to construct a Card");
  }

  const marketid: bigint = u64data.shift()!;
  const askprice: bigint = u64data.shift()!;
  const settleinfo: bigint = u64data.shift()!;



  // Map each byte to a signed 8-bit integer.
  // For byte values greater than 127, subtract 256 to get the signed representation.
  //const attributes = leBytes.map(b => (b > 127 ? b - 256 : b));

  let bidder = null;
  if (settleinfo != 0n) {
    bidder = {
      bidprice: u64data.shift()!,
      bidder: [u64data.shift()!, u64data.shift()!]
    }
  }

  // Consume data from the beginning of the array.
  const duration: bigint = u64data.shift()!;
  const valueForAttributes: bigint = u64data.shift()!;

  // Return the constructed Card object.
  return {
    marketid: marketid,
    askprice: askprice,
    settleinfo: settleinfo,
    bidder: bidder,
    card: {
      duration: duration,
      attributes: valueForAttributes,
    }
  };
}


export class IndexedObject {
    // token idx
    marketid: bigint;
    askprice: bigint;
    settleinfo: bigint;
    bidder: Bidder | null;
    card: Card;

    constructor(m: MarketInfo) {
        this.marketid = m.marketid;
        this.card = m.card;
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
            card: this.card,
            bidder: this.bidder,
        };
    }

    toJSON() {
      return JSON.stringify(this.toObject());
    }

    static fromEvent(data: BigUint64Array): IndexedObject {
        let marketinfo = fromData(Array.from(data.slice(1)));
        return new IndexedObject(marketinfo)
    }
}

// Create the Token model
export const IndexedObjectModel = mongoose.model('IndexedObject', indexedObjectSchema);
