use crate::bop::scenes::field::PositionMessage;
use crate::bop::state::card_game_shared_state::CardKind::*;
use crate::bop::state::card_game_shared_state::PhaseType::{AttackTarget, Bid, GameStart, UseCard};
use crate::bop::state::character::Character;
use crate::bop::SaveData;
use crate::engine::application_types::StateType;
use crate::engine::state::State;
use serde::{Deserialize, Serialize};
use wasm_bindgen_test::console_log;
use crate::features::animation::Animation;
use crate::svg::simple_binder::SimpleBinder;

pub struct CardGameSharedState {
    pub treasure_box_opened: Vec<Vec<usize>>,
    pub save_data: SaveData,
    pub online_users: Vec<PositionMessage>,
    pub characters: Vec<Character>,

    // ここからカードゲーム用
    pub players: Vec<CardGamePlayer>,
    pub own_player_index: usize,
    pub cards_bid_on: Vec<Card>,
    // 入札確定前の入力を管理する
    pub bid_input: BidMessage,
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
}

pub struct CardGamePlayer {
    pub player_name: String,
    pub game_start_is_approved: bool,
    pub battle_is_viewed: bool,
    pub own_card_list: Vec<Card>,
    pub player_state: PlayerState,
}

pub struct GameLog {
    pub turn: usize,
    pub log_type: LogType,
}

pub enum LogType {
    Joined(usize),
    BidSuccessful(usize),
    InitiativeChanged(usize),
    UseCard(usize),
    AttackTarget(usize, u32),
    GameEnd(usize),
}
pub struct PlayerState {
    pub max_hp: u32,
    pub current_hp: u32,
    pub attack_point: u32,
    pub defence_point: u32,
    pub current_money_amount: u32,
    pub estimated_money_amount: u32,
}

impl PlayerState {
    pub fn init() -> PlayerState {
        PlayerState {
            max_hp: 50,
            current_hp: 50,
            attack_point: 5,
            defence_point: 5,
            current_money_amount: 5,
            estimated_money_amount: 3,
        }
    }
}

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
        vec![LongSword, LeatherArmour, Dagger, Balance, Cure]
            .into_iter()
            .map(|card_kind| Card::from(card_kind))
            .collect()
    }
}

#[derive(Debug)]
pub enum CardKind {
    LongSword,
    LeatherArmour,
    Dagger,
    Balance,
    Cure,
}

impl CardKind {
    pub fn get_card_name(&self) -> String {
        match self {
            LongSword => "ロングソード",
            LeatherArmour => "レザーアーマー",
            Dagger => "ダガー",
            Balance => "バランス",
            Cure => "キュア",
        }.to_string()
    }
    pub fn get_card_description(&self) -> String {
        match self {
            LongSword => "自己ATK+10",
            LeatherArmour => "自己DEF+5",
            Dagger => "自己ATK+5",
            Balance => "自己ATK,DEFを高い方に合わせる,ATK+1,DEF+1",
            Cure => "自己HP+20",
        }.to_string()
    }
}

#[derive(Deserialize, Serialize)]
pub struct BattleIsViewedMessage {
    pub battle_is_viewed: bool,
}

#[derive(Deserialize, Serialize)]
pub struct GameStartIsApprovedMessage {
    pub player_index: usize,
    pub game_start_is_approved: bool,
}

#[derive(Deserialize, Serialize)]
pub struct BidMessage {
    pub player_index: usize,
    pub bid_card_index: usize,
    pub bid_amount: u32,
}

