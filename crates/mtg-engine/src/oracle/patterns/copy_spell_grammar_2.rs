//! Spell copies and target changes II (CR 115.7, 707.10).
//!
//! * "The new target must be a player." after "Change the target of target spell that
//!   targets only a player." (Rebound): a restriction on the new target (CR 115.7).
//! * "If target spell has only one target and that target is a creature, change that
//!   spell's target to another creature." (Meddle; Quicksilver Dragon: "... and that
//!   target is ~ ...").
//! * "if it has a single target" (Captured by the Consulate's intervening "if" clause,
//!   CR 115.9a) and "change the target to enchanted creature if able".

use super::{ConditionPattern, EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// "player", "creature": what a new target must be.
fn new_target_kind(s: &str) -> Option<Sel> {
    let s = end(s.trim());
    let s = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .unwrap_or(s);
    if s == "player" {
        return Some(Sel::Players(PlayerRef::EachPlayer));
    }
    let (f, plural, tail) = parse_object_phrase(s)?;
    (!plural && end(tail).is_empty()).then(|| Sel::All(f))
}

/// "The new target must be a player." after changing a spell's target.
fn new_target_must_be(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let lower = l.to_lowercase();
    let Some(r) = end(&lower).strip_prefix("the new target must be ") else {
        return false;
    };
    let Some(sel) = new_target_kind(r) else {
        return false;
    };
    let slot = match prev {
        Effect::ChangeTargets { to, .. } => to,
        Effect::If { then, .. } => match then.as_mut() {
            Effect::ChangeTargets { to, .. } => to,
            _ => return false,
        },
        _ => return false,
    };
    if slot.is_some() {
        return false;
    }
    *slot = Some(sel);
    true
}

inventory::submit! { FollowupPattern { name: "copy spell grammar 2: the new target must be", priority: 45, apply: new_target_must_be } }

/// "if target spell has only one target and that target is a creature, change that
/// spell's target to another creature", "... and that target is ~, ...".
fn change_only_target(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if target spell has only one target and that target is ")?;
    let (what, rest) = r.split_once(", ")?;
    let rest = rest.strip_prefix("change that spell's target to another ")?;
    let only = if what == "~" {
        Filter::Source
    } else {
        let w = what
            .strip_prefix("a ")
            .or_else(|| what.strip_prefix("an "))?;
        let (f, plural, tail) = parse_object_phrase(w)?;
        if plural || !end(tail).is_empty() {
            return None;
        }
        f
    };
    let to = new_target_kind(rest)?;
    let mut spec = TargetSpec::any_target();
    spec.what = TargetKind::Spell(Filter::Spell);
    let slot = b.add_target(spec, "target spell");
    let spell = Sel::Target(slot);
    b.it = spell.clone();
    Some(Effect::If {
        cond: Condition::SelMatches(
            spell.clone(),
            Filter::and(vec![
                Filter::StackTargets(Box::new(TargetsFilter::Count(1))),
                Filter::StackTargets(Box::new(TargetsFilter::Only {
                    objects: Some(only),
                    players: None,
                })),
            ]),
        ),
        then: Box::new(Effect::ChangeTargets {
            what: spell,
            who: PlayerRef::You,
            how: TargetChange::All,
            to: Some(to),
        }),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "copy spell grammar 2: if target spell has only one target, change it", priority: 80, parse: change_only_target } }

/// "it has a single target" (about the spell a cast trigger is about, CR 115.9a).
fn has_a_single_target(c: &str) -> Option<Condition> {
    match end(c) {
        "it has a single target" | "that spell has a single target" => Some(Condition::SelMatches(
            Sel::TriggerSpell,
            Filter::StackTargets(Box::new(TargetsFilter::Count(1))),
        )),
        _ => None,
    }
}

inventory::submit! { ConditionPattern { name: "copy spell grammar 2: it has a single target", priority: 80, parse: has_a_single_target } }

/// "if it has a single target" as an intervening "if" clause of an ability that triggers
/// on a spell: that spell (`b.it`).
pub(crate) fn it_has_a_single_target(c: &str, b: &Builder) -> Option<Condition> {
    if !matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    if let Some(f) = targets_one_or_more(c) {
        return Some(Condition::SelMatches(
            Sel::TriggerSpell,
            Filter::StackTargets(Box::new(TargetsFilter::Targets {
                objects: Some(f),
                players: None,
            })),
        ));
    }
    if end(c) != "it has a single target" {
        return None;
    }
    has_a_single_target(c)
}

/// "it targets one or more other permanents you control" → the permanents' filter.
fn targets_one_or_more(c: &str) -> Option<Filter> {
    let r = end(c).strip_prefix("it targets one or more ")?;
    let (f, plural, tail) = parse_object_phrase(r)?;
    (plural && end(tail).is_empty()).then(|| Filter::and(vec![f, Filter::Permanent]))
}

/// "create a token that's a copy of one of those permanents" after "if it targets one or
/// more other permanents you control" (Orvar, the All-Form): one of the spell's targets
/// of that kind, chosen as the ability resolves.
fn copy_of_one_of_those_targets(l: &str, b: &mut Builder) -> Option<Effect> {
    if !matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    if end(l) != "create a token that's a copy of one of those permanents" {
        return None;
    }
    let raw = crate::oracle::raw_text().to_lowercase();
    let i = raw.find("if it targets one or more ")?;
    let tail = &raw[i + "if ".len()..];
    let cond = tail.split(", ").next()?;
    let f = targets_one_or_more(cond)?;
    Some(Effect::CreateTokenCopy {
        of: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![f, Filter::TargetOf(Box::new(Sel::TriggerSpell))]),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
        count: Value::c(1),
        controller: PlayerRef::You,
        tapped: false,
        attacking: false,
        mods: vec![],
    })
}

inventory::submit! { EffectPattern { name: "copy spell grammar 2: a token copy of one of those targeted permanents", priority: 80, parse: copy_of_one_of_those_targets } }

/// "change the target to enchanted creature if able" in an ability that triggers on a
/// spell (Captured by the Consulate): the spell's target becomes that creature, if it's a
/// legal target for it (CR 115.7).
fn change_the_target_to(l: &str, b: &mut Builder) -> Option<Effect> {
    if !matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    let r = end(l)
        .strip_prefix("change the target to ")?
        .strip_suffix(" if able")?;
    let to = match r {
        "enchanted creature" | "enchanted permanent" => Sel::AttachedTo,
        "~" => Sel::This,
        _ => return None,
    };
    Some(Effect::ChangeTargets {
        what: Sel::TriggerSpell,
        who: PlayerRef::You,
        how: TargetChange::All,
        to: Some(to),
    })
}

inventory::submit! { EffectPattern { name: "copy spell grammar 2: change the target to [object] if able", priority: 80, parse: change_the_target_to } }

/// "each other player may copy that spell and may choose new targets for the copy they
/// control" (Curse of Echoes), "each opponent may copy that spell and ..." (Tempt with
/// Mayhem): each of those players in turn may copy it; they control their copy and choose
/// its new targets (CR 707.10, 707.10c).
fn each_player_may_copy(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (players, r) = if let Some(r) = l.strip_prefix("each other player may copy ") {
        // Other than the player the ability is about (the caster), or than you.
        let other = match &b.it_player {
            p @ (PlayerRef::TriggerPlayer | PlayerRef::Target(_)) => p.clone(),
            _ => PlayerRef::You,
        };
        (
            PlayerRef::Each(PlayerFilter::Not(Box::new(PlayerFilter::Ref(Box::new(
                other,
            ))))),
            r,
        )
    } else if let Some(r) = l.strip_prefix("each opponent may copy ") {
        (PlayerRef::Each(PlayerFilter::Opponent), r)
    } else {
        return None;
    };
    let (what, new_targets) =
        match r.strip_suffix(" and may choose new targets for the copy they control") {
            Some(w) => (w, true),
            None => (r, false),
        };
    let spell = match (what, &b.it) {
        ("that spell" | "it", s @ (Sel::TriggerSpell | Sel::Target(_))) => s.clone(),
        _ => return None,
    };
    Some(Effect::ForEachPlayer {
        who: players,
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(Effect::May {
                who: PlayerRef::You,
                effect: Box::new(Effect::CopySpell {
                    what: spell,
                    count: Value::c(1),
                    new_targets,
                }),
            }),
        }),
    })
}

