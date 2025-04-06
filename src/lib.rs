use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use app::{
    data::jni::{JniDataManager, Request, RequestBody, Response, ResponseBody},
    entry::leave,
};
use jni::{
    JNIEnv, JavaVM,
    objects::{JClass, JObject},
};

mod app;

fn start_thread(rx: Receiver<Request>, tx: Sender<Response>, vm: JavaVM) {
    thread::spawn(move || {
        let rx = rx;
        let tx = tx;
        let mut env = vm.attach_current_thread_as_daemon().unwrap();
        let player_service = env
            .find_class("com/smoother/TacticalGrid2048/GlobalVariables")
            .unwrap();
        let service: JObject<'_> = env
            .get_static_field(
                player_service,
                "playerService",
                "Lcom/smoother/TacticalGrid2048/service/PlayerService;",
            )
            .unwrap()
            .try_into()
            .unwrap();

        loop {
            let req = rx.recv().unwrap();
            let rsp = match req.0 {
                RequestBody::GetCurrentPlayer => {
                    let result = app::data::jni::get_current_player(&mut env, &service).unwrap();
                    ResponseBody::GetCurrentPlayer(result)
                }
                RequestBody::VerifyAccount(username, password) => {
                    let result =
                        app::data::jni::verify_account(&mut env, &service, username, password)
                            .unwrap();
                    ResponseBody::VerifyAccount(result)
                }
                RequestBody::RegisterAccount(username, password) => {
                    let result =
                        app::data::jni::register_account(&mut env, &service, username, password)
                            .unwrap();
                    ResponseBody::RegisterAccount(result)
                }
                RequestBody::GetPlayersBestExceptSelf => {
                    let result =
                        app::data::jni::get_players_best_except_self(&mut env, &service).unwrap();
                    ResponseBody::GetPlayersBestExceptSelf(result)
                }
                RequestBody::GetPlayers => {
                    let result =
                        app::data::jni::get_players(&mut env, &service).unwrap();
                    ResponseBody::GetPlayers(result)
                }
                RequestBody::SaveRecord(player) => {
                    let result =
                        app::data::jni::save_record(&mut env, &service, player).unwrap();
                    ResponseBody::SaveRecord(result)
                }
                RequestBody::FindPlayer(player) => {
                    let result = app::data::jni::find_player(&mut env, &service, player).unwrap();
                    ResponseBody::FindPlayer(result)
                }
                RequestBody::UpdatePlayer(player) => {
                    let result = app::data::jni::update_player(&mut env, &service, player).unwrap();
                    ResponseBody::UpdatePlayer(result)
                }
                RequestBody::RemovePlayer(player) => {
                    let result = app::data::jni::remove_player(&mut env, &service, player).unwrap();
                    ResponseBody::RemovePlayer(result)
                }
                RequestBody::Exit => break,
            };
            tx.send((rsp, req.1)).unwrap();
        }
    });
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_smoother_TacticalGrid2048_view_View_startAndJoin<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
) {
    let is_first_launch = {
        let utils = env
            .find_class("com/smoother/TacticalGrid2048/GlobalVariables")
            .unwrap();
        env.get_static_field(utils, "isFirstLaunch", "Z").unwrap().z().unwrap()
    };
    let vm = env.get_java_vm().unwrap();

    let (req_tx, req_rx) = mpsc::channel::<Request>();
    let (rsp_tx, rsp_rx) = mpsc::channel::<Response>();

    std::panic::set_hook(Box::new(|panic_info| {
        let _ = leave();
        let s: Option<&str> = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            Some(s)
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            Some(s)
        } else {
            None
        };
        println!("===== FRONTEND PANIC =====");
        println!("===== 前端  ::  故障 =====");
        println!("Message: {s:?}");
    }));

    start_thread(req_rx, rsp_tx, vm);

    let data_manager = Box::new(JniDataManager::new(req_tx.clone(), rsp_rx, is_first_launch));
    if let Err(e) = app::entry::run_app(data_manager) {
        env.throw(("java/io/IOException", format!("{e:?}")))
            .unwrap();
        return;
    }
    if let Err(e) = req_tx.send((RequestBody::Exit, 0)) {
        env.throw(("java/lang/IllegalStateException", format!("{e:?}")))
            .unwrap();
    }
}
