use crate::config::ADMIN_PUBKEY;
use crate::config::CONFIG;
use crate::error::*;
use crate::events::Event;
use crate::object::Object;
use crate::player::AutomataPlayer;
use crate::player::Owner;
use crate::card::Card;
use crate::card::MarketCard;
use std::cell::RefCell;
use serde::Serialize;
use zkwasm_rest_abi::StorageData;
use zkwasm_rest_abi::WithdrawInfo;
use zkwasm_rest_abi::MERKLE_MAP;
use zkwasm_rest_convention::EventQueue;
use crate::player::PlayerData;
use zkwasm_rest_convention::MarketInfo;
use zkwasm_rest_convention::WithBalance;
use zkwasm_rest_convention::SettlementInfo;
use zkwasm_rest_convention::IndexedObject;
use zkwasm_rest_convention::BidObject;
use zkwasm_rest_convention::clear_events;
use zkwasm_rest_abi::enforce;

/*
// Custom serializer for `[u64; 4]` as a [String; 4].
fn serialize_u64_array_as_string<S>(value: &[u64; 4], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(value.len()))?;
        for e in value.iter() {
            seq.serialize_element(&e.to_string())?;
        }
        seq.end()
    }
*/

pub struct Transaction {
    pub nonce: u64,
    pub command: Command,
}

#[derive (Clone)]
pub enum Command {
    UpgradeObject(UpgradeObject),
    InstallObject(InstallObject),
    RestartObject(RestartObject),
    InstallCard(InstallCard),
    ListCardInMarket(ListCardInMarket),
    SellCard(SellCard),
    BidCard(BidCard),
    Withdraw(Withdraw),
    Deposit(Deposit),
    Bounty(Bounty),
    InstallPlayer,
    CollectEnergy,
    Tick,
}

trait CommandHandler {
    fn handle(&self, pid: &[u64; 2], nonce: u64, rand: &[u64; 4]) -> Result<(), u32>;
}

#[derive (Clone)]
pub struct UpgradeObject {
    object_index: usize,
    feature_index: usize,
}

impl CommandHandler for UpgradeObject {
    fn handle(&self, pid: &[u64; 2], nonce: u64, _rand: &[u64; 4]) -> Result<(), u32> {
        let mut player = AutomataPlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_ALREADY_EXIST),
            Some(player) => {
                player.check_and_inc_nonce(nonce);
                player.data.pay_cost(0)?;
                player.data.upgrade_object(self.object_index, self.feature_index);
                player.store();
                Ok(())
            }
        }
    }
}

#[derive (Clone)]
pub struct InstallObject {
    object_index: usize,
    modifiers: [u8; 8],
}

impl CommandHandler for InstallObject {
    fn handle(&self, pid: &[u64; 2], nonce: u64, _rand: &[u64; 4]) -> Result<(), u32> {
        let mut player = AutomataPlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_NOT_EXIST),
            Some(player) => {
                player.check_and_inc_nonce(nonce);
                let objindex = player.data.objects.len();
                enforce(objindex == self.object_index, "check object index");
                let level = player.data.level as usize;
                if objindex > (level + 1) / 2 {
                    Err(ERROR_NOT_ENOUGH_LEVEL)
                } else if objindex > 24 {
                    Err(ERROR_INDEX_OUT_OF_BOUND)
                } else {
                    player.data.pay_cost(1000)?;
                    let cards = self.modifiers;
                    let mut object = Object::new(cards);
                    let counter = STATE.0.borrow().queue.counter;
                    object.start_new_modifier(0, counter);
                    let delay = player.data.cards[object.cards[0] as usize].duration;
                    player.data.objects.push(object);
                    player.store();
                    STATE.0.borrow_mut().queue.insert(Event {
                        object_index: self.object_index ,
                        owner: *pid,
                        delta: delay as usize,
                    });
                    Ok(()) // no error occurred
                }
            }
        }
    }
}

#[derive (Clone)]
pub struct RestartObject {
    object_index: usize,
    modifiers: [u8; 8],
}

