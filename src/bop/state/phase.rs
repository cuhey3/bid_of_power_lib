use crate::bop::state::bop_shared_state::BoPSharedState;
use crate::bop::state::message::{AttackTargetMessage, UseCardMessage};
use crate::bop::state::phase::PhaseType::{AttackTarget, Bid, Empty, GameEnd, GameStart, UseCard};
use wasm_bindgen_test::console_log;

#[derive(Debug, Clone)]
pub struct Phase {
    pub phase_type: PhaseType,
    pub check_phase_complete_func: fn(&mut BoPSharedState) -> CheckPhaseCompleteResult,
    pub args_usize: Vec<usize>,
}

impl Phase {
    pub fn empty() -> Phase {
        fn empty_func(_: &mut BoPSharedState) -> CheckPhaseCompleteResult {
            CheckPhaseCompleteResult {
                is_phase_complete: false,
                next_phase_index: None,
                is_required_own_input_for_complete: None,
            }
        }
        Phase {
            phase_type: Empty,
            check_phase_complete_func: empty_func,
            args_usize: vec![],
        }
    }
    pub fn get_phases() -> Vec<Phase> {
        // PhaseType::GameEnd の Phase は現状必ずしもいらない…
        vec![
            Phase::get_game_start_phase(),
            Phase::get_bid_phase(),
            Phase::get_use_item_phase(),
            Phase::get_attack_target_phase(),
        ]
    }
    pub fn get_game_start_phase() -> Phase {
        // TODO
        // この関数は、3人以上のプレイヤーを意識して書かれていますが、動作確認は不十分です
        fn check_game_start_phase_complete_func(
            game_state: &mut BoPSharedState,
        ) -> CheckPhaseCompleteResult {
            let mut result = CheckPhaseCompleteResult::empty();
            // 全員受信が難しいので誰か受信でOK
            result.is_phase_complete = game_state
                .players
                .iter()
                .find(|player| player.game_start_is_approved)
                .is_some();

            if result.is_phase_complete {
                result.next_phase_index = Some(Bid as i32 as usize);
            } else {
                // 自分が approved でなければいつでも入力可能
                result.is_required_own_input_for_complete =
                    Some(!game_state.players[game_state.own_player_index].game_start_is_approved);
            }
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
            game_state: &mut BoPSharedState,
        ) -> CheckPhaseCompleteResult {
            let mut result = CheckPhaseCompleteResult::empty();
            let temporary_history_len = game_state.temporary_bid_history.len();
            let player_len = game_state.players.len();
            let own_player_index = game_state.own_player_index;
            // 入札が一巡していない場合のロジック
            if temporary_history_len < player_len {
                // 優先順位順に入札しているので、次に入札すべきプレイヤーは temporary_history の長さで決まる（一巡しない間）
                let next_player_index =
                    game_state.initiatives_to_player_index[temporary_history_len];
                result.is_required_own_input_for_complete =
                    Some(next_player_index == own_player_index);
                return result;
            }
            // 以降は、入札が一巡している
            // 各プレイヤーについて、"最終の"入札済みカードのインデックスを集める
            let mut player_index_to_target_item_index = vec![0; player_len];
            // 各プレイヤーにについて、最後の入札のインデックスを集める（あとで使う）
            let mut player_index_to_last_bid_index = vec![0; player_len];
            for player_index in 0..player_len {
                let found = game_state
                    .temporary_bid_history
                    .iter()
                    .enumerate()
                    .filter(|(_, bid)| bid.player_index == player_index)
                    .last();
                if found.is_none() {
                    let input_player_index = game_state
                        .temporary_bid_history
                        .iter()
                        .map(|bid| bid.player_index)
                        .collect::<Vec<usize>>();
                    console_log!("temporary history len >= player len, but index={} player target item not found. input player index: {:?} {:?}", player_index, input_player_index, game_state.temporary_bid_history);
                    panic!()
                }
                player_index_to_target_item_index[player_index] = found.unwrap().1.bid_item_index;
                player_index_to_last_bid_index[player_index] = found.unwrap().0;
            }
            // 各プレイヤーについて、競合を持つかをフラグで集める
            let mut player_index_to_has_competitor_flag = vec![false; player_len];
            for player_a_index in 1..player_len {
                for player_b_index in 0..player_a_index {
                    if player_index_to_target_item_index[player_a_index]
                        == player_index_to_target_item_index[player_b_index]
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
                // イニシアチブの入れ替え
                // TODO
                // もっといい位置に移動する
                // もっとコードをシンプルにする
                let first_player_last_bid_index =
                    player_index_to_last_bid_index[game_state.initiatives_to_player_index[0]];
                let first_player_bid_amount =
                    game_state.temporary_bid_history[first_player_last_bid_index].bid_amount;
                let second_player_last_bid_index =
                    player_index_to_last_bid_index[game_state.initiatives_to_player_index[1]];
                let second_player_bid_amount =
                    game_state.temporary_bid_history[second_player_last_bid_index].bid_amount;
                if second_player_bid_amount >= first_player_bid_amount {
                    let new_first_player = game_state.initiatives_to_player_index[1];
                    let new_second_player = game_state.initiatives_to_player_index[0];
                    game_state.initiatives_to_player_index[0] = new_first_player;
                    game_state.initiatives_to_player_index[1] = new_second_player;
                    // interrupt_animations.push(vec![Animation::create_message(format!(
                    //     "落札金額により行動順が変更されました。あなたは {} です",
                    //     if new_first_player == game_state.own_player_index {
                    //         "先攻"
                    //     } else {
                    //         "後攻"
                    //     }
                    // ))])
                }

                // 引き続き Bid フェーズを行うかの判定
                let is_continuous_bid = game_state.players[0].own_item_list.len() < 2;
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

    pub fn get_use_item_phase() -> Phase {
        // TODO
        // この関数は、3人以上のプレイヤーを意識して書かれていますが、動作確認は不十分です

        fn check_use_item_complete_func(
            game_state: &mut BoPSharedState,
        ) -> CheckPhaseCompleteResult {
            let mut result = CheckPhaseCompleteResult::empty();
            if let Some(_) = game_state
                .players
                .iter()
                .find(|player| player.player_state.current_hp == 0)
            {
                result.is_phase_complete = true;
                result.next_phase_index = Some(GameEnd as i32 as usize);
                return result;
            };
            let player_len = game_state.players.len();
            let own_player_index = game_state.own_player_index;

            // このターンの使用履歴を収集
            let this_turn_item_history = game_state
                .use_item_history
                .iter()
                .filter(|history| history.turn == game_state.turn)
                .collect::<Vec<&UseCardMessage>>();

            // このターンの使用履歴が空の場合は不完了
            if this_turn_item_history.is_empty() {
                let next_input_player_index = game_state.initiatives_to_player_index[0];
                // 優先順位の先頭が自分であれば次の入力者
                result.is_required_own_input_for_complete =
                    Some(next_input_player_index == own_player_index);
                return result;
            }
            // 空でない場合は、先にカード連続使用中（他者の使用ブロック中）でないかを確認
            let last_history = this_turn_item_history.last().unwrap();
            if last_history.check_is_blocked {
                // そうである場合はそれが自分であれば次の入力者
                result.is_required_own_input_for_complete =
                    Some(last_history.player_index == own_player_index);
                return result;
            }
            // 使用フラグを収集
            let mut player_index_to_item_used_flag = vec![false; player_len];

            for player_index in 0..player_len {
                // カードを使用していなければ当然履歴も見つからない点に注意
                if let Some(last_history) = this_turn_item_history
                    .iter()
                    .find(|history| history.player_index == player_index)
                {
                    player_index_to_item_used_flag[last_history.player_index] = true;
                }
                if game_state.players[player_index].own_item_list.is_empty() {
                    player_index_to_item_used_flag[player_index] = true;
                }
            }
            // 全員が使用完了
            if player_index_to_item_used_flag.iter().all(|flag| *flag) {
                result.is_phase_complete = true;
                result.next_phase_index = Some(AttackTarget as i32 as usize);
                return result;
            }
            // 次のカード使用者を探索
            for player_index in game_state.initiatives_to_player_index.iter() {
                // カード使用済みの場合は除外
                if player_index_to_item_used_flag[*player_index] {
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
            check_phase_complete_func: check_use_item_complete_func,
            args_usize: vec![],
        }
    }

    pub fn get_attack_target_phase() -> Phase {
        // TODO
        // この関数は、3人以上のプレイヤーを意識して書かれていますが、動作確認は不十分です
        fn check_attack_target_complete_func(
            game_state: &mut BoPSharedState,
        ) -> CheckPhaseCompleteResult {
            let mut result = CheckPhaseCompleteResult::empty();
            let player_len = game_state.players.len();
            let own_player_index = game_state.own_player_index;
            if let Some(_) = game_state
                .players
                .iter()
                .find(|player| player.player_state.current_hp == 0)
            {
                result.is_phase_complete = true;
                result.next_phase_index = Some(GameEnd as i32 as usize);
                return result;
            };
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
                result.is_required_own_input_for_complete =
                    Some(game_state.initiatives_to_player_index[0] == own_player_index);
                // TODO
                // 2以上ないと入札ができない
                // アイテム数が偶数なら1になることはないのだが、なぜか1になるケースがある
                // 根本的解決が必要
                if game_state.items_bid_on.len() < 2 {
                    if game_state.players[0].own_item_list.is_empty()
                        && game_state.players[1].own_item_list.is_empty()
                    {
                        result.next_phase_index = Some(AttackTarget as i32 as usize);
                    } else {
                        result.next_phase_index = Some(UseCard as i32 as usize);
                    }
                } else {
                    result.next_phase_index = Some(Bid as i32 as usize);
                }
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

    pub fn get_game_end_phase() -> Phase {
        fn check_game_end_complete_func(_: &mut BoPSharedState) -> CheckPhaseCompleteResult {
            CheckPhaseCompleteResult {
                is_phase_complete: true,
                next_phase_index: None,
                is_required_own_input_for_complete: None,
            }
        }
        Phase {
            phase_type: GameEnd,
            check_phase_complete_func: check_game_end_complete_func,
            args_usize: vec![],
        }
    }
}

#[derive(Debug)]
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
            is_required_own_input_for_complete: Some(false),
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
