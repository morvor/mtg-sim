//! Notes (CR 607.2e): "As ~ enters, note your life total." and "the last noted life total
//! for ~" in an ability linked to it (Sigarda's Splendor). A note is kept for the object's
//! linked abilities ([`Effect::Note`], read with [`Value::Chosen`]); noting again replaces
//! it.

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// The value grammar's atom: "the last noted life total for ~", "the noted life total".
pub fn noted_atom(s: &str) -> Option<(Value, String)> {
    for p in [
        "the last noted life total for ~",
        "the noted life total for ~",
    ] {
        if let Some(r) = s.strip_prefix(p) {
            if r.is_empty() || r.starts_with([' ', ',', '.']) {
                return Some((Value::Chosen, r.to_string()));
            }
        }
    }
    None
}

/// The number a "note [number]" instruction notes: "your life total".
fn noted_value(r: &str) -> Option<Value> {
    Some(match r {
        "your life total" => Value::LifeTotal(PlayerRef::You),
        _ => return None,
    })
}

/// "note your life total".
fn note(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let value = noted_value(l.strip_prefix("note ")?)?;
    Some(Effect::Note { value })
}

inventory::submit! { EffectPattern { name: "notes 4: note your life total", priority: 120, parse: note } }

/// "As ~ enters, note your life total." (CR 614.12, 607.2e).
fn as_enters_note(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let value = noted_value(l.strip_prefix("as ~ enters, note ")?)?;
    let s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
        event: ReplacementEvent::EntersBattlefield(Filter::Source),
        action: ReplacementAction::AsEnters(Box::new(Effect::Note { value })),
        self_replacement: true,
        optional: false,
    }));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "notes 4: as ~ enters, note your life total", priority: 120, parse: as_enters_note } }

/// "Choose target instant or sorcery spell. ... If denial gets more votes, counter the
/// spell." (Split Decision): "the spell" is the spell target the text chose, as "that
/// spell" would be.
fn the_chosen_spell(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let words: Vec<&str> = l.split(' ').collect();
    if !words.windows(2).any(|w| w == ["the", "spell"]) || l.contains("that spell") {
        return None;
    }
    // The text named a spell "that spell" refers to ("Choose target ... spell").
    if !b
        .named
        .iter()
        .any(|(n, sel)| n == "that spell" && matches!(sel, Sel::Target(_)))
    {
        return None;
    }
    let rewritten = words
        .iter()
        .enumerate()
        .map(|(i, w)| {
            if *w == "the" && words.get(i + 1) == Some(&"spell") {
                "that"
            } else {
                w
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    crate::oracle::effects::parse_clause(&rewritten, b)
}

inventory::submit! { EffectPattern { name: "notes 4: the spell (the chosen target spell)", priority: 120, parse: the_chosen_spell } }

/// The players a "faces a villainous choice" subject with a relative clause names.
const FACING: Var = vars::USER + 41030;

/// "each opponent who lost 3 or more life this turn faces a villainous choice — ..."
/// (Davros, Dalek Creator): the players the subject names (read as the core reads "each
/// opponent who ... loses 1 life") each face the choice, in APNAP order (CR 701.55a,
/// 701.55d).
fn villainous_for_players_who(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let marker = " faces a villainous choice — ";
    let i = l.find(marker)?;
    let subject = &l[..i];
    if !subject.contains(" who ") || !subject.starts_with("each ") {
        return None;
    }
    let probe = crate::oracle::effects::parse_clause(&format!("{subject} loses 1 life"), b)?;
    let Effect::ForEachPlayer { who, effect } = probe else {
        return None;
    };
    let Effect::AsPlayer {
        who: PlayerRef::Iterated,
        effect: inner,
    } = *effect
    else {
        return None;
    };
    let Effect::If {
        cond,
        then,
        otherwise,
    } = *inner
    else {
        return None;
    };
    if !matches!(*then, Effect::LoseLife { .. }) || !matches!(*otherwise, Effect::Noop) {
        return None;
    }
    let choice = crate::oracle::effects::parse_clause(
        &format!(
            "each opponent faces a villainous choice — {}",
            &l[i + marker.len()..]
        ),
        b,
    )?;
    let Effect::KeywordActionEx(mut spec) = choice else {
        return None;
    };
    if spec.action != KeywordAction::VillainousChoice {
        return None;
    }
    spec.who = PlayerRef::Var(FACING);
    Some(Effect::seq(vec![
        Effect::Store {
            var: FACING,
            sel: Sel::None,
        },
        Effect::ForEachPlayer {
            who,
            effect: Box::new(Effect::AsPlayer {
                who: PlayerRef::Iterated,
                effect: Box::new(Effect::If {
                    cond,
                    then: Box::new(Effect::Store {
                        var: FACING,
                        sel: Sel::Union(vec![Sel::Var(FACING), Sel::Players(PlayerRef::Iterated)]),
                    }),
                    otherwise: Box::new(Effect::Noop),
                }),
            }),
        },
        Effect::KeywordActionEx(spec),
    ]))
}

inventory::submit! { EffectPattern { name: "notes 4: each opponent who ... faces a villainous choice", priority: 55, parse: villainous_for_players_who } }

#[cfg(test)]
mod tests {
    use super::super::followup_grammar::tests::compiled;

    #[test]
    fn notes() {
        for (ty, text) in [
            ("Instant", "Will of the council — Choose target instant or sorcery spell. Starting with you, each player votes for denial or duplication. If denial gets more votes, counter the spell. If duplication gets more votes or the vote is tied, copy the spell. You may choose new targets for the copy."),
            ("Legendary Creature", "At the beginning of your end step, create a 3/3 black Dalek artifact creature token with menace if an opponent lost 3 or more life this turn. Then each opponent who lost 3 or more life this turn faces a villainous choice — You draw a card, or that player discards a card."),
            ("Enchantment", "As ~ enters, note your life total.\nAt the beginning of your upkeep, draw a card if your life total is greater than or equal to the last noted life total for ~. Then note your life total."),
        ] {
            assert!(compiled(ty, text).is_some(), "{text}");
        }
    }
}
