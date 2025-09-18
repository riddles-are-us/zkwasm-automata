use crate::card::{Card, DEFAULT_CARDS};
use crate::config::{COST_INCREASE_ROUND, COST_INCREASE_ROUND_INITIAL};
use crate::config::{default_local, random_modifier, INITIAL_ENERGY};
use crate::error::*;
use crate::object::Object;
use crate::Player;
use crate::StorageData;
use crate::MERKLE_MAP;
use serde::Serialize;
use std::slice::IterMut;
use zkwasm_rest_abi::enforce;
use zkwasm_rest_convention::IndexedObject;
use zkwasm_rest_convention::Wrapped;
use zkwasm_rest_convention::BidObject;
use zkwasm_rest_convention::WithBalance;
use crate::card::MarketCard;

#[derive(Clone, Debug, Serialize)]
pub struct Attributes(pub Vec<i64>);

impl Attributes {
    pub fn apply_modifier(&mut self, m: &Attributes) -> bool {
        for (a, b) in self.0.iter().zip(m.0.iter()) {
            if *a + *b < 0 {
                return false;
            }
        }
        for (a, b) in self.0.iter_mut().zip(m.0.iter()) {
            *a += *b;
        }
        return true;
    }
}

#[derive(Debug, Serialize)]
pub struct PlayerData {
    pub level: u16,
    pub exp: u16,
    pub last_check_point: u32,
    pub energy: u16, // this is collected from the supplier
    pub cost_info: u16,
    pub current_cost: u32,
    pub redeem_info: [u8; 8],
    pub last_interest_stamp: u64,
    pub objects: Vec<Object>,
    pub local: Attributes,
    pub cards: Vec<Card>,
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            level: 1,
            exp: 0,
            last_check_point: 0,
            energy: INITIAL_ENERGY,
            cost_info: COST_INCREASE_ROUND_INITIAL,
            current_cost: 0,
            redeem_info: [0; 8],
            last_interest_stamp: 0,
            objects: vec![],
            local: Attributes::default_local(),
            cards: DEFAULT_CARDS.clone(),
        }
    }
}

impl Attributes {
    fn default_local() -> Self {
        Attributes(default_local().to_vec())
    }
}

impl WithBalance for PlayerData {
    fn cost_balance(&mut self, b: u64) -> Result<(), u32> {
        if let Some(treasure) = self.local.0.last_mut() {
            if *treasure >= b as i64 {
                *treasure -= b as i64;
                Ok(())
            } else {
                Err(ERROR_NOT_ENOUGH_BALANCE)
            }
        } else {
            unreachable!();
        }
    }

    fn inc_balance(&mut self, b: u64) {
        if let Some(treasure) = self.local.0.last_mut() {
            *treasure += b as i64;
        } else {
            unreachable!();
        }
    }

}

impl PlayerData {
    pub fn generate_card(&mut self, rand: &[u64; 4]) {
        let new_card = random_modifier(self.level as i64, self.local.0.clone().try_into().unwrap(), rand[1]);
        self.cards.push(new_card)
    }

    pub fn pay_cost(&mut self, base: u64) -> Result<(), u32> {
        self.cost_balance(self.current_cost as u64 + base)?;
        self.cost_info -= 1;
        if self.cost_info == 0 {
            self.cost_info = COST_INCREASE_ROUND;
            if self.current_cost != 0 {
                self.current_cost = self.current_cost * 2
            } else {
                self.current_cost = 1;
            }
        }
        let energy = (self.energy as u32) + 20;
        if energy > 0xffff {
            self.energy = 0xffff
        } else {
            self.energy += 20;
        }
        self.inc_exp((self.current_cost + 1).ilog2() as u16);
        Ok(())
    }



    fn get_balance(&self) -> u64 {
        if let Some(treasure) = self.local.0.last() {
            *treasure as u64
        } else {
            unreachable!();
        }
    }

    pub fn update_interest(&mut self, counter: u64) {
        self.last_interest_stamp = (self.get_balance() << 32) + counter
    }

    pub fn upgrade_object(&mut self, object_index: usize, index: usize) {
        let object = self.objects.get_mut(object_index).unwrap();
        enforce(object.attributes[0] < 128, "check attributes bound");
        object.attributes[0] += 1;
        object.attributes[index] += 1;
    }

    pub fn inc_exp(&mut self, a: u16) {
        self.exp += a;
        if self.exp >= 100 + self.level * 10 {
            self.level += 1;
            self.exp = 0;
        }
    }

