use std::sync::{Arc, Mutex};
use steamworks::{Client, SingleClient};

#[derive(Default)]
pub struct SteamState {
    client: Arc<Mutex<Option<(u32, Client)>>>,
    single_client: Mutex<Option<(u32, SingleClient)>>,
}

impl SteamState {
    pub fn new() -> Self {
        SteamState {
            client: Arc::new(Mutex::new(None)),
            single_client: Mutex::new(None),
        }
    }

    pub fn has_client(&self, steam_game_id: u32) -> bool {
        if let Some((current_steam_game_id, _)) = self.client.lock().unwrap().as_ref() {
            *current_steam_game_id == steam_game_id
        } else {
            false
        }
    }

    pub fn get_client(&self, steam_game_id: u32) -> Option<Client> {
        let state = self.client.lock().unwrap();
        if let Some((current_steam_game_id, ref client)) = *state
            && current_steam_game_id == steam_game_id
        {
            return Some(client.clone());
        }
        None
    }

    pub fn run_callbacks(&self, steam_game_id: u32) -> Result<(), String> {
        let mut state = self.single_client.lock().unwrap();
        if let Some((current_steam_game_id, ref mut single_client)) = *state
            && current_steam_game_id == steam_game_id
        {
            single_client.run_callbacks();
            return Ok(());
        }
        Err("Single client not found for given steam_game_id".to_string())
    }

    pub fn set_clients(&self, steam_game_id: u32, client: Client, single_client: SingleClient) {
        {
            let mut client_state = self.client.lock().unwrap();
            *client_state = Some((steam_game_id, client));
        }
        {
            let mut single_client_state = self.single_client.lock().unwrap();
            *single_client_state = Some((steam_game_id, single_client));
        }
    }

    pub fn drop_all_clients(&self) {
        {
            let mut client_state = self.client.lock().unwrap();
            *client_state = None;
        }
        {
            let mut single_client_state = self.single_client.lock().unwrap();
            *single_client_state = None;
        }
    }
}
