/// Copyright (C) 2023 Bryan A. Jones.
///
/// This file is part of the CodeChat Editor. The CodeChat Editor is free
/// software: you can redistribute it and/or modify it under the terms of the
/// GNU General Public License as published by the Free Software Foundation,
/// either version 3 of the License, or (at your option) any later version.
///
/// The CodeChat Editor is distributed in the hope that it will be useful, but
/// WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
/// or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for
/// more details.
///
/// You should have received a copy of the GNU General Public License along with
/// the CodeChat Editor. If not, see
/// [http://www.gnu.org/licenses](http://www.gnu.org/licenses).
///
/// # `vscode.rs` -- Implement server-side functionality for the Visual Studio Code IDE
// ## Imports
//
// ### Standard library
//
// None.
//
// ### Third-party
use actix_web::{
    error::{Error, ErrorBadRequest},
    get, web, HttpRequest, HttpResponse,
};
use log::error;
use open;
use tokio::sync::mpsc;

// ### Local
use super::{
    client_websocket, send_response, AppState, EditorMessage, EditorMessageContents, IdeType,
    WebsocketQueues,
};

use crate::queue_send;

// ## Code
#[get("/vsc/ws-ide/{connection_id}")]
pub async fn vscode_ide_websocket(
    connection_id: web::Path<String>,
    req: HttpRequest,
    body: web::Payload,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let connection_id_str = connection_id.to_string();

    // There are three cases for this `connection_id`:
    //
    // 1. It hasn't been used before. In this case, create the appropriate queues and start websocket and processing tasks.
    // 2. It's in use, but was disconnected. In this case, re-use the queues and start the websocket task; the processing task is still running.
    // 3. It's in use by another IDE. This is an error, but I don't have a way to detect it yet.
    //
    // Check case 3.
    if app_state
        .vscode_connection_id
        .lock()
        .unwrap()
        .contains(&connection_id_str)
    {
        let msg = format!("Connection ID {connection_id_str} already in use.");
        error!("{msg}");
        return Err(ErrorBadRequest(msg));
    }

    // Now case 2.
    if app_state
        .vscode_ide_queues
        .lock()
        .unwrap()
        .contains_key(&connection_id_str)
    {
        return client_websocket(
            connection_id,
            req,
            body,
            app_state.vscode_ide_queues.clone(),
        )
        .await;
    }

    // Then this is case 1. Add the connection ID to the list of active connections.
    let (from_ide_tx, mut from_ide_rx) = mpsc::channel(10);
    let (to_ide_tx, to_ide_rx) = mpsc::channel(10);
    assert!(app_state
        .vscode_ide_queues
        .lock()
        .unwrap()
        .insert(
            connection_id_str.clone(),
            WebsocketQueues {
                from_websocket_tx: from_ide_tx,
                to_websocket_rx: to_ide_rx,
            },
        )
        .is_none());
    let (from_client_tx, mut from_client_rx) = mpsc::channel(10);
    let (to_client_tx, to_client_rx) = mpsc::channel(10);
    assert!(app_state
        .vscode_client_queues
        .lock()
        .unwrap()
        .insert(
            connection_id_str,
            WebsocketQueues {
                from_websocket_tx: from_client_tx,
                to_websocket_rx: to_client_rx,
            },
        )
        .is_none());

    actix_rt::spawn(async move {
        // Use a
        // [labeled block expression](https://doc.rust-lang.org/reference/expressions/loop-expr.html#labelled-block-expressions)
        // to provide a way to exit the current task.
        'task: {
            // Get the first message sent by the IDE.
            let Some(message): std::option::Option<EditorMessage> = from_ide_rx.recv().await else {
                error!("{}", "IDE websocket received no data.");
                break 'task;
            };

            // Make sure it's the `Opened` message.
            let EditorMessageContents::Opened(ide_type) = message.message else {
                let msg = format!("Unexpected message {message:?}");
                error!("{msg}");
                send_response(&to_ide_tx, message.id, &msg).await;

                // Send a `Closed` message to shut down the websocket.
                queue_send!(to_ide_tx.send(EditorMessage { id: 0, message: EditorMessageContents::Closed}), 'task);
                break 'task;
            };

            // Ensure the IDE type (VSCode) is correct.
            match ide_type {
                IdeType::VSCode(is_self_hosted) => {
                    if is_self_hosted {
                        // Send a response (successful) to the `Opened` message.
                        send_response(&to_ide_tx, message.id, "").await;
                        queue_send!(to_ide_tx.send(EditorMessage { id: 0, message: EditorMessageContents::ClientHtml("testing".to_string())}), 'task);
                    } else {
                        // Open the Client in an external browser.
                        if let Err(err) = open::that_detached("https://example.com") {
                            let msg = format!("Unable to open web browser: {err}");
                            error!("{msg}");
                            send_response(&to_ide_tx, message.id, &msg).await;

                            // Send a `Closed` message.
                            queue_send!(to_ide_tx.send(EditorMessage { id: 0, message: EditorMessageContents::Closed}), 'task);

                            break 'task;
                        }
                        send_response(&to_ide_tx, message.id, "").await;
                    }
                }
                _ => {
                    // This is the wrong IDE type. Report then error.
                    let msg = format!("Invalid IDE type: {ide_type:?}");
                    error!("{msg}");
                    send_response(&to_ide_tx, message.id, &msg).await;

                    // Close the connection.
                    queue_send!(to_ide_tx.send(EditorMessage { id: 0, message: EditorMessageContents::Closed}), 'task);
                }
            }
        }
    });

    // Move data between the IDE and the processing task via queues.
    client_websocket(
        connection_id,
        req,
        body,
        app_state.vscode_ide_queues.clone(),
    )
    .await
}

// ## Tests
#[cfg(test)]
mod test {
    use actix_web::{App, HttpServer};
    use assertables::assert_starts_with;
    use assertables::assert_starts_with_as_result;
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

    use super::super::{configure_app, make_app_data, EditorMessage, EditorMessageContents};
    use crate::{test_utils::configure_testing_logger, webserver::UpdateMessageContents};

    #[actix_web::test]
    async fn test_vscode_ide_websocket() {
        configure_testing_logger();

        // Start the full webserver, so we can send non-test requests to it.
        // (The test library doesn't provide a way to send websocket requests
        // that I know of.) One problem: since the server gets run in a separate
        // thread, we can't examine the logs.
        let app_data = make_app_data();
        let webserver_handle = actix_rt::spawn(async move {
            HttpServer::new(move || configure_app(App::new(), &app_data))
                .bind(("127.0.0.1", 8080))?
                // No need to create a bunch of threads for testing.
                .workers(1)
                .run()
                .await
        });

        // Connect to the VSCode IDE websocket.
        let (mut ws_stream, _) = connect_async("ws://127.0.0.1:8080/vsc/ws-ide/test-connection-id")
            .await
            .expect("Failed to connect");

        // Note: we can't check the logs, since the server runs in a separate
        // thread. Changing the logger to log across threads means we get logs
        // from other tests (which run in parallel by default). The benefit of
        // running all tests single-threaded plus fixing the logger is low.
        //
        // Send a message that's not an `Opened` message.
        ws_stream
            .send(Message::Text(
                serde_json::to_string(&EditorMessage {
                    id: 0,
                    message: EditorMessageContents::Update(UpdateMessageContents {
                        path: None,
                        contents: None,
                        cursor_position: None,
                        scroll_position: None,
                    }),
                })
                .unwrap(),
            ))
            .await
            .unwrap();

        // Get the response. It should be an error.
        let em: EditorMessage = serde_json::from_str(
            &ws_stream
                .next()
                .await
                .unwrap()
                .unwrap()
                .into_text()
                .unwrap(),
        )
        .unwrap();
        let EditorMessageContents::Result(result) = em.message else {
            panic!();
        };
        assert_starts_with!(result, "Unexpected message");

        // Next, expect a closed message.
        let em: EditorMessage = serde_json::from_str(
            &ws_stream
                .next()
                .await
                .unwrap()
                .unwrap()
                .into_text()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(em.message, EditorMessageContents::Closed);

        // Send a response to the closed message.
        ws_stream
            .send(Message::Text(
                serde_json::to_string(&EditorMessage {
                    id: 0,
                    message: EditorMessageContents::Result("".to_string()),
                })
                .unwrap(),
            ))
            .await
            .unwrap();

        // Close the connection. TODO: check that the webserver closes it.
        ws_stream.close(None).await.unwrap();

        // Shut down the webserver.
        webserver_handle.abort();
    }
}
