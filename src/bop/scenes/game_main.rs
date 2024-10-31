use crate::bop::cpu_player::CPUPlayer;
use crate::bop::state::message::{
    AttackTargetMessage, BidMessage, GameStartIsApprovedMessage, UseCardMessage,
};
use crate::engine::application_types::SceneType::BoPGameMain;
use crate::engine::application_types::StateType::BoPShared;
use crate::engine::input::Input;
use crate::engine::scene::Scene;
use crate::engine::state::State;
use crate::features::animation::Animation;
use crate::svg::element_wrapper::ElementWrapper;
use crate::svg::svg_renderer::{Cursor, SvgRenderer};
use wasm_bindgen_test::console_log;

pub struct GameMainState {
    renderers: Vec<SvgRenderer>,
    is_bid_confirm_opened: bool,
    is_item_use_confirm_opened: bool,
    is_item_use_skip_confirm_opened: bool,
    use_item_cursors: Vec<Cursor>,
}

impl GameMainState {
    pub fn create_game_main_scene(shared_state: &mut State) -> Scene {
        let mut renderer = SvgRenderer::new("game-main-bid".to_string(), 45.0);
        renderer.cursor.update_choice_length(3);

        let mut game_main_state = GameMainState {
            renderers: vec![
                renderer,
                SvgRenderer::new("game-main-common-confirm".to_string(), 30.0),
                SvgRenderer::new("game-main-battle".to_string(), 30.0),
            ],
            is_bid_confirm_opened: false,
            is_item_use_confirm_opened: false,
            is_item_use_skip_confirm_opened: false,
            use_item_cursors: vec![
                Cursor::new(
                    &shared_state.elements.document,
                    "use-item-cursor-a",
                    1,
                    39.0,
                ),
                Cursor::new(
                    &shared_state.elements.document,
                    "use-item-cursor-b",
                    1,
                    39.0,
                ),
            ],
        };
        let consume_func = game_main_state.create_consume_func();
        let init_func = game_main_state.create_init_func();
        Scene {
            own_element: ElementWrapper::new(
                shared_state
                    .elements
                    .document
                    .get_element_by_id("game-main")
                    .unwrap(),
            ),
            is_partial_scene: false,
            consume_func,
            init_func,
            update_map_func: Scene::create_update_map_func_empty(),
            // consume_channel_message_func: game_main_state.create_consume_channel_message_func(),
            on_update_state_func: game_main_state.create_on_update_state_func(),
            scene_type: BoPGameMain(game_main_state),
        }
    }
    pub fn create_init_func(&self) -> fn(&mut Scene, &mut State) {
        fn init_func(scene: &mut Scene, state: &mut State) {
            scene.show();
            if let BoPShared(bop_shared_state) = &mut state.state_type {
                state.to_send_channel_messages.push(
                    serde_json::to_string(&GameStartIsApprovedMessage {
                        player_index: bop_shared_state.own_player_index,
                        game_start_is_approved: true,
                    })
                    .unwrap(),
                );
            }
        }
        init_func
    }
    pub fn create_consume_func(&self) -> fn(&mut Scene, &mut State, Input) {
        fn consume_func(scene: &mut Scene, shared_state: &mut State, input: Input) {
            shared_state.keep_connection_request = true;
            if let State {
                state_type: BoPShared(bop_shared_state),
                to_send_channel_messages,
                interrupt_animations,
                ..
            } = shared_state
            {
                if bop_shared_state.input_is_guard {
                    return;
                }
                if let Scene {
                    scene_type: BoPGameMain(game_main_state),
                    ..
                } = scene
                {
                    if bop_shared_state.phase_index == 1 {
                        let cursor_index = game_main_state.renderers[0].cursor.chose_index;
                        match input {
                            Input::ArrowRight => {
                                if game_main_state.is_bid_confirm_opened {
                                    return;
                                }
                                let bid_amount =
                                    bop_shared_state.bid_input[cursor_index].bid_amount;
                                let player_money = bop_shared_state.players
                                    [bop_shared_state.own_player_index]
                                    .player_status
                                    .current_money_amount;
                                bop_shared_state.bid_input[cursor_index].bid_amount =
                                    // プレイヤーの持ち金より最低入札価格が高い場合はそちらを参照しなければならない
                                    (bid_amount + 1).min(player_money.max(bid_amount));
                            }
                            Input::ArrowLeft => {
                                if game_main_state.is_bid_confirm_opened {
                                    return;
                                }
                                let bid_amount =
                                    bop_shared_state.bid_input[cursor_index].bid_amount;
                                let current_bid_amount = BidMessage::current_bid_amount(
                                    cursor_index,
                                    &bop_shared_state.temporary_bid_history,
                                );
                                // 現在価格が0なら最低入札価格は1
                                // 現在価格が1以上なら、最低入札価格は現在価格+2
                                let lowest_amount = if current_bid_amount == 0 {
                                    1
                                } else {
                                    current_bid_amount + 2
                                };
                                bop_shared_state.bid_input[cursor_index].bid_amount =
                                    (bid_amount - 1).max(lowest_amount);
                            }

                            Input::ArrowDown | Input::ArrowUp => {
                                let renderer_index = if game_main_state.is_bid_confirm_opened {
                                    1
                                } else {
                                    0
                                };
                                game_main_state.renderers[renderer_index]
                                    .cursor
                                    .consume(input);
                            }
                            Input::Enter => {
                                if game_main_state.is_bid_confirm_opened {
                                    if game_main_state.renderers[1].cursor.chose_index == 0 {
                                        to_send_channel_messages.push(
                                            serde_json::to_string(&BidMessage {
                                                seq_no: bop_shared_state.get_seq_no_to_send(),
                                                player_index: bop_shared_state.own_player_index,
                                                bid_item_index: cursor_index,
                                                bid_amount: bop_shared_state.bid_input
                                                    [cursor_index]
                                                    .bid_amount,
                                            })
                                            .unwrap(),
                                        );
                                    }
                                    game_main_state.is_bid_confirm_opened = false;
                                    game_main_state.renderers[1].hide();
                                    game_main_state.renderers[1].cursor.reset();
                                } else {
                                    let item_name = bop_shared_state.items_bid_on[cursor_index]
                                        .item_kind
                                        .get_name();
                                    let amount =
                                        bop_shared_state.bid_input[cursor_index].bid_amount;
                                    if bop_shared_state.players[bop_shared_state.own_player_index]
                                        .player_status
                                        .current_money_amount
                                        < amount
                                    {
                                        interrupt_animations.push(vec![Animation::create_message(
                                            "Moneyが足りません".to_string(),
                                            true,
                                        )]);
                                        return;
                                    }
                                    game_main_state.renderers[1].render(
                                        vec!["はい".to_string(), "いいえ".to_string()],
                                        vec![],
                                        format!(
                                            "{} を {} Moneyで入札しますか？",
                                            item_name, amount
                                        )
                                        .as_str(),
                                    );
                                    game_main_state.is_bid_confirm_opened = true;
                                }
                            }
                            _ => {}
                        }
                    } else if bop_shared_state.phase_index == 2 {
                        match input {
                            Input::ArrowUp | Input::ArrowDown => {
                                if game_main_state.is_item_use_confirm_opened
                                    || game_main_state.is_item_use_skip_confirm_opened
                                {
                                    game_main_state.renderers[1].cursor.consume(input);
                                } else {
                                    let player_index = bop_shared_state.own_player_index;
                                    game_main_state.use_item_cursors[player_index]
                                        .update_choice_length(
                                            bop_shared_state.players[player_index]
                                                .own_item_list
                                                .len(),
                                        );
                                    game_main_state.use_item_cursors[player_index].consume(input);
                                }
                            }
                            Input::Enter => {
                                if game_main_state.is_item_use_confirm_opened {
                                    if game_main_state.renderers[1].cursor.chose_index == 0 {
                                        let player_index = bop_shared_state.own_player_index;
                                        let cursor_index = game_main_state.use_item_cursors
                                            [player_index]
                                            .chose_index;
                                        game_main_state.use_item_cursors[player_index].reset();
                                        to_send_channel_messages.push(
                                            serde_json::to_string(&UseCardMessage {
                                                seq_no: bop_shared_state.get_seq_no_to_send(),
                                                turn: bop_shared_state.turn,
                                                check_is_blocked: false,
                                                player_index: bop_shared_state.own_player_index,
                                                use_item_index: cursor_index,
                                                is_skipped: false,
                                                args_i32: vec![],
                                                args_usize: vec![],
                                            })
                                            .unwrap(),
                                        );
                                    }
                                    game_main_state.is_item_use_confirm_opened = false;
                                    game_main_state.renderers[1].hide();
                                    game_main_state.renderers[1].cursor.reset();
                                } else if game_main_state.is_item_use_skip_confirm_opened {
                                    if game_main_state.renderers[1].cursor.chose_index == 0 {
                                        to_send_channel_messages.push(
                                            serde_json::to_string(&UseCardMessage {
                                                seq_no: bop_shared_state.get_seq_no_to_send(),
                                                turn: bop_shared_state.turn,
                                                check_is_blocked: false,
                                                player_index: bop_shared_state.own_player_index,
                                                use_item_index: 0,
                                                is_skipped: true,
                                                args_i32: vec![],
                                                args_usize: vec![],
                                            })
                                            .unwrap(),
                                        );
                                    }
                                    game_main_state.is_item_use_skip_confirm_opened = false;
                                    game_main_state.renderers[1].hide();
                                    game_main_state.renderers[1].cursor.reset();
                                } else {
                                    let player_index = bop_shared_state.own_player_index;
                                    let cursor_index =
                                        game_main_state.use_item_cursors[player_index].chose_index;
                                    let item = &bop_shared_state.players[player_index]
                                        .own_item_list[cursor_index];
                                    game_main_state.renderers[1].render(
                                        vec!["はい".to_string(), "いいえ".to_string()],
                                        vec![],
                                        format!("{} を使用しますか？", item.item_kind.get_name())
                                            .as_str(),
                                    );
                                    game_main_state.is_item_use_confirm_opened = true;
                                }
                            }
                            Input::Cancel => {
                                if game_main_state.is_item_use_confirm_opened
                                    || game_main_state.is_item_use_skip_confirm_opened
                                {
                                    game_main_state.is_item_use_confirm_opened = false;
                                    game_main_state.is_item_use_skip_confirm_opened = false;
                                    game_main_state.renderers[1].hide();
                                    game_main_state.renderers[1].cursor.reset();
                                } else {
                                    game_main_state.renderers[1].render(
                                        vec!["はい".to_string(), "いいえ".to_string()],
                                        vec![],
                                        "アイテム使用をスキップしますか？",
                                    );
                                    game_main_state.is_item_use_skip_confirm_opened = true;
                                }
                            }
                            _ => {}
                        }
                    } else if bop_shared_state.phase_index == 3 {
                        match input {
                            Input::ArrowUp | Input::ArrowDown => {
                                game_main_state.renderers[2].cursor.consume(input);
                            }
                            Input::Enter => {
                                let player_index = bop_shared_state.own_player_index;
                                let opponent_player_index =
                                    bop_shared_state.opponent_player_index(player_index);
                                let is_skipped =
                                    game_main_state.renderers[2].cursor.chose_index == 1;
                                let attack_target_message = AttackTargetMessage {
                                    seq_no: bop_shared_state.get_seq_no_to_send(),
                                    turn: bop_shared_state.turn,
                                    player_index,
                                    check_is_blocked: false,
                                    attack_target_player_index: opponent_player_index,
                                    is_skipped,
                                };
                                to_send_channel_messages
                                    .push(serde_json::to_string(&attack_target_message).unwrap());
                                game_main_state.renderers[2].hide();
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        consume_func
    }

    pub fn create_on_update_state_func(&mut self) -> fn(&mut Scene, &mut State) {
        fn on_update_state_func(scene: &mut Scene, shared_state: &mut State) {
            if let Scene {
                scene_type: BoPGameMain(game_main_state),
                ..
            } = scene
            {
                if let State {
                    state_type: BoPShared(bop_shared_state),
                    ..
                } = shared_state
                {
                    let check_result =
                        bop_shared_state.check_phase_complete(shared_state.is_matched);
                    game_main_state.renderers[0].cursor.reset();
                    if let Some(next_phase_index) = check_result.next_phase_index {
                        if next_phase_index == 4 {
                            bop_shared_state.phase_index = 4;
                            return;
                        }
                    }
                    console_log!(
                        "input required player {}",
                        bop_shared_state.own_player_index
                    );
                    console_log!(
                        "now phase is... {:?}",
                        bop_shared_state.phases[bop_shared_state.phase_index]
                    );
                    console_log!(
                        "next input is... {:?}",
                        check_result.is_required_own_input_for_complete
                    );
                    bop_shared_state.input_is_guard =
                        !check_result.is_required_own_input_for_complete.unwrap();
                    if bop_shared_state.input_is_guard && bop_shared_state.has_cpu {
                        let cpu_player = &mut CPUPlayer::new(bop_shared_state);
                        cpu_player.bop_shared_state.own_player_index = 1;
                        cpu_player.bop_shared_state.has_cpu = false;
                        let player_index = bop_shared_state.own_player_index;
                        let opponent_player_index =
                            bop_shared_state.opponent_player_index(player_index);
                        let index =
                            cpu_player.simulate_multiple_times(opponent_player_index, 40000);
                        console_log!("cpu index is... {}", index);
                        shared_state
                            .to_send_channel_messages
                            .push(cpu_player.create_cpu_message(index));
                    }
                    let item_names = bop_shared_state
                        .items_bid_on
                        .iter()
                        .map(|item| item.item_kind.get_name())
                        .collect();
                    let item_descriptions = bop_shared_state
                        .items_bid_on
                        .iter()
                        .map(|item| item.item_kind.get_description())
                        .collect();
                    game_main_state.renderers[0].render(item_names, item_descriptions, "");
                    match bop_shared_state.phase_index {
                        3 => {
                            let opponent_player_name = &bop_shared_state.players[bop_shared_state
                                .opponent_player_index(bop_shared_state.own_player_index)]
                            .player_name;
                            if !bop_shared_state.input_is_guard {
                                game_main_state.renderers[2].render(
                                    vec![
                                        opponent_player_name.to_owned(),
                                        "攻撃しない(Money+1)".to_string(),
                                    ],
                                    vec![],
                                    format!(
                                        "{}さん、攻撃対象を選んでください。",
                                        bop_shared_state.players[bop_shared_state.own_player_index]
                                            .player_name
                                    )
                                    .as_str(),
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        on_update_state_func
    }
}
