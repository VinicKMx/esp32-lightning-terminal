mod ui;

use esp_idf_hal::delay::FreeRtos;
use terminal_core::{AppEvent, Terminal};
use ui::Screen;

fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("booting ESP32 Lightning terminal firmware");

    let mut terminal = Terminal::new();
    if let Err(error) = terminal.apply(AppEvent::BootCompleted { provisioned: false }) {
        log::error!("terminal state transition failed: {error}");
    }

    let screen = Screen::from_terminal_state(terminal.state());
    log::info!("initial screen: {:?}", screen);

    loop {
        FreeRtos::delay_ms(1_000);
    }
}
