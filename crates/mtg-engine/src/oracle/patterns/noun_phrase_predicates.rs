//! Predicates after an object phrase's head noun, read as grammar: a copula ("that's",
//! "that is", "that are", negated "that isn't", "that aren't", "that's not", ...) and a
//! list of predicates joined by "or" (CR 105, 205, 506):
//!
//! ```text
//! that's attacking or blocking          that isn't enchanted
//! that aren't legendary                 that's not historic
//! that isn't all colors                 that's a token or a Rabbit
//! that's an artifact, creature, or enchantment
//! that isn't a Kraken, Leviathan, Merfolk, Octopus, or Serpent
//! that's attacking you or a planeswalker you control
//! that's attacking one of your opponents
//! ```
//!
//! A predicate is an adjective (`phrases::adjective`: "enchanted", "legendary",
//! "historic", "tapped", "attacking"...), a color quality ("all colors"), a combat
//! relation ("attacking you", "attacking a planeswalker you control"), or a type with an
//! optional article ("a Rabbit", "an artifact", "token"). A negated list names objects
//! none of the predicates describe.

use super::FilterSuffixPattern;
use crate::ability::*;
use crate::oracle::phrases::{adjective, head_noun};
use crate::types::*;

/// `Filter::Custom`: a creature attacking a planeswalker the ability's controller
/// controls (CR 506.2, 508.1b).
pub const ATTACKING_YOUR_PLANESWALKER: &str = "attacking a planeswalker you control";

/// Whether `r` is at a word boundary (the end, a space or punctuation).
fn boundary(r: &str) -> bool {
    r.is_empty() || r.starts_with([' ', ',', '.', ';', '"'])
}

fn word<'a>(s: &'a str, p: &str) -> Option<&'a str> {
    s.strip_prefix(p).filter(|r| boundary(r))
}

/// One predicate at the start of `s`, and the rest.
fn predicate(s: &str) -> Option<(Filter, &str)> {
    // Combat relations (the player or planeswalker a creature attacks, CR 506.2).
    for (p, f) in [
        (
            "attacking you or a planeswalker you control",
            Filter::Custom(crate::kw::activated_ability_kind::ATTACKING_YOU_OR_YOUR_PLANESWALKER.into()),
        ),
        (
            "attacking you and/or a planeswalker you control",
            Filter::Custom(crate::kw::activated_ability_kind::ATTACKING_YOU_OR_YOUR_PLANESWALKER.into()),
        ),
        (
            "attacking a planeswalker you control",
            Filter::Custom(ATTACKING_YOUR_PLANESWALKER.into()),
        ),
        (
            "attacking one of your opponents",
            Filter::AttackingPlayer(PlayerRel::Opponent),
        ),
        ("attacking an opponent", Filter::AttackingPlayer(PlayerRel::Opponent)),
        ("attacking you", Filter::AttackingPlayer(PlayerRel::You)),
    ] {
        if let Some(r) = word(s, p) {
            return Some((f, r));
        }
    }
    // Color qualities (CR 105.2): "all colors" is each of the five.
    if let Some(r) = word(s, "all colors") {
        return Some((
            Filter::and(Color::ALL.iter().map(|c| Filter::Color(*c)).collect()),
            r,
        ));
    }
    if let Some(r) = word(s, "one or more colors") {
        return Some((Filter::not(Filter::Colorless), r));
    }
    if let Some(r) = word(s, "colorless") {
        return Some((Filter::Colorless, r));
    }
    // A type with an optional article ("a Rabbit", "an artifact", "a token").
    let (article, s2) = match s.strip_prefix("an ").or_else(|| s.strip_prefix("a ")) {
        Some(r) => (true, r),
        None => (false, s),
    };
    let n = s2
        .find(|c: char| !(c.is_alphanumeric() || c == '-'))
        .unwrap_or(s2.len());
    let w = &s2[..n];
    if w.is_empty() {
        return None;
    }
    let rest = &s2[n..];
    if let Some(f @ (Filter::Type(_) | Filter::Subtype(_) | Filter::Token | Filter::Commander)) =
        head_noun(w)
    {
        return Some((f, rest));
    }
    if article {
        return None;
    }
    // An adjective ("enchanted", "legendary", "historic", "tapped", "attacking").
    let f = adjective(w)?;
    Some((f, rest))
}

/// A list of predicates joined by "or" (", or", "and/or"), and the rest.
fn predicate_list(s: &str) -> Option<(Filter, &str)> {
    let (first, mut rest) = predicate(s)?;
    let mut alts = vec![first];
    loop {
        let next = [", or ", ", and/or ", " and/or ", " or ", ", "]
            .into_iter()
            .find_map(|sep| rest.strip_prefix(sep).and_then(predicate));
        match next {
            Some((f, r)) => {
                alts.push(f);
                rest = r;
            }
            None => break,
        }
    }
    let f = if alts.len() == 1 {
        alts.pop()?
    } else {
        Filter::Or(alts)
    };
    Some((f, rest))
}

