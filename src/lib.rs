use std::{sync::mpsc, thread};

use app::{
    data::jni::{JniDataManager, Request, RequestBody, Response, ResponseBody},
    entry::leave,
};
use jni::{
    JNIEnv,
    objects::{JClass, JValueGen},
};

mod app;

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_smoother_TacticalGrid2048_view_View_startAndJoin<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
) {
    let is_first_launch = {
        let utils = env
            .find_class("com/smoother/TacticalGrid2048/Utils")
            .unwrap();
        env.call_static_method(utils, "isFirstLaunch", "()Z", &[])
            .unwrap()
            .z()
            .unwrap()
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

    thread::spawn(move || {
        let rx = req_rx;
        let tx = rsp_tx;
        let mut env = vm.attach_current_thread().unwrap();
        let player_service = env
            .find_class("com/smoother/TacticalGrid2048/GlobalVariants")
            .unwrap();
        let service = env
            .get_static_field(
                player_service,
                "playerService",
                "Lcom/smoother/TacticalGrid2048/service/PlayerService;",
            )
            .unwrap();
        let JValueGen::Object(service) = service else {
            panic!("Type of PlayerService is incorrect")
        };

        loop {
            let req = rx.recv().unwrap();
            let rsp: ResponseBody;
            match req.0 {
                RequestBody::GetCurrentPlayer => {
                    let result = app::data::jni::get_current_player(&mut env, &service).unwrap();
                    rsp = ResponseBody::GetCurrentPlayer(result);
                }
                _ => todo!(),
            }
            tx.send((rsp, req.1)).unwrap();
        }
    });

    let data_manager = Box::new(JniDataManager::new(req_tx, rsp_rx, is_first_launch));
    if let Err(e) = app::entry::run_app(data_manager) {
        env.throw(("java/io/IOException", format!("{e:?}")))
            .unwrap();
        return;
    }
}
