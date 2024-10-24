use crate::bop::state::card_game_shared_state::{
    AttackTargetMessage, BidMessage, CheckPhaseCompleteResult, GameStartIsApprovedMessage,
    GameStateMessage, UseCardMessage,
};
use crate::engine::application_types::SceneType::BoPGameMain;
use crate::engine::application_types::StateType::BoPShared;
use crate::engine::input::Input;
use crate::engine::scene::Scene;
use crate::engine::state::State;
use crate::features::animation::Animation;
use crate::features::websocket::{ChannelMessage, MessageType};
use crate::svg::element_wrapper::ElementWrapper;
use crate::svg::svg_renderer::{Cursor, SvgRenderer};
use wasm_bindgen_test::console_log;

pub struct GameMainState {
    renderers: Vec<SvgRenderer>,
    is_bid_confirm_opened: bool,
    is_card_use_confirm_opened: bool,
    is_card_use_skip_confirm_opened: bool,
    use_item_cursors: Vec<Cursor>,
}

impl GameMainState {
    pub fn create_game_main_scene(shared_state: &mut State) -> Scene {
        let mut renderer = SvgRenderer::new("game-main-bid".to_string(), 45.0);
        renderer.cursor.update_choice_length(3);
        console_log!(
            "cursor info {:?} {:?} {:?}",
            renderer.cursor.cursor_type,
            renderer.cursor.choice_length,
            renderer.cursor.step_length
        );

        let mut game_main_state = GameMainState {
            renderers: vec![
                renderer,
                SvgRenderer::new("game-main-common-confirm".to_string(), 30.0),
                SvgRenderer::new("game-main-battle".to_string(), 30.0),
            ],
            is_bid_confirm_opened: false,
            is_card_use_confirm_opened: false,
            is_card_use_skip_confirm_opened: false,
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
            consume_channel_message_func: game_main_state.create_consume_channel_message_func(),
            scene_type: BoPGameMain(game_main_state),
        }
    }
    pub fn create_init_func(&self) -> fn(&mut Scene, &mut State) {
        fn init_func(scene: &mut Scene, state: &mut State) {
            console_log!("init game main scene");
            scene.show();
            match &mut scene.scene_type {
                BoPGameMain(..) => {}
                _ => panic!(),
            }
            if let BoPShared(card_game_shared_state) = &mut state.state_type {
                state.to_send_channel_messages.push(
                    serde_json::to_string(&GameStartIsApprovedMessage {
                        player_index: card_game_shared_state.own_player_index,
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
                state_type: BoPShared(card_game_shared_state),
                to_send_channel_messages,
                interrupt_animations,
                ..
            } = shared_state
            {
                if card_game_shared_state.input_is_guard {
                    return;
                }
                if let Scene {
                    scene_type: BoPGameMain(game_main_state),
                    ..
                } = scene
                {
                    if card_game_shared_state.phase_index == 1 {
                        let cursor_index = game_main_state.renderers[0].cursor.chose_index;
                        match input {
                            Input::ArrowRight => {
                                if game_main_state.is_bid_confirm_opened {
                                    return;
                                }
                                let bid_amount =
                                    card_game_shared_state.bid_input[cursor_index].bid_amount;
                                let player_money = card_game_shared_state.players
                                    [card_game_shared_state.own_player_index]
                                    .player_state
                                    .current_money_amount;
                                card_game_shared_state.bid_input[cursor_index].bid_amount =
                                    // プレイヤーの持ち金より最低入札価格が高い場合はそちらを参照しなければならない
                                    (bid_amount + 1).min(player_money.max(bid_amount));
                            }
                            Input::ArrowLeft => {
                                if game_main_state.is_bid_confirm_opened {
                                    return;
                                }
                                let bid_amount =
                                    card_game_shared_state.bid_input[cursor_index].bid_amount;
                                let current_bid_amount = BidMessage::current_bid_amount(
                                    cursor_index,
                                    &card_game_shared_state.temporary_bid_history,
                                );
                                // 現在価格が0なら最低入札価格は1
                                // 現在価格が1以上なら、最低入札価格は現在価格+2
                                let lowest_amount = if current_bid_amount == 0 {
                                    1
                                } else {
                                    current_bid_amount + 2
                                };
                                card_game_shared_state.bid_input[cursor_index].bid_amount =
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
                                                seq_no: card_game_shared_state.get_seq_no_to_send(),
                                                player_index: card_game_shared_state
                                                    .own_player_index,
                                                bid_card_index: cursor_index,
                                                bid_amount: card_game_shared_state.bid_input
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
                                    let item_name = card_game_shared_state.cards_bid_on
                                        [cursor_index]
                                        .card_kind
                                        .get_card_name();
                                    let amount =
                                        card_game_shared_state.bid_input[cursor_index].bid_amount;
                                    if card_game_shared_state.players
                                        [card_game_shared_state.own_player_index]
                                        .player_state
                                        .current_money_amount
                                        < amount
                                    {
                                        interrupt_animations.push(vec![Animation::create_message(
                                            "Moneyが足りません".to_string(),
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
                    } else if card_game_shared_state.phase_index == 2 {
                        match input {
                            Input::ArrowUp | Input::ArrowDown => {
                                if game_main_state.is_card_use_confirm_opened
                                    || game_main_state.is_card_use_skip_confirm_opened
                                {
                                    game_main_state.renderers[1].cursor.consume(input);
                                } else {
                                    let player_index = card_game_shared_state.own_player_index;
                                    game_main_state.use_item_cursors[player_index]
                                        .update_choice_length(
                                            card_game_shared_state.players[player_index]
                                                .own_card_list
                                                .len(),
                                        );
                                    game_main_state.use_item_cursors[player_index].consume(input);
                                }
                            }
                            Input::Enter => {
                                if game_main_state.is_card_use_confirm_opened {
                                    if game_main_state.renderers[1].cursor.chose_index == 0 {
                                        let player_index = card_game_shared_state.own_player_index;
                                        let cursor_index = game_main_state.use_item_cursors
                                            [player_index]
                                            .chose_index;
                                        game_main_state.use_item_cursors[player_index].reset();
                                        to_send_channel_messages.push(
                                            serde_json::to_string(&UseCardMessage {
                                                seq_no: card_game_shared_state.get_seq_no_to_send(),
                                                turn: card_game_shared_state.turn,
                                                check_is_blocked: false,
                                                player_index: card_game_shared_state
                                                    .own_player_index,
                                                use_card_index: cursor_index,
                                                is_skipped: false,
                                                args_i32: vec![],
                                                args_usize: vec![],
                                            })
                                            .unwrap(),
                                        );
                                    }
                                    game_main_state.is_card_use_confirm_opened = false;
                                    game_main_state.renderers[1].hide();
                                    game_main_state.renderers[1].cursor.reset();
                                } else if game_main_state.is_card_use_skip_confirm_opened {
                                    if game_main_state.renderers[1].cursor.chose_index == 0 {
                                        to_send_channel_messages.push(
                                            serde_json::to_string(&UseCardMessage {
                                                seq_no: card_game_shared_state.get_seq_no_to_send(),
                                                turn: card_game_shared_state.turn,
                                                check_is_blocked: false,
                                                player_index: card_game_shared_state
                                                    .own_player_index,
                                                use_card_index: 0,
                                                is_skipped: true,
                                                args_i32: vec![],
                                                args_usize: vec![],
                                            })
                                            .unwrap(),
                                        );
                                    }
                                    game_main_state.is_card_use_skip_confirm_opened = false;
                                    game_main_state.renderers[1].hide();
                                    game_main_state.renderers[1].cursor.reset();
                                } else {
                                    let player_index = card_game_shared_state.own_player_index;
                                    let cursor_index =
                                        game_main_state.use_item_cursors[player_index].chose_index;
                                    let item = &card_game_shared_state.players[player_index]
                                        .own_card_list[cursor_index];
                                    game_main_state.renderers[1].render(
                                        vec!["はい".to_string(), "いいえ".to_string()],
                                        vec![],
                                        format!(
                                            "{} を使用しますか？",
                                            item.card_kind.get_card_name()
                                        )
                                        .as_str(),
                                    );
                                    game_main_state.is_card_use_confirm_opened = true;
                                }
                            }
                            Input::Cancel => {
                                if game_main_state.is_card_use_confirm_opened
                                    || game_main_state.is_card_use_skip_confirm_opened
                                {
                                    game_main_state.is_card_use_confirm_opened = false;
                                    game_main_state.is_card_use_skip_confirm_opened = false;
                                    game_main_state.renderers[1].hide();
                                    game_main_state.renderers[1].cursor.reset();
                                } else {
                                    game_main_state.renderers[1].render(
                                        vec!["はい".to_string(), "いいえ".to_string()],
                                        vec![],
                                        "アイテム使用をスキップしますか？",
                                    );
                                    game_main_state.is_card_use_skip_confirm_opened = true;
                                }
                            }
                            _ => {}
                        }
                    } else if card_game_shared_state.phase_index == 3 {
                        match input {
                            Input::ArrowUp | Input::ArrowDown => {
                                game_main_state.renderers[2].cursor.consume(input);
                            }
                            Input::Enter => {
                                let player_index = card_game_shared_state.own_player_index;
                                let opponent_player_index =
                                    (player_index + 1) % card_game_shared_state.players.len();
                                let is_skipped =
                                    game_main_state.renderers[2].cursor.chose_index == 1;
                                let attack_target_message = AttackTargetMessage {
                                    seq_no: card_game_shared_state.get_seq_no_to_send(),
                                    turn: card_game_shared_state.turn,
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
    pub fn create_consume_channel_message_func(
        &mut self,
    ) -> fn(&mut Scene, &mut State, message: &ChannelMessage) {
        fn consume_channel_message(
            scene: &mut Scene,
            shared_state: &mut State,
            message: &ChannelMessage,
        ) {
            if let Scene {
                scene_type: BoPGameMain(game_main_state),
                ..
            } = scene
            {
                console_log!("consume_channel_message start {}", message.message);
                if let State {
                    state_type: BoPShared(card_game_shared_state),
                    interrupt_animations,
                    ..
                } = shared_state
                {
                    match message.message_type {
                        MessageType::Join => {
                            console_log!("enter join message logic {:?}", message);
                            if message.user_name == shared_state.user_name
                                && card_game_shared_state.consumed_seq_no != 0
                            {
                                // 自分が復帰したことを相手に知らせる
                                shared_state.to_send_channel_messages.push(
                                    serde_json::to_string(&GameStateMessage {
                                        player_index: card_game_shared_state.own_player_index,
                                        last_consumed_seq_no: card_game_shared_state
                                            .consumed_seq_no,
                                    })
                                    .unwrap(),
                                )
                            }
                            console_log!("complete join message logic {:?}", message);
                        }
                        MessageType::Message => {
                            console_log!("enter main message logic {:?}", message);
                            if let Ok(message) =
                                serde_json::from_str::<GameStateMessage>(&message.message)
                            {
                                console_log!("enter game state message logic {:?}", message);
                                if message.player_index == card_game_shared_state.phase_index
                                    || card_game_shared_state.consumed_seq_no
                                        == message.last_consumed_seq_no
                                {
                                    // 自分のメッセージ、または同期が取れているものは無視
                                    // empty
                                } else {
                                    let last_consumed = message.last_consumed_seq_no;
                                    for n in last_consumed + 1
                                        ..card_game_shared_state.consumed_seq_no + 1
                                    {
                                        if let Some(found) = card_game_shared_state
                                            .temporary_bid_history
                                            .iter()
                                            .find(|message| message.seq_no == n)
                                        {
                                            shared_state
                                                .to_send_channel_messages
                                                .push(serde_json::to_string(found).unwrap());
                                        };
                                        if let Some(found) = card_game_shared_state
                                            .bid_history
                                            .iter()
                                            .find(|message| message.seq_no == n)
                                        {
                                            shared_state
                                                .to_send_channel_messages
                                                .push(serde_json::to_string(found).unwrap());
                                        };
                                        if let Some(found) = card_game_shared_state
                                            .use_card_history
                                            .iter()
                                            .find(|message| message.seq_no == n)
                                        {
                                            shared_state
                                                .to_send_channel_messages
                                                .push(serde_json::to_string(found).unwrap());
                                        };
                                        if let Some(found) = card_game_shared_state
                                            .attack_target_history
                                            .iter()
                                            .find(|message| message.seq_no == n)
                                        {
                                            shared_state
                                                .to_send_channel_messages
                                                .push(serde_json::to_string(found).unwrap());
                                        };
                                    }
                                }
                                console_log!("complete game state message logic {:?}", message);
                            } else if let Ok(message) =
                                serde_json::from_str::<GameStartIsApprovedMessage>(&message.message)
                            {
                                console_log!(
                                    "enter game start is approved message logic {:?}",
                                    message
                                );
                                card_game_shared_state.players[message.player_index]
                                    .game_start_is_approved = message.game_start_is_approved;
                                console_log!(
                                    "complete game start is approved message logic {:?}",
                                    message
                                );
                            } else if let Ok(message) =
                                serde_json::from_str::<BidMessage>(&message.message)
                            {
                                console_log!("enter bid message logic {:?}", message);
                                if !card_game_shared_state.check_and_update_seq_no(
                                    message.seq_no,
                                    message.player_index == card_game_shared_state.own_player_index,
                                ) {
                                    return;
                                }
                                card_game_shared_state.temporary_bid_history.push(message);
                                BidMessage::ready_bid_input(
                                    &mut card_game_shared_state.bid_input,
                                    &card_game_shared_state.temporary_bid_history,
                                );
                                console_log!("complete bid message logic");
                            } else if let Ok(message) =
                                serde_json::from_str::<UseCardMessage>(&message.message)
                            {
                                console_log!("enter use card message logic {:?}", message);
                                if !card_game_shared_state.check_and_update_seq_no(
                                    message.seq_no,
                                    message.player_index == card_game_shared_state.own_player_index,
                                ) {
                                    return;
                                }
                                console_log!("use card logic 1");
                                if !message.is_skipped {
                                    let card = card_game_shared_state.players[message.player_index]
                                        .own_card_list
                                        .remove(message.use_card_index);
                                    console_log!("use card logic 2");
                                    let mut card_use_functions =
                                        card.get_use_func(message.player_index);
                                    card_use_functions(card_game_shared_state);
                                }
                                console_log!("use card logic 3");
                                card_game_shared_state.use_card_history.push(message);
                                console_log!("complete use card message logic");
                            } else if let Ok(message) =
                                serde_json::from_str::<AttackTargetMessage>(&message.message)
                            {
                                console_log!("enter attack target message logic {:?}", message);
                                if !card_game_shared_state.check_and_update_seq_no(
                                    message.seq_no,
                                    message.player_index == card_game_shared_state.own_player_index,
                                ) {
                                    return;
                                }
                                if message.is_skipped {
                                    card_game_shared_state.players[message.player_index]
                                        .player_state
                                        .current_money_amount += 1;
                                    interrupt_animations.push(vec![Animation::create_message(
                                        format!(
                                            "{}さんは 1 Moneyを得た",
                                            card_game_shared_state.players[message.player_index]
                                                .player_name
                                        ),
                                    )])
                                } else {
                                    let opponent_player_index = (message.player_index + 1)
                                        % card_game_shared_state.players.iter().len();
                                    let opponent_player_defence_point = card_game_shared_state
                                        .players[opponent_player_index]
                                        .player_state
                                        .defence_point;
                                    let player_attack_point = card_game_shared_state.players
                                        [message.player_index]
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
                                    interrupt_animations.push(vec![Animation::create_message(
                                        format!(
                                            "{}さんに{}のダメージ（残りHP: {}）",
                                            card_game_shared_state.players[opponent_player_index]
                                                .player_name,
                                            damage,
                                            card_game_shared_state.players[opponent_player_index]
                                                .player_state
                                                .current_hp,
                                        ),
                                    )]);
                                }
                                card_game_shared_state.attack_target_history.push(message);
                                console_log!("complete attack target message logic");
                            }
                        }
                        _ => {}
                    }
                    console_log!("complete message logic");
                    console_log!("start phase check");
                    let mut check_result = CheckPhaseCompleteResult::empty();
                    'outer: for player_index in 0..card_game_shared_state.players.len() {
                        if shared_state.is_matched
                            && card_game_shared_state.own_player_index != player_index
                        {
                            continue;
                        }
                        card_game_shared_state.own_player_index = player_index;
                        'inner: loop {
                            check_result =
                                card_game_shared_state.check_phase_complete(interrupt_animations);
                            console_log!("check result {}", check_result.is_phase_complete);
                            console_log!("check result {:?}", check_result.next_phase_index);
                            console_log!(
                                "check result {:?}",
                                check_result.is_required_own_input_for_complete
                            );
                            if check_result.is_phase_complete {
                                if check_result.next_phase_index.unwrap() == 4 {
                                    console_log!("battle end");
                                    break 'outer;
                                }
                                console_log!(
                                    "phase shift to start, turn: {}",
                                    card_game_shared_state.turn
                                );
                                card_game_shared_state.phase_shift_to(
                                    interrupt_animations,
                                    check_result.next_phase_index.unwrap(),
                                );
                                console_log!(
                                    "phase shift to end, turn:  {}",
                                    card_game_shared_state.turn
                                );
                                game_main_state.renderers[0].cursor.reset();
                            } else {
                                break 'inner;
                            }
                        }
                        if check_result.is_required_own_input_for_complete.unwrap() {
                            break;
                        }
                    }
                    if let Some(next_phase_index) = check_result.next_phase_index {
                        if next_phase_index == 4 {
                            card_game_shared_state.phase_index = 4;
                            return;
                        }
                    }
                    console_log!(
                        "input required player {}",
                        card_game_shared_state.own_player_index
                    );
                    console_log!(
                        "now phase is... {:?}",
                        card_game_shared_state.phases[card_game_shared_state.phase_index]
                    );
                    console_log!(
                        "next input is... {:?}",
                        check_result.is_required_own_input_for_complete
                    );

                    card_game_shared_state.input_is_guard =
                        !check_result.is_required_own_input_for_complete.unwrap();
                    let card_names = card_game_shared_state
                        .cards_bid_on
                        .iter()
                        .map(|card| card.card_kind.get_card_name())
                        .collect();
                    let card_descriptions = card_game_shared_state
                        .cards_bid_on
                        .iter()
                        .map(|card| card.card_kind.get_card_description())
                        .collect();
                    game_main_state.renderers[0].render(card_names, card_descriptions, "");
                    match card_game_shared_state.phase_index {
                        3 => {
                            let opponent_player_name = &card_game_shared_state.players
                                [(card_game_shared_state.own_player_index + 1)
                                    % card_game_shared_state.players.len()]
                            .player_name;
                            if !card_game_shared_state.input_is_guard {
                                game_main_state.renderers[2].render(
                                    vec![
                                        opponent_player_name.to_owned(),
                                        "攻撃しない(Money+1)".to_string(),
                                    ],
                                    vec![],
                                    format!(
                                        "{}さん、攻撃対象を選んでください。",
                                        card_game_shared_state.players
                                            [card_game_shared_state.own_player_index]
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
        consume_channel_message
    }
}
