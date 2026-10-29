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
    let copy = Effect::CopySpell {
        what: spell,
        count: Value::c(1),
        new_targets,
    };
    // Each opponent: those who accept are remembered for "for each opponent who copied
    // the spell this way" (tempting offer).
    if matches!(players, PlayerRef::Each(PlayerFilter::Opponent)) {
        return Some(super::each_opponent_may::each_opponent_may_effect(copy, b));
    }
    Some(Effect::ForEachPlayer {
        who: players,
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(Effect::May {
                who: PlayerRef::You,
                effect: Box::new(copy),
            }),
        }),
    })
}

/// "You copy that spell once plus an additional time for each opponent who copied the
/// spell this way." (Tempt with Mayhem).
fn copy_once_plus_for_each_opponent(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l)
        .strip_prefix("you copy that spell once plus an additional time for each opponent who ")?;
    if !matches!(
        r,
        "copied the spell this way" | "copied that spell this way" | "does"
    ) {
        return None;
    }
    let accepted = super::each_opponent_may::accepted_opponents(b)?;
    let spell = match &b.it {
        s @ (Sel::TriggerSpell | Sel::Target(_)) => s.clone(),
        _ => return None,
    };
    Some(Effect::CopySpell {
        what: spell,
        count: Value::Sum(vec![Value::c(1), Value::CountSel(Box::new(accepted))]),
        new_targets: false,
    })
}

inventory::submit! { EffectPattern { name: "copy spell grammar 2: copy that spell once plus an additional time for each opponent who copied", priority: 80, parse: copy_once_plus_for_each_opponent } }

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
    if !link_last_exile(prev) {
        return false;
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, copy_each_linked_exiled_card()]);
    true
}

