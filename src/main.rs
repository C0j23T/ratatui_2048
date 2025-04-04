use std::io::Result;

use app::{data::dummy::DummyDataManager, entry::leave};

mod app;

fn main() -> Result<()> {
    std::panic::set_hook(Box::new(|panic_info| {
        let _ = leave();
        println!("😱😱😱😱😱微距了😱😱😱😱😱");
        println!("{panic_info}");
        println!("😱😱😱😱😱😱😱😱😱😱😱😱😱");
    }));

    let data_app = Box::new(DummyDataManager);
    app::entry::run_app(data_app)?;
    Ok(())
}
