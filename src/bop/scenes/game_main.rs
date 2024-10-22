use crate::bop::mechanism::choice_kind::ChoiceKind::{Confirm, Menu};
use crate::bop::state::card_game_shared_state::{
    BidMessage, CheckPhaseCompleteResult, GameStartIsApprovedMessage, UseCardMessage,
};
use crate::engine::application_types::SceneType::BoPGameMain;
use crate::engine::application_types::StateType::BoPShared;
use crate::engine::input::Input;
use crate::engine::scene::Scene;
use crate::engine::state::State;
use crate::features::websocket::{ChannelMessage, MessageType};
use crate::svg::element_wrapper::ElementWrapper;
use crate::svg::svg_renderer::{Cursor, SvgRenderer};
use wasm_bindgen_test::console_log;

pub struct GameMainState {
    renderers: Vec<SvgRenderer>,
    is_bid_confirm_opened: bool,
    use_item_cursors: Vec<Cursor>,
}

impl GameMainState {
    pub fn create_game_main_scene(shared_state: &mut State) -> Scene {
        let mut renderer = SvgRenderer::new(Menu, "game-main-bid".to_string(), 45.0);
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
                SvgRenderer::new(Confirm, "game-main-common-confirm".to_string(), 30.0),
            ],
            is_bid_confirm_opened: false,
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
        fn init_func(scene: &mut Scene, _: &mut State) {
            console_log!("init game main scene");
            scene.show();
            match &mut scene.scene_type {
                BoPGameMain(..) => {}
                _ => panic!(),
            }
        }
        init_func
    }
    pub fn create_consume_func(&self) -> fn(&mut Scene, &mut State, Input) {
        fn consume_func(scene: &mut Scene, shared_state: &mut State, input: Input) {
            if let State {
                state_type: BoPShared(card_game_shared_state),
                to_send_channel_messages,
                ..
            } = shared_state
            {
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
                                    (bid_amount + 1).min(player_money);
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
                                card_game_shared_state.bid_input[cursor_index].bid_amount =
                                    (bid_amount - 1).max(current_bid_amount).max(1);
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
                                } else {
                                    let item_name = card_game_shared_state.cards_bid_on
                                        [cursor_index]
                                        .card_kind
                                        .get_card_name();
                                    let amount =
                                        card_game_shared_state.bid_input[cursor_index].bid_amount;
                                    game_main_state.renderers[1].render(
                                        vec!["はい".to_string(), "いいえ".to_string()],
                                        vec![],
                                        format!("{} を {} で入札しますか？", item_name, amount)
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
                                let player_index = card_game_shared_state.own_player_index;
                                game_main_state.use_item_cursors[player_index]
                                    .update_choice_length(
                                        card_game_shared_state.players[player_index]
                                            .own_card_list
                                            .len(),
                                    );
                                game_main_state.use_item_cursors[player_index].consume(input);
                            }
                            Input::Enter => {
                                let player_index = card_game_shared_state.own_player_index;
                                let cursor_index =
                                    game_main_state.use_item_cursors[player_index].chose_index;
                                game_main_state.use_item_cursors[player_index].reset();
                                to_send_channel_messages.push(
                                    serde_json::to_string(&UseCardMessage {
                                        turn: 0,
                                        check_is_blocked: false,
                                        player_index: card_game_shared_state.own_player_index,
                                        use_card_index: cursor_index,
                                        is_skipped: false,
                                        args_i32: vec![],
                                        args_usize: vec![],
                                    })
                                    .unwrap(),
                                );
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
                    let found = card_game_shared_state
                        .online_users
                        .iter_mut()
                        .enumerate()
                        .find(|(_, user)| user.user_name == message.user_name);
                    match message.message_type {
                        MessageType::Left => {
                            if found.is_some() {
                                let remove_index = found.unwrap().0;
                                card_game_shared_state.online_users.remove(remove_index);
                            }
                        }
                        MessageType::Message => {
                            if let Ok(message) =
                                serde_json::from_str::<GameStartIsApprovedMessage>(&message.message)
                            {
                                card_game_shared_state.players[message.player_index]
                                    .game_start_is_approved = message.game_start_is_approved;
                            } else if let Ok(message) =
                                serde_json::from_str::<BidMessage>(&message.message)
                            {
                                card_game_shared_state.temporary_bid_history.push(message);
                                BidMessage::ready_bid_input(
                                    &mut card_game_shared_state.bid_input,
                                    &card_game_shared_state.temporary_bid_history,
                                );
                            } else if let Ok(message) =
                                serde_json::from_str::<UseCardMessage>(&message.message)
                            {
                                let card = card_game_shared_state.players
                                    [card_game_shared_state.own_player_index]
                                    .own_card_list
                                    .remove(message.use_card_index);
                                let mut card_use_functions = card.get_use_func();
                                card_use_functions(card_game_shared_state);
                                card_game_shared_state.use_card_history.push(message);
                            }
                        }
                        _ => {}
                    }
                    console_log!("start consume");
                    let mut check_result = CheckPhaseCompleteResult::empty();
                    for player_index in 0..card_game_shared_state.players.len() {
                        card_game_shared_state.own_player_index = player_index;
                        'inner: loop {
                            check_result = card_game_shared_state.check_phase_complete();
                            console_log!("check result {}", check_result.is_phase_complete);
                            console_log!("check result {:?}", check_result.next_phase_index);
                            console_log!(
                                "check result {:?}",
                                check_result.is_required_own_input_for_complete
                            );
                            if check_result.is_phase_complete {
                                card_game_shared_state.phase_shift_to(
                                    interrupt_animations,
                                    check_result.next_phase_index.unwrap(),
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
                }
            }
        }
        consume_channel_message
    }
}
