use serde::{Deserialize, Serialize};
use wasm_bindgen_test::console_log;
use crate::bop::mechanism::card::Card;
use crate::bop::state::card_game_shared_state::{CardGamePlayer, GameLog};
use crate::bop::state::phase::{CheckPhaseCompleteResult, Phase};
use crate::bop::state::phase::PhaseType::{AttackTarget, Bid, UseCard};
use crate::features::animation::Animation;
use crate::svg::simple_binder::SimpleBinder;

#[derive(Clone)]
pub struct CardGameSharedState {
    pub players: Vec<CardGamePlayer>,
    pub own_player_index: usize,
    pub cards_bid_on: Vec<Card>,
    // 入札確定前の入力を管理する
    pub bid_input: Vec<BidMessage>,
    pub bid_scheduled_cards: Vec<Card>,
    pub temporary_bid_history: Vec<BidMessage>,
    pub bid_history: Vec<BidMessage>,
    // カード使用確定前の入力を管理する
    pub use_card_input: UseCardMessage,
    pub use_card_history: Vec<UseCardMessage>,
    pub attack_target_input: AttackTargetMessage,
    pub attack_target_history: Vec<AttackTargetMessage>,
    // このVectorだけ少し特殊で、プレイヤーのインデックス自体が追加される
    // 前に出現したプレイヤーほど優先される
    pub initiatives_to_player_index: Vec<usize>,
    pub game_logs: Vec<GameLog>,
    pub turn: usize,
    // enum PhaseType と同じ順序でセットされていないとエラーになる
    // TODO
    // 初期化時にチェックを追加
    pub phase_index: usize,
    pub phases: Vec<Phase>,
    pub simple_binders: Vec<SimpleBinder>,
    pub input_is_guard: bool,
    pub consumed_seq_no: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GameStartIsApprovedMessage {
    pub player_index: usize,
    pub game_start_is_approved: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BidMessage {
    pub seq_no: usize,
    pub player_index: usize,
    pub bid_card_index: usize,
    pub bid_amount: u32,
}

impl BidMessage {
    pub fn init() -> BidMessage {
        BidMessage {
            seq_no: 0,
            player_index: 0,
            bid_card_index: 0,
            bid_amount: 1,
        }
    }
    pub fn ready_bid_input(
        bid_input: &mut Vec<BidMessage>,
        temporary_bid_history: &Vec<BidMessage>,
    ) {
        for input_index in 0..bid_input.len() {
            bid_input[input_index] = BidMessage::init();
            bid_input[input_index].bid_card_index = input_index;
            bid_input[input_index].bid_amount = 1;
        }
        if temporary_bid_history.is_empty() {
            return;
        }
        for input_index in 0..bid_input.len() {
            if let Some(past_bid) = temporary_bid_history
                .iter()
                .filter(|history| history.bid_card_index == input_index)
                .last()
            {
                bid_input[input_index].bid_amount = past_bid.bid_amount + 2
            };
        }
    }

    pub fn current_bid_amount(card_index: usize, temporary_bid_history: &Vec<BidMessage>) -> u32 {
        if temporary_bid_history.is_empty() {
            0
        } else if let Some(last_bid) = temporary_bid_history
            .iter()
            .filter(|history| history.bid_card_index == card_index)
            .last()
        {
            last_bid.bid_amount
        } else {
            0
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UseCardMessage {
    pub seq_no: usize,
    pub turn: usize,
    // 1度のターンで複数のカードを使うことができるように用意したフラグ
    // 当然ブロックしているユーザーが次のカード使用者である
    pub check_is_blocked: bool,
    pub player_index: usize,
    pub use_card_index: usize,
    pub is_skipped: bool,
    pub args_i32: Vec<i32>,
    pub args_usize: Vec<usize>,
}

impl UseCardMessage {
    pub fn new_with_turn(turn: usize) -> UseCardMessage {
        UseCardMessage {
            seq_no: 0,
            turn,
            check_is_blocked: false,
            player_index: 0,
            use_card_index: 0,
            is_skipped: false,
            args_i32: vec![],
            args_usize: vec![],
        }
    }
    pub fn empty() -> UseCardMessage {
        UseCardMessage::new_with_turn(0)
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

impl CardGameSharedState {
    pub fn get_seq_no_to_send(&self) -> usize {
        self.consumed_seq_no + 1
    }

    fn is_valid_new_message(&self, message_seq_no: usize) -> bool {
        message_seq_no == self.consumed_seq_no + 1
    }
    fn update_consumed_seq_no(&mut self, message_seq_no: usize) {
        self.consumed_seq_no = message_seq_no;
    }

    pub fn check_and_update_seq_no(&mut self, seq_no: usize, own_message_flag: bool) -> bool {
        if !self.is_valid_new_message(seq_no) {
            if own_message_flag {
                // 自分の再送信メッセージなので無視
                return false;
            }
            console_log!(
                "message seq no does not match {} and {}",
                seq_no,
                self.consumed_seq_no
            );
            panic!()
        }
        self.update_consumed_seq_no(seq_no);
        true
    }
    pub fn check_phase_complete(
        &mut self,
        interrupt_animations: &mut Vec<Vec<Animation>>,
    ) -> CheckPhaseCompleteResult {
        let index = self.phase_index;
        let check_func = self.phases[index].check_phase_complete_func;
        check_func(self, interrupt_animations)
    }
    pub fn phase_shift_to(&mut self, _: &mut Vec<Vec<Animation>>, next_phase_index: usize) {
        let now_phase_index = self.phase_index;
        if next_phase_index == 4 {
            console_log!("battle end");
            return;
        }
        match self.phases[next_phase_index].phase_type {
            Bid => match now_phase_index {
                0 => {
                    while self.cards_bid_on.len() < 3 && self.bid_scheduled_cards.len() > 0 {
                        let card = self.bid_scheduled_cards.remove(0);
                        self.cards_bid_on.push(card);
                    }
                }
                1 => {
                    // 入札中リストの後ろから対象の履歴を探す
                    // 途中で cards_bid_on に対して remove するのでインデックスがズレないように
                    let bid_on_len = self.cards_bid_on.len();
                    for bid_on_index_reverse in 0..bid_on_len {
                        if let Some((history_index, history)) = self
                            .temporary_bid_history
                            .iter_mut()
                            .enumerate()
                            .filter(|(_, history)| {
                                bid_on_len - bid_on_index_reverse - 1 == history.bid_card_index
                            })
                            .last()
                        {
                            let player_index = history.player_index;
                            let history = self.temporary_bid_history.remove(history_index);
                            self.players[player_index].player_state.current_money_amount =
                                self.players[player_index].player_state.current_money_amount
                                    - history.bid_amount
                                    + self.players[player_index]
                                        .player_state
                                        .estimated_money_amount;
                            self.bid_history.push(history);
                            let item = self
                                .cards_bid_on
                                .remove(bid_on_len - bid_on_index_reverse - 1);
                            self.players[player_index].own_card_list.push(item);
                        } else {
                            // cards_bid_on の中には落札されていないアイテムも当然存在する
                        };
                    }
                    self.temporary_bid_history.clear();
                    while self.cards_bid_on.len() < 3 && self.bid_scheduled_cards.len() > 0 {
                        let card = self.bid_scheduled_cards.remove(0);
                        self.cards_bid_on.push(card);
                    }
                    BidMessage::ready_bid_input(&mut self.bid_input, &self.temporary_bid_history);
                    self.turn += 1;
                }
                3 => {
                    // TODO
                    // 根本的な解決になっていないが空になっていないことがあるので…
                    self.temporary_bid_history.clear();
                    self.turn += 1;
                }
                _ => {}
            },
            UseCard => match now_phase_index {
                1 => {
                    // 入札中リストの後ろから対象の履歴を探す
                    // 途中で cards_bid_on に対して remove するのでインデックスがズレないように
                    let bid_on_len = self.cards_bid_on.len();
                    for bid_on_index_reverse in 0..bid_on_len {
                        if let Some((history_index, history)) = self
                            .temporary_bid_history
                            .iter_mut()
                            .enumerate()
                            .filter(|(_, history)| {
                                bid_on_len - bid_on_index_reverse - 1 == history.bid_card_index
                            })
                            .last()
                        {
                            let player_index = history.player_index;
                            let history = self.temporary_bid_history.remove(history_index);
                            self.players[player_index].player_state.current_money_amount =
                                self.players[player_index].player_state.current_money_amount
                                    - history.bid_amount
                                    + self.players[player_index]
                                        .player_state
                                        .estimated_money_amount;
                            self.bid_history.push(history);
                            let item = self
                                .cards_bid_on
                                .remove(bid_on_len - bid_on_index_reverse - 1);
                            self.players[player_index].own_card_list.push(item);
                        } else {
                            // cards_bid_on の中には落札されていないアイテムも当然存在する
                        };
                    }
                    self.temporary_bid_history.clear();
                    while self.cards_bid_on.len() < 3 && self.bid_scheduled_cards.len() > 0 {
                        let card = self.bid_scheduled_cards.remove(0);
                        self.cards_bid_on.push(card);
                    }
                    BidMessage::ready_bid_input(&mut self.bid_input, &self.temporary_bid_history);
                }
                3 => {
                    self.turn += 1;
                }
                _ => {}
            },
            AttackTarget => match now_phase_index {
                3 => {
                    self.turn += 1;
                }
                _ => {}
            },
            _ => {}
        }
        self.phase_index = self.phases[next_phase_index].phase_type.to_owned() as i32 as usize;
    }
}