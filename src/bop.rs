use crate::bop::scenes::game_main::GameMainState;
use crate::bop::state::bind::get_binds;
use crate::bop::state::card_game_shared_state::{
    AttackTargetMessage, BidMessage, CardGamePlayer, GameStartIsApprovedMessage, Phase,
    PlayerState, UseCardMessage,
};
use crate::engine::application_types::StateType;
use crate::engine::state::{Primitives, References, State};
use crate::engine::Engine;
use crate::features::animation::Animation;
use crate::features::websocket::WebSocketWrapper;
use crate::svg::Position;
use crate::svg::SharedElements;
use mechanism::card::Card;
use mechanism::item::Item;
use rand::Rng;
use scenes::battle::BattleState;
use scenes::field::FieldState;
use scenes::menu::MenuState;
use scenes::title::TitleState;
use serde::{Deserialize, Serialize};
use state::card_game_shared_state::CardGameSharedState;
use state::character::Character;
use std::cell::RefCell;
use std::rc::Rc;

pub mod mechanism;
pub mod scenes;
pub mod state;

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    character_u32: Vec<u32>,
    pub treasure_box_usize: Vec<Vec<usize>>,
    pub map_usize: Vec<usize>,
    map_i32: Vec<i32>,
    inventory_string: Vec<String>,
    check_token: u32,
    event_flags: Vec<bool>,
}

impl SaveData {
    pub fn load(&mut self, characters: &mut Vec<Character>, try_get_storage: bool) {
        if try_get_storage {
            let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
            let raw_save = storage.get_item("save").unwrap();
            if raw_save.is_some() {
                let raw_save = raw_save.unwrap();
                let local_save_data: SaveData = serde_json::from_str(raw_save.as_str()).unwrap();
                self.character_u32 = local_save_data.character_u32.to_vec();
                self.treasure_box_usize = local_save_data.treasure_box_usize.to_vec();
                self.map_usize = local_save_data.map_usize.to_vec();
                self.map_i32 = local_save_data.map_i32.to_vec();
                self.inventory_string = local_save_data.inventory_string.to_vec();
                self.event_flags = local_save_data.event_flags.to_vec();
                self.check_token = local_save_data.check_token.to_owned();
            }
        }
        characters[0].current_hp = *self.character_u32.get(0).unwrap();
        characters[0].max_hp = *self.character_u32.get(1).unwrap();
        characters[0].position.x = *self.map_i32.get(0).unwrap();
        characters[0].position.y = *self.map_i32.get(1).unwrap();
        characters[0].inventory = self
            .inventory_string
            .iter()
            .map(|s| Item::new(s.as_str()))
            .collect();
        characters[0].event_flags = self.event_flags.to_vec();
    }
    pub fn update(
        &mut self,
        characters: &Vec<Character>,
        treasure_box_opened: &Vec<Vec<usize>>,
        map_index: usize,
    ) {
        self.character_u32 = vec![characters[0].current_hp, characters[0].max_hp];
        self.treasure_box_usize = treasure_box_opened.to_vec();
        self.map_usize = vec![map_index];
        self.map_i32 = vec![characters[0].position.x, characters[0].position.y];
        self.inventory_string = characters[0]
            .inventory
            .iter()
            .map(|item| item.name.clone())
            .collect::<Vec<String>>();
        self.event_flags = characters[0].event_flags.to_vec();
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        let json = serde_json::to_string(self).unwrap();
        storage.set_item("save", json.as_str()).unwrap();
    }
    fn new(
        character_u32: Vec<u32>,
        treasure_box_usize: Vec<Vec<usize>>,
        map_usize: Vec<usize>,
        map_i32: Vec<i32>,
        inventory_string: Vec<String>,
        event_flags: Vec<bool>,
    ) -> SaveData {
        SaveData {
            character_u32,
            treasure_box_usize,
            map_usize,
            map_i32,
            inventory_string,
            event_flags,
            check_token: 0,
        }
    }
    pub fn empty() -> SaveData {
        SaveData {
            character_u32: vec![25, 80],
            treasure_box_usize: vec![vec![]],
            map_usize: vec![0],
            map_i32: vec![360, 280],
            inventory_string: vec![],
            event_flags: vec![],
            check_token: 0,
        }
    }
}

pub fn mount() -> Engine {
    let mut rng = rand::thread_rng();
    let random_number = rng.random::<u16>();
    let user_name = random_number.to_string();

    let rpg_shared_state = CardGameSharedState {
        treasure_box_opened: vec![],
        save_data: SaveData::empty(),
        online_users: vec![],
        characters: vec![Character {
            current_hp: 25,
            max_hp: 80,
            position: Position { x: -1, y: -1 },
            inventory: vec![],
            event_flags: vec![],
        }],
        // ここからBoP
        is_online: false,
        players: vec![
            CardGamePlayer {
                player_name: "プレイヤー1".to_string(),
                game_start_is_approved: false,
                battle_is_viewed: false,
                own_card_list: vec![],
                player_state: PlayerState::init(),
            },
            CardGamePlayer {
                player_name: "プレイヤー2".to_string(),
                game_start_is_approved: false,
                battle_is_viewed: false,
                own_card_list: vec![],
                player_state: PlayerState::init(),
            },
        ],
        own_player_index: 0,
        cards_bid_on: vec![],
        bid_input: vec![BidMessage::init(), BidMessage::init(), BidMessage::init()],
        bid_scheduled_cards: Card::card_set_default(),
        temporary_bid_history: vec![],
        bid_history: vec![],
        use_card_input: UseCardMessage::empty(),
        use_card_history: vec![],
        attack_target_input: AttackTargetMessage::empty(),
        attack_target_history: vec![],
        initiatives_to_player_index: vec![0, 1],
        game_logs: vec![],
        turn: 0,
        phase_index: 0,
        phases: Phase::get_phases(),
        simple_binders: get_binds(),
    };
    let to_send_channel_messages = vec![
        serde_json::to_string(&GameStartIsApprovedMessage {
            player_index: 0,
            game_start_is_approved: true,
        })
        .unwrap(),
        serde_json::to_string(&GameStartIsApprovedMessage {
            player_index: 1,
            game_start_is_approved: true,
        })
        .unwrap(),
    ];
    let mut shared_state = State {
        user_name: user_name.to_owned(),
        to_send_channel_messages,
        elements: SharedElements::new(),
        interrupt_animations: vec![vec![Animation::always_blink()]],
        state_type: StateType::BoPShared(rpg_shared_state),
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
    };

    let mut scenes = vec![
        TitleState::create_title_scene(&mut shared_state),
        GameMainState::create_game_main_scene(&mut shared_state),
        FieldState::create_field_scene(&mut shared_state),
        BattleState::create_battle_scene(&mut shared_state),
        MenuState::create_menu_scene(&mut shared_state),
    ];
    let init_func = scenes[0].init_func;
    init_func(&mut scenes[0], &mut shared_state);
    let web_socket_wrapper = WebSocketWrapper::new(shared_state.user_name.to_owned());
    Engine::new(shared_state, scenes, web_socket_wrapper)
}
