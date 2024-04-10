use actix_web::{
    dev::Payload,
    web::{self, Data},
    FromRequest, HttpRequest, HttpResponse,
};
use futures::future::LocalBoxFuture;
use futures_util::FutureExt;
use serde_json::json;

use crate::{error::HandlerError, server::ServerState};

/// JSON data handler for PUT requests.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TrackData {
    /// What to record into the counter.
    pub trackable: String,
}

impl FromRequest for TrackData {
    type Error = HandlerError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let req = req.clone();
        let mut payload = payload.take();

        async move {
            let data: web::Json<Self> = web::Json::from_request(&req, &mut payload)
                .await
                .map_err(|e| HandlerError::internal(&e.to_string()))?;
            Ok(data.into_inner())
        }
        .boxed_local()
    }
}

/// Record the 'trackable' JSON field.
pub async fn track(state: Data<ServerState>, payload: TrackData) -> HttpResponse {
    let mut counter = match state
        .counter
        .lock()
        .map_err(|e| HandlerError::internal(&e.to_string()))
    {
        Ok(v) => v,
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    };
    let frequent = counter.track(&payload.trackable);

    HttpResponse::Ok().json(json!({
        "trackable": payload.trackable,
        "frequent": frequent,
    }))
}

/// Return a JSON report of the top items in the counter.
pub async fn report(state: Data<ServerState>) -> HttpResponse {
    let counter = state.counter.lock().unwrap();
    let report_size = state.report_size;
    let report = counter.clone().top(report_size as usize);

    HttpResponse::Ok().json(json!({
        "report": report,
    }))
}
