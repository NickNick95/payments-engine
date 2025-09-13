use crate::errors::AppResult;
use crate::state::AppState;

pub trait TxCommandTrait {
    fn execute(&self, app_state: &mut AppState) -> AppResult<()>;
}
