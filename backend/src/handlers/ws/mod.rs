pub mod manager;
pub mod session;

use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::MessageStream;
use tokio::sync::mpsc;
use crate::domain::UserId;
use crate::handlers::ws::manager::ConnectionManager;
use crate::handlers::ws::session::ws_session_loop;

// ruta upgrade a websocket
pub async fn ws_upgrade_handler(
    req: HttpRequest,
    body: web::Payload,
    manager: web::Data<ConnectionManager>,
    // el uuid viene de path param para pruebas auth
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    
    // extrae id de ruta simulada
    let user_id_str = path.into_inner();
    let user_uuid = match uuid::Uuid::parse_str(&user_id_str) {
        Ok(u) => u,
        Err(_) => return Ok(HttpResponse::BadRequest().body("UUID Invalido")),
    };
    let user_id = UserId(user_uuid);
    let (response, session, msg_stream) = actix_ws::handle(&req, body)?;

    // buzón interno para la sesión
    let (tx, rx) = mpsc::unbounded_channel();

    // se enlaza al manager global
    let manager_clone = manager.get_ref().clone(); // el manager envuelve un arc cost-free
    manager_clone.add_client(user_id, tx);

    // aquise aisla la tarea en el executor actix local (!send)
    actix_web::rt::spawn(async move {
        ws_session_loop(user_id, session, msg_stream, manager_clone, rx).await;
    });

    Ok(response)
}