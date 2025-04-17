import mongoose from 'mongoose';

interface Bidder {
  bidprice: number;
  bidder: string[];
}
interface Card {
  duration: number;
  attributes: number[];
  marketid: number;
  askprice: number;
  bidder: Bidder | null;
}

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

function fromData(u64datasource: bigint[]): Card {
  const u64data = u64datasource.slice();
  // Ensure there are at least three elements.
  if (u64data.length < 3) {
    throw new Error("Not enough data to construct a Card");
  }

  // Consume data from the beginning of the array.
  const duration: bigint = u64data.shift()!;
  const valueForAttributes: bigint = u64data.shift()!;
  const marketid: bigint = u64data.shift()!;
  const askprice: bigint = u64data.shift()!;

  // Convert the second value into its 8 little-endian bytes.
  const leBytes = toLEBytes(valueForAttributes);

  // Map each byte to a signed 8-bit integer.
  // For byte values greater than 127, subtract 256 to get the signed representation.
  const attributes = leBytes.map(b => (b > 127 ? b - 256 : b));

  const bidderprice = u64data.shift();
  let bidder = null;
  if (bidderprice != 0n) {
    bidder = {
      bidprice: Number(bidderprice),
      bidder: [u64data.shift()!.toString(), u64data.shift()!.toString()]
    }
  }

  // Return the constructed Card object.
  return {
    duration: Number(duration),
    attributes,
    marketid: Number(marketid),
    askprice: Number(askprice),
    bidder: bidder,
  };
}


export class IndexedObject {
    // token idx
    index: number;
    data: bigint[];

    constructor(index: number, data: bigint[]) {
        this.index = index;
        this.data = data;
    }

    static fromMongooseDoc(doc: mongoose.Document): IndexedObject {
        const obj = doc.toObject({
            transform: (_doc, ret) => {
                delete ret._id;
                return ret;
            }
        });
        let unsigneddata = obj.data.map((x: bigint) => BigInt.asUintN(64, x));
        return new IndexedObject(obj.index, unsigneddata);
    }

    toMongooseDoc(): mongoose.Document {
        return new IndexedObjectModel(this.toObject());
    }

    toObject(): { index: number, data: bigint[], bidder: string[] | null} {
        console.log("toObject", this.data);
        const obj = fromData(this.data);
        let bidder = null;
        if (obj.bidder) {
            bidder = obj.bidder.bidder;
        }
        return {
            bidder: bidder,
            index: this.index,
            data: this.data,
        };
    }

    toJSON() {
      const iobj = fromData(this.data);
      return JSON.stringify(iobj);
    }

    static fromEvent(data: BigUint64Array): IndexedObject {
        return new IndexedObject(Number(data[0]),  Array.from(data.slice(1)))
    }
}

// Define the schema for the Token model
const indexedObjectSchema = new mongoose.Schema({
    index: { type: Number, required: true, unique: true},
    bidder:  {
      type: [String],
      required: false,
    },
    data: {
        type: [BigInt],
        required: true,
    },
});

// Create the Token model
export const IndexedObjectModel = mongoose.model('IndexedObject', indexedObjectSchema);
