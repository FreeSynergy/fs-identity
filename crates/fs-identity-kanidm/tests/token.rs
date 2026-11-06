//! The token is never printed, and the real transport refuses plain HTTP.

use fs_identity_kanidm::{Error, ReqwestSeam, Token};

#[test]
fn the_token_does_not_show_in_debug_output() {
    let t = Token::new("super-secret-value");
    assert!(!format!("{t:?}").contains("super-secret"));
    let seam = ReqwestSeam::new("https://idm.example.org", t, None).unwrap();
    assert!(!format!("{seam:?}").contains("super-secret"));
}

#[test]
fn the_transport_needs_https_and_a_valid_ca() {
    let t = || Token::new("x");
    assert!(matches!(
        ReqwestSeam::new("http://idm.example.org", t(), None),
        Err(Error::Setup(_))
    ));
    assert!(matches!(
        ReqwestSeam::new("https://idm.example.org", t(), Some(b"not a certificate")),
        Err(Error::Setup(_))
    ));
}
