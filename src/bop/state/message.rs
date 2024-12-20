use crate::bop::mechanism::item::ItemKind;
use crate::engine::application_types::StateType::BoPShared;
use crate::engine::state::State;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct GameStartIsApprovedMessage {
    pub player_index: usize,
    pub game_start_is_approved: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BidMessage {
    pub seq_no: usize,
    pub player_index: usize,
    pub bid_item_index: usize,
    pub bid_amount: u32,
}

impl BidMessage {
    pub fn init(index: usize) -> BidMessage {
        BidMessage {
            seq_no: 0,
            player_index: 0,
            bid_item_index: index,
            bid_amount: 1,
        }
    }
    pub fn ready_bid_input(
        bid_input: &mut Vec<BidMessage>,
        temporary_bid_history: &Vec<BidMessage>,
    ) {
        for input_index in 0..bid_input.len() {
            bid_input[input_index] = BidMessage::init(input_index);
        }
        if temporary_bid_history.is_empty() {
            return;
        }
        for input_index in 0..bid_input.len() {
            if let Some(past_bid) = temporary_bid_history
                .iter()
                .filter(|history| history.bid_item_index == input_index)
                .last()
            {
                bid_input[input_index].bid_amount = past_bid.bid_amount + 2
            };
        }
    }

    pub fn current_bid_amount(item_index: usize, temporary_bid_history: &Vec<BidMessage>) -> u32 {
        if temporary_bid_history.is_empty() {
            0
        } else if let Some(last_bid) = temporary_bid_history
            .iter()
            .filter(|history| history.bid_item_index == item_index)
            .last()
        {
            last_bid.bid_amount
        } else {
            0
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UseItemMessage {
    pub seq_no: usize,
    pub turn: usize,
    // 1度のターンで複数のカードを使うことができるように用意したフラグ
    // 当然ブロックしているユーザーが次のカード使用者である
    pub check_is_blocked: bool,
    pub player_index: usize,
    pub use_item_index: usize,
    pub is_skipped: bool,
    pub args_i32: Vec<i32>,
    pub args_usize: Vec<usize>,
}

impl UseItemMessage {
    pub fn new_with_turn(turn: usize) -> UseItemMessage {
        UseItemMessage {
            seq_no: 0,
            turn,
            check_is_blocked: false,
            player_index: 0,
            use_item_index: 0,
            is_skipped: false,
            args_i32: vec![],
            args_usize: vec![],
        }
    }
    pub fn empty() -> UseItemMessage {
        UseItemMessage::new_with_turn(0)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AttackTargetMessage {
    pub seq_no: usize,
    pub turn: usize,
    pub player_index: usize,
    // 1度のターンで複数回攻撃決定ができるように用意したフラグ
    // 当然ブロックしているユーザーが次の攻撃対象決定者である
    pub check_is_blocked: bool,
    pub attack_target_player_index: usize,
    pub is_skipped: bool,
}

impl AttackTargetMessage {
    pub fn new_with_turn(turn: usize) -> AttackTargetMessage {
        AttackTargetMessage {
            seq_no: 0,
            turn,
            player_index: 0,
            check_is_blocked: false,
            attack_target_player_index: 0,
            is_skipped: false,
        }
    }
    pub fn empty() -> AttackTargetMessage {
        AttackTargetMessage::new_with_turn(0)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GameStateMessage {
    pub player_index: usize,
    pub last_consumed_seq_no: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GameRuleMessage {
    pub host_player_name: String,
    pub host_player_index: usize,
    pub guest_player_name: String,
    pub guest_player_index: usize,
    pub item_kind_list: Vec<ItemKind>,
}

impl GameRuleMessage {
    pub fn from_state(state: &mut State, guest_player_name: String) -> GameRuleMessage {
        let mut rng = thread_rng();
        if let BoPShared(bop_shared_state) = &state.state_type {
            let item_kind_list = bop_shared_state
                .bid_scheduled_items
                .iter()
                .map(|item| item.item_kind.clone())
                .collect::<Vec<ItemKind>>();

            let host_is_first = rng.gen_bool(0.5);
            GameRuleMessage {
                host_player_name: state.user_name.to_string(),
                host_player_index: if host_is_first { 0 } else { 1 },
                guest_player_name,
                guest_player_index: if host_is_first { 1 } else { 0 },
                item_kind_list,
            }
        } else {
            panic!()
        }
    }
}
