//! "Create a token that's a copy of it, except it's a Nightmare in addition to its other
//! types. Then exile all other Nightmare tokens you control." (Abyssal Harvester): right
//! after an instruction that creates tokens, "all other [tokens]" are the tokens other
//! than the ones just created, not other than the ability's source (which the source,
//! not being one of those tokens, would leave meaningless). Only for a class of tokens:
//! "all other nontoken creatures" can't mean other than new tokens, so it keeps its usual
//! sense (other than the source).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;

/// Whether only tokens match `f` ("Nightmare tokens you control"), as opposed to a
/// filter that merely mentions tokens ("nontoken creatures").
fn only_tokens(f: &Filter) -> bool {
    match f {
        Filter::Token => true,
        Filter::And(v) => v.iter().any(only_tokens),
        Filter::Or(v) => !v.is_empty() && v.iter().all(only_tokens),
        _ => false,
    }
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
    // "Create a 1/1 ... token and a Food token": `vars::CREATED` would hold only the last
    // kind, so "other" than both can't be said with it.
    if super::tokens_copies_create::last_create(prev).is_none()
        || super::tokens_copies_create::several_kinds(prev)
    {
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
    if !only_tokens(what) {
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

#[cfg(test)]
mod tests {
    use crate::ability::*;
    use crate::oracle::{compile, CompileContext};
    use crate::types::TypeLine;

    /// The filters of the "all [objects]" the compiled text exiles or destroys.
    fn removal_filters(text: &str) -> Vec<Filter> {
        let tl = TypeLine::parse("Creature — Demon Warlock");
        let ctx = CompileContext {
            card_name: "Test",
            full_name: "Test",
            type_line: &tl,
            layout: crate::card::Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: Some("3"),
            toughness: Some("3"),
        };
        let c = compile(text, &ctx);
        assert!(c.unsupported.is_empty(), "{text}: {:?}", c.unsupported);
        let json = serde_json::to_value(&c.abilities).unwrap();
        let mut out = Vec::new();
        fn walk(v: &serde_json::Value, out: &mut Vec<Filter>) {
            match v {
                serde_json::Value::Object(m) => {
                    for (k, x) in m {
                        if matches!(k.as_str(), "Exile" | "Destroy") {
                            if let Some(f) = x.get("what").and_then(|w| w.get("All")) {
                                out.push(serde_json::from_value(f.clone()).unwrap());
                            }
                        }
                        walk(x, out);
                    }
                }
                serde_json::Value::Array(a) => a.iter().for_each(|x| walk(x, out)),
                _ => {}
            }
        }
        walk(&json, &mut out);
        out
    }

    fn mentions(f: &Filter, what: &Filter) -> bool {
        serde_json::to_string(f)
            .is_ok_and(|s| s.contains(&serde_json::to_string(what).unwrap()))
    }

    #[test]
    fn other_than_the_tokens_just_created() {
        let not_created = Filter::not(Filter::In(Box::new(Sel::Var(vars::CREATED))));
        // A class of tokens: other than the new token.
        let f = removal_filters(
            "{T}: Create a 1/1 black Nightmare creature token. Then exile all other Nightmare tokens you control.",
        );
        assert_eq!(f.len(), 1);
        assert!(mentions(&f[0], &not_created) && !mentions(&f[0], &Filter::Other));
        // "Nontoken" creatures can't be other than the new token: other than the source.
        let f = removal_filters(
            "{T}: Create a 1/1 black Nightmare creature token. Then destroy all other nontoken creatures.",
        );
        assert_eq!(f.len(), 1);
        assert!(!mentions(&f[0], &not_created), "{:?}", f[0]);
    }
}
