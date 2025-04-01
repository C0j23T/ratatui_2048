use std::io::Result;

use app::data::DummyDataManager;

mod app;

fn main() -> Result<()> {
    let data_app = Box::new(DummyDataManager);
    app::entry::run_app(data_app)?;
    Ok(())
}
