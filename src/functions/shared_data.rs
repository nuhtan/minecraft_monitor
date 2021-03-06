// TODO Documentation
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct ServerSharedData {
    pub server_output: Arc<Mutex<VecDeque<(u32, String)>>>,
    pub current_player_count: Arc<Mutex<u32>>,
    pub current_players: Arc<Mutex<Vec<String>>>,
    pub max_player_count: Arc<Mutex<u32>>,
    pub mcserver_state: Arc<Mutex<MinecraftServerState>>,
    pub gen_state: Arc<Mutex<GeneralState>>,
}

impl ServerSharedData {
    pub fn new() -> ServerSharedData {
        ServerSharedData {
            server_output: Arc::new(Mutex::new(VecDeque::<(u32, String)>::new())),
            current_player_count: Arc::new(Mutex::new(0)),
            current_players: Arc::new(Mutex::new(Vec::<String>::new())),
            max_player_count: Arc::new(Mutex::new(0)),
            mcserver_state: Arc::new(Mutex::new(MinecraftServerState::Starting)),
            gen_state: Arc::new(Mutex::new(GeneralState::Running)),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum MinecraftServerState {
    Off,
    Starting,
    Running,
    Eula
}

#[derive(Clone, Copy, PartialEq)]
pub enum GeneralState {
    Running,
    ShutDown,
    Restart,
}
