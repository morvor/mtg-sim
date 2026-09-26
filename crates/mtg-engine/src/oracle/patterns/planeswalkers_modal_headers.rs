//! Modal headers beyond the plain "choose one/two/one or both" (CR 700.2):
//!
//! * "choose up to one —": no mode need be chosen; a triggered ability with no mode chosen
//!   is removed from the stack (CR 700.2b);
//! * "choose one at random —": the mode is chosen at random, among the modes that can be
//!   chosen (CR 700.2b);
//! * "choose one that hasn't been chosen [this turn] —": modes already chosen for that
//!   object's ability (that turn) can't be chosen again (see `modal_history`);
//! * "Choose one. If [condition], [you may] choose both/two/one or more/any number
//!   instead.": the condition is checked as the modes are chosen (CR 601.2b, 603.3c).

use super::{ModalHeader, ModalHeaderPattern};
use crate::ability::*;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

fn header(min: Value, max: Value, chooser: ModeChooser) -> ModalHeader {
    ModalHeader {
        min,
        max,
        allow_repeat: false,
        chooser,
    }
}

/// "choose up to one", "choose up to two", ...
fn up_to(h: &str, _ctx: &CompileContext) -> Option<ModalHeader> {
    let r = h.strip_prefix("choose up to ")?;
    let (n, rest) = parse_number(r)?;
    let n = n.as_const()?;
    if !rest.trim().is_empty() || n < 1 {
        return None;
    }
    Some(header(Value::c(0), Value::c(n), ModeChooser::Controller))
}

inventory::submit! { ModalHeaderPattern { name: "choose up to n", priority: 100, parse: up_to } }

/// "choose up to four. you may choose the same mode more than once", "choose x. you may
/// choose the same mode more than once" (CR 700.2d; X is announced before the modes are
/// chosen, CR 601.2b).
fn repeatable(h: &str, _ctx: &CompileContext) -> Option<ModalHeader> {
    let r = h
        .strip_suffix(". you may choose the same mode more than once")?
        .strip_prefix("choose ")?;
    let (min, max) = if r == "x" {
        (Value::X, Value::X)
    } else {
        let (up_to, r) = match r.strip_prefix("up to ") {
            Some(r) => (true, r),
            None => (false, r),
        };
        let (n, rest) = parse_number(r)?;
        let n = n.as_const()?;
        if !rest.trim().is_empty() || n < 1 {
            return None;
        }
        (Value::c(if up_to { 0 } else { n }), Value::c(n))
    };
    Some(ModalHeader {
        min,
        max,
        allow_repeat: true,
        chooser: ModeChooser::Controller,
    })
}

inventory::submit! { ModalHeaderPattern { name: "choose n, same mode more than once", priority: 100, parse: repeatable } }

/// "choose one at random"
fn at_random(h: &str, _ctx: &CompileContext) -> Option<ModalHeader> {
    (h == "choose one at random").then(|| header(Value::c(1), Value::c(1), ModeChooser::Random))
}

inventory::submit! { ModalHeaderPattern { name: "choose one at random", priority: 100, parse: at_random } }

/// "choose one that hasn't been chosen", "choose one that hasn't been chosen this turn"
fn unchosen(h: &str, _ctx: &CompileContext) -> Option<ModalHeader> {
    let this_turn = match h {
        "choose one that hasn't been chosen" => false,
        "choose one that hasn't been chosen this turn" => true,
        _ => return None,
    };
    Some(header(
        Value::c(1),
        Value::c(1),
        ModeChooser::Unchosen { this_turn },
    ))
}

inventory::submit! { ModalHeaderPattern { name: "choose one that hasn't been chosen", priority: 100, parse: unchosen } }

/// "choose one. if [condition], [you may] choose both instead": the condition decides how
/// many modes may (or must) be chosen. "As you cast ~" in the condition is when it's
/// checked anyway: modes are chosen while the spell is being cast (CR 601.2b).
fn conditional_count(h: &str, ctx: &CompileContext) -> Option<ModalHeader> {
    let r = h.strip_prefix("choose one. if ")?;
    let (cond, rest) = r.rsplit_once(", ")?;
    let (optional, what) = match rest.strip_prefix("you may choose ") {
        Some(w) => (true, w),
        None => (false, rest.strip_prefix("choose ")?),
    };
    let what = what.strip_suffix(" instead")?;
    // (min, max) when the condition holds; 99 is "all of them" (capped by the modes).
    let (min, max) = match what {
        "both" | "two" => (2, 2),
        "one or more" => (1, 99),
        "any number" => (0, 99),
        _ => return None,
    };
    let cond = cond
        .strip_suffix(" as you cast ~")
        .or_else(|| cond.strip_suffix(" as you cast this spell"))
        .unwrap_or(cond);
    // "it" in the condition would have no referent in the header.
    if cond.split_whitespace().any(|w| w == "it" || w == "it's") {
        return None;
    }
    let cond = crate::oracle::statics::parse_condition(cond, ctx)?;
    let when = |a: i32, b: i32| {
        if a == b {
            Value::c(a)
        } else {
            Value::If(Box::new(cond.clone()), Box::new(Value::c(a)), Box::new(Value::c(b)))
        }
    };
    let min = if optional { Value::c(1) } else { when(min, 1) };
    Some(header(min, when(max, 1), ModeChooser::Controller))
}

inventory::submit! { ModalHeaderPattern { name: "choose one, if condition choose more", priority: 100, parse: conditional_count } }
