//! Tokens whose power and toughness are a value (CR 111.3, 107.3, 608.2h):
//!
//! ```text
//! [subject] create(s) COUNT [tapped] X/X DESC token(s) TAIL* [, where X is VALUE]
//! ```
//!
//! "Create an X/X black Horror creature token, where X is the number of creatures that
//! died this turn." (Spoils of Blood), "Create X X/X green Ooze creature tokens."
//! (Gelatinous Genesis). The rest of the token description is read as for a token with a
//! printed P/T; X is determined once, as the token is created, and the result is the
//! token's power and toughness as created — its copiable values (`TokenSpec::pt_values`).
//!
//! Without "where X is", X is the X chosen for the spell or ability (CR 107.3a); a
//! triggered ability has no such X, so there the text must define it.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// Calls `f` on each token creation in `e` that a rewrite described as 0/0.
fn zero_zero_specs(e: &mut Effect, f: &mut dyn FnMut(&mut TokenSpec)) {
    match e {
        Effect::CreateToken { spec, .. } | Effect::CreateTokenAttached { spec, .. }
            if spec.power == Some(0) && spec.toughness == Some(0) && spec.pt_values.is_none() =>
        {
            f(spec)
        }
        Effect::Seq(v) => v.iter_mut().for_each(|x| zero_zero_specs(x, f)),
        Effect::If {
            then, otherwise, ..
        } => {
            zero_zero_specs(then, f);
            zero_zero_specs(otherwise, f);
        }
        Effect::May { effect, .. }
        | Effect::AsPlayer { effect, .. }
        | Effect::ForEachPlayer { effect, .. } => zero_zero_specs(effect, f),
        _ => {}
    }
}

/// Gives the one 0/0 token of `e` the P/T X/X; false unless there's exactly one.
fn mark_value_pt(e: &mut Effect) -> bool {
    let mut n = 0;
    zero_zero_specs(e, &mut |_| n += 1);
    if n != 1 {
        return false;
    }
    zero_zero_specs(e, &mut |spec| {
        spec.pt_values = Some(Box::new((Value::X, Value::X)));
    });
    true
}

/// "create an x/x ... token[, where x is ...]".
fn create_value_pt_token(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (clause, value_s) = match l.rsplit_once(", where x is ") {
        Some((c, v)) => (c, Some(v)),
        None => (l, None),
    };
    // A triggered ability has no X of its own: the text must say what X is.
    if value_s.is_none() && b.in_trigger {
        return None;
    }
    if !clause.contains("create") || clause.contains("0/0") {
        return None;
    }
    let i = clause.find(" x/x ")?;
    // Only the token's P/T: "X/X" in a quoted ability of the token isn't it.
    if clause[..i].contains('"') {
        return None;
    }
    let rewritten = format!("{} 0/0 {}", &clause[..i], &clause[i + " x/x ".len()..]);
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let restore = |b: &mut Builder, saved: (usize, Sel, PlayerRef)| {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
    };
    let value = match value_s {
        Some(v) => {
            let Some((v, tail)) = super::r107_numbers::value_phrase(v, b) else {
                restore(b, saved);
                return None;
            };
            if !end(&tail).is_empty() {
                restore(b, saved);
                return None;
            }
            Some(super::r107_numbers::nonnegative(v))
        }
        None => None,
    };
    let Some(mut e) = crate::oracle::effects::parse_clause(&rewritten, b) else {
        restore(b, saved);
        return None;
    };
    if !mark_value_pt(&mut e) {
        restore(b, saved);
        return None;
    }
    match value {
        Some(x) => {
            let out = super::r107_numbers::substitute_x(&e, &x);
            if out.is_none() {
                restore(b, saved);
            }
            out
        }
        None => Some(e),
    }
}

inventory::submit! { EffectPattern { name: "create x/x token", priority: 65, parse: create_value_pt_token } }
