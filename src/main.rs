use std::{collections::HashMap, sync::{Mutex, Arc}};

use tonic::{transport::Server, Request, Response, Status};

use pool::{*, pool_game_server::{PoolGame, PoolGameServer}};

use rand::{distributions::Alphanumeric, Rng};

pub mod pool {
    tonic::include_proto!("pool"); // The string specified here must match the proto package name
}

const DEFAULT_PLAYER_NAME: &str = "Guest";
const DEFAULT_ROOM_NAME: &str = "Game Panda Pool";

#[derive(Debug, Default)]
struct PoolServer {
    pub rooms: Arc<Mutex<HashMap<String, ServerRoom>>>,
}

#[derive(Debug)]
struct ServerRoom {
    players: Vec<ServerPlayer>,
    room_id: String,
    room_name: String,
    game_started: bool,
    game_state: Option<GameState>,
    previous_turn: Option<Turn>,
}

impl ServerRoom {
    fn new(room_id: String) -> Self {
        Self {
            players: Vec::new(),
            room_id,
            room_name: DEFAULT_ROOM_NAME.to_string(),
            game_started: false,
            game_state: None,
            previous_turn: None,
        }
    }
}

#[derive(Debug, Default)]
struct ServerPlayer {
    pub token: String, 
    pub player: Player,
}

impl ServerPlayer {
    fn new(player_id: i32) -> Self {
        let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
        let mut player = Player::default();
        player.name = DEFAULT_PLAYER_NAME.to_string();
        player.player_id = player_id;
        Self {
            token,
            player,
        }
    }
}

