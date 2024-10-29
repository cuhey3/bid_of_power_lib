use crate::bop::mechanism::card::Card;
use crate::bop::mechanism::player_state::PlayerState;
use crate::bop::state::message::{AttackTargetMessage, BidMessage, UseCardMessage};
use crate::bop::state::phase::PhaseType::*;
use crate::bop::state::phase::{CheckPhaseCompleteResult, Phase};
use crate::svg::simple_binder::SimpleBinder;
use wasm_bindgen_test::console_log;

#[derive(Clone, Debug)]
pub struct CardGamePlayer {
    pub player_name: String,
    pub game_start_is_approved: bool,
    pub battle_is_viewed: bool,
    pub own_card_list: Vec<Card>,
    pub player_state: PlayerState,
}

#[derive(Clone)]
pub struct GameLog {
    pub turn: usize,
    pub log_type: LogType,
}

#[derive(Clone)]
pub enum LogType {
    Joined(usize),
    BidSuccessful(usize),
    InitiativeChanged(usize),
    UseCard(usize),
    AttackTarget(usize, u32),
    GameEnd(usize),
}

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
    pub has_cpu: bool,
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
    pub fn check_phase_complete(&mut self, is_matched: bool) -> CheckPhaseCompleteResult {
        let mut check_result = CheckPhaseCompleteResult::empty();
        'outer: for player_index in 0..self.players.len() {
            if self.has_cpu && self.own_player_index != player_index {
                continue;
            }
            if is_matched && self.own_player_index != player_index {
                continue;
            }
            self.own_player_index = player_index;
            'inner: loop {
                let index = self.phase_index;
                let check_func = self.phases[index].check_phase_complete_func;
                check_result = check_func(self);
                if check_result.is_phase_complete {
                    if check_result.next_phase_index.unwrap() == 4 {
                        break 'outer;
                    }
                    self.phase_shift_to(check_result.next_phase_index.unwrap());
                } else {
                    break 'inner;
                }
            }
            if check_result.is_required_own_input_for_complete.unwrap() {
                break 'outer;
            }
        }
        check_result
    }
    pub fn phase_shift_to(&mut self, next_phase_index: usize) {
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
                        // console_log!("bid on update {} {} {}", self.cards_bid_on.len(), self.bid_scheduled_cards.len(), self.turn)
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
                        // console_log!("bid on update {} {} {}", self.cards_bid_on.len(), self.bid_scheduled_cards.len(), self.turn)
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
