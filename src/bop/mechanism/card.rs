use crate::bop::state::card_game_shared_state::CardKind::*;
use crate::bop::state::card_game_shared_state::{CardGameSharedState, CardKind};
use wasm_bindgen_test::console_log;

#[derive(Debug)]
pub struct Card {
    pub sold_for: u32,
    pub is_used: bool,
    pub card_kind: CardKind,
}

impl Card {
    pub fn from(card_kind: CardKind) -> Card {
        Card {
            sold_for: 0,
            is_used: false,
            card_kind,
        }
    }
    pub fn card_set_default() -> Vec<Card> {
        vec![
            Treasure,
            GoldenSkin,
            Chaos,
            GoldenDagger,
            ATKSwap,
            DEFSwap,
            Excalibur,
            Shrink,
            ArmourBreak,
            LongSword,
            GainUp,
            Weakness,
            BuildUp,
            LeatherArmour,
            Dagger,
            Balance,
            ChainMail,
            MagicBolt,
            Cure,
            HPSwap,
            GoldenHeal,
            MagicBolt,
        ]
        .into_iter()
        .map(|card_kind| Card::from(card_kind))
        .collect()
    }

    pub fn create_update_status_func(
        target_is_own: bool,
        status: String,
        amount: i32,
    ) -> impl FnMut(&mut CardGameSharedState) {
        move |card_game_shared_state: &mut CardGameSharedState| {
            let own_player_index = card_game_shared_state.own_player_index;
            let target_player_index = if target_is_own {
                own_player_index
            } else {
                (own_player_index + 1) % card_game_shared_state.players.len()
            };
            let state = &mut card_game_shared_state.players[target_player_index].player_state;
            match status.as_str() {
                "ATK" => state.attack_point = (state.attack_point as i32 + amount) as u32,
                "DEF" => state.defence_point = (state.defence_point as i32 + amount) as u32,
                "HP" => {
                    state.current_hp = ((state.current_hp as i32 + amount) as u32).min(state.max_hp)
                }
                "MHP" => state.max_hp = (state.max_hp as i32 + amount) as u32,
                "Money" => {
                    state.current_money_amount = (state.current_money_amount as i32 + amount) as u32
                }
                "Gain" => {
                    state.estimated_money_amount =
                        (state.estimated_money_amount as i32 + amount) as u32
                }
                _ => {
                    panic!()
                }
            }
        }
    }