inventory::submit! { EffectPattern { name: "copy spell grammar 2: each other player may copy that spell", priority: 80, parse: each_player_may_copy } }

/// Whether `e` copies the spell an ability triggered on (optionally).
fn copies_trigger_spell(e: &Effect) -> bool {
    match e {
        Effect::CopySpell { what, .. } | Effect::CopySpellExcept { what, .. } => {
            matches!(what, Sel::TriggerSpell)
        }
        Effect::May { effect, .. } => copies_trigger_spell(effect),
        Effect::Seq(v) => v.last().is_some_and(copies_trigger_spell),
        _ => false,
    }
}

/// "If you do, those spells gain wither." after "you may copy it" (Spinerock Tyrant): the
/// spell and its copy gain the keyword (CR 702.80).
fn those_spells_gain(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !copies_trigger_spell(prev) {
        return false;
    }
    let lower = l.to_lowercase();
    let Some(kw) = end(&lower)
        .strip_prefix("if you do, those spells gain ")
        .or_else(|| end(&lower).strip_prefix("those spells gain "))
    else {
        return false;
    };
    let Some(mods) = crate::oracle::effects::keyword_mods(kw) else {
        return false;
    };
    let gain = Effect::Modify {
        what: Sel::Union(vec![Sel::TriggerSpell, Sel::Var(vars::CREATED)]),
        mods,
        duration: Duration::Permanent,
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::If {
            cond: Condition::PrevHappened,
            then: Box::new(gain),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "copy spell grammar 2: those spells gain [keyword]", priority: 45, apply: those_spells_gain } }

/// Whether `e` ends with (an optional) copy of a spell.
fn ends_with_spell_copy(e: &Effect) -> bool {
    match e {
        Effect::CopySpell { .. } | Effect::CopySpellExcept { .. } => true,
        Effect::May { effect, .. } => ends_with_spell_copy(effect),
        Effect::PayOptional { then, .. } => ends_with_spell_copy(then),
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => ends_with_spell_copy(then),
        Effect::Seq(v) => v.last().is_some_and(ends_with_spell_copy),
        _ => false,
    }
}

/// "If the copy is a permanent spell, it gains haste." (Mizzix, Replica Rider;
/// Spider-Verse): the copy gains it only if it's a permanent spell, and so does the token
/// it becomes (CR 707.10f).
fn if_the_copy_is_a_permanent_spell(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !ends_with_spell_copy(prev) {
        return false;
    }
    let lower = l.to_lowercase();
    let Some(kw) = end(&lower).strip_prefix("if the copy is a permanent spell, it gains ") else {
        return false;
    };
    // "haste", "haste and \"At the beginning of your end step, sacrifice ~.\"".
    let mods = match crate::oracle::effects::keyword_mods(kw) {
        Some(m) => m,
        None => {
            let Some((masked, quotes)) = super::statics::mask_quotes(end(l.trim())) else {
                return false;
            };
            let Some(list) = masked
                .to_lowercase()
                .strip_prefix("if the copy is a permanent spell, it gains ")
                .map(str::to_string)
            else {
                return false;
            };
            let Some(abilities) = super::tokens_copies_create::ability_list(
                &list,
                &quotes,
                &[crate::types::CardType::Creature],
                _b.ctx,
            ) else {
                return false;
            };
            abilities
                .into_iter()
                .map(|a| match &a.kind {
                    AbilityKind::Keyword(k) => Modification::AddKeyword(k.clone()),
                    _ => Modification::AddAbility(a),
                })
                .collect()
        }
    };
    let gain = Effect::Modify {
        what: Sel::Matching(Box::new(Sel::Var(vars::CREATED)), Filter::PermanentCard),
        mods,
        duration: Duration::Permanent,
    };
    // Inside the optional part ("you may pay {1}{U/R}. If you do, copy that spell"), right
    // after the copy.
    fn append(e: &mut Effect, gain: Effect) -> bool {
        match e {
            Effect::CopySpell { .. } | Effect::CopySpellExcept { .. } => {
                let old = std::mem::take(e);
                *e = Effect::seq(vec![old, gain]);
                true
            }
            Effect::May { effect, .. } => append(effect, gain),
            Effect::PayOptional { then, .. } => append(then, gain),
            Effect::If { then, .. } => append(then, gain),
            Effect::Seq(v) => match v.last_mut() {
                Some(last) if ends_with_spell_copy(last) => append(last, gain),
                _ => false,
            },
            _ => false,
        }
    }
    append(prev, gain)
}

inventory::submit! { FollowupPattern { name: "copy spell grammar 2: if the copy is a permanent spell, it gains", priority: 45, apply: if_the_copy_is_a_permanent_spell } }

/// "Exile an instant or sorcery card at random from your graveyard. Then copy each card
/// exiled with ~." (Arcane Bombardment): the exile is linked to the ability (CR 607.2a),
/// and every card exiled with it so far that's still in exile is copied (CR 707.12).
fn copy_each_card_exiled_with(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let lower = l.to_lowercase();
    let l = end(&lower);
    let l = l.strip_prefix("then ").unwrap_or(l);
    if l != "copy each card exiled with ~" {
        return false;
    }
    fn link_last_exile(e: &mut Effect) -> bool {
        match e {
            Effect::Exile {
                face_down: false,
                link,
                ..
            } => {
                *link = true;
                true
            }
            Effect::Seq(v) => v.last_mut().is_some_and(link_last_exile),
            _ => false,
        }
    }
    if !link_last_exile(prev) {
        return false;
    }
    let copy = Effect::CopyCard {
        what: Sel::All(Filter::And(vec![
            Filter::In(Box::new(Sel::Linked)),
            Filter::InZone(ZoneKind::Exile),
        ])),
        named: None,
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, copy]);
    true
}

