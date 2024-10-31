use crate::bop::mechanism::item::ItemKind::{
    ATKSwap, ArmourBreak, Balance, BuildUp, ChainMail, Chaos, Cure, DEFSwap, Dagger, Excalibur,
    GainUp, GoldenDagger, GoldenHeal, GoldenSkin, HPSwap, LeatherArmour, LongSword, MagicBolt,
    Shrink, Treasure, Weakness,
};
use crate::bop::state::bop_shared_state::BoPSharedState;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use wasm_bindgen_test::console_log;

#[derive(Clone, Debug)]
pub struct Item {
    pub sold_for: u32,
    pub is_used: bool,
    pub item_kind: ItemKind,
}

impl Item {
    pub fn from(item_kind: ItemKind) -> Item {
        Item {
            sold_for: 0,
            is_used: false,
            item_kind,
        }
    }
    pub fn item_set_default() -> Vec<Item> {
        let mut rng = rand::thread_rng();
        let mut items = vec![
            Treasure,
            GoldenSkin,
            Chaos,
            GoldenDagger,
            ATKSwap,
            DEFSwap,
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
        .map(|item_kind| Item::from(item_kind))
        .collect::<Vec<Item>>();
        items.shuffle(&mut rng);
        items.push(Item::from(Excalibur));
        items
    }

    pub fn create_update_status_func(
        use_player_index: usize,
        target_is_own: bool,
        status: String,
        amount: i32,
    ) -> impl FnMut(&mut BoPSharedState) {
        move |item_game_shared_state: &mut BoPSharedState| {
            let target_player_index = if target_is_own {
                use_player_index
            } else {
                (use_player_index + 1) % item_game_shared_state.players.len()
            };
            let state = &mut item_game_shared_state.players[target_player_index].player_state;
            match status.as_str() {
                "ATK" => state.attack_point = (state.attack_point as i32 + amount).max(0) as u32,
                "DEF" => state.defence_point = (state.defence_point as i32 + amount).max(0) as u32,
                "HP" => {
                    state.current_hp =
                        ((state.current_hp as i32 + amount).max(0) as u32).min(state.max_hp)
                }
                "MHP" => state.max_hp = (state.max_hp as i32 + amount).max(0) as u32,
                "Money" => {
                    state.current_money_amount =
                        (state.current_money_amount as i32 + amount).max(0) as u32
                }
                "Gain" => {
                    state.estimated_money_amount =
                        (state.estimated_money_amount as i32 + amount).max(0) as u32
                }
                _ => {
                    panic!()
                }
            }
        }
    }

    pub fn create_update_status_golden_func(
        use_player_index: usize,
        target_is_own: bool,
        status: String,
        scale: f64,
    ) -> impl FnMut(&mut BoPSharedState) {
        move |bop_shared_state: &mut BoPSharedState| {
            let target_player_index = if target_is_own {
                use_player_index
            } else {
                (use_player_index + 1) % bop_shared_state.players.len()
            };
            let state = &mut bop_shared_state.players[target_player_index].player_state;
            match status.as_str() {
                "ATK" => {
                    state.attack_point +=
                        (state.current_money_amount as f64 * scale).max(0.0) as u32
                }
                "DEF" => {
                    state.defence_point +=
                        (state.current_money_amount as f64 * scale).max(0.0) as u32
                }
                "HP" => {
                    state.current_hp = (state.current_hp
                        + (state.current_money_amount as f64 * scale).max(0.0) as u32)
                        .min(state.max_hp)
                }
                "MHP" => {
                    state.max_hp += (state.current_money_amount as f64 * scale).max(0.0) as u32
                }
                _ => {
                    panic!()
                }
            }
        }
    }
    // HP/MHP は一度に処理する
    // プレイヤーの入力を取れないうちはプレイヤー1と2に固定
    pub fn create_swap_status_func(status: String) -> impl FnMut(&mut BoPSharedState) {
        move |bop_shared_state: &mut BoPSharedState| match status.as_str() {
            "ATK" => {
                let state_a_amount = bop_shared_state.players[0].player_state.attack_point;
                let state_b_amount = bop_shared_state.players[1].player_state.attack_point;
                bop_shared_state.players[0].player_state.attack_point = state_b_amount;
                bop_shared_state.players[1].player_state.attack_point = state_a_amount;
            }
            "DEF" => {
                let state_a_amount = bop_shared_state.players[0].player_state.defence_point;
                let state_b_amount = bop_shared_state.players[1].player_state.defence_point;
                bop_shared_state.players[0].player_state.defence_point = state_b_amount;
                bop_shared_state.players[1].player_state.defence_point = state_a_amount;
            }
            "HP" | "MHP" => {
                let state_a_amount = bop_shared_state.players[0].player_state.max_hp;
                let state_b_amount = bop_shared_state.players[1].player_state.max_hp;
                bop_shared_state.players[0].player_state.max_hp = state_b_amount;
                bop_shared_state.players[1].player_state.max_hp = state_a_amount;
                let state_a_amount = bop_shared_state.players[0].player_state.current_hp;
                let state_b_amount = bop_shared_state.players[1].player_state.current_hp;
                bop_shared_state.players[0].player_state.current_hp = state_b_amount;
                bop_shared_state.players[1].player_state.current_hp = state_a_amount;
            }
            _ => {
                panic!()
            }
        }
    }
    pub fn create_cut_status_func(
        use_player_index: usize,
        target_is_own: bool,
        status: String,
    ) -> impl FnMut(&mut BoPSharedState) {
        move |bop_shared_state: &mut BoPSharedState| {
            let target_player_index = if target_is_own {
                use_player_index
            } else {
                (use_player_index + 1) % bop_shared_state.players.len()
            };
            let state = &mut bop_shared_state.players[target_player_index].player_state;
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
        use_player_index: usize,
        target_is_own: bool,
        status_a: String,
        status_b: String,
        is_balance: bool,
        modifier: i32,
    ) -> impl FnMut(&mut BoPSharedState) {
        move |bop_shared_state: &mut BoPSharedState| {
            let target_player_index = if target_is_own {
                use_player_index
            } else {
                (use_player_index + 1) % bop_shared_state.players.len()
            };
            let target_player_status =
                &mut bop_shared_state.players[target_player_index].player_state;
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
                    target_player_status.current_hp = ((new_amount as i32 + modifier).max(0) as u32)
                        .min(target_player_status.max_hp)
                }
                "MHP" => target_player_status.max_hp = (new_amount as i32 + modifier).max(0) as u32,
                "ATK" => {
                    target_player_status.attack_point = (new_amount as i32 + modifier).max(0) as u32
                }
                "DEF" => {
                    target_player_status.defence_point =
                        (new_amount as i32 + modifier).max(0) as u32
                }
                _ => {
                    panic!()
                }
            };
            match status_b.as_str() {
                "HP" => {
                    target_player_status.current_hp = ((new_amount as i32 + modifier).max(0) as u32)
                        .min(target_player_status.max_hp)
                }
                "MHP" => target_player_status.max_hp = (new_amount as i32 + modifier).max(0) as u32,
                "ATK" => {
                    target_player_status.attack_point = (new_amount as i32 + modifier).max(0) as u32
                }
                "DEF" => {
                    target_player_status.defence_point =
                        (new_amount as i32 + modifier).max(0) as u32
                }
                _ => {
                    panic!()
                }
            };
        }
    }
    pub fn combine_func(
        mut functions: Vec<Box<dyn FnMut(&mut BoPSharedState)>>,
    ) -> impl FnMut(&mut BoPSharedState) {
        move |bop_shared_state: &mut BoPSharedState| {
            for func in functions.iter_mut() {
                func(bop_shared_state)
            }
        }
    }
    pub fn get_use_func(
        &self,
        use_player_index: usize,
    ) -> Box<dyn FnMut(&mut BoPSharedState)> {
        match self.item_kind {
            Dagger => Box::new(Item::create_update_status_func(
                use_player_index,
                true,
                "ATK".to_string(),
                5,
            )),
            LongSword => Box::new(Item::create_update_status_func(
                use_player_index,
                true,
                "ATK".to_string(),
                10,
            )),
            BuildUp => Box::new(Item::combine_func(vec![
                Box::new(Item::create_update_status_func(
                    use_player_index,
                    true,
                    "MHP".to_string(),
                    10,
                )),
                Box::new(Item::create_update_status_func(
                    use_player_index,
                    true,
                    "HP".to_string(),
                    10,
                )),
            ])),
            GainUp => Box::new(Item::create_update_status_func(
                use_player_index,
                true,
                "Gain".to_string(),
                1,
            )),
            ArmourBreak => Box::new(Item::create_cut_status_func(
                use_player_index,
                false,
                "DEF".to_string(),
            )),
            Weakness => Box::new(Item::create_cut_status_func(
                use_player_index,
                false,
                "ATK".to_string(),
            )),
            LeatherArmour => Box::new(Item::create_update_status_func(
                use_player_index,
                true,
                "DEF".to_string(),
                5,
            )),
            ChainMail => Box::new(Item::create_update_status_func(
                use_player_index,
                true,
                "DEF".to_string(),
                10,
            )),
            MagicBolt => Box::new(Item::create_update_status_func(
                use_player_index,
                false,
                "HP".to_string(),
                -15,
            )),
            Cure => Box::new(Item::create_update_status_func(
                use_player_index,
                true,
                "HP".to_string(),
                20,
            )),
            HPSwap => Box::new(Item::create_swap_status_func("HP".to_string())),
            ATKSwap => Box::new(Item::create_swap_status_func("ATK".to_string())),
            DEFSwap => Box::new(Item::create_swap_status_func("DEF".to_string())),
            Balance => Box::new(Item::create_balance_func(
                use_player_index,
                true,
                "ATK".to_string(),
                "DEF".to_string(),
                true,
                1,
            )),
            Shrink => Box::new(Item::create_balance_func(
                use_player_index,
                false,
                "ATK".to_string(),
                "DEF".to_string(),
                false,
                -1,
            )),
            GoldenDagger => Box::new(Item::create_update_status_golden_func(
                use_player_index,
                true,
                "ATK".to_string(),
                1.0,
            )),
            GoldenSkin => Box::new(Item::create_update_status_golden_func(
                use_player_index,
                true,
                "DEF".to_string(),
                1.0,
            )),
            GoldenHeal => Box::new(Item::create_update_status_golden_func(
                use_player_index,
                true,
                "HP".to_string(),
                2.0,
            )),
            Treasure => Box::new(Item::create_update_status_func(
                use_player_index,
                true,
                "Money".to_string(),
                5,
            )),
            Chaos => Box::new(Item::combine_func(vec![
                Box::new(Item::create_update_status_func(
                    use_player_index,
                    true,
                    "HP".to_string(),
                    -5,
                )),
                Box::new(Item::create_update_status_func(
                    use_player_index,
                    false,
                    "HP".to_string(),
                    -5,
                )),
                Box::new(Item::create_update_status_func(
                    use_player_index,
                    true,
                    "ATK".to_string(),
                    5,
                )),
                Box::new(Item::create_update_status_func(
                    use_player_index,
                    false,
                    "ATK".to_string(),
                    5,
                )),
                Box::new(Item::create_update_status_func(
                    use_player_index,
                    true,
                    "DEF".to_string(),
                    -5,
                )),
                Box::new(Item::create_update_status_func(
                    use_player_index,
                    false,
                    "DEF".to_string(),
                    -5,
                )),
            ])),
            Excalibur => Box::new(Item::combine_func(vec![
                Box::new(Item::create_update_status_func(
                    use_player_index,
                    true,
                    "HP".to_string(),
                    10,
                )),
                Box::new(Item::create_update_status_func(
                    use_player_index,
                    true,
                    "ATK".to_string(),
                    10,
                )),
                Box::new(Item::create_update_status_func(
                    use_player_index,
                    true,
                    "DEF".to_string(),
                    10,
                )),
            ])),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemKind {
    LongSword,
    LeatherArmour,
    Dagger,
    Balance,
    Cure,
    Shrink,
    ArmourBreak,
    GainUp,
    Weakness,
    ChainMail,
    MagicBolt,
    BuildUp,
    HPSwap,
    GoldenHeal,
    Treasure,
    GoldenSkin,
    Chaos,
    GoldenDagger,
    ATKSwap,
    DEFSwap,
    Excalibur,
}

impl ItemKind {
    pub fn get_item_name(&self) -> String {
        match self {
            LongSword => "ロングソード",
            LeatherArmour => "レザーアーマー",
            Dagger => "ダガー",
            Balance => "バランス",
            Cure => "キュア",
            Shrink => "シュリンク",
            ArmourBreak => "アーマーブレイク",
            GainUp => "ゲインアップ",
            Weakness => "ウィークネス",
            ChainMail => "チェインメイル",
            MagicBolt => "マジックボルト",
            BuildUp => "ビルドアップ",
            HPSwap => "HPスワップ",
            GoldenHeal => "ゴールデンヒール",
            Treasure => "トレジャー",
            GoldenSkin => "ゴールデンスキン",
            Chaos => "カオス",
            GoldenDagger => "ゴールデンダガー",
            ATKSwap => "ATKスワップ",
            DEFSwap => "DEFスワップ",
            Excalibur => "エクスカリバー",
        }
        .to_string()
    }
    pub fn get_item_description(&self) -> String {
        match self {
            LongSword => "自己ATK+10",
            LeatherArmour => "自己DEF+5",
            Dagger => "自己ATK+5",
            Balance => "自己ATK,DEFを高い方に合わせ+1",
            Cure => "自己HP+20",
            Shrink => "相手ATK,DEFを低い方に合わせ-1",
            ArmourBreak => "相手DEF半減",
            GainUp => "自己獲得Money+1",
            Weakness => "相手ATK半減",
            ChainMail => "自己DEF+10",
            MagicBolt => "相手HP-15",
            BuildUp => "自己MHP+10,HP+10",
            HPSwap => "お互いのMHP,HPを入れ替える",
            GoldenHeal => "自己HP+自己現在Money×2",
            Treasure => "自己Money+5",
            GoldenSkin => "自己DEF+自己現在Money",
            Chaos => "全員HP-5,ATK+5,DEF-5",
            GoldenDagger => "自己ATK+自己現在Money",
            ATKSwap => "お互いのATKを入れ替える",
            DEFSwap => "お互いのDEFを入れ替える",
            Excalibur => "自己HP+10,ATK+10,DEF+10",
        }
        .to_string()
    }
}
