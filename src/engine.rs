use crate::engine::application_types::StateType::BoPShared;
use crate::features::animation::Animation;
use crate::features::websocket::{ChannelMessage, MessageType, WebSocketWrapper};
use input::Input;
use scene::Scene;
use state::State;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_test::console_log;

pub mod application_types;
pub mod input;
pub mod scene;
pub mod state;

#[wasm_bindgen]
pub struct Engine {
    pub(crate) scenes: Vec<Scene>,
    pub(crate) web_socket_wrapper: WebSocketWrapper,
    pub(crate) shared_state: State,
}

#[wasm_bindgen]
impl Engine {
    pub(crate) fn new(
        shared_state: State,
        scenes: Vec<Scene>,
        web_socket_wrapper: WebSocketWrapper,
    ) -> Engine {
        Engine {
            scenes,
            shared_state,
            web_socket_wrapper,
        }
    }

    pub fn keydown(&mut self, key: String) {
        let input = Input::from(key);
        if self.shared_state.references.borrow_mut().has_block_message {
            if !self
                .shared_state
                .references
                .borrow_mut()
                .has_continuous_message
            {
                self.shared_state.elements.message.hide();
            }
            (*self.shared_state.references.borrow_mut()).has_block_message = false;
            return;
        }
        if self.has_animation_blocking_scene_update() {
            console_log!("keydown interrupt {:?}", input);
            return;
        }
        let scene_index = self.shared_state.primitives.scene_index;
        let consume_func = self.scenes[scene_index].consume_func;
        consume_func(&mut self.scenes[scene_index], &mut self.shared_state, input);
        if !self.has_animation_blocking_scene_update() {
            if self.shared_state.primitives.scene_index
                != self.shared_state.primitives.requested_scene_index
            {
                self.shared_state.primitives.scene_index =
                    self.shared_state.primitives.requested_scene_index;
                self.on_scene_update();
            }
            if self.shared_state.primitives.map_index
                != self.shared_state.primitives.requested_map_index
            {
                self.shared_state.primitives.map_index =
                    self.shared_state.primitives.requested_map_index;
                self.on_map_update();
            }
        }
    }

    fn on_scene_update(&mut self) {
        console_log!(
            "scene_updated {:?}",
            self.shared_state.primitives.scene_index
        );
        let scene_index = self.shared_state.primitives.scene_index;
        if scene_index != 0 && !self.web_socket_wrapper.state.borrow_mut().is_joined {
            // TODO
            // 対戦用のWebSocketに切り替えると、タイミング的にメッセージ送信が失敗する
            // self.web_socket_wrapper.join();
        }
        if scene_index == 0 && self.web_socket_wrapper.state.borrow_mut().is_joined {
            // TODO
            // 対戦用のWebSocketに切り替えると、タイミング的にメッセージ送信が失敗する
            // self.web_socket_wrapper.left();
        }
        if !self.scenes[scene_index].is_partial_scene {
            // メニューからタイトルなどもあるので遷移先が is_partial_scene でないなら一括で隠す
            for scene in self.scenes.iter() {
                scene.hide();
            }
        }
        let init_func = self.scenes[scene_index].init_func;
        init_func(&mut self.scenes[scene_index], &mut self.shared_state);
    }

    fn on_map_update(&mut self) {
        let update_map_func = self.scenes[self.shared_state.primitives.scene_index].update_map_func;
        update_map_func(
            &mut self.scenes[self.shared_state.primitives.scene_index],
            &mut self.shared_state,
        );
    }

    fn has_animation_blocking_scene_update(&self) -> bool {
        self.shared_state
            .interrupt_animations
            .iter()
            .find(|animation| animation.get(0).unwrap().block_scene_update)
            .is_some()
    }