    pub fn create_update_status_golden_func(
        target_is_own: bool,
        status: String,
        scale: f64,
    ) -> impl FnMut(&mut CardGameSharedState) {
        move |card_game_shared_state: &mut CardGameSharedState| {
            let own_player_index = card_game_shared_state.own_player_index;
            let target_player_index = if target_is_own {
                own_player_index
            } else {
                (own_player_index + 1) % card_game_shared_state.players.len()
            };
            let state = &mut card_game_shared_state.players[target_player_index].player_state;
            match status.as_str() {
                "ATK" => state.attack_point += (state.current_money_amount as f64 * scale) as u32,
                "DEF" => state.defence_point += (state.current_money_amount as f64 * scale) as u32,
                "HP" => {
                    state.current_hp = (state.current_hp
                        + (state.current_money_amount as f64 * scale) as u32)
                        .min(state.max_hp)
                }
                "MHP" => state.max_hp += (state.current_money_amount as f64 * scale) as u32,
                _ => {
                    panic!()
                }
            }
        }
    }
    // HP/MHP は一度に処理する
    // プレイヤーの入力を取れないうちはプレイヤー1と2に固定
    pub fn create_swap_status_func(status: String) -> impl FnMut(&mut CardGameSharedState) {
        move |card_game_shared_state: &mut CardGameSharedState| match status.as_str() {
            "ATK" => {
                let state_a_amount = card_game_shared_state.players[0].player_state.attack_point;
                let state_b_amount = card_game_shared_state.players[1].player_state.attack_point;
                card_game_shared_state.players[0].player_state.attack_point = state_b_amount;
                card_game_shared_state.players[1].player_state.attack_point = state_a_amount;
            }
            "DEF" => {
                let state_a_amount = card_game_shared_state.players[0].player_state.defence_point;
                let state_b_amount = card_game_shared_state.players[1].player_state.defence_point;
                card_game_shared_state.players[0].player_state.defence_point = state_b_amount;
                card_game_shared_state.players[1].player_state.defence_point = state_a_amount;
            }
            "HP" | "MHP" => {
                let state_a_amount = card_game_shared_state.players[0].player_state.max_hp;
                let state_b_amount = card_game_shared_state.players[1].player_state.max_hp;
                card_game_shared_state.players[0].player_state.max_hp = state_b_amount;
                card_game_shared_state.players[1].player_state.max_hp = state_a_amount;
                let state_a_amount = card_game_shared_state.players[0].player_state.current_hp;
                let state_b_amount = card_game_shared_state.players[1].player_state.current_hp;
                card_game_shared_state.players[0].player_state.current_hp = state_b_amount;
                card_game_shared_state.players[1].player_state.current_hp = state_a_amount;
            }
            _ => {
                panic!()
            }
        }
    }
    pub fn create_cut_status_func(
        target_is_own: bool,
        status: String,
    ) -> impl FnMut(&mut CardGameSharedState) {
        move |card_game_shared_state: &mut CardGameSharedState| {
            let own_player_index = card_game_shared_state.own_player_index;
            let target_player_index = if target_is_own {
                own_player_index
            } else {
                (own_player_index + 1) % card_game_shared_state.players.len()
            };
            let state = &mut card_game_shared_state.players[target_player_index].player_state;
            match status.as_str() {
                "ATK" => state.attack_point /= 2,
                "DEF" => state.defence_point /= 2,
                "HP" => state.current_hp /= 2,
                "MHP" => state.max_hp /= 2,
                _ => {
                    panic!()
                }
            }
        }
    }

