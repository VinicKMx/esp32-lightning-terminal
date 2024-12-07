use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use terminal_core::TerminalState;
use terminal_models::{Invoice, Sats};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Boot,
    Provisioning,
    Home,
    EnterAmount { amount: Option<Sats> },
    CreatingInvoice,
    Invoice { invoice: Invoice },
    PaymentReceived { amount: Sats },
    Expired,
    NetworkError,
    SystemError,
    Updating,
}

impl Screen {
    pub fn from_terminal_state(state: &TerminalState) -> Self {
        match state {
            TerminalState::Booting => Self::Boot,
            TerminalState::Provisioning => Self::Provisioning,
            TerminalState::Connecting | TerminalState::Idle => Self::Home,
            TerminalState::EnteringAmount { amount } => Self::EnterAmount { amount: *amount },
            TerminalState::CreatingInvoice { .. } => Self::CreatingInvoice,
            TerminalState::AwaitingPayment { invoice } => Self::Invoice {
                invoice: invoice.clone(),
            },
            TerminalState::PaymentReceived { payment } => Self::PaymentReceived {
                amount: payment.amount,
            },
            TerminalState::Expired { .. } => Self::Expired,
            TerminalState::NetworkUnavailable { .. } => Self::NetworkError,
            TerminalState::Error { .. } => Self::SystemError,
            TerminalState::Updating => Self::Updating,
        }
    }
}

#[allow(dead_code)]
pub trait TerminalUi {
    type Error;

    fn render(&mut self, screen: &Screen) -> Result<(), Self::Error>;
}

#[allow(dead_code)]
pub fn draw_status_text<D>(display: &mut D, screen: &Screen) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let title = match screen {
        Screen::Boot => "Bitcoin Lightning Terminal",
        Screen::Provisioning => "Provisioning",
        Screen::Home => "Ready",
        Screen::EnterAmount { .. } => "Amount",
        Screen::CreatingInvoice => "Creating invoice",
        Screen::Invoice { .. } => "Waiting for payment",
        Screen::PaymentReceived { .. } => "Payment received",
        Screen::Expired => "Invoice expired",
        Screen::NetworkError => "Network unavailable",
        Screen::SystemError => "System error",
        Screen::Updating => "Updating firmware",
    };

    Text::new(title, Point::new(0, 12), style).draw(display)?;
    Ok(())
}
