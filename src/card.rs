use crate::config::LOCAL_ATTRIBUTES_SIZE;
use serde::Serialize;
use std::slice::IterMut;
use zkwasm_rest_abi::StorageData;

#[derive(Clone, Debug, Serialize)]
pub struct Card {
    pub duration: u64,
    pub attributes: [i8; 8],
}

impl Card {
    fn new(duration: u64, attributes: [i8; LOCAL_ATTRIBUTES_SIZE]) -> Self {
        Card {
            duration,
            attributes,
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
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(self.duration);
        data.push(u64::from_le_bytes(self.attributes.map(|x| x as u8)));
    }
}

lazy_static::lazy_static! {
    pub static ref DEFAULT_CARDS: Vec<Card> = vec![
        Card::new(40, [-10, -10, 20, 0, 0, 0, 0, 0]),
        Card::new(60, [15, 0, -10, 0, 0, 0, 0, 0]),
        Card::new(70, [0, 15, -10, 0, 0, 0, 0, 0]),
        Card::new(65, [10, 0, -30, 0, 20, 0, 0, 0]),
    ];
    pub static ref CARD_NAME: Vec<&'static str> = vec![
        "Biogen",
        "Crystara",
        "AstroMine",
        "CrystaBloom",
    ];
}