    pub fn create_balance_func(
        target_is_own: bool,
        status_a: String,
        status_b: String,
        is_balance: bool,
        modifier: i32,
    ) -> impl FnMut(&mut CardGameSharedState) {
        move |card_game_shared_state: &mut CardGameSharedState| {
            let own_player_index = card_game_shared_state.own_player_index;
            let target_player_index = if target_is_own {
                own_player_index
            } else {
                (own_player_index + 1) % card_game_shared_state.players.len()
            };
            let target_player_status =
                &mut card_game_shared_state.players[target_player_index].player_state;
            let status_a_amount = match status_a.as_str() {
                "HP" => target_player_status.current_hp,
                "MHP" => target_player_status.max_hp,
                "ATK" => target_player_status.attack_point,
                "DEF" => target_player_status.defence_point,
                _ => {
                    console_log!("does not implemented {}", status_a);
                    panic!()
                }
            };
            let status_b_amount = match status_b.as_str() {
                "HP" => target_player_status.current_hp,
                "MHP" => target_player_status.max_hp,
                "ATK" => target_player_status.attack_point,
                "DEF" => target_player_status.defence_point,
                _ => {
                    console_log!("does not implemented {}", status_b);
                    panic!()
                }
            };
            let new_amount = if is_balance {
                status_a_amount.max(status_b_amount)
            } else {
                status_a_amount.min(status_b_amount)
            };
            match status_a.as_str() {
                "HP" => {
                    target_player_status.current_hp =
                        ((new_amount as i32 + modifier) as u32).min(target_player_status.max_hp)
                }
                "MHP" => target_player_status.max_hp = (new_amount as i32 + modifier) as u32,
                "ATK" => target_player_status.attack_point = (new_amount as i32 + modifier) as u32,
                "DEF" => target_player_status.defence_point = (new_amount as i32 + modifier) as u32,
                _ => {
                    panic!()
                }
            };
            match status_b.as_str() {
                "HP" => {
                    target_player_status.current_hp =
                        ((new_amount as i32 + modifier) as u32).min(target_player_status.max_hp)
                }
                "MHP" => target_player_status.max_hp = (new_amount as i32 + modifier) as u32,
                "ATK" => target_player_status.attack_point = (new_amount as i32 + modifier) as u32,
                "DEF" => target_player_status.defence_point = (new_amount as i32 + modifier) as u32,
                _ => {
                    panic!()
                }
            };
        }
    }
    pub fn combine_func(
        mut functions: Vec<Box<dyn FnMut(&mut CardGameSharedState)>>,
    ) -> impl FnMut(&mut CardGameSharedState) {
        move |card_game_shared_state: &mut CardGameSharedState| {
            for func in functions.iter_mut() {
                func(card_game_shared_state)
            }
        }
    }
    pub fn get_use_func(&self) -> Box<dyn FnMut(&mut CardGameSharedState)> {
        match self.card_kind {
            Dagger => Box::new(Card::create_update_status_func(true, "ATK".to_string(), 5)),
            LongSword => Box::new(Card::create_update_status_func(true, "ATK".to_string(), 10)),
            BuildUp => Box::new(Card::combine_func(vec![
                Box::new(Card::create_update_status_func(true, "MHP".to_string(), 10)),
                Box::new(Card::create_update_status_func(true, "HP".to_string(), 10)),
            ])),
            GainUp => Box::new(Card::create_update_status_func(true, "Gain".to_string(), 1)),
            ArmourBreak => Box::new(Card::create_cut_status_func(false, "DEF".to_string())),
            Weakness => Box::new(Card::create_cut_status_func(false, "ATK".to_string())),
            LeatherArmour => Box::new(Card::create_update_status_func(true, "DEF".to_string(), 5)),
            ChainMail => Box::new(Card::create_update_status_func(true, "DEF".to_string(), 10)),
            MagicBolt => Box::new(Card::create_update_status_func(
                false,
                "HP".to_string(),
                -15,
            )),
            Cure => Box::new(Card::create_update_status_func(true, "HP".to_string(), 20)),
            HPSwap => Box::new(Card::create_swap_status_func("HP".to_string())),
            ATKSwap => Box::new(Card::create_swap_status_func("ATK".to_string())),
            DEFSwap => Box::new(Card::create_swap_status_func("DEF".to_string())),
            Balance => Box::new(Card::create_balance_func(
                true,
                "ATK".to_string(),
                "DEF".to_string(),
                true,
                1,
            )),
            Shrink => Box::new(Card::create_balance_func(
                false,
                "ATK".to_string(),
                "DEF".to_string(),
                false,
                -1,
            )),
            GoldenDagger => Box::new(Card::create_update_status_golden_func(
                true,
                "ATK".to_string(),
                1.0,
            )),
            GoldenSkin => Box::new(Card::create_update_status_golden_func(
                true,
                "DEF".to_string(),
                1.0,
            )),
            GoldenHeal => Box::new(Card::create_update_status_golden_func(
                true,
                "HP".to_string(),
                2.0,
            )),
            Treasure => Box::new(Card::create_update_status_func(
                true,
                "Money".to_string(),
                5,
            )),
            Chaos => Box::new(Card::combine_func(vec![
                Box::new(Card::create_update_status_func(true, "HP".to_string(), -5)),
                Box::new(Card::create_update_status_func(false, "HP".to_string(), -5)),
                Box::new(Card::create_update_status_func(true, "ATK".to_string(), 5)),
                Box::new(Card::create_update_status_func(false, "ATK".to_string(), 5)),
                Box::new(Card::create_update_status_func(true, "DEF".to_string(), -5)),
                Box::new(Card::create_update_status_func(
                    false,
                    "DEF".to_string(),
                    -5,
                )),
            ])),
            Excalibur => Box::new(Card::combine_func(vec![
                Box::new(Card::create_update_status_func(true, "HP".to_string(), 10)),
                Box::new(Card::create_update_status_func(true, "ATK".to_string(), 10)),
                Box::new(Card::create_update_status_func(true, "DEF".to_string(), 10)),
            ])),
        }
    }
}