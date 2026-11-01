#![cfg(unix)]

mod common;

use common::{send_request, session_name, start_server, TestHarness, PTY_TEST_LOCK};
use rmux_proto::{
    NewSessionRequest, PaneOptionGetRequest, PaneOptionSetRequest, PaneTarget, PaneTargetRef,
    Request, Response, SetOptionMode, TerminalDefaultColors, TERMINAL_DEFAULT_COLORS_OPTION,
};

#[tokio::test]
async fn invalid_palette_metadata_preserves_the_accepted_defaults(
) -> Result<(), Box<dyn std::error::Error>> {
    let _guard = PTY_TEST_LOCK.lock().await;
    let harness = TestHarness::new("terminal-defaults");
    let server = start_server(&harness).await?;
    let response = send_request(
        harness.socket_path(),
        &Request::NewSession(NewSessionRequest {
            session_name: session_name("colors"),
            detached: true,
            size: None,
            environment: Some(vec!["SHELL=/bin/sh".to_owned()]),
        }),
    )
    .await?;
    assert!(matches!(response, Response::NewSession(_)));
    let target = PaneTargetRef::slot(PaneTarget::with_window(session_name("colors"), 0, 0));
    let colors = TerminalDefaultColors {
        foreground: [1, 2, 3],
        background: [4, 5, 6],
        cursor: [7, 8, 9],
    }
    .encode();
    for (value, valid) in [
        (colors.clone(), true),
        ("bad".to_owned(), false),
        ("+fffff;112233;445566".to_owned(), false),
    ] {
        let response = send_request(
            harness.socket_path(),
            &Request::PaneOptionSet(PaneOptionSetRequest {
                target: target.clone(),
                name: TERMINAL_DEFAULT_COLORS_OPTION.to_owned(),
                value: Some(value),
                mode: SetOptionMode::Replace,
                unset: false,
            }),
        )
        .await?;
        assert_eq!(
            matches!(response, Response::PaneOptionSet(_)),
            valid,
            "{response:?}"
        );
        let response = send_request(
            harness.socket_path(),
            &Request::PaneOptionGet(PaneOptionGetRequest {
                target: target.clone(),
                name: TERMINAL_DEFAULT_COLORS_OPTION.to_owned(),
            }),
        )
        .await?;
        let Response::PaneOptionGet(response) = response else {
            panic!("{response:?}");
        };
        assert_eq!(response.value.as_deref(), Some(colors.as_str()));
    }
    server.shutdown().await?;
    Ok(())
}