impl BidMessage {
    pub fn init() -> BidMessage {
        BidMessage {
            player_index: 0,
            bid_card_index: 0,
            bid_amount: 1,
        }
    }
}
#[derive(Deserialize, Serialize)]
pub struct UseCardMessage {
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

#[derive(Deserialize, Serialize)]
pub struct AttackTargetMessage {
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
#[derive(Debug)]
pub struct Phase {
    pub phase_type: PhaseType,
    pub check_phase_complete_func: fn(&mut CardGameSharedState) -> CheckPhaseCompleteResult,
    pub args_usize: Vec<usize>,
}

pub struct CheckPhaseCompleteResult {
    pub is_phase_complete: bool,
    pub next_phase_index: Option<usize>,
    pub is_required_own_input_for_complete: Option<bool>,
}

impl CheckPhaseCompleteResult {
    pub fn empty() -> CheckPhaseCompleteResult {
        CheckPhaseCompleteResult {
            is_phase_complete: false,
            next_phase_index: None,
            is_required_own_input_for_complete: None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum PhaseType {
    GameStart,
    Bid,
    UseCard,
    AttackTarget,
    GameEnd,
    Empty,
}

impl Phase {
    pub fn empty() -> Phase {
        fn empty_func(_: &mut CardGameSharedState) -> CheckPhaseCompleteResult {
            CheckPhaseCompleteResult {
                is_phase_complete: false,
                next_phase_index: None,
                is_required_own_input_for_complete: None,
            }
        }
        Phase {
            phase_type: PhaseType::Empty,
            check_phase_complete_func: empty_func,
            args_usize: vec![],
        }
    }
    pub fn get_phases() -> Vec<Phase> {
        // PhaseType::GameEnd の Phase は現状必ずしもいらない…
        vec![
            Phase::get_game_start_phase(),
            Phase::get_bid_phase(),
            Phase::get_use_card_phase(),
            Phase::get_attack_target_phase(),
        ]
    }
    pub fn get_game_start_phase() -> Phase {
        // TODO
        // この関数は、3人以上のプレイヤーを意識して書かれていますが、動作確認は不十分です
        fn check_game_start_phase_complete_func(
            game_state: &mut CardGameSharedState,
        ) -> CheckPhaseCompleteResult {
            console_log!("cgspcf 1");
            let mut result = CheckPhaseCompleteResult {
                is_phase_complete: false,
                next_phase_index: None,
                is_required_own_input_for_complete: None,
            };
            console_log!("cgspcf 2");
            result.is_phase_complete = game_state
                .players
                .iter()
                .all(|player| player.game_start_is_approved);
            console_log!("cgspcf 3");

            if result.is_phase_complete {
                result.next_phase_index = Some(Bid as i32 as usize);
                console_log!("cgspcf 4");
            } else {
                console_log!("cgspcf 5");

                // 自分が approved でなければいつでも入力可能
                result.is_required_own_input_for_complete =
                    Some(!game_state.players[game_state.own_player_index].game_start_is_approved);
            }
            console_log!("cgspcf 6");
            result
        }
        Phase {
            phase_type: GameStart,
            check_phase_complete_func: check_game_start_phase_complete_func,
            args_usize: vec![],
        }
    }
    pub fn get_bid_phase() -> Phase {
        // TODO
        // この関数は、3人以上のプレイヤーを意識して書かれていますが、動作確認は不十分です
        fn check_bid_phase_complete_func(
            game_state: &mut CardGameSharedState,
        ) -> CheckPhaseCompleteResult {
            let mut result = CheckPhaseCompleteResult {
                is_phase_complete: false,
                next_phase_index: None,
                is_required_own_input_for_complete: None,
            };

            let temporary_history_len = game_state.temporary_bid_history.len();
            let player_len = game_state.players.len();
            let own_player_index = game_state.own_player_index;
            // 入札が一巡していない場合のロジック
            if temporary_history_len < player_len {
                result.is_phase_complete = false;
                // 優先順位順に入札しているので、次に入札すべきプレイヤーは temporary_history の長さで決まる（一巡しない間）
                let next_player_index =
                    game_state.initiatives_to_player_index[temporary_history_len];
                result.is_required_own_input_for_complete =
                    Some(next_player_index == own_player_index);
                return result;
            }

            // 以降は、入札が一巡している
            // 各プレイヤーについて、"最終の"入札済みカードのインデックスを集める
            let mut player_index_to_target_card_index = vec![0; player_len];
            // 各プレイヤーにについて、最後の入札のインデックスを集める（あとで使う）
            let mut player_index_to_last_bid_index = vec![0; player_len];
            for player_index in 0..player_len {
                let found = game_state
                    .temporary_bid_history
                    .iter()
                    .enumerate()
                    .filter(|(temporary_history_index, bid)| bid.player_index == player_index)
                    .last();
                if found.is_none() {
                    let input_player_index = game_state
                        .temporary_bid_history
                        .iter()
                        .map(|bid| bid.player_index)
                        .collect::<Vec<usize>>();
                    console_log!("temporary history len >= player len, but index={} player target card not found. input player index: {:?}", player_index, input_player_index);
                    panic!()
                }
                player_index_to_target_card_index[player_index] = found.unwrap().1.bid_card_index;
                player_index_to_last_bid_index[player_index] = found.unwrap().0;
            }

            // 各プレイヤーについて、競合を持つかをフラグで集める
            let mut player_index_to_has_competitor_flag = vec![false; player_len];
            for player_a_index in 1..player_len {
                for player_b_index in 0..player_a_index {
                    if player_index_to_target_card_index[player_a_index]
                        == player_index_to_target_card_index[player_b_index]
                    {
                        // 重複発見時ロジック
                        player_index_to_has_competitor_flag[player_a_index] = true;
                        player_index_to_has_competitor_flag[player_b_index] = true;
                    }
                }
            }

            // 競合がなければ（次が何のフェースでも）完了
            result.is_phase_complete = player_index_to_has_competitor_flag
                .iter()
                .all(|flag| *flag == false);
            if result.is_phase_complete {
                // 引き続き Bid フェーズを行うかの判定
                let is_continuous_bid = game_state.players[0].own_card_list.len() < 2;
                // まだカード使用フェーズが来ないなら引き続き Bid、そうでないなら UseCard
                if is_continuous_bid {
                    result.next_phase_index = Some(Bid as i32 as usize);
                } else {
                    result.next_phase_index = Some(UseCard as i32 as usize);
                }
                return result;
            }

            // 競合が見つかっている場合のロジック
            // 自分が競合していなければ、単純に false をセットして返却
            if !player_index_to_has_competitor_flag[own_player_index] {
                result.is_required_own_input_for_complete = Some(false);
                return result;
            }
            // 自分が競合している場合は、追加で入札順を考慮する
            let own_last_bid_index = player_index_to_last_bid_index[own_player_index];
            // 自分より前に入札している競合ありプレイヤーが存在していない場合は、自分の入力が必要
            let mut is_required_own_input = true;

            // 自分より前に入札している他のプレイヤー（競合を持つ）を探す
            for player_index in 0..player_len {
                // 自分自身は除外
                if player_index == own_player_index {
                    continue;
                }
                // 競合を持っていないプレイヤーは除外
                if !player_index_to_has_competitor_flag[player_index] {
                    continue;
                }
                // 自分より前に入札している競合ありプレイヤーが存在している場合
                if player_index_to_last_bid_index[player_index] < own_last_bid_index {
                    // 自分の入力は不要
                    is_required_own_input = false;
                    break;
                }
            }
            result.is_required_own_input_for_complete = Some(is_required_own_input);
            result
        }
        Phase {
            phase_type: Bid,
            check_phase_complete_func: check_bid_phase_complete_func,
            args_usize: vec![],
        }
    }

    pub fn get_use_card_phase() -> Phase {
        // TODO
        // この関数は、3人以上のプレイヤーを意識して書かれていますが、動作確認は不十分です

        fn check_use_card_complete_func(
            game_state: &mut CardGameSharedState,
        ) -> CheckPhaseCompleteResult {
            let mut result = CheckPhaseCompleteResult {
                is_phase_complete: false,
                next_phase_index: None,
                is_required_own_input_for_complete: None,
            };
            let player_len = game_state.players.len();
            let own_player_index = game_state.own_player_index;

            // このターンの使用履歴を収集
            let this_turn_card_history = game_state
                .use_card_history
                .iter()
                .filter(|history| history.turn == game_state.turn)
                .collect::<Vec<&UseCardMessage>>();

            // このターンの使用履歴が空の場合は不完了
            if this_turn_card_history.is_empty() {
                let next_input_player_index = game_state.initiatives_to_player_index[0];
                // 優先順位の先頭が自分であれば次の入力者
                result.is_required_own_input_for_complete =
                    Some(next_input_player_index == own_player_index);
                return result;
            }
            // 空でない場合は、先にカード連続使用中（他者の使用ブロック中）でないかを確認
            let last_history = this_turn_card_history.last().unwrap();
            if last_history.check_is_blocked {
                // そうである場合はそれが自分であれば次の入力者
                result.is_required_own_input_for_complete =
                    Some(last_history.player_index == own_player_index);
                return result;
            }
            // 使用フラグを収集
            let mut player_index_to_card_used_flag = vec![false; player_len];

            for player_index in 0..player_len {
                // カードを使用していなければ当然履歴も見つからない点に注意
                if let Some(last_history) = this_turn_card_history
                    .iter()
                    .find(|history| history.player_index == player_index)
                {
                    player_index_to_card_used_flag[last_history.player_index] = true;
                };
            }

            // 全員が使用完了
            if player_index_to_card_used_flag.iter().all(|flag| *flag) {
                result.is_phase_complete = true;
                result.next_phase_index = Some(AttackTarget as i32 as usize);
                return result;
            }
            // 次のカード使用者を探索
            for player_index in game_state.initiatives_to_player_index.iter() {
                // カード使用済みの場合は除外
                if player_index_to_card_used_flag[*player_index] {
                    continue;
                }
                // 優先順で最初に見つかったカード未使用プレイヤーが次のプレイヤー
                result.is_required_own_input_for_complete = Some(*player_index == own_player_index);
                break;
            }
            result
        }
        Phase {
            phase_type: UseCard,
            check_phase_complete_func: check_use_card_complete_func,
            args_usize: vec![],
        }
    }

    pub fn get_attack_target_phase() -> Phase {
        // TODO
        // この関数は、3人以上のプレイヤーを意識して書かれていますが、動作確認は不十分です

        fn check_attack_target_complete_func(
            game_state: &mut CardGameSharedState,
        ) -> CheckPhaseCompleteResult {
            let mut result = CheckPhaseCompleteResult {
                is_phase_complete: false,
                next_phase_index: None,
                is_required_own_input_for_complete: None,
            };
            let player_len = game_state.players.len();
            let own_player_index = game_state.own_player_index;

            // このターンの攻撃対象決定履歴を収集
            let this_turn_attack_target_history = game_state
                .attack_target_history
                .iter()
                .filter(|history| history.turn == game_state.turn)
                .collect::<Vec<&AttackTargetMessage>>();

            // このターンの攻撃対象決定履歴が空の場合は不完了
            if this_turn_attack_target_history.is_empty() {
                let next_input_player_index = game_state.initiatives_to_player_index[0];
                // 優先順位の先頭が自分であれば次の入力者
                result.is_required_own_input_for_complete =
                    Some(next_input_player_index == own_player_index);
                return result;
            }
            // 空でない場合は、先に連続攻撃中（他者の使用ブロック中）でないかを確認
            let last_history = this_turn_attack_target_history.last().unwrap();
            if last_history.check_is_blocked {
                // そうである場合はそれが自分であれば次の入力者
                result.is_required_own_input_for_complete =
                    Some(last_history.player_index == own_player_index);
                return result;
            }
            // 使用フラグを収集
            let mut player_index_to_chose_attack_target_flag = vec![false; player_len];

            for player_index in 0..player_len {
                // 攻撃対象を決定していなければ当然履歴も見つからない点に注意
                if let Some(last_history) = this_turn_attack_target_history
                    .iter()
                    .find(|history| history.player_index == player_index)
                {
                    player_index_to_chose_attack_target_flag[last_history.player_index] = true;
                };
            }

            // 全員が攻撃対象決定完了
            if player_index_to_chose_attack_target_flag
                .iter()
                .all(|flag| *flag)
            {
                result.is_phase_complete = true;
                result.next_phase_index = Some(Bid as i32 as usize);
                return result;
            }
            // 次の攻撃対象決定者を探索
            for player_index in game_state.initiatives_to_player_index.iter() {
                // 攻撃対象済みの場合は除外
                if player_index_to_chose_attack_target_flag[*player_index] {
                    continue;
                }
                // 優先順で最初に見つかった攻撃対象未決定プレイヤーが次のプレイヤー
                result.is_required_own_input_for_complete = Some(*player_index == own_player_index);
                break;
            }
            result
        }
        Phase {
            phase_type: AttackTarget,
            check_phase_complete_func: check_attack_target_complete_func,
            args_usize: vec![],
        }
    }
}

impl CardGameSharedState {
    pub fn check_phase_complete(&mut self) -> CheckPhaseCompleteResult {
        console_log!("cpc 1");
        let index = self.phase_index;
        let check_func = self.phases[index].check_phase_complete_func;
        console_log!("cpc 2");
        check_func(self)
    }
    pub fn phase_shift_to(&mut self, animations: &mut Vec<Vec<Animation>>, next_phase_index: usize) {
        let now_phase_index = self.phase_index;
        match self.phases[next_phase_index].phase_type {
            Bid => match now_phase_index {
                0 => {
                    while self.cards_bid_on.len() < 3 && self.bid_scheduled_cards.len() > 0 {
                        let card = self.bid_scheduled_cards.remove(0);
                        self.cards_bid_on.push(card);
                    }
                }
                1 => {
                    for player_index in 0..self.players.len() {
                        let last_bid = self.temporary_bid_history.iter().filter(|history|history.player_index == player_index).last().unwrap();
                        let card = self.cards_bid_on.remove(last_bid.bid_card_index);
                        animations.push(vec![Animation::create_message(format!("{}: {}を {} で落札", self.players[last_bid.player_index].player_name, card.card_kind.get_card_name(),last_bid.bid_amount))]);
                        self.players[player_index].own_card_list.push(card);
                    }
                    self.temporary_bid_history.clear();
                    while self.cards_bid_on.len() < 3 && self.bid_scheduled_cards.len() > 0 {
                        let card = self.bid_scheduled_cards.remove(0);
                        self.cards_bid_on.push(card);
                    }
                }
                _ => {}
            },
            _ => {}
        }
        self.phase_index = self.phases[next_phase_index].phase_type.to_owned() as i32 as usize;
    }
}
// 使わないimpl
impl CardGameSharedState {
    pub fn update_save_data(shared_state: &mut State) {
        if let StateType::BoPShared(rpg_shared_state) = &mut shared_state.state_type {
            rpg_shared_state.save_data.update(
                &mut rpg_shared_state.characters,
                &rpg_shared_state.treasure_box_opened,
                shared_state.primitives.map_index,
            );
        }
    }
    pub fn load_save_data(shared_state: &mut State) {
        if let StateType::BoPShared(rpg_shared_state) = &mut shared_state.state_type {
            rpg_shared_state
                .save_data
                .load(&mut rpg_shared_state.characters, true);
            rpg_shared_state.treasure_box_opened =
                rpg_shared_state.save_data.treasure_box_usize.to_vec();
            shared_state.primitives.map_index =
                *rpg_shared_state.save_data.map_usize.get(0).unwrap();
            shared_state.primitives.requested_map_index =
                *rpg_shared_state.save_data.map_usize.get(0).unwrap();
        }
    }
    pub fn new_game(shared_state: &mut State) {
        if let StateType::BoPShared(rpg_shared_state) = &mut shared_state.state_type {
            let mut new_save_data = SaveData::empty();
            new_save_data.load(&mut rpg_shared_state.characters, false);
            rpg_shared_state.treasure_box_opened = new_save_data.treasure_box_usize.to_vec();
            shared_state.primitives.map_index = *new_save_data.map_usize.get(0).unwrap();
            shared_state.primitives.requested_map_index = *new_save_data.map_usize.get(0).unwrap();
        }
    }
}
