use crate::config::LOCAL_ATTRIBUTES_SIZE;
use serde::Serialize;
use crate::player::PlayerData;
use std::slice::IterMut;
use crate::error::*;
use zkwasm_rest_abi::StorageData;
use zkwasm_rest_convention::BidInfo;
use zkwasm_rest_convention::IndexedObject;
use zkwasm_rest_convention::MarketInfo;
use zkwasm_rest_convention::BidObject;
use std::marker::PhantomData;

#[derive(Clone, Debug, Serialize)]
pub struct Card {
    pub duration: u64,
    pub attributes: [i8; 8],
    pub marketid: u64,
}

impl Card {
    fn new(duration: u64, attributes: [i8; LOCAL_ATTRIBUTES_SIZE]) -> Self {
        Card {
            duration,
            attributes,
            marketid: 0,
        }
    }
}

impl StorageData for Card {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        let duration = *u64data.next().unwrap();
        let attributes = (*u64data.next().unwrap()).to_le_bytes();
        Card {
            duration,
            attributes: attributes.map(|x| x as i8),
            marketid: *u64data.next().unwrap(),
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(self.duration);
        data.push(u64::from_le_bytes(self.attributes.map(|x| x as u8)));
        data.push(self.marketid);
    }
}

lazy_static::lazy_static! {
    pub static ref DEFAULT_CARDS: Vec<Card> = vec![
        Card::new(100, [-2, -2, 4, 0, 0, 0, 0, 0]),
        Card::new(110, [3, 0, -2, 0, 0, 0, 0, 0]),
        Card::new(120, [0, 3, -2, 0, 0, 0, 0, 0]),
    ];
    pub static ref CARD_NAME: Vec<&'static str> = vec![
        "Biogen",
        "Crystara",
        "AstroMine",
    ];
}

impl BidObject<PlayerData> for MarketInfo<Card, PlayerData> {
    const INSUFF:u32 = ERROR_BID_PRICE_INSUFFICIENT;
    const NOBID:u32 = ERROR_NO_BIDDER;
    fn get_bidder(&self) -> Option<BidInfo> {
        self.bid
    }

    fn set_bidder(&mut self, bidder: Option<BidInfo>) {
        self.bid = bidder;
    }
    fn get_owner(&self) -> [u64; 2] {
        self.owner
    }

    fn set_owner(&mut self, pid: [u64; 2]) { 
        self.owner = pid 
    }
}

pub struct MarketCard (pub MarketInfo<Card, PlayerData>);

impl MarketCard {
    pub fn new(marketid: u64, askprice: u64, settleinfo: u64, bid: Option<BidInfo>, object: Card, owner: [u64; 2]) -> Self {
        MarketCard (MarketInfo {
            marketid,
            askprice,
            settleinfo,
            bid,
            object,
            owner,
            user: PhantomData
        })
    }
}

impl StorageData for MarketCard {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        MarketCard (MarketInfo::<Card, PlayerData>::from_data(u64data))
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        self.0.to_data(data)
    }
}

impl IndexedObject<MarketCard> for MarketCard {
    const PREFIX: u64 = 0x1ee1;
    const POSTFIX: u64 = 0xfee1;
    const EVENT_NAME: u64 = 0x02;
}
