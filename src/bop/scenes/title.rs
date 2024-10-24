use crate::bop::state::message::GameStartIsApprovedMessage;
use crate::engine::application_types::SceneType::BoPTitle;
use crate::engine::application_types::StateType::BoPShared;
use crate::engine::input::Input;
use crate::engine::scene::Scene;
use crate::engine::state::State;
use crate::features::animation::Animation;
use crate::features::websocket::{ChannelMessage, MessageType};
use crate::svg::element_wrapper::ElementWrapper;
use crate::svg::svg_renderer::Cursor;

pub struct TitleState {
    cursor: Cursor,
}

impl TitleState {
    pub fn create_title_scene(shared_state: &mut State) -> Scene {
        let document = &shared_state.elements.document;
        let title_state = TitleState {
            cursor: Cursor::new(document, "title-cursor", 2, 60.0),
        };
        let consume_func = title_state.create_consume_func();
        let init_func = title_state.create_init_func();
        let scene_type = BoPTitle(title_state);
        Scene {
            own_element: ElementWrapper::new(
                shared_state
                    .elements
                    .document
                    .get_element_by_id("title")
                    .unwrap(),
            ),
            scene_type,
            is_partial_scene: false,
            consume_func,
            init_func,
            update_map_func: Scene::create_update_map_func_empty(),
            consume_channel_message_func: Scene::create_consume_channel_message_func_empty(),
        }
    }
    pub fn create_init_func(&self) -> fn(&mut Scene, &mut State) {
        fn init_func(scene: &mut Scene, _: &mut State) {
            scene.show();
            match &mut scene.scene_type {
                BoPTitle(..) => {}
                _ => panic!(),
            }
        }
        init_func
    }
    pub fn create_consume_func(&self) -> fn(&mut Scene, &mut State, Input) {
        fn consume_func(scene: &mut Scene, shared_state: &mut State, input: Input) {
            if let State {
                state_type: BoPShared(..),
                to_send_channel_messages,
                ..
            } = shared_state
            {
                if let BoPTitle(title_state) = &mut scene.scene_type {
                    match input {
                        Input::ArrowUp | Input::ArrowDown => {
                            title_state.cursor.consume(input);
                        }
                        Input::Enter => {
                            if title_state.cursor.chose_index == 0 {
                                shared_state.primitives.requested_scene_index = 1;
                                to_send_channel_messages.push(
                                    serde_json::to_string(&GameStartIsApprovedMessage {
                                        player_index: 0,
                                        game_start_is_approved: true,
                                    })
                                    .unwrap(),
                                );
                                to_send_channel_messages.push(
                                    serde_json::to_string(&GameStartIsApprovedMessage {
                                        player_index: 1,
                                        game_start_is_approved: true,
                                    })
                                    .unwrap(),
                                );
                            } else if title_state.cursor.chose_index == 1 {
                                to_send_channel_messages.push(
                                    serde_json::to_string(&ChannelMessage {
                                        user_name: shared_state.user_name.to_string(),
                                        message_type: MessageType::MatchRequest,
                                        message: shared_state.user_name.to_string(),
                                    })
                                    .unwrap(),
                                );
                                shared_state.is_request_matching = true;
                                // shared_state.interrupt_animations.push(vec![
                                //     Animation::create_message("Coming soon...".to_string()),
                                // ]);
                                return;
                            } else {
                                shared_state.interrupt_animations.push(vec![
                                    Animation::create_message("Coming soon...".to_string()),
                                ]);
                                return;
                            }
                            shared_state
                                .interrupt_animations
                                .push(vec![Animation::create_fade_out_in()]);
                        }
                        _ => (),
                    }
                }
            }
        }
        consume_func
    }
}
