#![allow(dead_code)]

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

#[derive(PartialEq, Eq)]
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
    Exit,
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
            Self::Exit => 9,
        }
    }
}

#[derive(PartialEq, Eq)]
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

    fn update_responses(&mut self) {
        while let Ok(resp) = self.rx.try_recv() {
            self.responses.push(resp);
        }
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
        $this.update_responses();
        $this.clear_expired();

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
                $this.requests.retain(|x| x.0.1 != seq);
                let ResponseBody::$rsp_variant(result) = resp else {
                    unreachable!()
                };
                return Ok(result);
            }
        }

        if $this.check_expired(request_ty) {
            return Err(TryRecvError::Timeout);
        }

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

fn parse_java_list<'local, T, F>(
    env: &mut JNIEnv<'local>,
    list: &JObject<'local>,
    parser: F,
) -> jni::errors::Result<Vec<T>>
where
    F: Fn(&mut JNIEnv<'local>, &JObject<'local>) -> jni::errors::Result<T>,
{
    if list.is_null() {
        return Ok(Vec::default());
    }
    let size = env.call_method(list, "size", "()I", &[])?.i()?;
    let mut result = Vec::new();
    for i in 0..size {
        let obj = env.call_method(
            list,
            "get",
            "(I)Ljava/lang/Object;",
            &[jni::objects::JValueGen::Int(i)],
        )?;
        result.push(parser(env, &obj.try_into()?)?);
    }
    Ok(result)
}

fn get_player_from_java<'local>(
    env: &mut JNIEnv<'local>,
    o: &JObject<'local>,
) -> jni::errors::Result<Option<Player>> {
    if o.is_null() {
        return Ok(None);
    }
    let clazz = env.find_class("com/smoother/TacticalGrid2048/entity/Player")?;
    let flag = env.is_instance_of(o, clazz)?;
    if !flag {
        return Ok(None);
    }

    let id = env.get_field(o, "id", "I")?.i()?;
    let name = {
        let o: JObject<'_> = env.get_field(o, "name", "Ljava/lang/String;")?.try_into()?;
        if o.is_null() {
            String::new()
        } else {
            env.get_string(&o.into())?.into()
        }
    };

    let list = env.get_field(o, "records", "Ljava/util/List;")?;
    let records = if let JValueGen::Object(list) = list {
        parse_java_list(env, &list, |env, o| {
            let score = env.get_field(o, "score", "I")?.i()?;
            let time = env.get_field(o, "time", "J")?.j()?;
            let timestamp = env.get_field(o, "timestamp", "J")?.j()?;
            Ok(PlayerRecord {
                score,
                time,
                timestamp,
            })
        })?
    } else {
        Vec::default()
    };

    let (best_score, best_time, best_timestamp) = records
        .iter()
        .max_by_key(|x| x.score)
        .map(|x| (x.score, x.time, x.timestamp))
        .unwrap_or_default();

    Ok(Some(Player {
        id,
        name,
        best_score,
        best_time,
        best_timestamp,
        records,
    }))
}

fn new_player_record<'local>(
    env: &mut JNIEnv<'local>,
    id: i32,
    record: PlayerRecord,
) -> jni::errors::Result<JObject<'local>> {
    env.new_object(
        "com/smoother/TacticalGrid2048/entity/PlayerRecord",
        "(IIJJ)V",
        &[
            JValueGen::Int(id),
            JValueGen::Int(record.score),
            JValueGen::Long(record.time),
            JValueGen::Long(record.timestamp),
        ],
    )
}

fn new_player<'local>(
    env: &mut JNIEnv<'local>,
    player: Player,
) -> jni::errors::Result<JObject<'local>> {
    let array_list = env.new_object("java/util/ArrayList", "()V", &[])?;
    let it = player.records.into_iter();
    for x in it {
        let record = new_player_record(env, player.id, x)?;
        env.call_method(
            &array_list,
            "add",
            "(Ljava/lang/Object;)Z",
            &[JValueGen::Object(&record)],
        )?;
    }
    let name = env.new_string(player.name)?;
    env.new_object(
        "com/smoother/TacticalGrid2048/entity/Player",
        "(ILjava/lang/String;Ljava/util/List;)V",
        &[
            JValueGen::Int(player.id),
            JValueGen::Object(&name),
            JValueGen::Object(&array_list),
        ],
    )
}

pub fn get_current_player(
    env: &mut JNIEnv<'_>,
    service: &JObject<'_>,
) -> jni::errors::Result<Player> {
    let player: JObject<'_> = env
        .call_method(
            service,
            "getCurrentPlayer",
            "()Lcom/smoother/TacticalGrid2048/entity/Player;",
            &[],
        )?
        .try_into()?;
    get_player_from_java(env, &player).map(|x| x.unwrap_or_default())
}