#[tonic::async_trait]
impl PoolGame for PoolServer {
    async fn create_room(&self, _req: Request<()>) -> Result<Response<RoomCreation>, Status> {
        let room_code: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();
        let mut room = ServerRoom::new(room_code.clone());
        let player = ServerPlayer::new(0);
        let player_token = player.token.clone();
        room.players.push(player);

        let mut rooms_look = self.rooms.lock();
        let rooms_ref = rooms_look.as_mut().unwrap();
        rooms_ref.insert(room_code.clone(), room);

        return Ok(Response::new(RoomCreation {room_id: Some(RoomId {room_code}), player_token: Some(PlayerToken { player_token })}));
    }
    async fn join_room(&self, req: Request<RoomId>) -> Result<Response<RoomJoin>, Status> {
        let mut rooms_look = self.rooms.lock();
        let rooms_ref = rooms_look.as_mut().unwrap();

        if rooms_ref.contains_key(&req.get_ref().room_code) {
            let room = rooms_ref.get_mut(&req.get_ref().room_code).unwrap();
            if room.players.len() < 2 {
                let player = ServerPlayer::new(1);
                let player_token = player.token.clone();
                room.players.push(player);

                return Ok(Response::new(RoomJoin {player_token: Some(PlayerToken { player_token })}));
            } else {
                return Err(Status::permission_denied("room full"));
            }
        } else {
            return Err(Status::invalid_argument("room does not exist"));
        }
    }
    async fn get_room(&self, req: Request<RoomId>) -> Result<Response<Room>, Status> {
        let rooms_look = self.rooms.lock();
        let rooms_ref = rooms_look.unwrap();

        if rooms_ref.contains_key(&req.get_ref().room_code) {
            let room = rooms_ref.get(&req.get_ref().room_code).unwrap();
            let mut players = Vec::new();
            for server_player in &room.players {
                players.push(server_player.player.clone());
            }
            return Ok(Response::new(Room {room_id: Some(RoomId { room_code: room.room_id.clone() }), room_name: room.room_name.clone(), game_started: room.game_started, players}));
        } else {
            return Err(Status::invalid_argument("room does not exist"));
        }
    }
    async fn set_player_info(&self, req: Request<SetPlayerInfoReq>) -> Result<Response<()>, Status> {
        let mut rooms_look = self.rooms.lock();
        let rooms_ref = rooms_look.as_mut().unwrap();

        if req.get_ref().room_id.is_some() && req.get_ref().authed_player.is_some() {
            if rooms_ref.contains_key(&req.get_ref().room_id.as_ref().unwrap().room_code) {
                let room = rooms_ref.get_mut(&req.get_ref().room_id.as_ref().unwrap().room_code).unwrap();
                let req_pid = req.get_ref().authed_player.as_ref().unwrap().player_id;
                if req_pid != 0 || req_pid != 1 {
                    return Err(Status::invalid_argument("player id is out of bounds"));
                }
                if req.get_ref().authed_player.as_ref().unwrap().player_token.is_none() {
                    return Err(Status::invalid_argument("authed_player.player_token is null"));
                }
                if room.players[req_pid as usize].token == req.get_ref().authed_player.as_ref().unwrap().player_token.as_ref().unwrap().player_token {
                    room.players[req_pid as usize].player.name = req.get_ref().authed_player.as_ref().unwrap().name.clone();
                    return Ok(Response::new(()));
                } else {
                    return Err(Status::permission_denied("you are not authorized as this player"));
                }
            } else {
                return Err(Status::invalid_argument("room does not exist"));
            }
        } else {
            return Err(Status::invalid_argument("field(s) is null"));
        }
    }
    async fn start_game(&self, req: Request<AuthedPlayer>) -> Result<Response<()>, Status> {
        todo!();
    }
    async fn get_game_state(&self, req: Request<RoomId>) -> Result<Response<GameState>, Status> {
        let mut rooms_look = self.rooms.lock();
        let rooms_ref = rooms_look.as_mut().unwrap();

        if rooms_ref.contains_key(&req.get_ref().room_code) {
            let room = rooms_ref.get_mut(&req.get_ref().room_code).unwrap();
            if room.game_started && room.game_state.is_some() {
                let game_state = room.game_state.as_ref().unwrap();
                return Ok(Response::new(GameState { balls: game_state.balls.clone(), scores: game_state.scores.clone(), p1_solid: game_state.p1_solid, p1_turn: game_state.p1_turn }))
            } else {
                return Err(Status::unavailable("game is not started yet"));
            }
        } else {
            return Err(Status::invalid_argument("room does not exist"));
        }
    }
    async fn post_turn(&self, req: Request<Turn>) -> Result<Response<()>, Status> {
        todo!();
    }
    async fn check_win_state(&self, req: Request<RoomId>) -> Result<Response<WinState>, Status> {
        let mut rooms_look = self.rooms.lock();
        let rooms_ref = rooms_look.as_mut().unwrap();

        if rooms_ref.contains_key(&req.get_ref().room_code) {
            let room = rooms_ref.get_mut(&req.get_ref().room_code).unwrap();
            if room.game_started && room.game_state.is_some() {
                let game_state = room.game_state.as_ref().unwrap();
                let mut in_play = Vec::new();
                let mut win_state = 0;
                for ball in &game_state.balls {
                    in_play.push(ball.in_play);
                }
                if !in_play[0..6].contains(&true) {
                    win_state = game_state.p1_solid;
                }
                if !in_play[8..14].contains(&true) {
                    win_state = !(game_state.p1_solid != 1) as i32 + 1;
                }
                return Ok(Response::new(WinState { win_state }));
            } else {
                return Err(Status::unavailable("game is not started yet"));
            }
        } else {
            return Err(Status::invalid_argument("room does not exist"));
        }
    }
    async fn get_previous_turn(&self, req: Request<RoomId>) -> Result<Response<Turn>, Status> {
        let mut rooms_look = self.rooms.lock();
        let rooms_ref = rooms_look.as_mut().unwrap();

        if rooms_ref.contains_key(&req.get_ref().room_code) {
            let room = rooms_ref.get_mut(&req.get_ref().room_code).unwrap();
            if room.previous_turn.is_some() {
                return Ok(Response::new(room.previous_turn.as_ref().unwrap().clone()));
            } else {
                return Err(Status::unavailable("no previous turn"));
            }
        } else {
            return Err(Status::invalid_argument("room does not exist"));
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:8080".parse()?;
    let greeter = PoolServer::default();

    Server::builder()
        .add_service(PoolGameServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
