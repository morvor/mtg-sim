//! The object an ability notes for the abilities linked to it (`linked_notes.rs`, CR
//! 607.2c, 607.2d), named by what the noting ability did: "the chosen creature" (it chose
//! one; "the last chosen" when it can choose again), "the creature put onto the
//! battlefield with ~" (it put one onto the battlefield).

use super::*;

/// The noun of a kind of object: "creature", "card", "permanent".
fn kind_noun(f: &Filter) -> &'static str {
    let atoms: Vec<&Filter> = match f {
        Filter::And(v) => v.iter().collect(),
        other => vec![other],
    };
    // A card (not a permanent) chosen: "the last chosen card".
    if atoms.iter().any(|a| matches!(a, Filter::Card)) {
        "card"
    } else if atoms
        .iter()
        .any(|a| matches!(a, Filter::Type(CardType::Creature)))
    {
        "creature"
    } else {
        "permanent"
    }
}

/// What one ability notes: (the noun, whether it can note again as it's activated).
fn notes_of(body_targets: &[TargetSpec], effect: &Effect, again: bool) -> Option<String> {
    let steps: Vec<&Effect> = match effect {
        Effect::Seq(v) => v.iter().collect(),
        other => vec![other],
    };
    let chosen = |f: &Filter| {
        if again {
            format!("the last chosen {}", kind_noun(f))
        } else {
            format!("the chosen {}", kind_noun(f))
        }
    };
    for (i, e) in steps.iter().enumerate() {
        let Effect::NoteLinked { what, .. } = e else {
            continue;
        };
        match what {
            Sel::Choose { filter, .. } => return Some(chosen(filter)),
            Sel::Var(v) => {
                let before = &steps[..i];
                // "Choose a creature ... the chosen creature".
                if let Some(Effect::Store {
                    sel: Sel::Choose { filter, .. },
                    ..
                }) = before
                    .iter()
                    .rev()
                    .find(|x| matches!(x, Effect::Store { var, .. } if var == v))
                {
                    return Some(chosen(filter));
                }
                // "Return target creature card ... to the battlefield. ... the creature put
                // onto the battlefield with ~" (CR 607.2c): the object it became.
                if *v == vars::IT {
                    if let Some(Effect::Move { what, to }) = before.last() {
                        if to.zone == ZoneKind::Battlefield {
                            let f = match what {
                                Sel::Target(k) => {
                                    match body_targets.get(*k as usize).map(|t| &t.what) {
                                        Some(TargetKind::Object(f)) => f.clone(),
                                        _ => return None,
                                    }
                                }
                                Sel::All(f) => f.clone(),
                                _ => return None,
                            };
                            // The permanent the card became.
                            let n = if matches!(&f, Filter::And(v)
                                if v.iter().any(|a| matches!(a, Filter::Type(CardType::Creature))))
                            {
                                "creature"
                            } else {
                                "permanent"
                            };
                            return Some(format!("the {n} put onto the battlefield with ~"));
                        }
                    }
                }
                return None;
            }
            _ => return None,
        }
    }
    None
}

/// The object the face's one noting ability notes, if it has exactly one.
pub(crate) fn noted_noun(abilities: &[Ability]) -> Option<String> {
    let mut found: Vec<Option<String>> = Vec::new();
    for a in abilities {
        let n =
            match &a.kind {
                AbilityKind::Triggered(t) => has_note(&t.body.effect)
                    .then(|| notes_of(&t.body.targets, &t.body.effect, false)),
                AbilityKind::Spell(s) => has_note(&s.body.effect)
                    .then(|| notes_of(&s.body.targets, &s.body.effect, false)),
                AbilityKind::Activated(x) => has_note(&x.body.effect)
                    .then(|| notes_of(&x.body.targets, &x.body.effect, true)),
                AbilityKind::Static(s) => match &s.effect {
                    StaticEffect::Replacement(ReplacementDef {
                        action: ReplacementAction::AsEnters(e),
                        ..
                    }) if has_note(e) => Some(notes_of(&[], e, false)),
                    _ => None,
                },
                _ => None,
            };
        if let Some(n) = n {
            found.push(n);
        }
    }
    match found.as_slice() {
        [Some(n)] => Some(n.clone()),
        _ => None,
    }
}

fn has_note(e: &Effect) -> bool {
    match e {
        Effect::NoteLinked { .. } => true,
        Effect::Seq(v) => v.iter().any(|x| matches!(x, Effect::NoteLinked { .. })),
        _ => false,
    }
}