    pub fn collect_energy(&mut self, counter: u64) -> Result <(), u32> {
        let delta = counter - (self.last_check_point as u64);
        if delta >= 1000 {
            let energy = (self.energy as u32) + ((self.get_balance() / 10000) + 1).ilog2() * (self.level as u32);
            if energy > 0xffff {
                self.energy = 0xffff
            } else {
                self.energy = energy as u16
            }
            self.last_check_point = counter as u32;
            self.cost_balance(1)
        }
    }

    pub fn collect_interest(&mut self, counter: u64) -> Result <(), u32> {
        let mut balance = self.last_interest_stamp >> 32;
        let current_balance = self.get_balance();
        if balance > current_balance {
            balance = current_balance
        }
        let timestamp = self.last_interest_stamp & 0xffffffff;
        let delta = counter - timestamp;
        let interest = ((self.level as u64) * balance * delta / (10000 * 17280)) as i64;
        self.last_interest_stamp = (self.get_balance() << 32) + counter;

        // TODO: cost balance needs check interest bound
        self.cost_balance(interest as u64 - 100)
    }

    fn card_used(&self, card_index: usize) -> bool {
        for obj in self.objects.iter() {
            for ci in obj.cards.iter() {
                if *ci as usize == card_index {
                    return true;
                }
            }
        }
        return false;
    }

    pub fn list_card_in_market(&mut self, card_index: usize, price: u64, marketid: u64, owner: [u64; 2]) -> Result<MarketCard, u32> {
        if card_index < self.cards.len() {
            if self.card_used(card_index) {
                Err(ERROR_CARD_IS_IN_USE)
            } else {
                let card = self.cards.get_mut(card_index).unwrap();
                if card.marketid != 0 {
                    Err(ERROR_CARD_IS_IN_USE)
                } else {
                    card.marketid = marketid;
                    let market_card = MarketCard::new(
                        marketid,
                        price,
                        0,
                        None,
                        card.clone(),
                        owner,
                    );
                    Ok(market_card)
                }
            }
        } else {
            Err(ERROR_INDEX_OUT_OF_BOUND)
        }
    }

    pub fn remove_card(&mut self, card_index: usize) {
        if self.cards.len() == card_index + 1 {// the last element
          self.cards.swap_remove(card_index);
        } else {
            let last = self.cards.len() - 1;
            self.cards.swap_remove(card_index);
            for obj in self.objects.iter_mut() {
                for cid in obj.cards.iter_mut() {
                    if *cid == last as u8 {
                        *cid = card_index as u8;
                    }
                }
            }
        }
    }

    pub fn sell_card(&mut self, card_index: usize) -> Result<Wrapped<MarketCard>, u32> {
        if card_index < self.cards.len() {
            let card = self.cards.get_mut(card_index).unwrap();
            let marketid = card.marketid;
            card.marketid = 0;
            if marketid != 0 {
                let wrapped_market_card = MarketCard::get_object(marketid).unwrap();
                if let Some(b) = wrapped_market_card.data.0.get_bidder() {
                    self.inc_balance(b.bidprice);
                    self.remove_card(card_index);
                    Ok(wrapped_market_card)
                } else {
                    Err(ERROR_NO_BIDDER)
                }
            } else {
                Err(ERROR_INDEX_OUT_OF_BOUND)
            }
        } else {
            Err(ERROR_INDEX_OUT_OF_BOUND)
        } 
    }

    pub fn apply_object_card(&mut self, object_index: usize, counter: u64) -> Option<usize> {
        let object = self.objects[object_index].clone();
        let mut speed = (object.attributes[1] + 1).ilog2() as u64;
        if speed > 9 { speed = 9 };
        let current_index = object.get_modifier_index() as usize;
        if object.is_restarting() {
            //zkwasm_rust_sdk::dbg!("is restarting !\n");
            let next_index = 0;
            let duration = self.cards[object.cards[next_index] as usize].duration;
            let duration = duration * (10 - speed) / 10;
            let object = self.objects.get_mut(object_index).unwrap();
            object.start_new_modifier(next_index, counter);
            Some(duration as usize)
        } else {
            let card = self.cards[object.cards[current_index] as usize].clone();
            let applied = self.apply_modifier(&card, &object);
            //zkwasm_rust_sdk::dbg!("applied modifier!\n");
            let object = self.objects.get_mut(object_index).unwrap();
            if applied {
                //zkwasm_rust_sdk::dbg!("object after: {:?}\n", object);
                //zkwasm_rust_sdk::dbg!("player after: {:?}\n", {&self.local});
                let next_index = (current_index + 1) % object.cards.len();
                let duration = self.cards[object.cards[next_index] as usize].duration;
                let duration = duration * (10 - speed) / 10;
                object.start_new_modifier(next_index, counter);
                Some(duration as usize)
            } else {
                object.halt();
                None
            }
        }
    }

