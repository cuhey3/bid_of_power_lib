use crate::bop::state::card_game_shared_state::CardGameSharedState;
use crate::bop::state::message::{AttackTargetMessage, BidMessage, UseCardMessage};
use rand::{thread_rng, Rng};
use wasm_bindgen_test::console_log;

pub struct CPUPlayer {
    pub card_game_shared_state: CardGameSharedState,
}

impl CPUPlayer {
    pub fn create_cpu_message(&mut self, index: usize) -> String {
        let player_index = self.card_game_shared_state.own_player_index;
        let opponent_player_index = (player_index + 1) % self.card_game_shared_state.players.len();
        let turn = self.card_game_shared_state.turn;
        let seq_no = self.card_game_shared_state.consumed_seq_no + 1;
        match self.card_game_shared_state.phase_index {
            1 => serde_json::to_string(&BidMessage {
                seq_no,
                player_index,
                bid_card_index: index,
                bid_amount: self.card_game_shared_state.bid_input[index].bid_amount,
            })
            .unwrap(),
            2 => {
                let item_len = self.card_game_shared_state.players[player_index]
                    .own_card_list
                    .len();
                serde_json::to_string(&UseCardMessage {
                    seq_no,
                    turn,
                    check_is_blocked: false,
                    player_index,
                    use_card_index: index,
                    is_skipped: index == item_len,
                    args_i32: vec![],
                    args_usize: vec![],
                })
                .unwrap()
            }
            3 => serde_json::to_string(&AttackTargetMessage {
                seq_no,
                turn,
                player_index,
                check_is_blocked: false,
                attack_target_player_index: opponent_player_index,
                is_skipped: index == 1,
            })
            .unwrap(),
            _ => {
                panic!()
            }
        }
    }
    pub fn new(card_game_shared_state: &CardGameSharedState) -> CPUPlayer {
        let card_game_shared_state = &mut card_game_shared_state.clone();
        card_game_shared_state.players[0].game_start_is_approved = true;
        card_game_shared_state.players[1].game_start_is_approved = true;
        CPUPlayer {
            card_game_shared_state: card_game_shared_state.clone(),
        }
    }
    pub fn simulate_multiple_times(
        &mut self,
        simulating_player: usize,
        multiple_times: usize,
    ) -> usize {
        let mut string_keys: Vec<String> = vec![];
        let mut key_index_to_count: Vec<(usize, usize, f64)> = vec![];
        let mut key_index_to_input_player = vec![];
        let mut key_index_to_raw_inputs = vec![];
        let mut key_index_to_shortest_win: Vec<usize> = vec![];
        let mut key_index_to_longest_defeat: Vec<usize> = vec![];
        for _ in 0..multiple_times {
            let (input, is_simulating_player, result, seq_no) =
                self.simulate_once(simulating_player);
            let input_key = input
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join("-");
            let input_key_as_str = input_key.as_str();
            if let Some(index) = string_keys.iter().position(|k| k == input_key_as_str) {
                if result {
                    key_index_to_count[index].0 += 1;
                    key_index_to_shortest_win[index] = key_index_to_shortest_win[index].max(seq_no);
                } else {
                    key_index_to_longest_defeat[index] =
                        key_index_to_longest_defeat[index].min(seq_no);
                }
                key_index_to_count[index].1 += 1;
            } else {
                string_keys.push(input_key);
                key_index_to_input_player.push(is_simulating_player.clone());
                key_index_to_count.push((if result { 1 } else { 0 }, 1, 0_f64));
                key_index_to_raw_inputs.push(input.clone());
                key_index_to_shortest_win.push(0);
                key_index_to_longest_defeat.push(9999);
            };
        }

        for index in 0..string_keys.len() {
            key_index_to_count[index].2 =
                key_index_to_count[index].0 as f64 / key_index_to_count[index].1 as f64;
        }
        let len = string_keys.len();
        for index in 0..len {
            let reverse_index = len - index - 1;
            if key_index_to_count[reverse_index].1 >= 15 {
                continue;
            }
            if key_index_to_count[reverse_index].1 >= 10 {
                if key_index_to_count[reverse_index].2 == 1.0
                    || key_index_to_count[reverse_index].2 == 0.0
                {
                    continue;
                }
            }
            string_keys.remove(reverse_index);
            key_index_to_count.remove(reverse_index);
            key_index_to_input_player.remove(reverse_index);
            key_index_to_raw_inputs.remove(reverse_index);
            key_index_to_shortest_win.remove(reverse_index);
            key_index_to_longest_defeat.remove(reverse_index);
        }
        let mut to_delete_indexes: Vec<usize> = vec![];
        let mut target_depth = 5;
        let mut index = 0;
        while target_depth > 0 {
            loop {
                if key_index_to_raw_inputs[index].len() == target_depth {
                    index += 1;
                    if index >= string_keys.len() {
                        break;
                    }
                    continue;
                }
                let short_raw_inputs = key_index_to_raw_inputs[index]
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| *index < target_depth)
                    .map(|(_, value)| *value)
                    .collect::<Vec<usize>>();
                let key_prefix = short_raw_inputs
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join("-");
                let target_indexes = string_keys
                    .iter()
                    .enumerate()
                    .filter(|(_, key)| key.starts_with(key_prefix.as_str()))
                    .map(|(index, _)| {
                        key_index_to_raw_inputs[index] = short_raw_inputs.clone();
                        return index;
                    })
                    .collect::<Vec<usize>>();
                if target_indexes.len() == 1 {
                    index += 1;
                    if index >= string_keys.len() {
                        break;
                    }
                    continue;
                }
                let is_cpu_turn = key_index_to_input_player[index].last().unwrap();
                let mut to_delete_index = index;
                let mut compare_probability = key_index_to_count[index].2;
                // let mut longest_defeat = key_index_to_longest_defeat[index];
                for n in target_indexes {
                    if n == index {
                        continue;
                    }
                    if *is_cpu_turn {
                        // if longest_defeat < key_index_to_longest_defeat[n] {
                        //     longest_defeat = key_index_to_longest_defeat[n];
                        //     to_delete_indexes.push(to_delete_index.clone());
                        //     to_delete_index = n;
                        // } else
                        if compare_probability < key_index_to_count[n].2 {
                            // 値の更新があった場合
                            compare_probability = key_index_to_count[n].2;
                            to_delete_indexes.push(to_delete_index.clone());
                            to_delete_index = n;
                        } else {
                            // 値の更新がなかった場合
                            to_delete_indexes.push(n);
                        }
                    } else {
                        if compare_probability > key_index_to_count[n].2 {
                            // 値の更新があった場合
                            compare_probability = key_index_to_count[n].2;
                            to_delete_indexes.push(to_delete_index.clone());
                            to_delete_index = n;
                        } else {
                            // 値の更新がなかった場合
                            to_delete_indexes.push(n);
                        }
                    }
                }
                let to_delete_len = to_delete_indexes.len();
                for n in 0..to_delete_len {
                    let reversed_to_delete_index = to_delete_indexes[to_delete_len - n - 1];
                    string_keys.remove(reversed_to_delete_index);
                    key_index_to_raw_inputs.remove(reversed_to_delete_index);
                    key_index_to_input_player.remove(reversed_to_delete_index);
                    key_index_to_count.remove(reversed_to_delete_index);
                }
                to_delete_indexes.clear();
                if index < to_delete_len {
                    index = 0;
                } else {
                    index -= to_delete_len;
                }
                if index >= string_keys.len() {
                    break;
                }
            }
            target_depth -= 1;
            index = 0;
        }
        console_log!(
            "size: {}, {}, {}",
            string_keys.len(),
            key_index_to_count.len(),
            key_index_to_input_player.len()
        );
        let mut result_input = key_index_to_raw_inputs[0][0];
        let mut max_probability = key_index_to_count[0].2;
        for n in 0..string_keys.len() {
            if max_probability < key_index_to_count[n].2 {
                max_probability = key_index_to_count[n].2;
                result_input = key_index_to_raw_inputs[n][0];
            }
            console_log!(
                "key: {:?} {} input: {:?} raw_input: {:?} count: {:?}",
                string_keys[n],
                n,
                key_index_to_input_player[n],
                key_index_to_raw_inputs[n],
                key_index_to_count[n]
            );
        }
        result_input
    }
    pub fn simulate_once(
        &mut self,
        simulating_player: usize,
    ) -> (Vec<usize>, Vec<bool>, bool, usize) {
        let mut random_inputs = vec![];
        let mut input_players = vec![];
        let mut rng = thread_rng();
        let card_game_shared_state = &mut self.card_game_shared_state.clone();
        card_game_shared_state.consumed_seq_no = 0;
        loop {
            let check_result = card_game_shared_state.check_phase_complete(false);
            if let Some(next_phase_index) = check_result.next_phase_index {
                if next_phase_index == 4 {
                    card_game_shared_state.phase_index = 4;
                    break;
                }
            }
            let player_index = card_game_shared_state.own_player_index;
            let opponent_player_index = (player_index + 1) % card_game_shared_state.players.len();
            let turn = card_game_shared_state.turn;
            match card_game_shared_state.phase_index {
                1 => {
                    let biddable = card_game_shared_state
                        .bid_input
                        .iter()
                        .filter(|input| {
                            input.bid_amount
                                <= card_game_shared_state.players[player_index]
                                    .player_state
                                    .current_money_amount
                        })
                        .collect::<Vec<&BidMessage>>();
                    let has_bid = biddable.iter().find(|input| input.bid_amount > 1);
                    let biddable_index = rng.gen_index(0..biddable.len());
                    let mut bid_card_index = biddable[biddable_index].bid_card_index;
                    if has_bid.is_some() {
                        bid_card_index = if rng.gen_bool(0.3_f64) {
                            has_bid.unwrap().bid_card_index
                        } else {
                            bid_card_index
                        };
                    }
                    if random_inputs.len() < 6 {
                        random_inputs.push(bid_card_index);
                        input_players.push(simulating_player == player_index);
                    }
                    CPUPlayer::headless_consume_message(
                        card_game_shared_state,
                        serde_json::to_string(&BidMessage {
                            seq_no: card_game_shared_state.consumed_seq_no + 1,
                            player_index,
                            bid_card_index,
                            bid_amount: card_game_shared_state.bid_input[bid_card_index].bid_amount,
                        })
                        .unwrap(),
                    );
                }
                2 => {
                    let item_len = card_game_shared_state.players[player_index]
                        .own_card_list
                        .len();
                    let index = rng.gen_index(..item_len + 1);
                    if random_inputs.len() < 6 {
                        random_inputs.push(index);
                        input_players.push(simulating_player == player_index);
                    }
                    CPUPlayer::headless_consume_message(
                        card_game_shared_state,
                        serde_json::to_string(&UseCardMessage {
                            seq_no: card_game_shared_state.consumed_seq_no + 1,
                            turn,
                            check_is_blocked: false,
                            player_index,
                            use_card_index: index,
                            is_skipped: index == item_len,
                            args_i32: vec![],
                            args_usize: vec![],
                        })
                        .unwrap(),
                    );
                }
                3 => {
                    let index = if rng.gen_bool(0.8) == true { 0 } else { 1 };
                    if random_inputs.len() < 6 {
                        random_inputs.push(index);
                        input_players.push(simulating_player == player_index);
                    }
                    CPUPlayer::headless_consume_message(
                        card_game_shared_state,
                        serde_json::to_string(&AttackTargetMessage {
                            seq_no: card_game_shared_state.consumed_seq_no + 1,
                            turn,
                            player_index,
                            check_is_blocked: false,
                            attack_target_player_index: opponent_player_index,
                            is_skipped: index == 1,
                        })
                        .unwrap(),
                    )
                }
                _ => {}
            }
        }
        (
            random_inputs,
            input_players,
            card_game_shared_state.players[simulating_player]
                .player_state
                .current_hp
                != 0,
            card_game_shared_state.consumed_seq_no,
        )
    }
    pub fn headless_consume_message(
        card_game_shared_state: &mut CardGameSharedState,
        message: String,
    ) {
        if let Ok(message) = serde_json::from_str::<BidMessage>(&message) {
            card_game_shared_state.temporary_bid_history.push(message);
            BidMessage::ready_bid_input(
                &mut card_game_shared_state.bid_input,
                &card_game_shared_state.temporary_bid_history,
            );
        } else if let Ok(message) = serde_json::from_str::<UseCardMessage>(&message) {
            if !message.is_skipped {
                let card = card_game_shared_state.players[message.player_index]
                    .own_card_list
                    .remove(message.use_card_index);
                let mut card_use_functions = card.get_use_func(message.player_index);
                card_use_functions(card_game_shared_state);
            }
            card_game_shared_state.use_card_history.push(message);
        } else if let Ok(message) = serde_json::from_str::<AttackTargetMessage>(&message) {
            if message.is_skipped {
                card_game_shared_state.players[message.player_index]
                    .player_state
                    .current_money_amount += 1;
            } else {
                let opponent_player_index =
                    (message.player_index + 1) % card_game_shared_state.players.iter().len();
                let opponent_player_defence_point = card_game_shared_state.players
                    [opponent_player_index]
                    .player_state
                    .defence_point;
                let player_attack_point = card_game_shared_state.players[message.player_index]
                    .player_state
                    .attack_point;
                let damage = if player_attack_point == 0 {
                    0
                } else if opponent_player_defence_point >= player_attack_point {
                    1
                } else {
                    player_attack_point - opponent_player_defence_point
                };
                if damage
                    >= card_game_shared_state.players[opponent_player_index]
                        .player_state
                        .current_hp
                {
                    card_game_shared_state.players[opponent_player_index]
                        .player_state
                        .current_hp = 0;
                } else {
                    card_game_shared_state.players[opponent_player_index]
                        .player_state
                        .current_hp -= damage;
                }
            }
            card_game_shared_state.attack_target_history.push(message);
        }
    }
}