inventory::submit! { FollowupPattern { name: "copy spell grammar 2: then copy each card exiled with ~", priority: 45, apply: copy_each_card_exiled_with } }

/// "You may copy an instant or sorcery card in it." after "Target opponent reveals their
/// hand." (Reversal of Fortune): a copy of a card the controller chooses from that hand,
/// created in that zone (CR 707.12).
fn copy_a_card_in_revealed_hand(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("copy ")?.strip_suffix(" in it")?;
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains("reveals their hand") {
        return None;
    }
    let owner = match &b.it_player {
        p @ (PlayerRef::Target(_) | PlayerRef::TriggerPlayer) => p.clone(),
        _ => return None,
    };
    let (f, plural, tail) = parse_object_phrase(r)?;
    if plural || !end(tail).is_empty() {
        return None;
    }
    Some(Effect::CopyCard {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                f,
                Filter::InZone(ZoneKind::Hand),
                Filter::OwnedByPlayer(Box::new(owner)),
            ]),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
        named: None,
    })
}

inventory::submit! { EffectPattern { name: "copy spell grammar 2: copy a card in the revealed hand", priority: 80, parse: copy_a_card_in_revealed_hand } }

/// "Copy that spell for each other permanent or player the spell could target." after
/// choosing a target spell (Radiate), "Copy that spell or ability for each other permanent
/// or player the spell or ability could target." (Radiant Performer): CR 707.10d for the
/// targeted spell or ability, the copies controlled by this ability's controller.
fn copy_target_for_each_other(l: &str, b: &mut Builder) -> Option<Effect> {
    let Sel::Target(n) = b.it.clone() else {
        return None;
    };
    if !matches!(
        b.targets.get(n as usize).map(|t| &t.what),
        Some(TargetKind::Spell(_) | TargetKind::Ability(_) | TargetKind::SpellOrAbility(_))
    ) {
        return None;
    }
    let l = end(l);
    let r = l
        .strip_prefix("copy that spell for each other ")
        .or_else(|| l.strip_prefix("copy that spell or ability for each other "))
        .or_else(|| l.strip_prefix("copy that ability for each other "))?;
    let kind = r
        .strip_suffix(" the spell or ability could target")
        .or_else(|| r.strip_suffix(" the spell could target"))
        .or_else(|| r.strip_suffix(" that spell could target"))
        .or_else(|| r.strip_suffix(" the ability could target"))?;
    let among = if let Some(objects) = kind.strip_suffix(" or player") {
        let (f, plural, tail) = parse_object_phrase(objects)?;
        if plural || !end(tail).is_empty() {
            return None;
        }
        Filter::Or(vec![
            f,
            Filter::Custom(crate::copy_rules::ANY_PLAYER.into()),
        ])
    } else {
        let (f, plural, tail) = parse_object_phrase(kind)?;
        if plural || !end(tail).is_empty() {
            return None;
        }
        f
    };
    Some(Effect::CopySpellRetargeted {
        what: Sel::Target(n),
        target: None,
        among: Some(among),
    })
}

inventory::submit! { EffectPattern { name: "copy spell grammar 2: copy the target spell for each other thing it could target", priority: 80, parse: copy_target_for_each_other } }

/// "Each copy targets a different one of those permanents and players." after copying a
/// spell for each other permanent or player it could target (CR 707.10d).
fn each_copy_targets_a_different_one_2(s: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = s.to_lowercase();
    if !matches!(
        end(&l),
        "each copy targets a different one of those permanents and players"
            | "each copy targets a different one of those permanents and/or players"
    ) {
        return false;
    }
    matches!(prev, Effect::CopySpellRetargeted { target: None, .. })
}

inventory::submit! { FollowupPattern { name: "copy spell grammar 2: each copy targets a different one of those permanents and players", priority: 45, apply: each_copy_targets_a_different_one_2 } }