impl CommandHandler for RestartObject {
    fn handle(&self, pid: &[u64; 2], nonce: u64, _rand: &[u64; 4]) -> Result<(), u32> {
        let mut player = AutomataPlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_ALREADY_EXIST),
            Some(player) => {
                player.check_and_inc_nonce(nonce);
                player.data.pay_cost(0)?;
                let counter = STATE.0.borrow().queue.counter;
                if let Some(delay) = player.data.restart_object_card(
                    self.object_index,
                    self.modifiers,
                    counter,
                ) {
                    STATE.0.borrow_mut().queue.insert(Event {
                        object_index: self.object_index,
                        owner: *pid,
                        delta: delay,
                    });
                }
                player.store();
                Ok(())
            }
        }
    }
}


#[derive (Clone)]
pub struct InstallCard {
}

impl CommandHandler for InstallCard {
    fn handle(&self, pid: &[u64; 2], nonce: u64, rand: &[u64; 4]) -> Result<(), u32> {
        let mut player = AutomataPlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_NOT_EXIST),
            Some(player) => {
                player.check_and_inc_nonce(nonce);
                let level = player.data.level as usize;
                if player.data.cards.len() >= 4 * level + 4 {
                    Err(ERROR_NOT_ENOUGH_LEVEL)
                } else {
                    player.data.pay_cost(0)?;
                    player.data.generate_card(rand);
                    player.store();
                    Ok(())
                }
            }
        }
    }
}

#[derive (Clone)]
pub struct ListCardInMarket {
    card_index: usize,
    ask_price: u64,
}

impl CommandHandler for ListCardInMarket {
    fn handle(&self, pid: &[u64; 2], nonce: u64, _rand: &[u64; 4]) -> Result<(), u32> {
        let mut player = AutomataPlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_NOT_EXIST),
            Some(player) => {
                player.check_and_inc_nonce(nonce);
                let id = STATE.0.borrow().market_id;
                let marketcard = player.data.list_card_in_market(self.card_index, self.ask_price, id, *pid)?;
                player.data.pay_cost(0)?;
                let marketcard = MarketCard::new_object(marketcard, id);
                player.store();
                marketcard.store();
                let mut state = STATE.0.borrow_mut();
                state.market_id += 1;
                state.event_id += 1;
                MarketCard::emit_event(state.event_id, &marketcard.data);
                Ok(())
            }
        }
    }
}



#[derive (Clone)]
pub struct SellCard {
    card_index: usize
}

impl CommandHandler for SellCard {
    fn handle(&self, pid: &[u64; 2], nonce: u64, _rand: &[u64; 4]) -> Result<(), u32> {
        let mut player = AutomataPlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_NOT_EXIST),
            Some(player) => {
                player.check_and_inc_nonce(nonce);
                let mut marketcard = player.data.sell_card(self.card_index)?; 
                // Shold not error from this point
                if let Some(b) = marketcard.data.0.get_bidder() {
                    let mut bidder = AutomataPlayer::get_from_pid(&b.bidder).unwrap();
                    marketcard.data.0.object.marketid = 0;
                    marketcard.store();
                    bidder.data.cards.push(marketcard.data.0.object.clone());
                    bidder.store();
                }
                //marketcard.data.0.set_bidder(None);
                marketcard.data.0.settleinfo = 2;
                marketcard.store();
                let mut global = STATE.0.borrow_mut();
                player.store();
                MarketCard::emit_event(global.event_id, &marketcard.data);
                global.event_id += 1;
                Ok(())
            }
        }
    }
}

#[derive (Clone)]
pub struct BidCard {
    marketindex: u64,
    price: u64,
}