/// "that's [predicates]", "that isn't [predicates]" after an object noun.
fn copula_predicates<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    const POSITIVE: [&str; 3] = ["that's ", "that is ", "that are "];
    const NEGATIVE: [&str; 6] = [
        "that isn't ",
        "that aren't ",
        "that's not ",
        "that is not ",
        "that are not ",
        "that are neither ",
    ];
    let (negate, r) = if let Some(r) = NEGATIVE.iter().find_map(|p| t.strip_prefix(p)) {
        (true, r)
    } else {
        (false, POSITIVE.iter().find_map(|p| t.strip_prefix(p))?)
    };
    let (f, rest) = predicate_list(r)?;
    // The predicate goes on ("that are enchanted by an Aura you control", "that are
    // blocked by ~ this turn"): another phrase.
    let next = rest.trim_start().split([' ', ',', '.']).next().unwrap_or("");
    // Or it's a list this grammar doesn't read ("that are instant cards, sorcery cards,
    // and/or have an Adventure", "that are Oozes or are named ~").
    let t = rest.trim_start();
    if !boundary(rest)
        || matches!(next, "by"
                | "this"
                | "from"
                | "than"
                | "as"
                | "with"
                | "or"
                | "and"
                | "and/or"
                // An object of the predicate ("that's blocking it", "that's blocking
                // equipped creature"): the blocking grammar's.
                | "it"
                | "~"
                | "target"
                | "enchanted"
                | "equipped")
        || rest.starts_with(',')
        || head_noun(next).is_some()
        || t.starts_with("or ")
    {
        return None;
    }
    Some((if negate { Filter::not(f) } else { f }, rest))
}

inventory::submit! { FilterSuffixPattern { name: "noun phrase: that's / that isn't [predicates]", priority: 900, parse: copula_predicates } }

/// "without mana abilities", "with a mana ability" (CR 605.1a).
fn mana_abilities<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let has = || Filter::Custom(crate::kw::activated_ability_kind::HAS_MANA_ABILITY.into());
    if let Some(r) = word(t, "without mana abilities").or_else(|| word(t, "without a mana ability")) {
        return Some((Filter::not(has()), r));
    }
    let r = word(t, "with a mana ability").or_else(|| word(t, "with mana abilities"))?;
    Some((has(), r))
}

inventory::submit! { FilterSuffixPattern { name: "noun phrase: with/without mana abilities", priority: 90, parse: mana_abilities } }

/// "with mana value of the chosen quality" after "Choose odd or even." (CR 202.3; the
/// choice belongs to the instruction's own ability, CR 607.2d).
fn chosen_quality<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let r = word(t, "with mana value of the chosen quality")
        .or_else(|| word(t, "with mana values of the chosen quality"))?;
    Some((Filter::ManaValueOfChosenQuality, r))
}

inventory::submit! { FilterSuffixPattern { name: "noun phrase: with mana value of the chosen quality", priority: 90, parse: chosen_quality } }

/// "Whenever enchanted creature attacks, each other creature ...": on a source that
/// can't be one of the objects described (an Aura or Equipment isn't a creature), "other"
/// means other than the creature the trigger is about, not other than the source.
pub fn other_than_trigger_subject(f: Filter, b: &crate::oracle::effects::Builder) -> Filter {
    let Filter::And(parts) = &f else {
        return f;
    };
    let subject = match &b.it {
        s @ (Sel::AttachedTo | Sel::TriggerObject) if b.in_trigger => s.clone(),
        _ => return f,
    };
    let creatures = parts
        .iter()
        .any(|p| matches!(p, Filter::Type(CardType::Creature)));
    if !creatures
        || b.ctx.type_line.card_types.contains(CardType::Creature)
        || !parts.iter().any(|p| matches!(p, Filter::Other))
    {
        return f;
    }
    Filter::And(
        parts
            .iter()
            .map(|p| match p {
                Filter::Other => Filter::not(Filter::In(Box::new(subject.clone()))),
                p => p.clone(),
            })
            .collect(),
    )
}

/// "all lands its controller controls" (Tectonic Instability: "Whenever a land enters,
/// tap all lands its controller controls"): objects controlled by the controller of the
/// one object "it" names.
pub fn its_controller_controls(
    f: Filter,
    rest: &str,
    b: &crate::oracle::effects::Builder,
) -> Option<(Filter, String)> {
    let r = word(rest.trim_start(), "its controller controls")?;
    let it = super::pronoun_groups::singular_it(b);
    if !super::filters_relational::names_one_object(&it) || matches!(it, Sel::This) {
        return None;
    }
    let who = PlayerRef::ControllerOf(Box::new(it));
    Some((
        Filter::and(vec![f, Filter::ControlledByPlayer(Box::new(who))]),
        r.to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predicate_lists() {
        let f = |s| copula_predicates(s, &Filter::Any).map(|(f, r)| (format!("{f:?}"), r));
        assert!(f("that isn't enchanted").is_some_and(|(f, r)| f.contains("Not(Enchanted)") && r.is_empty()));
        assert!(f("that's attacking or blocking").is_some_and(|(f, _)| f == "Or([Attacking, Blocking])"));
        assert!(f("that aren't legendary").is_some());
        assert!(f("that's not historic to its owner's hand")
            .is_some_and(|(_, r)| r == " to its owner's hand"));
        assert!(f("that's an artifact, creature, or enchantment").is_some_and(|(f, _)| f.matches("Type").count() == 3));
        assert!(f("that isn't a kraken, leviathan, merfolk, octopus, or serpent").is_some_and(|(f, _)| f.matches("Subtype").count() == 5));
        assert!(f("that's a token or a rabbit").is_some());
        assert!(f("that isn't all colors").is_some());
        assert!(f("that's attacking you or a planeswalker you control").is_some_and(|(_, r)| r.is_empty()));
        assert!(f("that's attacking one of your opponents gets +1/+1").is_some_and(|(_, r)| r == " gets +1/+1"));
        // Not a predicate.
        assert!(f("that's dealt damage").is_none());
        assert!(f("that's a 2/2").is_none());
    }
}
