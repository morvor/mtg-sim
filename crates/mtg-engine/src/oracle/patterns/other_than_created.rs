//! "Create a token that's a copy of it, except it's a Nightmare in addition to its other
//! types. Then exile all other Nightmare tokens you control." (Abyssal Harvester): right
//! after an instruction that creates tokens, "all other [tokens]" are the tokens other
//! than the ones just created, not other than the ability's source (which the source,
//! not being one of those tokens, would leave meaningless).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;

fn has_token(f: &Filter) -> bool {
    serde_json::to_string(f).is_ok_and(|s| s.contains("\"Token\""))
}

fn f_all_other_tokens(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let Some((verb, r)) = l.split_once(' ') else {
        return false;
    };
    if !matches!(verb, "exile" | "destroy" | "sacrifice") {
        return false;
    }
    let Some(rest) = r.strip_prefix("all other ") else {
        return false;
    };
    if super::tokens_copies_create::last_create(prev).is_none() {
        return false;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1.clone(), saved.2.clone());
    };
    let Some(mut e) = parse_clause(&format!("{verb} all {rest}"), b) else {
        restore(b);
        return false;
    };
    if b.targets.len() != saved.0 {
        restore(b);
        return false;
    }
    let what = match &mut e {
        Effect::Exile {
            what: Sel::All(f), ..
        }
        | Effect::Destroy {
            what: Sel::All(f), ..
        }
        | Effect::SacrificeObjects { what: Sel::All(f) } => f,
        _ => {
            restore(b);
            return false;
        }
    };
    // Only the tokens just created are the antecedent of "other".
    if !has_token(what) {
        restore(b);
        return false;
    }
    *what = Filter::and(vec![
        what.clone(),
        Filter::not(Filter::In(Box::new(Sel::Var(vars::CREATED)))),
    ]);
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "token grammar: all other tokens after creating tokens", priority: 60, apply: f_all_other_tokens } }
