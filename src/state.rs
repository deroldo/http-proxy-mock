use configcat::Client;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub configcat: Arc<Option<Client>>,
}