/// Links the effect's last exile (face up) to the ability (CR 607.2a): its cards are
/// "exiled with ~". Returns false if the effect doesn't end with one.
pub(crate) fn link_last_exile(e: &mut Effect) -> bool {
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

/// "Copy each card exiled with ~": the linked cards still in exile (CR 707.12).
pub(crate) fn copy_each_linked_exiled_card() -> Effect {
    Effect::CopyCard {
        what: Sel::All(Filter::And(vec![
            Filter::In(Box::new(Sel::Linked)),
            Filter::InZone(ZoneKind::Exile),
        ])),
        named: None,
    }
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

/// "you control one or more other creatures that spell could target", "another opponent
/// controls one or more nonland permanents that spell could target": the objects (other
/// than its current targets) the spell an ability triggered on could target.
fn could_target_group(c: &str) -> Option<Filter> {
    let c = end(c);
    let (who, r) = if let Some(r) = c.strip_prefix("you control one or more ") {
        (Filter::ControlledBy(PlayerRel::You), r)
    } else if let Some(r) = c.strip_prefix("another opponent controls one or more ") {
        // An opponent other than the one controlling what the spell targets.
        (
            Filter::and(vec![
                Filter::ControlledBy(PlayerRel::Opponent),
                Filter::not(Filter::ControlledByPlayer(Box::new(
                    PlayerRef::ControllerOf(Box::new(Sel::All(Filter::TargetOf(Box::new(
                        Sel::TriggerSpell,
                    ))))),
                ))),
            ]),
            r,
        )
    } else {
        return None;
    };
    let noun = r.strip_suffix(" that spell could target")?;
    let (f, plural, tail) = parse_object_phrase(noun)?;
    if !plural || !end(tail).is_empty() {
        return None;
    }
    Some(Filter::and(vec![
        f,
        who,
        Filter::Custom(crate::copy_rules::TRIGGER_SPELL_COULD_TARGET.into()),
    ]))
}

/// The intervening "if" clause "if you control one or more other creatures that spell
/// could target" (Beamsplitter Mage).
pub(crate) fn could_target_condition(c: &str, b: &Builder) -> Option<Condition> {
    if !matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    Some(Condition::Exists(could_target_group(c)?))
}

/// The object chosen by "choose one of those creatures" (the copy's new target).
const CHOSEN_NEW_TARGET: Var = vars::USER + 5130;

/// "choose one of those creatures" after "if you control one or more other creatures
/// that spell could target" (Beamsplitter Mage, Exterminator Magmarch).
fn choose_one_of_those_could_target(l: &str, b: &mut Builder) -> Option<Effect> {
    if !matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    let r = end(l).strip_prefix("choose one of those ")?;
    if r.contains(' ') {
        return None;
    }
    let raw = crate::oracle::raw_text().to_lowercase();
    let i = raw.find(" that spell could target")?;
    let head = &raw[..i + " that spell could target".len()];
    let start = head.rfind("if ")? + "if ".len();
    let f = could_target_group(&head[start..])?;
    b.named.push((
        format!("the chosen {}", r.trim_end_matches('s')),
        Sel::Var(CHOSEN_NEW_TARGET),
    ));
    Some(Effect::Store {
        var: CHOSEN_NEW_TARGET,
        sel: Sel::Choose {
            chooser: PlayerRef::You,
            filter: f,
            count: Value::c(1),
            up_to: false,
            store: None,
        },
    })
}

inventory::submit! { EffectPattern { name: "copy spell grammar 2: choose one of those (the spell could target)", priority: 80, parse: choose_one_of_those_could_target } }

/// "The copy targets the chosen creature." after "Copy that spell." (CR 707.10e).
fn the_copy_targets_the_chosen(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let lower = l.to_lowercase();
    let Some(what) = end(&lower).strip_prefix("the copy targets ") else {
        return false;
    };
    let Some(sel) = b
        .named
        .iter()
        .find(|(p, _)| p == what)
        .map(|(_, s)| s.clone())
    else {
        return false;
    };
    fn retarget(e: &mut Effect, sel: &Sel) -> bool {
        match e {
            Effect::CopySpell {
                what,
                count: Value::Const(1),
                ..
            } if matches!(what, Sel::TriggerSpell) => {
                *e = Effect::CopySpellRetargeted {
                    what: Sel::TriggerSpell,
                    target: Some(sel.clone()),
                    among: None,
                };
                true
            }
            Effect::Seq(v) => v.last_mut().is_some_and(|x| retarget(x, sel)),
            _ => false,
        }
    }
    retarget(prev, &sel)
}

inventory::submit! { FollowupPattern { name: "copy spell grammar 2: the copy targets the chosen [object]", priority: 45, apply: the_copy_targets_the_chosen } }

/// "a permanent or player", "a creature": what a spell's targets are checked against.
fn targets_noun(s: &str) -> Option<TargetsFilter> {
    let s = end(s);
    let s = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .unwrap_or(s);
    let (s, players) = match s.strip_suffix(" or player") {
        Some(x) => (x, Some(PlayerFilter::Any)),
        None => (s, None),
    };
    let (f, false, tail) = parse_object_phrase(s)? else {
        return None;
    };
    end(tail).is_empty().then(|| TargetsFilter::Targets {
        objects: Some(f),
        players,
    })
}

/// "copy that spell if it targets a permanent or player, and you may choose new targets
/// for the copy" (Shiko and Narset, Unified): only a spell with such a target is copied
/// (CR 115.9b).
fn copy_that_spell_if_it_targets(l: &str, b: &mut Builder) -> Option<Effect> {
    if !matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    let r = end(l).strip_prefix("copy that spell if it targets ")?;
    let (noun, new_targets) = match r.strip_suffix(", and you may choose new targets for the copy")
    {
        Some(n) => (n, true),
        None => (r, false),
    };
    let tf = targets_noun(noun)?;
    Some(Effect::If {
        cond: Condition::SelMatches(Sel::TriggerSpell, Filter::StackTargets(Box::new(tf))),
        then: Box::new(Effect::CopySpell {
            what: Sel::TriggerSpell,
            count: Value::c(1),
            new_targets,
        }),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "copy spell grammar 2: copy that spell if it targets [noun]", priority: 80, parse: copy_that_spell_if_it_targets } }

/// "If you don't copy a spell this way, draw a card." after a conditional copy.
fn if_you_dont_copy(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let lower = l.to_lowercase();
    let Some(r) = end(&lower).strip_prefix("if you don't copy a spell this way, ") else {
        return false;
    };
    let Effect::If {
        then, otherwise, ..
    } = prev
    else {
        return false;
    };
    if !matches!(**then, Effect::CopySpell { .. }) || !matches!(**otherwise, Effect::Noop) {
        return false;
    }
    let Some(e) = crate::oracle::effects::parse_clause(r, b) else {
        return false;
    };
    **otherwise = e;
    true
}

inventory::submit! { FollowupPattern { name: "copy spell grammar 2: if you don't copy a spell this way", priority: 90, apply: if_you_dont_copy } }

/// "If this creature was kicked, copy that spell twice instead." after "copy the next
/// instant or sorcery spell ... you cast this turn when you cast it" (Sea Gate
/// Stormcaller): the delayed trigger makes two copies if the source was kicked.
fn kicked_copy_next_spell_twice(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let lower = l.to_lowercase();
    let Some(r) = end(&lower).strip_prefix("if ") else {
        return false;
    };
    let Some((c, rest)) = r.split_once(", ") else {
        return false;
    };
    if rest != "copy that spell twice instead" {
        return false;
    }
    let Some(cond) = crate::oracle::statics::parse_condition(c, _b.ctx) else {
        return false;
    };
    if !matches!(cond, Condition::CostPaid(_)) {
        return false;
    }
    fn last_delayed(e: &mut Effect) -> Option<&mut Effect> {
        match e {
            Effect::Seq(v) => v.last_mut().and_then(last_delayed),
            Effect::DelayedTrigger { .. } => Some(e),
            _ => None,
        }
    }
    let Some(Effect::DelayedTrigger { trigger, body, .. }) = last_delayed(prev) else {
        return false;
    };
    if !matches!(trigger, TriggerCond::ThisTurn(_)) {
        return false;
    }
    let Effect::CopySpell {
        what: Sel::TriggerSpell,
        count,
        ..
    } = &mut body.effect
    else {
        return false;
    };
    if !matches!(count, Value::Const(1)) {
        return false;
    }
    *count = Value::If(Box::new(cond), Box::new(Value::c(2)), Box::new(Value::c(1)));
    true
}

inventory::submit! { FollowupPattern { name: "copy spell grammar 2: if kicked, copy that spell twice instead", priority: 90, apply: kicked_copy_next_spell_twice } }

/// The cards each opponent revealed from the top of their library.
const REVEALED_BY_EACH: Var = vars::USER + 5112;

/// "If any of those cards shares a card type with that spell, copy that spell, ..." after
/// "each opponent reveals the top card of their library" (Gandalf, Westward Voyager): the
/// cards all of them revealed.
fn if_any_revealed_shares_a_type(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let lower = l.to_lowercase();
    let Some(rest) =
        end(&lower).strip_prefix("if any of those cards shares a card type with that spell, ")
    else {
        return false;
    };
    // "that spell": the spell the ability triggered on (the reveal made "those cards"
    // the last referent).
    if !b.in_trigger
        || !crate::oracle::raw_text()
            .to_lowercase()
            .starts_with("whenever you cast ")
    {
        return false;
    }
    let Effect::ForEachPlayer {
        who: PlayerRef::EachOpponent,
        effect,
    } = prev
    else {
        return false;
    };
    let Effect::AsPlayer { effect: dig, .. } = &**effect else {
        return false;
    };
    if !matches!(&**dig, Effect::Dig { reveal: true, .. }) {
        return false;
    }
    let saved = std::mem::replace(&mut b.it, Sel::TriggerSpell);
    let then = crate::oracle::effects::parse_clause(rest, b);
    b.it = saved;
    let Some(then) = then else {
        return false;
    };
    let each = std::mem::take(&mut **effect);
    **effect = Effect::seq(vec![
        each,
        Effect::Store {
            var: REVEALED_BY_EACH,
            sel: Sel::Union(vec![Sel::Var(REVEALED_BY_EACH), Sel::Var(vars::DUG)]),
        },
    ]);
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        Effect::Store {
            var: REVEALED_BY_EACH,
            sel: Sel::None,
        },
        old,
        Effect::If {
            cond: Condition::SelNonEmpty(Sel::Matching(
                Box::new(Sel::Var(REVEALED_BY_EACH)),
                Filter::SharesCardType(Box::new(Sel::TriggerSpell)),
            )),
            then: Box::new(then),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "copy spell grammar 2: if any of those cards shares a card type with that spell", priority: 90, apply: if_any_revealed_shares_a_type } }
