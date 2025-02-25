use crate::controller::proxy::routes::ProxyRoutes;
use crate::state::AppState;
use axum::routing::Router;
use derust::httpx::AppContext;

pub struct Routes;

impl Routes {
    pub async fn routes() -> Router<AppContext<AppState>> {
        Router::new()
            // .nest("/config-cat", ConfigCatRoutes::routes())
            .merge(ProxyRoutes::routes())
    }
}