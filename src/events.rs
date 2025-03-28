use crate::player::AutomataPlayer;
use core::slice::IterMut;
use zkwasm_rest_abi::StorageData;
use zkwasm_rest_convention::EventHandler;

#[derive(Clone)]
pub struct Event {
    pub owner: [u64; 2],
    pub object_index: usize,
    pub delta: usize,
}

impl StorageData for Event {
    fn to_data(&self, buf: &mut Vec<u64>) {
        buf.push(self.owner[0]);
        buf.push(self.owner[1]);
        buf.push(((self.object_index as u64) << 32) | self.delta as u64);
    }
    fn from_data(u64data: &mut IterMut<u64>) -> Event {
        let owner = [*u64data.next().unwrap(), *u64data.next().unwrap()];
        let f = *u64data.next().unwrap();
        Event {
            owner,
            object_index: (f >> 32) as usize,
            delta: (f & 0xffffffff) as usize,
        }
    }
}

impl EventHandler for Event {
    fn u64size() -> usize {
        3
    }
    fn get_delta(&self) -> usize {
        self.delta
    }
    fn progress(&mut self, d: usize) {
        self.delta -= d;
    }
    fn handle(&mut self, counter: u64) -> Option<Self> {
        let owner_id = self.owner;
        let object_index = self.object_index;
        let mut player = AutomataPlayer::get_from_pid(&owner_id).unwrap();
        let m = if player.data.energy == 0 {
            player.data.objects.get_mut(object_index).unwrap().halt();
            None
        } else {
            player.data.apply_object_card(object_index, counter)
        };
        let event = if let Some(delta) = m {
            if player.data.objects[object_index].get_modifier_index() == 0 {
                if player.data.energy < object_index as u16 + 1 {
                    player.data.energy = 0;
                } else {
                    player.data.energy -= (object_index + 1) as u16;
                }
                player.data.inc_exp(1);
            }
            Some(Event {
                owner: owner_id,
                object_index,
                delta,
            })
        } else {
            None
        };
        player.store();
        //zkwasm_rust_sdk::dbg!("store end...\n");
        event
    }
}