    pub fn restart_object_card(
        &mut self,
        object_index: usize,
        data: [u8; 8],
        counter: u64,
    ) -> Option<usize> {
        let object = self.objects.get_mut(object_index).unwrap();
        let halted = object.is_halted();
        if halted {
            // modify object with new modifiers
            object.reset_modifier(data);
            let modifier_index = object.get_modifier_index();
            let duration = self.cards[object.cards[modifier_index as usize] as usize].duration;
            object.restart(counter);
            //zkwasm_rust_sdk::dbg!("object restarted\n");
            Some(duration as usize)
        } else {
            object.reset_modifier(data);
            object.reset_halt_bit_to_restart();
            None
        }
    }
    pub fn apply_modifier(&mut self, m: &Card, o: &Object) -> bool {
        let reduce = o.attributes[2] as i64;
        let productivity = o.attributes[3];
        let m = m.attributes.iter().map(|x| *x as i64).collect::<Vec<_>>();
        for (a, b) in self.local.0.iter().zip(m.iter()) {
            if *a + *b + reduce < 0 {
                return false;
            }
        }
        for (a, b) in self.local.0.iter_mut().zip(m.iter()) {
            if *b < 0 {
                let g = *b + reduce;
                if g < 0 {
                    *a += *b + reduce;
                }
            } else if *b > 0 {
                *a += *b + ((productivity + 1).ilog2() as i64);
            }
        }
        return true;
    }
}

impl StorageData for PlayerData {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        let player_info = *u64data.next().unwrap();
        let cost_info = *u64data.next().unwrap();
        let redeem_info = *u64data.next().unwrap();
        let last_interest_stamp = *u64data.next().unwrap();
        let objects_size = *u64data.next().unwrap();
        let mut objects = Vec::with_capacity(objects_size as usize);
        for _ in 0..objects_size {
            objects.push(Object::from_data(u64data));
        }

        let local_size = *u64data.next().unwrap();
        let mut local = Vec::with_capacity(local_size as usize);
        for _ in 0..local_size {
            local.push(*u64data.next().unwrap() as i64);
        }

        let card_size = *u64data.next().unwrap();
        let mut cards = Vec::with_capacity(card_size as usize);
        for _ in 0..card_size {
            cards.push(Card::from_data(u64data));
        }
        PlayerData {
            level: ((player_info >> 48) & 0xffff) as u16,
            exp: ((player_info >> 32) & 0xffff) as u16,
            last_check_point : (player_info & 0xffffffff) as u32,
            energy: ((cost_info >> 48) & 0xffff) as u16,
            cost_info: ((cost_info >> 32) & 0xffff) as u16,
            redeem_info: redeem_info.to_le_bytes(),
            current_cost: (cost_info & 0xffffffff) as u32,
            last_interest_stamp,
            objects,
            local: Attributes(local),
            cards,
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(
            ((self.level as u64) << 48)
                + ((self.exp as u64) << 32)
                + (self.last_check_point as u64),
        );
        data.push(
            ((self.energy as u64) << 48)
            + ((self.cost_info as u64) << 32)
            + (self.current_cost as u64),
        );
        data.push(u64::from_le_bytes(self.redeem_info));
        data.push(self.last_interest_stamp);
        data.push(self.objects.len() as u64);
        for c in self.objects.iter() {
            c.to_data(data);
        }
        data.push(self.local.0.len() as u64);
        for c in self.local.0.iter() {
            data.push(*c as u64);
        }
        data.push(self.cards.len() as u64);
        for c in self.cards.iter() {
            c.to_data(data);
        }
    }
}

pub type AutomataPlayer = Player<PlayerData>;

pub trait Owner: Sized {
    fn store(&self);
    fn new(pkey: &[u64; 4]) -> Self;
    fn get(pkey: &[u64; 4]) -> Option<Self>;
}

impl Owner for AutomataPlayer {
    fn store(&self) {
        //zkwasm_rust_sdk::dbg!("store player\n");
        let mut data = Vec::new();
        self.data.to_data(&mut data);
        let kvpair = unsafe { &mut MERKLE_MAP };
        kvpair.set(&Self::to_key(&self.player_id), data.as_slice());
        //zkwasm_rust_sdk::dbg!("end store player\n");
    }
    fn new(pkey: &[u64; 4]) -> Self {
        Self::new_from_pid(Self::pkey_to_pid(pkey))
    }

    fn get(pkey: &[u64; 4]) -> Option<Self> {
        Self::get_from_pid(&Self::pkey_to_pid(pkey))
    }
}
