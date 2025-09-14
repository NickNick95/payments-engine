use crate::errors::AppResult;
use crate::state::AppState;

/// A trait that defines the behavior of transaction commands in the application.
/// Implementors of this trait can execute specific transaction commands,
/// modifying the application state as needed.
pub trait TxCommandTrait {
    /// Executes the transaction command and updates the application state.
    ///
    /// # Arguments
    /// * `app_state` - A mutable reference to the application state.
    ///
    /// # Returns
    /// * `AppResult<()>` - Returns `Ok(())` if the command is successfully executed,
    ///   or an `AppErrors` variant if an error occurs.
    fn execute(&self, app_state: &mut AppState) -> AppResult<()>;
}
