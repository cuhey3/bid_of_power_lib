use crate::bop::mechanism::item::ItemKind::*;
use crate::bop::state::bop_shared_state::BoPSharedState;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct Item {
    // 使用済みを表すフラグなどあったが参照箇所がないので一旦消している
    pub item_kind: ItemKind,
}

impl Item {
    pub fn from(item_kind: ItemKind) -> Item {
        Item { item_kind }
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
        move |bop_shared_state: &mut BoPSharedState| {
            let target_player_index = if target_is_own {
                use_player_index
            } else {
                bop_shared_state.opponent_player_index(use_player_index)
            };
            bop_shared_state.players[target_player_index]
                .player_status
                .capped_update_status(&status, amount);
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
                bop_shared_state.opponent_player_index(use_player_index)
            };
            let amount = (bop_shared_state.players[target_player_index]
                .player_status
                .current_money_amount as f64
                * scale) as i32;
            bop_shared_state.players[target_player_index]
                .player_status
                .capped_update_status(&status, amount);
        }
    }
    // HP/MHP は一度に処理する
    // プレイヤーの入力を取れないうちはプレイヤー1と2に固定
    pub fn create_swap_status_func(status: String) -> impl FnMut(&mut BoPSharedState) {
        move |bop_shared_state: &mut BoPSharedState| match status.as_str() {
            "HP" | "MHP" => {
                let state_a_amount = bop_shared_state.players[0].player_status.max_hp;
                let state_b_amount = bop_shared_state.players[1].player_status.max_hp;
                bop_shared_state.players[0].player_status.max_hp = state_b_amount;
                bop_shared_state.players[1].player_status.max_hp = state_a_amount;
                let state_a_amount = bop_shared_state.players[0].player_status.current_hp;
                let state_b_amount = bop_shared_state.players[1].player_status.current_hp;
                bop_shared_state.players[0].player_status.current_hp = state_b_amount;
                bop_shared_state.players[1].player_status.current_hp = state_a_amount;
            }
            _ => {
                let state_a_amount = bop_shared_state.players[0]
                    .player_status
                    .get_amount(&status);
                let state_b_amount = bop_shared_state.players[1]
                    .player_status
                    .get_amount(&status);
                bop_shared_state.players[1]
                    .player_status
                    .set_amount(&status, state_a_amount);
                bop_shared_state.players[0]
                    .player_status
                    .set_amount(&status, state_b_amount);
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
                bop_shared_state.opponent_player_index(use_player_index)
            };
            // 変化量は半分よりも多い（端数切り上げ）
            let amount = (bop_shared_state.players[target_player_index]
                .player_status
                .get_amount(&status) as f64
                * 0.5)
                .ceil() as i32
                * -1;
            bop_shared_state.players[target_player_index]
                .player_status
                .capped_update_status(&status, amount);
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
                bop_shared_state.opponent_player_index(use_player_index)
            };
            let target_player_status =
                &mut bop_shared_state.players[target_player_index].player_status;
            let status_a_amount = target_player_status.get_amount(&status_a);
            let status_b_amount = target_player_status.get_amount(&status_b);
            let new_amount = if is_balance {
                status_a_amount.max(status_b_amount)
            } else {
                status_a_amount.min(status_b_amount)
            };
            for status in [status_a.clone(), status_b.clone()].iter() {
                target_player_status
                    .set_amount(status, (new_amount as i32 + modifier).max(0) as u32);
            }
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
    pub fn get_use_func(&self, use_player_index: usize) -> Box<dyn FnMut(&mut BoPSharedState)> {
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
    pub fn get_name(&self) -> String {
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
    pub fn get_description(&self) -> String {
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
