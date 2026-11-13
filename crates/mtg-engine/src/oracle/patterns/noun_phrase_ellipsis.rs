//! Coordination of complete noun phrases that share an elided verb:
//!
//! ```text
//! VERB all X or all Y [TAIL]       "Destroy all lands or all creatures." (Catastrophe)
//! VERB each X or each Y [TAIL]     "Destroy all green creatures or all white creatures."
//! ```
//!
//! "all X or all Y" is a choice between two groups, made as the spell or ability resolves
//! (Catastrophe's ruling: "You choose land or creatures when the spell resolves"; CR
//! 608.2d): it compiles to [`Effect::ChooseOne`] between the instruction applied to each
//! group. Sentences that modify the instruction ("They can't be regenerated.",
//! "Creatures destroyed this way can't be regenerated.") modify each alternative.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_simple, Builder};
use crate::oracle::phrases::*;

/// The labels of the alternatives start with this quantifier.
fn is_group_label(l: &str) -> bool {
    l.starts_with("all ") || l.starts_with("each ")
}

/// "VERB all X or all Y [TAIL]".
fn either_group(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (q, i) = [" or all ", " or each "]
        .into_iter()
        .find_map(|p| l.find(p).map(|i| (&p[4..], i)))?;
    let (a, c) = (&l[..i], &l[i + 4 + q.len()..]);
    // The verb before the first group: "destroy all lands".
    let j = a.find(&format!(" {q}"))?;
    let verb = &a[..=j];
    let first_group = &a[j + 1..];
    if verb.trim().contains(' ') || first_group.contains(" or ") || c.contains(" or all ") {
        return None;
    }
    // What follows the second group belongs to both ("to their owners' hands").
    let (_, _, tail) = parse_object_phrase(c)?;
    let second_group = format!("{q}{}", &c[..c.len() - tail.len()]);
    let saved = b.targets.len();
    let first_text = if parse_object_phrase(&first_group[q.len()..])
        .is_some_and(|(_, _, r)| end(r).is_empty())
    {
        format!("{verb}{first_group}{tail}")
    } else {
        return None;
    };
    let e1 = parse_simple(&first_text, b)?;
    let e2 = parse_simple(&format!("{verb}{second_group}{tail}"), b)?;
    // The groups aren't targets.
    if b.targets.len() != saved {
        return None;
    }
    Some(Effect::ChooseOne {
        who: PlayerRef::You,
        options: vec![
            (first_group.trim().to_string(), e1),
            (second_group.trim().to_string(), e2),
        ],
    })
}

inventory::submit! { EffectPattern { name: "noun phrase ellipsis: VERB all X or all Y", priority: 60, parse: either_group } }

/// A sentence modifying the previous instruction modifies each of its alternatives.
fn each_alternative(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Effect::ChooseOne { options, .. } = prev else {
        return false;
    };
    if options.len() < 2 || !options.iter().all(|(label, _)| is_group_label(label)) {
        return false;
    }
    let saved = b.targets.len();
    let mut out = Vec::new();
    for (label, e) in options.iter() {
        let mut e = e.clone();
        if !crate::oracle_ext::apply_followup_ext(l, &mut e, b) || b.targets.len() != saved {
            b.targets.truncate(saved);
            return false;
        }
        out.push((label.clone(), e));
    }
    *options = out;
    true
}

inventory::submit! { FollowupPattern { name: "noun phrase ellipsis: modify each alternative", priority: 40, apply: each_alternative } }

#[cfg(test)]
mod tests {
    use crate::card::Layout;
    use crate::oracle::{compile, CompileContext};
    use crate::types::TypeLine;

    fn compiled(ty: &str, text: &str) -> Option<String> {
        let tl = TypeLine::parse(ty);
        let ctx = CompileContext {
            card_name: "Test Card",
            full_name: "Test Card",
            type_line: &tl,
            layout: Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: None,
            toughness: None,
        };
        let r = compile(text, &ctx);
        r.unsupported
            .is_empty()
            .then(|| format!("{:?}", r.abilities))
    }

    #[test]
    fn either_group_is_a_choice_on_resolution() {
        for text in [
            "Destroy all lands or all creatures. Creatures destroyed this way can't be regenerated.",
            "Destroy all green creatures or all white creatures. They can't be regenerated.",
            "Exile all artifacts or all enchantments.",
        ] {
            let ast = compiled("Sorcery", text).unwrap_or_else(|| panic!("{text}"));
            assert!(ast.contains("ChooseOne"), "{text}: {ast}");
        }
        let ast = compiled("Sorcery", "Destroy all lands or all creatures. Creatures destroyed this way can't be regenerated.").unwrap();
        assert_eq!(ast.matches("no_regen: true").count(), 2, "{ast}");
        // Not two groups.
        assert!(compiled("Sorcery", "Destroy all lands or creatures.").is_none_or(|a| !a.contains("ChooseOne")));
    }
}