    fn receive_channel_message(&mut self, channel_message: &mut ChannelMessage) {
        let message = channel_message.message.to_owned();
        console_log!("receive_channel_message {}", message);
        if !self.shared_state.is_request_matching {
            channel_message.message = message;
            for scene in self.scenes.iter_mut() {
                let consume_channel_message_func = scene.consume_channel_message_func;
                consume_channel_message_func(scene, &mut self.shared_state, &channel_message);
            }
            return;
        }
        if let Ok(special_message) = serde_json::from_str::<ChannelMessage>(&message) {
            match special_message.message_type {
                MessageType::MatchRequest => {
                    if special_message.user_name != self.shared_state.user_name {
                        let to_send_message = serde_json::to_string(&ChannelMessage {
                            user_name: self.shared_state.user_name.to_string(),
                            message_type: MessageType::MatchResponse,
                            message: special_message.user_name.clone(),
                        })
                        .unwrap();
                        console_log!("to_send_message {}", to_send_message);
                        self.shared_state
                            .to_send_channel_messages
                            .push(to_send_message);
                    }
                    return;
                }
                MessageType::MatchResponse => {
                    if special_message.user_name != self.shared_state.user_name
                        && special_message.message == self.shared_state.user_name
                    {
                        if let BoPShared(card_game_shared_state) = &mut self.shared_state.state_type
                        {
                            card_game_shared_state.own_player_index = 0;
                            console_log!("you are first.");
                        }
                    } else if special_message.user_name == self.shared_state.user_name {
                        if let BoPShared(card_game_shared_state) = &mut self.shared_state.state_type
                        {
                            card_game_shared_state.own_player_index = 1;
                            console_log!("you are second.");
                        }
                    } else {
                        return;
                    }
                    console_log!(
                        "match response {} {}",
                        special_message.message,
                        special_message.user_name
                    );
                    self.web_socket_wrapper.ws.close().unwrap();
                    self.web_socket_wrapper.state.borrow_mut().channel_name =
                        format!("bop-{}", special_message.message);
                    self.web_socket_wrapper.force_update_not_ready();
                    self.web_socket_wrapper.request_reconnect();
                    self.shared_state.is_request_matching = false;
                    self.shared_state.primitives.requested_scene_index = 1;
                    self.shared_state.is_matched = true;
                }
                _ => {}
            }
        }
    }

    pub fn animate(&mut self, step: f64) {
        if self.shared_state.keep_connection_request {
            if !self.web_socket_wrapper.is_ready() {
                self.web_socket_wrapper.request_reconnect();
            }
            self.shared_state.keep_connection_request = false;
        }
        // 送るべきメッセージが存在していてWebSocketが切れていれば再接続
        if !self.web_socket_wrapper.is_ready()
            && !self.shared_state.to_send_channel_messages.is_empty()
        {
            self.web_socket_wrapper.request_reconnect();
        }

        // WebSocketに届いたメッセージをアプリケーションに処理させる
        while !(*self.web_socket_wrapper.messages.borrow_mut()).is_empty() {
            let mut message = (*self.web_socket_wrapper.messages.borrow_mut()).remove(0);
            self.receive_channel_message(&mut message);
        }

        // 送るべきメッセージが存在していてWebSocket接続が準備できていれば全て送信
        while self.web_socket_wrapper.is_ready()
            && !self.shared_state.to_send_channel_messages.is_empty()
        {
            let message = self.shared_state.to_send_channel_messages.remove(0);
            self.web_socket_wrapper.send_message(message);
        }

        let mut to_delete_indexes = vec![];
        for (index, animation) in self
            .shared_state
            .interrupt_animations
            .iter_mut()
            .enumerate()
        {
            let func = animation.get(0).unwrap().animation_func;
            let result = func(
                animation.get_mut(0).unwrap(),
                self.shared_state.references.clone(),
                step,
            );
            if result {
                to_delete_indexes.push(index)
            }
        }

        to_delete_indexes.reverse();
        for index in to_delete_indexes.iter() {
            let at_animations = self
                .shared_state
                .interrupt_animations
                .get_mut(*index)
                .unwrap();
            at_animations.remove(0);
            if at_animations.is_empty() {
                self.shared_state.interrupt_animations.remove(*index);
            }
        }

        if self
            .shared_state
            .interrupt_animations
            .iter()
            .filter(|animation| animation.get(0).unwrap().block_scene_update)
            .collect::<Vec<&Vec<Animation>>>()
            .len()
            == 0
        {
            if self.shared_state.primitives.scene_index
                != self.shared_state.primitives.requested_scene_index
            {
                self.shared_state.primitives.scene_index =
                    self.shared_state.primitives.requested_scene_index;
                self.on_scene_update()
            }
            if self.shared_state.primitives.map_index
                != self.shared_state.primitives.requested_map_index
            {
                self.shared_state.primitives.map_index =
                    self.shared_state.primitives.requested_map_index;
                self.on_map_update();
            }
        }
        if let State {
            state_type: BoPShared(card_game_shared_state),
            ..
        } = &mut self.shared_state
        {
            for n in 0..card_game_shared_state.simple_binders.len() {
                // 自己参照を含んでいるので一旦 SimpleBinder の clone をして、戻す
                let mut binder = card_game_shared_state.simple_binders[n].clone();
                let binder = binder.sync(card_game_shared_state);
                card_game_shared_state.simple_binders[n] = binder.clone();
            }
        }
    }
}
