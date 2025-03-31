use std::io::Result;

use app::data::DummyDataManager;

mod app;

fn main() -> Result<()> {
    let data_app = DummyDataManager;
    app::entry::start_app(data_app)?;
    Ok(())
}
