use std::sync::Arc;
use configcat::Client;

#[derive(Clone)]
pub struct AppState {
    pub configcat: Arc<Option<Client>>,
}