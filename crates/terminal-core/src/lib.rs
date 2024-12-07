//! Hardware-independent terminal application state machine.

use serde::{Deserialize, Serialize};
use terminal_models::{Invoice, InvoiceId, Payment, PaymentStatus, Sats};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminalState {
    Booting,
    Provisioning,
    Connecting,
    Idle,
    EnteringAmount { amount: Option<Sats> },
    CreatingInvoice { amount: Sats },
    AwaitingPayment { invoice: Invoice },
    PaymentReceived { payment: Payment },
    Expired { invoice_id: InvoiceId },
    NetworkUnavailable { invoice: Option<Invoice> },
    Error { reason: String },
    Updating,
}

impl TerminalState {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Booting => "Booting",
            Self::Provisioning => "Provisioning",
            Self::Connecting => "Connecting",
            Self::Idle => "Idle",
            Self::EnteringAmount { .. } => "EnteringAmount",
            Self::CreatingInvoice { .. } => "CreatingInvoice",
            Self::AwaitingPayment { .. } => "AwaitingPayment",
            Self::PaymentReceived { .. } => "PaymentReceived",
            Self::Expired { .. } => "Expired",
            Self::NetworkUnavailable { .. } => "NetworkUnavailable",
            Self::Error { .. } => "Error",
            Self::Updating => "Updating",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppEvent {
    BootCompleted { provisioned: bool },
    ProvisioningCompleted,
    WifiConnected,
    WifiDisconnected,
    EnterAmountRequested,
    AmountChanged(Sats),
    AmountConfirmed(Sats),
    InvoiceCreated(Invoice),
    InvoiceCreationFailed { reason: String },
    PaymentReceived(Payment),
    InvoiceExpired(InvoiceId),
    RetryRequested,
    CancelRequested,
    UpdateStarted,
    UpdateFinished,
    ErrorOccurred { reason: String },
}

impl AppEvent {
    pub fn name(&self) -> &'static str {
        match self {
            Self::BootCompleted { .. } => "BootCompleted",
            Self::ProvisioningCompleted => "ProvisioningCompleted",
            Self::WifiConnected => "WifiConnected",
            Self::WifiDisconnected => "WifiDisconnected",
            Self::EnterAmountRequested => "EnterAmountRequested",
            Self::AmountChanged(_) => "AmountChanged",
            Self::AmountConfirmed(_) => "AmountConfirmed",
            Self::InvoiceCreated(_) => "InvoiceCreated",
            Self::InvoiceCreationFailed { .. } => "InvoiceCreationFailed",
            Self::PaymentReceived(_) => "PaymentReceived",
            Self::InvoiceExpired(_) => "InvoiceExpired",
            Self::RetryRequested => "RetryRequested",
            Self::CancelRequested => "CancelRequested",
            Self::UpdateStarted => "UpdateStarted",
            Self::UpdateFinished => "UpdateFinished",
            Self::ErrorOccurred { .. } => "ErrorOccurred",
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum StateMachineError {
    #[error("invalid transition from {state} with event {event}")]
    InvalidTransition {
        state: &'static str,
        event: &'static str,
    },

    #[error("payment event does not match the active invoice")]
    InvoiceMismatch,

    #[error("payment event is not a paid confirmation")]
    PaymentNotConfirmed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Terminal {
    state: TerminalState,
    current_invoice: Option<Invoice>,
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            state: TerminalState::Booting,
            current_invoice: None,
        }
    }

    pub fn state(&self) -> &TerminalState {
        &self.state
    }

    pub fn current_invoice(&self) -> Option<&Invoice> {
        self.current_invoice.as_ref()
    }

    pub fn apply(&mut self, event: AppEvent) -> Result<&TerminalState, StateMachineError> {
        let next = self.next_state(event)?;
        self.current_invoice = active_invoice(&next);
        self.state = next;
        Ok(&self.state)
    }

    fn next_state(&self, event: AppEvent) -> Result<TerminalState, StateMachineError> {
        match (&self.state, event) {
            (TerminalState::Booting, AppEvent::BootCompleted { provisioned }) => {
                if provisioned {
                    Ok(TerminalState::Connecting)
                } else {
                    Ok(TerminalState::Provisioning)
                }
            }
            (TerminalState::Provisioning, AppEvent::ProvisioningCompleted) => {
                Ok(TerminalState::Connecting)
            }
            (TerminalState::Connecting, AppEvent::WifiConnected)
            | (TerminalState::NetworkUnavailable { invoice: None }, AppEvent::WifiConnected) => {
                Ok(TerminalState::Idle)
            }
            (
                TerminalState::NetworkUnavailable {
                    invoice: Some(invoice),
                },
                AppEvent::WifiConnected,
            ) => Ok(TerminalState::AwaitingPayment {
                invoice: invoice.clone(),
            }),
            (TerminalState::Idle, AppEvent::EnterAmountRequested) => {
                Ok(TerminalState::EnteringAmount { amount: None })
            }
            (TerminalState::EnteringAmount { .. }, AppEvent::AmountChanged(amount)) => {
                Ok(TerminalState::EnteringAmount {
                    amount: Some(amount),
                })
            }
            (TerminalState::Idle, AppEvent::AmountConfirmed(amount))
            | (TerminalState::EnteringAmount { .. }, AppEvent::AmountConfirmed(amount)) => {
                Ok(TerminalState::CreatingInvoice { amount })
            }
            (TerminalState::CreatingInvoice { amount }, AppEvent::InvoiceCreated(invoice)) => {
                if invoice.amount == *amount {
                    Ok(TerminalState::AwaitingPayment { invoice })
                } else {
                    Err(StateMachineError::InvoiceMismatch)
                }
            }
            (TerminalState::CreatingInvoice { .. }, AppEvent::InvoiceCreationFailed { reason }) => {
                Ok(TerminalState::Error { reason })
            }
            (TerminalState::AwaitingPayment { invoice }, AppEvent::PaymentReceived(payment)) => {
                if payment.invoice_id != invoice.id {
                    return Err(StateMachineError::InvoiceMismatch);
                }

                if payment.status != PaymentStatus::Paid {
                    return Err(StateMachineError::PaymentNotConfirmed);
                }

                Ok(TerminalState::PaymentReceived { payment })
            }
            (
                TerminalState::PaymentReceived { payment: current },
                AppEvent::PaymentReceived(payment),
            ) if payment.invoice_id == current.invoice_id => Ok(self.state.clone()),
            (TerminalState::AwaitingPayment { invoice }, AppEvent::InvoiceExpired(invoice_id)) => {
                if invoice_id == invoice.id {
                    Ok(TerminalState::Expired { invoice_id })
                } else {
                    Err(StateMachineError::InvoiceMismatch)
                }
            }
            (_, AppEvent::WifiDisconnected) => Ok(TerminalState::NetworkUnavailable {
                invoice: self.current_invoice.clone(),
            }),
            (
                TerminalState::Idle
                | TerminalState::EnteringAmount { .. }
                | TerminalState::CreatingInvoice { .. }
                | TerminalState::AwaitingPayment { .. }
                | TerminalState::Expired { .. }
                | TerminalState::PaymentReceived { .. }
                | TerminalState::Error { .. },
                AppEvent::CancelRequested,
            )
            | (
                TerminalState::Expired { .. }
                | TerminalState::PaymentReceived { .. }
                | TerminalState::Error { .. },
                AppEvent::RetryRequested,
            ) => Ok(TerminalState::Idle),
            (_, AppEvent::UpdateStarted) => Ok(TerminalState::Updating),
            (TerminalState::Updating, AppEvent::UpdateFinished) => Ok(TerminalState::Connecting),
            (_, AppEvent::ErrorOccurred { reason }) => Ok(TerminalState::Error { reason }),
            (state, event) => Err(StateMachineError::InvalidTransition {
                state: state.name(),
                event: event.name(),
            }),
        }
    }
}

fn active_invoice(state: &TerminalState) -> Option<Invoice> {
    match state {
        TerminalState::AwaitingPayment { invoice }
        | TerminalState::NetworkUnavailable {
            invoice: Some(invoice),
        } => Some(invoice.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use terminal_models::{Bolt11, UnixTimestamp};

    fn invoice(amount: u64) -> Result<Invoice, Box<dyn std::error::Error>> {
        Ok(Invoice::new(
            InvoiceId::new("01JTEST")?,
            Sats::new(amount)?,
            Bolt11::new("lnbc10000n1ptest")?,
            UnixTimestamp::from_secs(1_786_383_000),
        ))
    }

    #[test]
    fn boot_routes_to_provisioning_when_not_configured() -> Result<(), Box<dyn std::error::Error>> {
        let mut terminal = Terminal::new();

        let state = terminal.apply(AppEvent::BootCompleted { provisioned: false })?;

        assert_eq!(state, &TerminalState::Provisioning);
        Ok(())
    }

    #[test]
    fn idle_amount_confirmation_starts_invoice_creation() -> Result<(), Box<dyn std::error::Error>>
    {
        let mut terminal = Terminal::new();
        terminal.apply(AppEvent::BootCompleted { provisioned: true })?;
        terminal.apply(AppEvent::WifiConnected)?;

        let state = terminal.apply(AppEvent::AmountConfirmed(Sats::new(10_000)?))?;

        assert_eq!(
            state,
            &TerminalState::CreatingInvoice {
                amount: Sats::new(10_000)?
            }
        );
        Ok(())
    }

    #[test]
    fn created_invoice_moves_terminal_to_awaiting_payment() -> Result<(), Box<dyn std::error::Error>>
    {
        let mut terminal = Terminal::new();
        let invoice = invoice(10_000)?;

        terminal.apply(AppEvent::BootCompleted { provisioned: true })?;
        terminal.apply(AppEvent::WifiConnected)?;
        terminal.apply(AppEvent::AmountConfirmed(Sats::new(10_000)?))?;
        let state = terminal.apply(AppEvent::InvoiceCreated(invoice.clone()))?;

        assert_eq!(state, &TerminalState::AwaitingPayment { invoice });
        Ok(())
    }

    #[test]
    fn paid_invoice_moves_terminal_to_payment_received() -> Result<(), Box<dyn std::error::Error>> {
        let mut terminal = Terminal::new();
        let invoice = invoice(10_000)?;
        let payment = Payment::received(
            invoice.id.clone(),
            invoice.amount,
            UnixTimestamp::from_secs(1_786_383_010),
        );

        terminal.apply(AppEvent::BootCompleted { provisioned: true })?;
        terminal.apply(AppEvent::WifiConnected)?;
        terminal.apply(AppEvent::AmountConfirmed(Sats::new(10_000)?))?;
        terminal.apply(AppEvent::InvoiceCreated(invoice))?;
        let state = terminal.apply(AppEvent::PaymentReceived(payment.clone()))?;

        assert_eq!(state, &TerminalState::PaymentReceived { payment });
        Ok(())
    }

    #[test]
    fn duplicate_payment_event_is_idempotent() -> Result<(), Box<dyn std::error::Error>> {
        let mut terminal = Terminal::new();
        let invoice = invoice(10_000)?;
        let payment = Payment::received(
            invoice.id.clone(),
            invoice.amount,
            UnixTimestamp::from_secs(1_786_383_010),
        );

        terminal.apply(AppEvent::BootCompleted { provisioned: true })?;
        terminal.apply(AppEvent::WifiConnected)?;
        terminal.apply(AppEvent::AmountConfirmed(Sats::new(10_000)?))?;
        terminal.apply(AppEvent::InvoiceCreated(invoice))?;
        terminal.apply(AppEvent::PaymentReceived(payment.clone()))?;
        let state = terminal.apply(AppEvent::PaymentReceived(payment.clone()))?;

        assert_eq!(state, &TerminalState::PaymentReceived { payment });
        Ok(())
    }

    #[test]
    fn wifi_loss_during_payment_preserves_active_invoice() -> Result<(), Box<dyn std::error::Error>>
    {
        let mut terminal = Terminal::new();
        let invoice = invoice(10_000)?;

        terminal.apply(AppEvent::BootCompleted { provisioned: true })?;
        terminal.apply(AppEvent::WifiConnected)?;
        terminal.apply(AppEvent::AmountConfirmed(Sats::new(10_000)?))?;
        terminal.apply(AppEvent::InvoiceCreated(invoice.clone()))?;
        let state = terminal.apply(AppEvent::WifiDisconnected)?;

        assert_eq!(
            state,
            &TerminalState::NetworkUnavailable {
                invoice: Some(invoice)
            }
        );
        Ok(())
    }

    #[test]
    fn expired_invoice_moves_terminal_to_expired() -> Result<(), Box<dyn std::error::Error>> {
        let mut terminal = Terminal::new();
        let invoice = invoice(10_000)?;
        let invoice_id = invoice.id.clone();

        terminal.apply(AppEvent::BootCompleted { provisioned: true })?;
        terminal.apply(AppEvent::WifiConnected)?;
        terminal.apply(AppEvent::AmountConfirmed(Sats::new(10_000)?))?;
        terminal.apply(AppEvent::InvoiceCreated(invoice))?;
        let state = terminal.apply(AppEvent::InvoiceExpired(invoice_id.clone()))?;

        assert_eq!(state, &TerminalState::Expired { invoice_id });
        Ok(())
    }
}
