//! Tokens whose power and toughness are a value: "create an X/X green Ooze creature
//! token, where X is the greatest power among creatures you control", "Create X X/X green
//! Ooze creature tokens" (X of the spell), "create an X/1 red Elemental creature token".
//! The value is determined as the effect is performed (CR 608.2h) and becomes the
//! tokens' printed power and toughness (CR 111.4).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};

/// Replaces an X power and/or toughness ("x/x", "x/2", "2/x") in a token description
/// with a placeholder number; returns the new text and the values.
fn replace_x_pt(l: &str) -> Option<(String, Value, Value)> {
    let words: Vec<&str> = l.split(' ').collect();
    let i = words.iter().position(|w| {
        w.split_once('/').is_some_and(|(p, t)| {
            (p == "x" || t == "x") && [p, t].iter().all(|v| *v == "x" || v.parse::<u32>().is_ok())
        })
    })?;
    let (p, t) = words[i].split_once('/')?;
    let val = |v: &str| -> Value {
        if v == "x" {
            Value::X
        } else {
            Value::c(v.parse().unwrap_or(0))
        }
    };
    let (pv, tv) = (val(p), val(t));
    let mut out = words.clone();
    let placeholder = format!(
        "{}/{}",
        if p == "x" { "0" } else { p },
        if t == "x" { "0" } else { t }
    );
    out[i] = &placeholder;
    Some((out.join(" "), pv, tv))
}

fn x_pt_token(l: &str, b: &mut Builder) -> Option<Effect> {
    if !(l.starts_with("create ") || l.contains(" creates "))
        || !l.contains(" token")
        || l.contains("where x is")
    {
        return None;
    }
    let (text, power, toughness) = replace_x_pt(l)?;
    // An X that isn't defined (by the cost, the spell's X, or "where X is") can't be
    // read here.
    let defined = super::value_grammar::x_defined() || (b.ctx.is_spell() && !b.in_trigger);
    if !defined {
        return None;
    }
    let create = parse_clause(&text, b)?;
    if !matches!(create, Effect::CreateToken { .. }) {
        return None;
    }
    Some(Effect::CreateTokenWithPT {
        power,
        toughness,
        create: Box::new(create),
    })
}

inventory::submit! { EffectPattern { name: "create an X/X token", priority: 60, parse: x_pt_token } }
