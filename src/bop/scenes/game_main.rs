use crate::bop::state::card_game_shared_state::{BidMessage, CheckPhaseCompleteResult, GameStartIsApprovedMessage};
use crate::engine::application_types::SceneType::BoPGameMain;
use crate::engine::application_types::StateType::BoPShared;
use crate::engine::input::Input;
use crate::engine::scene::Scene;
use crate::engine::state::State;
use crate::features::websocket::{ChannelMessage, MessageType};
use crate::svg::element_wrapper::ElementWrapper;
use wasm_bindgen_test::console_log;
use crate::bop::mechanism::choice_kind::ChoiceKind::Menu;
use crate::svg::svg_renderer::SvgRenderer;

pub struct GameMainState {
    renderers: Vec<SvgRenderer>
}

impl GameMainState {
    pub fn create_game_main_scene(shared_state: &mut State) -> Scene {
        let mut renderer = SvgRenderer::new(Menu, "game-main-bid".to_string(), 55.0);
        renderer.cursor.update_choice_length(3);
        renderer.cursor.update_cursor_amount_with_min_max(vec![[1,5]; 3]);
        renderer.load_amount_element();

        let mut game_main_state = GameMainState { renderers: vec![renderer] };
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
        fn init_func(scene: &mut Scene, shared_state: &mut State) {
            console_log!("init event scene");
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
                interrupt_animations,
                to_send_channel_messages,
                ..
            } = shared_state
            {
                if let Scene {
                    scene_type: BoPGameMain(game_main_state),
                    ..
                } = scene {
                    match input {
                        Input::ArrowDown | Input::ArrowUp | Input::ArrowRight | Input::ArrowLeft => {
                            game_main_state.renderers[0].consume(input);
                        }
                        Input::Enter => {
                            // interrupt_animations.push(vec![
                            //     Animation::create_message(format!("入札します。よろしいですか？ {:?} {:?}", card_names[game_main_state.renderers[0].cursor.chose_index], game_main_state.renderers[0].cursor.cursor_amount[game_main_state.renderers[0].cursor.chose_index].amount))
                            // ]);
                            to_send_channel_messages.push(serde_json::to_string(&BidMessage {
                                player_index: card_game_shared_state.own_player_index,
                                bid_card_index: game_main_state.renderers[0].cursor.chose_index,
                                bid_amount: game_main_state.renderers[0].cursor.cursor_amount[game_main_state.renderers[0].cursor.chose_index].amount,
                            }).unwrap());
                        }
                        _ => {}
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
                console_log!("ccm start {}", message.message);
                // if message.user_name == shared_state.user_name {
                //     return;
                // }
                if let State {
                    state_type: BoPShared(card_game_shared_state),
                    primitives,
                    elements,
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
                                console_log!("ccm message 1");
                                card_game_shared_state.players[message.player_index]
                                    .game_start_is_approved = message.game_start_is_approved;
                                console_log!("ccm message 2");
                            } else if let Ok(message) =
                                serde_json::from_str::<BidMessage>(&message.message)
                            {
                                game_main_state.renderers[0].cursor.cursor_amount[message.bid_card_index].current_amount = message.bid_amount;
                                game_main_state.renderers[0].cursor.cursor_amount[message.bid_card_index].min_amount = message.bid_amount + 2;
                                game_main_state.renderers[0].cursor.cursor_amount[message.bid_card_index].amount = message.bid_amount + 2;
                                game_main_state.renderers[0].cursor.cursor_amount[message.bid_card_index].initial_amount = message.bid_amount + 2;
                                card_game_shared_state.temporary_bid_history.push(message);
                            }
                        }
                        _ => {}
                    }
                    // // Joinの分は rpg_shared_state 使用の後に持ってこないと、second immutable borrow でビルド失敗する
                    // match message.message_type {
                    //     MessageType::Join => {
                    //         shared_state.send_own_position(None);
                    //     }
                    //     _ => {}
                    // }
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
                            card_game_shared_state
                                .phase_shift_to(interrupt_animations, check_result.next_phase_index.unwrap());
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
                    let card_names = card_game_shared_state.cards_bid_on.iter().map(|card|card.card_kind.get_card_name()).collect();
                    let card_descriptions = card_game_shared_state.cards_bid_on.iter().map(|card|card.card_kind.get_card_description()).collect();
                    game_main_state.renderers[0].render(card_names, card_descriptions, format!("{}さんの番です。入札してください", card_game_shared_state.players[card_game_shared_state.own_player_index].player_name).as_str());

                }

            }

        }
        consume_channel_message
    }
}
