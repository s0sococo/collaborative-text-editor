mod ui;
mod backend_api;

use create::ui::AppView;
use create::backend_api::{DocBackend, MockStringBackend};
use eframe::{egui, NativeOptions};
 
 fn main()