impl CommandHandler for BidCard {
    fn handle(&self, pid: &[u64; 2], nonce: u64, _rand: &[u64; 4]) -> Result<(), u32> {
        let mut player = AutomataPlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_NOT_EXIST),
            Some(player) => {
                player.check_and_inc_nonce(nonce);
                let mut marketcard = MarketCard::get_object(self.marketindex).unwrap();
                if marketcard.data.0.askprice <= self.price { // direct get the card
                    marketcard.data.0.settleinfo = 2;
                    marketcard.data.0.object.marketid = 0;
                    let prev_bidder = marketcard.data.0.replace_bidder(player, self.price)?;
                    prev_bidder.map(|x| x.store());
                    let mut global = STATE.0.borrow_mut();
                    player.data.cards.push(marketcard.data.0.object.clone());
                    player.store();
                    let mut owner = marketcard.data.0.deal()?;
                    if let Some(card_index) = owner.data.cards.iter().position(|c| c.marketid == marketcard.data.0.marketid) {
                        owner.data.remove_card(card_index);
                    }
                    owner.store();
                    marketcard.store();
                    MarketCard::emit_event(global.event_id, &marketcard.data);
                    global.event_id += 1;
                    Ok(())
                } else if marketcard.data.0.object.marketid != 0 {
                    let prev_bidder = marketcard.data.0.get_bidder();
                    if prev_bidder.map_or(false, |x| x.bidder == player.player_id) {
                        player.data.cost_balance(self.price - prev_bidder.expect("").bidprice);
                    } else {
                        let prev_bidder = marketcard.data.0.replace_bidder(player, self.price)?;
                        prev_bidder.map(|x| x.store());
                    }

                    player.store();
                    let mut global = STATE.0.borrow_mut();
                    marketcard.data.0.settleinfo = 1;
                    marketcard.store();
                    MarketCard::emit_event(global.event_id, &marketcard.data);
                    global.event_id += 1;
                    Ok(())
                } else {
                    Err(ERROR_CARD_IS_IN_USE)
                }
            }
        }
    }
}




#[derive (Clone)]
pub struct Bounty {
    bounty_index: usize,
}

impl CommandHandler for Bounty {
    fn handle(&self, pid: &[u64; 2], nonce: u64, _rand: &[u64; 4]) -> Result<(), u32> {
        let mut player = AutomataPlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_ALREADY_EXIST),
            Some(player) => {
                player.check_and_inc_nonce(nonce);
                if self.bounty_index < 7 {
                    if let Some(v) = player.data.local.0.get(self.bounty_index) {
                        let redeem_info = player.data.redeem_info[self.bounty_index];
                        let cost = CONFIG.get_bounty_cost(redeem_info as u64);
                        if *v > cost as i64 {
                            player.data.local.0[self.bounty_index] = v - (cost as i64);
                            player.data.redeem_info[self.bounty_index] += 1;
                            let reward = CONFIG.get_bounty_reward(redeem_info as u64);
                            player.data.inc_balance(reward);
                            player.data.inc_exp(5);
                            player.store();
                            Ok(())
                        } else {
                            Err(ERROR_NOT_ENOUGH_RESOURCE)
                        }
                    } else {
                        Err(ERROR_INDEX_OUT_OF_BOUND)
                    }
                } else {
                    enforce(self.bounty_index == 7, "check bounty index");
                    let counter = STATE.0.borrow().queue.counter;
                    player.data.collect_interest(counter)?;
                    player.store();
                    Ok(())
                }
            }
        }
    }
}


#[derive (Clone)]
pub struct Deposit {
    data: [u64; 3],
}

impl CommandHandler for Deposit {
    fn handle(&self, pid: &[u64; 2], nonce: u64, _rand: &[u64; 4]) -> Result<(), u32> {
        //zkwasm_rust_sdk::dbg!("deposit\n");
        let mut admin = AutomataPlayer::get_from_pid(pid).unwrap();
        admin.check_and_inc_nonce(nonce);
        let mut player = AutomataPlayer::get_from_pid(&[self.data[0], self.data[1]]);
        let mut state = STATE.0.borrow_mut();
        let counter = state.queue.counter;
        match player.as_mut() {
            None => {
                let mut player = AutomataPlayer::new_from_pid([self.data[0], self.data[1]]);
                player.data.cost_balance(self.data[2])?;
                player.data.update_interest(counter);
                player.store();
            }
            Some(player) => {
                player.data.cost_balance(self.data[2])?;
                player.data.update_interest(counter);
                player.store();
            }
        };
        state.bounty_pool += self.data[2];
        admin.store();
        Ok(()) // no error occurred
    }
}

