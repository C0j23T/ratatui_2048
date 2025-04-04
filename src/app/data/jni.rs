use std::{
    sync::mpsc::{Receiver, Sender},
    time::{Duration, Instant},
};

use jni::{
    JNIEnv,
    objects::{JObject, JValueGen},
};

use crate::app::structs::{Player, PlayerRecord};

use super::{DataManager, TryRecvError};

#[derive(Clone, PartialEq, Eq)]
pub enum RequestBody {
    GetCurrentPlayer,
    GetPlayersBestExceptSelf,
    GetPlayersExceptSelf,
    SaveCurrentPlayer(Player),
    VerifyAccount(String, String),
    RegisterAccount(String, String),
    FindPlayer(Player),
    UpdatePlayer(Player),
    RemovePlayer(Player),
}

impl RequestBody {
    fn to_ty(&self) -> i32 {
        match self {
            Self::GetCurrentPlayer => 0,
            Self::GetPlayersBestExceptSelf => 1,
            Self::GetPlayersExceptSelf => 2,
            Self::SaveCurrentPlayer { .. } => 3,
            Self::VerifyAccount { .. } => 4,
            Self::RegisterAccount { .. } => 5,
            Self::FindPlayer { .. } => 6,
            Self::UpdatePlayer { .. } => 7,
            Self::RemovePlayer { .. } => 8,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum ResponseBody {
    GetCurrentPlayer(Player),
    GetPlayersBestExceptSelf(Vec<Player>),
    GetPlayersExceptSelf(Vec<Player>),
    SaveCurrentPlayer(bool),
    VerifyAccount(Option<Player>),
    RegisterAccount(Option<Player>),
    FindPlayer(Vec<Player>),
    UpdatePlayer(bool),
    RemovePlayer(bool),
}

pub type Request = (RequestBody, usize);

pub type Response = (ResponseBody, usize);

pub struct JniDataManager {
    tx: Sender<Request>,
    rx: Receiver<Response>,
    is_first_launch: bool,
    responses: Vec<Response>,
    requests: Vec<((i32, usize), Instant)>,
    i: usize,
}

impl JniDataManager {
    pub fn new(tx: Sender<Request>, rx: Receiver<Response>, is_first_launch: bool) -> Self {
        Self {
            tx,
            rx,
            is_first_launch,
            responses: Vec::default(),
            requests: Vec::default(),
            i: 0,
        }
    }

    fn check_expired(&mut self, req_ty: i32) -> bool {
        let timeout = Duration::from_secs(3);
        let now = Instant::now();
        self.requests
            .iter()
            .any(|x| x.0.0 == req_ty && now - x.1 > timeout)
    }

    fn next_seq(&mut self) -> usize {
        let seq = self.i;
        self.i += 1;
        seq
    }

    fn clear_expired(&mut self) {
        let now = Instant::now();
        let timeout = Duration::from_secs(4);
        let expired = self
            .requests
            .iter()
            .filter(|x| now - x.1 > timeout)
            .map(|x| x.0.1)
            .collect::<Vec<_>>();
        self.requests.retain(|x| !expired.contains(&x.0.1));
        self.responses.retain(|x| !expired.contains(&x.1));
    }
}

macro_rules! impl_request_response {
    ($this:ident, $req_variant:ident, $rsp_variant:ident) => {
        let request = RequestBody::$req_variant;
        impl_request_response!($this, request, $rsp_variant,);
    };
    ($this:ident, $req_variant:ident($($args:expr),*), $rsp_variant:ident) => {
        let request = RequestBody::$req_variant($($args),*);
        impl_request_response!($this, request, $rsp_variant,);
    };
    ($this:ident, $request:ident, $rsp_variant:ident,) => {
        // This comma 🤯
        let request_ty = $request.to_ty();
        let has_pending = $this.requests.iter().any(|((r, _), _)| *r == request_ty);

        if !has_pending {
            let seq = $this.next_seq();
            if $this.tx.send(($request, seq)).is_err() {
                return Err(TryRecvError::Disconnected);
            } else {
                $this.requests.push(((request_ty, seq), Instant::now()));
                return Err(TryRecvError::Empty);
            }
        }

        let r = $this
            .responses
            .iter()
            .position(|(r, _)| matches!(r, ResponseBody::$rsp_variant { .. }))
            .and_then(|p| Some((p, $this.responses[p].1)));

        if let Some((index, seq)) = r {
            if $this.requests.iter().any(|((_, id), _)| *id == seq) {
                let (resp, _) = $this.responses.remove(index);
                let ResponseBody::$rsp_variant(result) = resp else {
                    unreachable!()
                };
                return Ok(result);
            }
        }

        if $this.check_expired(request_ty) {
            return Err(TryRecvError::Timeout);
        }
        $this.clear_expired();

        return Err(TryRecvError::Empty);
    }
}

impl DataManager for JniDataManager {
    fn is_first_launch(&mut self) -> bool {
        self.is_first_launch
    }

    fn get_current_player(&mut self) -> Result<Player, TryRecvError> {
        impl_request_response!(self, GetCurrentPlayer, GetCurrentPlayer);
    }

    fn get_players_best_except_self(&mut self) -> Result<Vec<Player>, TryRecvError> {
        impl_request_response!(self, GetPlayersBestExceptSelf, GetPlayersBestExceptSelf);
    }

    fn get_players_except_self(&mut self) -> Result<Vec<Player>, TryRecvError> {
        impl_request_response!(self, GetPlayersExceptSelf, GetPlayersExceptSelf);
    }

    fn save_current_player(&mut self, player: Player) -> Result<bool, TryRecvError> {
        impl_request_response!(self, SaveCurrentPlayer(player), SaveCurrentPlayer);
    }

    fn verify_account(
        &mut self,
        username: String,
        password: String,
    ) -> Result<Option<Player>, TryRecvError> {
        impl_request_response!(self, VerifyAccount(username, password), VerifyAccount);
    }

    fn register_account(
        &mut self,
        username: String,
        password: String,
    ) -> Result<Option<Player>, TryRecvError> {
        impl_request_response!(self, RegisterAccount(username, password), RegisterAccount);
    }

    fn find_player(&mut self, player: Player) -> Result<Vec<Player>, TryRecvError> {
        impl_request_response!(self, FindPlayer(player), FindPlayer);
    }

    fn update_player(&mut self, player: Player) -> Result<bool, TryRecvError> {
        impl_request_response!(self, UpdatePlayer(player), UpdatePlayer);
    }

    fn remove_player(&mut self, player: Player) -> Result<bool, TryRecvError> {
        impl_request_response!(self, RemovePlayer(player), RemovePlayer);
    }
}

fn get_player_from_java<'local>(
    env: &mut JNIEnv<'local>,
    o: JObject<'local>,
) -> jni::errors::Result<Option<Player>> {
    let clazz = env
        .find_class("com/smoother/TacticalGrid2048/entity/Player")
        .unwrap();
    let flag = env
        .is_instance_of(env.get_object_class(&o).unwrap(), clazz)
        .unwrap();
    if !flag {
        return Ok(None);
    }

    let id = env.get_field(&o, "id", "I")?.i()?;
    let name = env.get_field(&o, "name", "Ljava/lang/String;")?;
    let best_score = env.get_field(&o, "score", "I")?.i()?;
    let best_time = env.get_field(&o, "time", "J")?.j()?;
    let best_timestamp = env.get_field(&o, "timestamp", "J")?.j()?;
    let name: String = if let JValueGen::Object(o) = name {
        if o.is_null() {
            String::new()
        } else {
            env.get_string((&o).into())?.into()
        }
    } else {
        return Ok(None);
    };

    let j_records = env.get_field(o, "records", "Ljava/util/List;")?;
    let JValueGen::Object(j_records) = j_records else {
        return Ok(None);
    };
    let mut records = Vec::new();
    let size = env
        .call_method(&j_records, "size", "()I", &[])
        .unwrap()
        .i()
        .unwrap();
    for i in 0..size {
        let record = env
            .call_method(
                &j_records,
                "get",
                "(I)Ljava/lang/Object;",
                &[jni::objects::JValueGen::Int(i)],
            )
            .unwrap();
        let JValueGen::Object(o) = record else {
            continue;
        };
        let score = env.get_field(&o, "score", "I")?.i()?;
        let time = env.get_field(&o, "time", "J")?.j()?;
        let timestamp = env.get_field(&o, "timestamp", "J")?.j()?;
        records.push(PlayerRecord {
            score,
            time,
            timestamp,
        });
    }

    Ok(Some(Player {
        id,
        name,
        best_score,
        best_time,
        best_timestamp,
        records,
    }))
}

pub fn get_current_player(
    env: &mut JNIEnv<'_>,
    service: &JObject<'_>,
) -> jni::errors::Result<Player> {
    let player = env
        .call_method(
            service,
            "getCurrentPlayer",
            "()Lcom/smoother/TacticalGrid2048/entity/Player;",
            &[],
        )
        .unwrap();
    let JValueGen::Object(o) = player else {
        return Ok(Player::default());
    };
    get_player_from_java(env, o).map(|x| x.unwrap_or_default())
}
