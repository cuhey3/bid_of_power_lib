use crate::bop::CardGameSharedState;
use crate::engine::application_types::SceneType::BoPTitle;
use crate::engine::application_types::StateType::BoPShared;
use crate::engine::input::Input;
use crate::engine::scene::Scene;
use crate::engine::state::State;
use crate::features::animation::Animation;
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
        fn init_func(scene: &mut Scene, shared_state: &mut State) {
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
            match &mut scene.scene_type {
                BoPTitle(title_state) => match input {
                    Input::ArrowUp | Input::ArrowDown => {
                        title_state.cursor.consume(input);
                    }
                    Input::Enter => {
                        if title_state.cursor.chose_index == 0 {
                            shared_state.primitives.requested_scene_index = 1;
                            CardGameSharedState::new_game(shared_state);
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
                },
                _ => panic!(),
            }
        }
        consume_func
    }
}