#[derive (Clone)]
pub struct Withdraw {
    data: [u64; 3],
}

impl CommandHandler for Withdraw {
    fn handle(&self, pid: &[u64; 2], nonce: u64, _rand: &[u64; 4]) -> Result<(), u32> {
        let mut player = AutomataPlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_NOT_EXIST),
            Some(player) => {
                player.check_and_inc_nonce(nonce);
                let mut state = STATE.0.borrow_mut();
                let amount = self.data[0] & 0xffffffff;
                if amount <= state.bounty_pool {
                    let counter = state.queue.counter;
                    player.data.cost_balance(amount)?;
                    let withdrawinfo =
                        WithdrawInfo::new(&[self.data[0], self.data[1], self.data[2]], 0);
                    SettlementInfo::append_settlement(withdrawinfo);
                    player.data.update_interest(counter);
                    state.bounty_pool -= amount;
                    player.store();
                    Ok(())
                } else {
                    Err(ERROR_NOT_ENOUGH_POOL)
                }
            }
        }
    }
}





const INSTALL_PLAYER: u64 = 1;
const INSTALL_OBJECT: u64 = 2;
const RESTART_OBJECT: u64 = 3;
const UPGRADE_OBJECT: u64 = 4;
const INSTALL_CARD: u64 = 5;
const WITHDRAW: u64 = 6;
const DEPOSIT: u64 = 7;
const BOUNTY: u64 = 8;
const COLLECT_ENERGY: u64 = 9;
const LIST_CARD_IN_MARKET: u64 = 10;
const BID_CARD: u64 = 11; // index, price
const SELL_CARD: u64 = 12;

