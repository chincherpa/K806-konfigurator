//! Sprache der Meldungen (Deutsch/Englisch), von der GUI gesetzt.

use std::sync::atomic::{AtomicBool, Ordering};

static ENGLISCH: AtomicBool = AtomicBool::new(false);

pub fn setzen(englisch: bool) {
    ENGLISCH.store(englisch, Ordering::Relaxed);
}

pub fn englisch() -> bool {
    ENGLISCH.load(Ordering::Relaxed)
}

/// `tr!("deutsch {x}", "english {x}", …)` – formatiert je nach eingestellter Sprache.
#[macro_export]
macro_rules! tr {
    ($de:literal, $en:literal $(, $arg:expr)* $(,)?) => {
        if $crate::geraet::sprache::englisch() { format!($en $(, $arg)*) } else { format!($de $(, $arg)*) }
    };
}
