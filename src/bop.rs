use crate::bop::scenes::game_main::GameMainState;
use crate::bop::state::bind::get_binds;
use crate::bop::state::bop_shared_state::BoPPlayer;
use crate::bop::state::message::{GameStartIsApprovedMessage, GameStateMessage};
use crate::engine::application_types::StateType::BoPShared;
use crate::engine::state::{Primitives, References, State};
use crate::engine::Engine;
use crate::features::animation::Animation;
use crate::features::websocket::{ChannelMessage, MessageType, WebSocketWrapper};
use crate::svg::SharedElements;
use mechanism::item::Item;
use mechanism::player_status::PlayerStatus;
use rand::Rng;
use scenes::title::TitleState;
use state::bop_shared_state::BoPSharedState;
use state::message::{AttackTargetMessage, BidMessage, UseItemMessage};
use state::phase::Phase;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen_test::console_log;

mod cpu_player;
pub mod mechanism;
pub mod scenes;
pub mod state;

pub fn mount() -> Engine {
    let mut rng = rand::thread_rng();
    let random_number = rng.random::<u16>();
    let user_name = random_number.to_string();

    let rpg_shared_state = BoPSharedState {
        players: vec![
            BoPPlayer {
                player_name: "プレイヤー1".to_string(),
                game_start_is_approved: false,
                battle_is_viewed: false,
                own_item_list: vec![],
                player_status: PlayerStatus::init(),
            },
            BoPPlayer {
                player_name: "プレイヤー2".to_string(),
                game_start_is_approved: false,
                battle_is_viewed: false,
                own_item_list: vec![],
                player_status: PlayerStatus::init(),
            },
        ],
        players_len: 2,
        own_player_index: 0,
        items_bid_on: vec![],
        bid_input: vec![
            BidMessage::init(0),
            BidMessage::init(1),
            BidMessage::init(2),
        ],
        bid_scheduled_items: Item::item_set_default(),
        temporary_bid_history: vec![],
        bid_history: vec![],
        use_item_input: UseItemMessage::empty(),
        use_item_history: vec![],
        attack_target_input: AttackTargetMessage::empty(),
        attack_target_history: vec![],
        initiatives_to_player_index: vec![0, 1],
        game_logs: vec![],
        turn: 0,
        phase_index: 0,
        phases: Phase::get_phases(),
        simple_binders: get_binds(),
        input_is_guard: false,
        consumed_seq_no: 0,
        has_cpu: false,
    };
    let mut shared_state = State {
        user_name: user_name.to_owned(),
        to_send_channel_messages: vec![],
        elements: SharedElements::new(),
        interrupt_animations: vec![vec![Animation::always_blink()]],
        state_type: BoPShared(rpg_shared_state.clone()),
        primitives: Primitives {
            scene_index: 0,
            requested_scene_index: 0,
            map_index: 0,
            requested_map_index: 0,
        },
        references: Rc::new(RefCell::new(References {
            has_block_message: false,
            has_continuous_message: false,
        })),
        is_request_matching: false,
        is_matched: false,
        keep_connection_request: false,
    };

    let mut scenes = vec![
        TitleState::create_title_scene(&mut shared_state),
        GameMainState::create_game_main_scene(&mut shared_state),
    ];
    let init_func = scenes[0].init_func;
    init_func(&mut scenes[0], &mut shared_state);
    let web_socket_wrapper =
        WebSocketWrapper::new(shared_state.user_name.to_owned(), "bop".to_string());
    Engine::new(shared_state, scenes, web_socket_wrapper)
}

impl State {
    pub fn consume_channel_message(&mut self, message: &ChannelMessage) {
        console_log!("consume_channel_message start {}", message.message);
        if let State {
            state_type: BoPShared(bop_shared_state),
            interrupt_animations,
            ..
        } = self
        {
            match message.message_type {
                MessageType::Join => {
                    console_log!("enter join message logic {:?}", message);
                    if message.user_name == self.user_name && bop_shared_state.consumed_seq_no != 0
                    {
                        // 自分が復帰したことを相手に知らせる
                        self.to_send_channel_messages.push(
                            serde_json::to_string(&GameStateMessage {
                                player_index: bop_shared_state.own_player_index,
                                last_consumed_seq_no: bop_shared_state.consumed_seq_no,
                            })
                            .unwrap(),
                        )
                    }
                    console_log!("complete join message logic {:?}", message);
                }
                MessageType::Message => {
                    console_log!("enter main message logic {:?}", message);
                    if let Ok(message) = serde_json::from_str::<GameStateMessage>(&message.message)
                    {
                        console_log!("enter game state message logic {:?}", message);
                        if message.player_index == bop_shared_state.phase_index
                            || bop_shared_state.consumed_seq_no == message.last_consumed_seq_no
                        {
                            // 自分のメッセージ、または同期が取れているものは無視
                            // empty
                        } else {
                            let last_consumed = message.last_consumed_seq_no;
                            for n in last_consumed + 1..bop_shared_state.consumed_seq_no + 1 {
                                if let Some(found) = bop_shared_state
                                    .temporary_bid_history
                                    .iter()
                                    .find(|message| message.seq_no == n)
                                {
                                    self.to_send_channel_messages
                                        .push(serde_json::to_string(found).unwrap());
                                };
                                if let Some(found) = bop_shared_state
                                    .bid_history
                                    .iter()
                                    .find(|message| message.seq_no == n)
                                {
                                    self.to_send_channel_messages
                                        .push(serde_json::to_string(found).unwrap());
                                };
                                if let Some(found) = bop_shared_state
                                    .use_item_history
                                    .iter()
                                    .find(|message| message.seq_no == n)
                                {
                                    self.to_send_channel_messages
                                        .push(serde_json::to_string(found).unwrap());
                                };
                                if let Some(found) = bop_shared_state
                                    .attack_target_history
                                    .iter()
                                    .find(|message| message.seq_no == n)
                                {
                                    self.to_send_channel_messages
                                        .push(serde_json::to_string(found).unwrap());
                                };
                            }
                        }
                        console_log!("complete game state message logic {:?}", message);
                    } else if let Ok(message) =
                        serde_json::from_str::<GameStartIsApprovedMessage>(&message.message)
                    {
                        console_log!("enter game start is approved message logic {:?}", message);
                        bop_shared_state.players[message.player_index].game_start_is_approved =
                            message.game_start_is_approved;
                        console_log!(
                            "complete game start is approved message logic {:?}",
                            message
                        );
                    } else {
                        bop_shared_state.update_game_state_by_message(
                            message.message.clone(),
                            interrupt_animations,
                            false,
                        );
                    }
                }
                _ => {}
            }
        }
    }
}