impl Transaction {
    pub fn decode_error(e: u32) -> &'static str {
        match e {
            ERROR_PLAYER_NOT_EXIST => "PlayerNotExist",
            ERROR_PLAYER_ALREADY_EXIST => "PlayerAlreadyExist",
            ERROR_NOT_ENOUGH_BALANCE => "NotEnoughBalance",
            ERROR_INDEX_OUT_OF_BOUND => "IndexOutofBound",
            ERROR_NOT_ENOUGH_RESOURCE => "NotEnoughResource",
            ERROR_NOT_ENOUGH_LEVEL => "NotEnoughLevel",
            ERROR_NOT_ENOUGH_POOL => "NotEnoughFundInPool",
            ERROR_CARD_IS_IN_USE => "CardIsInUse",
            ERROR_BID_PRICE_INSUFFICIENT => "BidPriceInSufficient",
            ERROR_NO_BIDDER=> "NoBidder",
            _ => "Unknown",
        }
    }
    pub fn decode(params: &[u64]) -> Self {
        let cmd = params[0] & 0xff;
        let nonce = params[0] >> 16;
        let command = if cmd == WITHDRAW {
            enforce(params[1] == 0, "check withdraw index"); // only token index 0 is supported
            Command::Withdraw (Withdraw {
                data: [params[2], params[3], params[4]]
            })
        } else if cmd == INSTALL_OBJECT {
            Command::InstallObject (InstallObject {
                object_index: params[1] as usize,
                modifiers: params[2].to_le_bytes(),
            })
        } else if cmd == RESTART_OBJECT {
            Command::RestartObject (RestartObject {
                object_index: params[1] as usize,
                modifiers: params[2].to_le_bytes(),
            })
        } else if cmd == DEPOSIT {
            zkwasm_rust_sdk::dbg!("deposit params: {:?}\n", params);
            enforce(params[3] == 0, "check deposit index"); // only token index 0 is supported
            Command::Deposit (Deposit {
                data: [params[1], params[2], params[4]]
            })
        } else if cmd == UPGRADE_OBJECT {
            Command::UpgradeObject(UpgradeObject {
                object_index: params[1] as usize,
                feature_index: params[2] as usize,
            })
        } else if cmd == BOUNTY {
            Command::Bounty (Bounty {
                bounty_index: params[1] as usize
            })
        } else if cmd == INSTALL_CARD {
            Command::InstallCard (InstallCard {})
        } else if cmd == BID_CARD {
            Command::BidCard (BidCard {
                marketindex: params[1],
                price: params[2],
            })
        } else if cmd == SELL_CARD {
            Command::SellCard (SellCard {
                card_index: params[1] as usize,
            })
        } else if cmd == LIST_CARD_IN_MARKET {
            Command::ListCardInMarket (ListCardInMarket{
                card_index: params[1] as usize,
                ask_price: params[2],
            })
        } else if cmd == INSTALL_PLAYER {
            Command::InstallPlayer
        } else if cmd == COLLECT_ENERGY {
            Command::CollectEnergy
        } else {
            Command::Tick
        };

        Transaction {
            command,
            nonce,
        }
    }

    pub fn install_player(pid: &[u64; 2]) -> Result<(), u32> {
        let player = AutomataPlayer::get_from_pid(pid);
        let counter = STATE.0.borrow().queue.counter;
        match player {
            Some(_) => Err(ERROR_PLAYER_ALREADY_EXIST),
            None => {
                let mut player = AutomataPlayer::new_from_pid(*pid);
                player.data.update_interest(counter);
                player.store();
                Ok(())
            }
        }

    }

    pub fn collect_energy(pid: &[u64; 2]) -> Result<(), u32> {
        let player = AutomataPlayer::get_from_pid(pid);
        let counter = STATE.0.borrow().queue.counter;
        match player {
            Some(mut player) => {
                player.data.collect_energy(counter)?;
                player.store();
                Ok(())
            }
            None => Err(ERROR_PLAYER_NOT_EXIST),
        }
    }

    pub fn collect_interest(pid: &[u64; 2]) -> Result<(), u32> {
        let player = AutomataPlayer::get_from_pid(pid);
        let counter = STATE.0.borrow().queue.counter;
        match player {
            Some(mut player) => {
                player.data.collect_interest(counter)?;
                player.store();
                Ok(())
            }
            None => Err(ERROR_PLAYER_NOT_EXIST),
        }
    }

    pub fn process(&self, pkey: &[u64; 4], rand: &[u64; 4]) -> Vec<u64> {
        let e = match self.command.clone() {
            Command::InstallPlayer => Self::install_player(&AutomataPlayer::pkey_to_pid(&pkey))
                .map_or_else(|e| e, |_| 0),
            Command::InstallObject(cmd) => cmd.handle(&AutomataPlayer::pkey_to_pid(&pkey), self.nonce, rand)
                .map_or_else(|e| e, |_| 0),
            Command::CollectEnergy => Self::collect_energy(&AutomataPlayer::pkey_to_pid(&pkey))
                .map_or_else(|e| e, |_| 0),
            Command::RestartObject(cmd) => cmd.handle(&AutomataPlayer::pkey_to_pid(&pkey), self.nonce, rand)
                .map_or_else(|e| e, |_| 0),
            Command::UpgradeObject(cmd) => cmd.handle(&AutomataPlayer::pkey_to_pid(&pkey), self.nonce, rand)
                .map_or_else(|e| e, |_| 0),
            Command::Withdraw(cmd)=> cmd.handle(&AutomataPlayer::pkey_to_pid(pkey), self.nonce, rand)
                .map_or_else(|e| e, |_| 0),
            Command::InstallCard(cmd) => cmd.handle(&AutomataPlayer::pkey_to_pid(pkey), self.nonce, rand)
                .map_or_else(|e| e, |_| 0),
            Command::ListCardInMarket(cmd) => cmd.handle(&AutomataPlayer::pkey_to_pid(pkey), self.nonce, rand)
                .map_or_else(|e| e, |_| 0),
            Command::SellCard(cmd) => cmd.handle(&AutomataPlayer::pkey_to_pid(pkey), self.nonce, rand)
                .map_or_else(|e| e, |_| 0),
            Command::BidCard(cmd) => cmd.handle(&AutomataPlayer::pkey_to_pid(pkey), self.nonce, rand)
                .map_or_else(|e| e, |_| 0),

            Command::Deposit(cmd) => {
                enforce(*pkey == *ADMIN_PUBKEY, "check admin key of deposit");
                cmd.handle(&AutomataPlayer::pkey_to_pid(pkey), self.nonce, rand)
                    .map_or_else(|e| e, |_| 0)
            },
            Command::Bounty(cmd) => cmd.handle(&AutomataPlayer::pkey_to_pid(pkey), self.nonce, rand)
                .map_or_else(|e| e, |_| 0),

            Command::Tick => {
                enforce(*pkey == *ADMIN_PUBKEY, "check admin key");
                zkwasm_rust_sdk::dbg!("perform borrow ....\n");
                let mut state = STATE.0.borrow_mut();
                zkwasm_rust_sdk::dbg!("perform borrow done.n");
                state.queue.tick();
                zkwasm_rust_sdk::dbg!("tick done. n");
                //STATE.0.borrow_mut().queue.tick();
                0
            }
        };
        let event_id = STATE.0.borrow().event_id;
        let events = clear_events(vec![e as u64, event_id]);
        zkwasm_rust_sdk::dbg!("events: {:?}", events);
        events
    }
}