pub fn get_players_best_except_self(
    env: &mut JNIEnv<'_>,
    service: &JObject<'_>,
) -> jni::errors::Result<Vec<Player>> {
    let player = env.call_method(
        service,
        "getPlayersBestExceptSelf",
        "()Ljava/util/List;",
        &[],
    )?;
    let JValueGen::Object(o) = player else {
        return Ok(Vec::default());
    };
    let list = parse_java_list(env, &o, |env, o| get_player_from_java(env, o))?;
    let mut result = Vec::new();
    list.into_iter().for_each(|x| {
        if let Some(x) = x {
            result.push(x);
        }
    });
    Ok(result)
}

pub fn get_players_except_self(
    env: &mut JNIEnv<'_>,
    service: &JObject<'_>,
) -> jni::errors::Result<Vec<Player>> {
    let player = env.call_method(service, "getPlayersExceptSelf", "()Ljava/util/List;", &[])?;
    let JValueGen::Object(o) = player else {
        return Ok(Vec::default());
    };
    let list = parse_java_list(env, &o, |env, o| get_player_from_java(env, o))?;
    let mut result = Vec::new();
    list.into_iter().for_each(|x| {
        if let Some(x) = x {
            result.push(x);
        }
    });
    Ok(result)
}

pub fn save_current_player(
    env: &mut JNIEnv<'_>,
    service: &JObject<'_>,
    player: Player,
) -> jni::errors::Result<bool> {
    let player = new_player(env, player)?;
    env.call_method(
        service,
        "saveCurrentPlayer",
        "(Lcom/smoother/TacticalGrid2048/entity/Player;)Z",
        &[JValueGen::Object(&player)],
    )?
    .z()
}

pub fn verify_account(
    env: &mut JNIEnv<'_>,
    service: &JObject<'_>,
    username: String,
    password: String,
) -> jni::errors::Result<Option<Player>> {
    let username = env.new_string(username)?;
    let password = env.new_string(password)?;
    let result: JObject<'_> = env
        .call_method(
            service,
            "verifyAccount",
            "(Ljava/lang/String;Ljava/lang/String;)Lcom/smoother/TacticalGrid2048/entity/Player;",
            &[JValueGen::Object(&username), JValueGen::Object(&password)],
        )?
        .try_into()?;
    get_player_from_java(env, &result)
}

pub fn register_account(
    env: &mut JNIEnv<'_>,
    service: &JObject<'_>,
    username: String,
    password: String,
) -> jni::errors::Result<Option<Player>> {
    let username = env.new_string(username)?;
    let password = env.new_string(password)?;
    let result: JObject<'_> = env
        .call_method(
            service,
            "registerAccount",
            "(Ljava/lang/String;Ljava/lang/String;)Lcom/smoother/TacticalGrid2048/entity/Player;",
            &[JValueGen::Object(&username), JValueGen::Object(&password)],
        )?
        .try_into()?;
    get_player_from_java(env, &result)
}

pub fn find_player(
    env: &mut JNIEnv<'_>,
    service: &JObject<'_>,
    player: Player,
) -> jni::errors::Result<Vec<Player>> {
    let player = new_player(env, player)?;
    let player = env.call_method(
        service,
        "findPlayer",
        "(Lcom/smoother/TacticalGrid2048/entity/Player;)Ljava/util/List;",
        &[JValueGen::Object(&player)],
    )?;
    let JValueGen::Object(o) = player else {
        return Ok(Vec::default());
    };
    let list = parse_java_list(env, &o, |env, o| get_player_from_java(env, o))?;
    let mut result = Vec::new();
    list.into_iter().for_each(|x| {
        if let Some(x) = x {
            result.push(x);
        }
    });
    Ok(result)
}

pub fn update_player(
    env: &mut JNIEnv<'_>,
    service: &JObject<'_>,
    player: Player,
) -> jni::errors::Result<bool> {
    let player = new_player(env, player)?;
    env.call_method(
        service,
        "updatePlayer",
        "(Lcom/smoother/TacticalGrid2048/entity/Player;)Z",
        &[JValueGen::Object(&player)],
    )?
    .z()
}

pub fn remove_player(
    env: &mut JNIEnv<'_>,
    service: &JObject<'_>,
    player: Player,
) -> jni::errors::Result<bool> {
    let player = new_player(env, player)?;
    env.call_method(
        service,
        "removePlayer",
        "(Lcom/smoother/TacticalGrid2048/entity/Player;)Z",
        &[JValueGen::Object(&player)],
    )?
    .z()
}