pub struct SafeState(RefCell<State>);
unsafe impl Sync for SafeState {}

lazy_static::lazy_static! {
    pub static ref STATE: SafeState = SafeState (RefCell::new(State::new()));
}

pub struct State {
    supplier: u64,
    bounty_pool: u64,
    start_time_stamp: u64,
    market_id: u64,
    event_id: u64,
    queue: EventQueue<Event>,
}

#[derive(Debug, Serialize)]
struct StateObserve {
    bounty_pool: u64,
    counter: u64,
}

impl State {
    pub fn new() -> Self {
        State {
            supplier: 1000,
            start_time_stamp: 0,
            bounty_pool: 20000000,
            market_id: 1,
            event_id: 1,
            queue: EventQueue::new(),
        }
    }
    pub fn snapshot() -> String {
        let counter = STATE.0.borrow().queue.counter;
        let bounty_pool = STATE.0.borrow().bounty_pool;
        let state = StateObserve {
            counter,
            bounty_pool
        };
        serde_json::to_string(&state).unwrap()
    }
    pub fn get_state(pid: Vec<u64>) -> String {
        let player = AutomataPlayer::get(&pid.try_into().unwrap()).unwrap();
        serde_json::to_string(&player).unwrap()
    }

    pub fn preempt() -> bool {
        let counter = STATE.0.borrow().queue.counter;
        let timestamp = STATE.0.borrow().start_time_stamp;
        if counter % 16 == 0  && counter != timestamp {
            true
        } else {
            false
        }
    }

    pub fn flush_settlement() -> Vec<u8> {
        SettlementInfo::flush_settlement()
    }

    pub fn rand_seed() -> u64 {
        0
    }

    pub fn store() {
        let mut state = STATE.0.borrow_mut();
        let mut v = Vec::with_capacity(state.queue.list.len() + 10);
        v.push(state.supplier);
        v.push(state.bounty_pool);
        v.push(state.market_id);
        v.push(state.event_id);
        state.queue.to_data(&mut v);
        let kvpair = unsafe { &mut MERKLE_MAP };
        kvpair.set(&[0, 0, 0, 0], v.as_slice());
        state.queue.store();
        let root = kvpair.merkle.root.clone();
        zkwasm_rust_sdk::dbg!("root after store: {:?}\n", root);
    }
    pub fn initialize() {
        let mut state = STATE.0.borrow_mut();
        let kvpair = unsafe { &mut MERKLE_MAP };
        let mut data = kvpair.get(&[0, 0, 0, 0]);
        if !data.is_empty() {
            let mut data = data.iter_mut();
            state.supplier = *data.next().unwrap();
            state.bounty_pool = *data.next().unwrap();
            state.market_id = *data.next().unwrap();
            state.event_id = *data.next().unwrap();
            state.queue = EventQueue::from_data(&mut data);
            state.start_time_stamp = state.queue.counter;
        }
    }
}
