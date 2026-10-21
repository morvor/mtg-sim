//! Combat grammar IV (cards3-combat-statics): objects put onto the battlefield attacking a
//! named player or blocking a named creature.
//!
//! * "put [cards] onto the battlefield tapped and attacking that player / that opponent /
//!   defending player [or a planeswalker they control]" (Kaalia of the Vast, Owlbear Cub,
//!   Zara, Hans Eriksson): the creature attacks that player (or one of their planeswalkers,
//!   as its controller chooses) rather than any player or planeswalker its controller
//!   chooses (CR 508.4). `Effect::TokensJoinCombat` around the move.
//! * "... It enters tapped and attacking that player." after a move (Shark Shredder).
//! * "put [a creature card] onto the battlefield blocking that creature" (Aetherplasm): it
//!   enters blocking that attacker (CR 509.4), never declared as a blocker.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::patterns::oracle_hardening_referents::{is_no_player_referent, is_no_referent};
use crate::oracle::phrases::end;

/// Whether `e` puts objects onto the battlefield attacking (any player): a move or a dig
/// whose destination says "attacking", possibly optional or conditional.
pub(crate) fn moves_attacking(e: &Effect) -> bool {
    match e {
        Effect::Move { to, .. } => to.zone == ZoneKind::Battlefield && to.attacking,
        Effect::Dig { take_to, .. } => {
            take_to.zone == ZoneKind::Battlefield && take_to.attacking
        }
        Effect::DigStep(s) => {
            matches!(&**s, DigStep::Take { to, .. } if to.zone == ZoneKind::Battlefield && to.attacking)
        }
        Effect::May { effect, .. } => moves_attacking(effect),
        Effect::Seq(v) => v.iter().any(moves_attacking) && !v.iter().any(creates_tokens),
        _ => false,
    }
}

/// Whether `e` puts objects onto the battlefield with a move or dig (not attacking).
fn moves_onto_battlefield(e: &Effect) -> bool {
    match e {
        Effect::Move { to, .. } => to.zone == ZoneKind::Battlefield && !to.attacking,
        Effect::Dig { take_to, .. } => {
            take_to.zone == ZoneKind::Battlefield && !take_to.attacking
        }
        Effect::May { effect, .. } => moves_onto_battlefield(effect),
        Effect::Seq(v) => v.iter().any(moves_onto_battlefield) && !v.iter().any(creates_tokens),
        _ => false,
    }
}

fn creates_tokens(e: &Effect) -> bool {
    matches!(
        e,
        Effect::CreateToken { .. }
            | Effect::CreateTokenWithPT { .. }
            | Effect::CreateTokenCopy { .. }
            | Effect::TokensJoinCombat { .. }
    )
}

/// The player a creature entering attacking attacks: "that player", "that opponent"
/// (the player the text refers to), "defending player".
fn named_defender(who: &str, b: &Builder) -> Option<PlayerRef> {
    match who {
        "that player" | "that opponent" => {
            let p = b.it_player.clone();
            if is_no_player_referent(&p) || matches!(p, PlayerRef::You) {
                return None;
            }
            Some(p)
        }
        "defending player" | "the defending player" => Some(PlayerRef::DefendingPlayer),
        _ => None,
    }
}

/// Splits "... attacking [defender][ or a planeswalker they control]" into the instruction
/// ending in "attacking", the defender's words and whether its planeswalkers may be
/// attacked instead.
pub(crate) fn split_defender(l: &str) -> Option<(&str, &str, bool)> {
    let (l, walkers) = match l
        .strip_suffix(" or a planeswalker they control")
        .or_else(|| l.strip_suffix(" or a planeswalker that player controls"))
    {
        Some(x) => (x, true),
        None => (l, false),
    };
    for who in [
        "that player",
        "that opponent",
        "defending player",
        "the defending player",
    ] {
        if let Some(base) = l.strip_suffix(who).and_then(|x| x.strip_suffix(" attacking ")) {
            return Some((&l[..base.len() + " attacking".len()], who, walkers));
        }
    }
    None
}

pub(crate) fn join_attacking(e: Effect, defender: PlayerRef, walkers: bool) -> Effect {
    let effect = if walkers {
        Effect::seq(vec![
            Effect::Custom(crate::kw::token_creation_rules::OR_THEIR_PLANESWALKERS.into()),
            e,
        ])
    } else {
        e
    };
    Effect::TokensJoinCombat {
        effect: Box::new(effect),
        attacking: Some(defender),
        blocking: None,
    }
}

/// "you may put an Angel, Demon, or Dragon creature card from your hand onto the battlefield
/// tapped and attacking that opponent", "put it onto the battlefield tapped and attacking
/// defending player or a planeswalker they control".
fn p_move_attacking_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    if !l.contains(" onto the battlefield ") || l.contains("create") {
        return None;
    }
    let (base, who, walkers) = split_defender(l)?;
    let defender = named_defender(who, b)?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let e = crate::oracle::effects::parse_clause(base, b);
    let Some(e) = e.filter(moves_attacking) else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
        return None;
    };
    Some(join_attacking(e, defender, walkers))
}

inventory::submit! { EffectPattern { name: "combat 4: put onto the battlefield attacking that player", priority: 89, parse: p_move_attacking_player } }

/// "you may put a creature card from your hand onto the battlefield blocking that creature"
/// (Aetherplasm).
fn p_move_blocking(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (base, blocked) = l.rsplit_once(" onto the battlefield blocking ")?;
    if l.contains("create") {
        return None;
    }
    let (attacker, rest) = object_ref(blocked, b)?;
    if !rest.trim().is_empty()
        || is_no_referent(&attacker)
        || !matches!(attacker, Sel::TriggerObject | Sel::Target(_) | Sel::Var(_))
    {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let e = crate::oracle::effects::parse_clause(&format!("{base} onto the battlefield"), b);
    let Some(e) = e.filter(moves_onto_battlefield) else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
        return None;
    };
    Some(Effect::TokensJoinCombat {
        effect: Box::new(e),
        attacking: None,
        blocking: Some(attacker),
    })
}

inventory::submit! { EffectPattern { name: "combat 4: put onto the battlefield blocking that creature", priority: 89, parse: p_move_blocking } }

/// The move instruction of `e` that puts objects onto the battlefield, last.
fn last_move(e: &mut Effect) -> Option<&mut Destination> {
    match e {
        Effect::Seq(v) => v.iter_mut().rev().find_map(last_move),
        Effect::May { effect, .. } => last_move(effect),
        Effect::Move { to, .. } if to.zone == ZoneKind::Battlefield && !to.attacking => Some(to),
        _ => None,
    }
}

/// "It enters tapped and attacking that player." after putting a card onto the
/// battlefield (Shark Shredder, Killer Clone).
fn f_it_enters_attacking_player(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l.trim());
    let Some(rest) = ["it enters ", "that creature enters ", "that card enters "]
        .iter()
        .find_map(|p| l.strip_prefix(p))
    else {
        return false;
    };
    let Some((base, who, walkers)) = split_defender(rest) else {
        return false;
    };
    if base != "tapped and attacking" {
        return false;
    }
    let Some(defender) = named_defender(who, b) else {
        return false;
    };
    if matches!(prev, Effect::TokensJoinCombat { .. }) {
        return false;
    }
    let Some(to) = last_move(prev) else {
        return false;
    };
    to.tapped = true;
    to.attacking = true;
    let inner = std::mem::take(prev);
    *prev = join_attacking(inner, defender, walkers);
    true
}

inventory::submit! { FollowupPattern { name: "combat 4: it enters tapped and attacking that player", priority: 44, apply: f_it_enters_attacking_player } }

/// "When you put a creature card onto the battlefield this way, it fights ~." (Hans
/// Eriksson): a reflexive triggered ability (CR 603.12) that triggers if the instruction
/// before put a card of that kind onto the battlefield ("it": that card) — not if it put
/// the card somewhere else instead ("Otherwise, put that card into your hand").
fn f_when_put_onto_battlefield_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l.trim());
    let Some((desc, text)) = l
        .strip_prefix("when you put ")
        .and_then(|r| r.split_once(" onto the battlefield this way, "))
    else {
        return false;
    };
    let Some(desc) = desc.strip_prefix("a ").or_else(|| desc.strip_prefix("an ")) else {
        return false;
    };
    let Some(f) = super::card_flow_search::card_filter(desc, b) else {
        return false;
    };
    if !puts_onto_battlefield(prev) {
        return false;
    }
    let Some(body) = super::r600_triggers::reflexive_body(text, b) else {
        return false;
    };
    let entered = Sel::Var(vars::IT);
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::If {
            cond: Condition::SelMatches(
                entered,
                Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]),
            ),
            then: Box::new(Effect::Reflexive {
                body: Box::new(body),
            }),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

/// Whether the last instruction of `e` may put a card onto the battlefield with a move.
fn puts_onto_battlefield(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.last().is_some_and(puts_onto_battlefield),
        Effect::If {
            then, otherwise, ..
        } => puts_onto_battlefield(then) || puts_onto_battlefield(otherwise),
        Effect::May { effect, .. } | Effect::TokensJoinCombat { effect, .. } => {
            puts_onto_battlefield(effect)
        }
        Effect::Move { to, .. } => to.zone == ZoneKind::Battlefield,
        _ => false,
    }
}

inventory::submit! { FollowupPattern { name: "combat 4: when you put a card onto the battlefield this way", priority: 60, apply: f_when_put_onto_battlefield_this_way } }

// ---------------------------------------------------------------------------
// "That combat": the additional combat phase just added (CR 500.8)
// ---------------------------------------------------------------------------

/// Whether `e` is an instruction adding one combat phase ("there is an additional combat
/// phase ..."), whose combat phase "that combat" names.
fn adds_one_combat(e: &Effect) -> bool {
    match e {
        Effect::AddTurnParts { parts, n, .. } => {
            matches!(n, Value::Const(1)) && parts.first() == Some(&TurnPart::CombatPhase)
        }
        Effect::Custom(name) => name.starts_with("combat4:after:"),
        _ => false,
    }
}

/// The sequence of instructions ending in the one that adds the combat phase: the effect
/// itself, or the branch of a condition ("if it's your main phase, ..."); `None` if the
/// effect's last instruction doesn't add one.
fn added_combat_branch(e: &mut Effect) -> Option<&mut Effect> {
    enum Where {
        Here,
        Then,
        Last,
    }
    let w = match &*e {
        Effect::If { .. } => Where::Then,
        Effect::Seq(v) if v.last().is_some_and(|x| matches!(x, Effect::If { .. })) => Where::Last,
        Effect::Seq(v) if v.iter().any(adds_one_combat) => Where::Here,
        other if adds_one_combat(other) => Where::Here,
        _ => return None,
    };
    match w {
        Where::Here => Some(e),
        Where::Then => match e {
            Effect::If { then, .. } => added_combat_branch(then),
            _ => None,
        },
        Where::Last => match e {
            Effect::Seq(v) => added_combat_branch(v.last_mut()?),
            _ => None,
        },
    }
}

/// Whether `e` only creates rule-modifying effects lasting until end of combat (or of
/// the turn): restrictions and requirements.
fn restrictions_only(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => !v.is_empty() && v.iter().all(restrictions_only),
        Effect::AddRestriction { duration, .. } => {
            matches!(duration, Duration::EndOfCombat | Duration::EndOfTurn)
        }
        _ => false,
    }
}

/// "Only land creatures can attack during that combat phase." (Bumi, Unleashed), "Each of
/// those creatures attacks that combat if able. They can't attack you or planeswalkers you
/// control that combat." (Illusionist's Gambit), "At the beginning of that combat, untap
/// all creatures you control." (Moraug, Fury of Akoum) after an instruction adding an
/// additional combat phase: effects that apply during that combat phase only, and a
/// triggered ability at its beginning (CR 603.7).
fn f_that_combat(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l.trim());
    if !l.contains("that combat") {
        return false;
    }
    if added_combat_branch(prev).is_none() {
        return false;
    }
    let effect = if let Some(text) = l.strip_prefix("at the beginning of that combat, ") {
        let Some(body) = super::r600_triggers::reflexive_body(text, b) else {
            return false;
        };
        Effect::Reflexive {
            body: Box::new(body),
        }
    } else {
        let rewritten = l
            .replace("during that combat phase", "this combat")
            .replace(" that combat", " this combat");
        if rewritten.contains("that combat") {
            return false;
        }
        let n = b.targets.len();
        let parsed = crate::oracle::effects::parse_sentence(&rewritten, b);
        let Some(e) = parsed.filter(|e| restrictions_only(e) && b.targets.len() == n) else {
            b.targets.truncate(n);
            return false;
        };
        e
    };
    let during = Effect::DuringNext {
        who: PlayerRef::ActivePlayer,
        part: NextTurnPart::AddedCombatPhase,
        effect: Box::new(effect),
    };
    let branch = added_combat_branch(prev).expect("checked");
    let old = std::mem::take(branch);
    *branch = match old {
        Effect::Seq(mut v) => {
            v.push(during);
            Effect::Seq(v)
        }
        other => Effect::Seq(vec![other, during]),
    };
    true
}

inventory::submit! { FollowupPattern { name: "combat 4: during that combat phase / at the beginning of that combat", priority: 40, apply: f_that_combat } }

/// "Only land creatures can attack this combat", "Only the chosen creatures can attack this
/// turn": no other creature can attack (CR 508.1c).
fn p_only_can_attack(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let r = l.strip_prefix("only ")?;
    let (np, duration) = if let Some(x) = r.strip_suffix(" can attack this combat") {
        (x, Duration::EndOfCombat)
    } else if let Some(x) = r.strip_suffix(" can attack this turn") {
        (x, Duration::EndOfTurn)
    } else {
        return None;
    };
    let f = match np {
        // "Choose two target creatures. ... Only the chosen creatures can attack ..."
        // (Last Night Together): the creatures the first sentence chose.
        "the chosen creatures" | "the chosen creature"
            if b.chosen_creature.is_none()
                && !b.targets.is_empty()
                && crate::oracle::raw_text()
                    .to_lowercase()
                    .starts_with("choose two target creatures.") =>
        {
            Filter::In(Box::new(Sel::Target(0)))
        }
        "the chosen creatures" | "those creatures" | "the chosen creature" | "that creature" => {
            let (sel, rest) = object_ref(np, b)?;
            if !rest.trim().is_empty() || is_no_referent(&sel) || matches!(sel, Sel::This) {
                return None;
            }
            Filter::In(Box::new(sel))
        }
        _ => {
            let (f, plural) = super::statics::whole_object_phrase(np)?;
            if !plural
                || serde_json::to_string(&f)
                    .unwrap_or_default()
                    .contains("Source")
            {
                return None;
            }
            f
        }
    };
    Some(Effect::AddRestriction {
        restriction: Restriction::CantAttack(Filter::not(f)),
        duration,
    })
}

inventory::submit! { EffectPattern { name: "combat 4: only [creatures] can attack", priority: 90, parse: p_only_can_attack } }

/// "After the second main phase this turn, there's an additional combat phase followed by
/// an additional main phase." (World at War), "After the first combat phase this turn,
/// there's an additional combat phase." (Swinging Ship): the phases are added after that
/// phase of the turn (CR 500.8).
fn p_after_nth_phase(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let r = l.strip_prefix("after the ")?;
    let (nth, r) = [("first ", 1), ("second ", 2), ("third ", 3)]
        .iter()
        .find_map(|(w, n)| r.strip_prefix(w).map(|x| (*n, x)))?;
    let (main, r) = if let Some(x) = r.strip_prefix("main phase this turn, ") {
        (true, x)
    } else if let Some(x) = r.strip_prefix("combat phase this turn, ") {
        (false, x)
    } else {
        return None;
    };
    let r = r
        .strip_prefix("there's ")
        .or_else(|| r.strip_prefix("there is "))?;
    let parts = match r {
        "an additional combat phase" => vec![TurnPart::CombatPhase],
        "an additional combat phase followed by an additional main phase" => {
            vec![TurnPart::CombatPhase, TurnPart::MainPhase]
        }
        _ => return None,
    };
    Some(Effect::Custom(
        crate::kw::added_combat_phases::after_nth(main, nth, &parts)?.into(),
    ))
}

inventory::submit! { EffectPattern { name: "combat 4: after the nth phase this turn, there's an additional combat phase", priority: 90, parse: p_after_nth_phase } }

/// "there is an additional combat phase after this phase followed by an additional main
/// phase" (All-Out Assault): the phase named after "after this phase".
fn p_additional_after_this_followed_by(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (head, tail) = l.split_once(" after this phase followed by ")?;
    if !head.ends_with("additional combat phase") || tail != "an additional main phase" {
        return None;
    }
    crate::oracle::effects::parse_clause(
        &format!("{head} followed by an additional main phase after this phase"),
        b,
    )
}

inventory::submit! { EffectPattern { name: "combat 4: an additional combat phase after this phase followed by a main phase", priority: 90, parse: p_additional_after_this_followed_by } }

/// "there's an additional combat phase after this phase" (Moraug, Fury of Akoum).
fn p_theres_additional(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let r = l.strip_prefix("there's an additional ")?;
    crate::oracle::effects::parse_clause(&format!("there is an additional {r}"), b)
}

inventory::submit! { EffectPattern { name: "combat 4: there's an additional phase", priority: 90, parse: p_theres_additional } }

/// "When you next attack this turn, untap each creature you control." (All-Out Assault): a
/// delayed triggered ability that triggers the next time you attack this turn (CR 603.7,
/// 508.1).
fn p_when_you_next_attack(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let text = l.strip_prefix("when you next attack this turn, ")?;
    let body = super::r600_triggers::reflexive_body(text, b)?;
    Some(Effect::DelayedTrigger {
        trigger: TriggerCond::ThisTurn(Box::new(TriggerCond::PlayerAttacks(PlayerRel::You))),
        body: Box::new(body),
        once: true,
    })
}

inventory::submit! { EffectPattern { name: "combat 4: when you next attack this turn", priority: 90, parse: p_when_you_next_attack } }

// ---------------------------------------------------------------------------
// Combat history (see `kw::combat_history_4`)
// ---------------------------------------------------------------------------

/// "defending player controls three or more creatures that share a creature type".
fn c_defending_share_a_type(c: &str) -> Option<Condition> {
    use crate::kw::combat_history_4 as h;
    let c = end(c.trim());
    let r = c
        .strip_prefix("defending player controls ")?
        .strip_suffix(" creatures that share a creature type")?;
    let num = r.strip_suffix(" or more")?;
    let (n, rest) = crate::oracle::phrases::parse_number(num)?;
    if !rest.trim().is_empty() {
        return None;
    }
    let n = n.as_const()?;
    (n >= 2).then(|| Condition::Custom(format!("{}{n}", h::DEFENDING_SHARE_A_TYPE).into()))
}

inventory::submit! { super::ConditionPattern { name: "combat 4: defending player controls N creatures that share a creature type", priority: 100, parse: c_defending_share_a_type } }

/// "each creature it blocked this combat" (Cathedral Membrane): "it" is the source.
fn s_it_blocked_this_combat<'a>(s: &'a str, f: &Filter) -> Option<(Filter, &'a str)> {
    let r = s
        .strip_prefix("it blocked this combat")
        .or_else(|| s.strip_prefix("~ blocked this combat"))?;
    if !(r.is_empty() || r.starts_with([' ', ',', '.'])) {
        return None;
    }
    if !serde_json::to_string(f).is_ok_and(|j| j.contains("\"Creature\"")) {
        return None;
    }
    Some((
        Filter::Custom(crate::kw::combat_history_4::SOURCE_BLOCKED_THIS_COMBAT.into()),
        r,
    ))
}

inventory::submit! { super::FilterSuffixPattern { name: "combat 4: creature it blocked this combat", priority: 100, parse: s_it_blocked_this_combat } }

/// "Whenever a Wolf you control attacks, ... blocks that Wolf this combat if able"
/// (Tolsimir, Midnight's Light): "that Wolf" is the trigger's creature.
fn p_that_trigger_noun(l: &str, b: &mut Builder) -> Option<Effect> {
    if !b.in_trigger || !matches!(b.it, Sel::TriggerObject) {
        return None;
    }
    let raw = crate::oracle::raw_text().to_lowercase();
    let l = end(l.trim());
    let i = l.find(" that ")?;
    let word = l[i + " that ".len()..]
        .split(|c: char| !c.is_alphanumeric())
        .next()?;
    if word.is_empty()
        || matches!(word, "creature" | "player" | "card" | "permanent" | "spell" | "opponent")
        || !(raw.contains(&format!("whenever a {word} you control attacks"))
            || raw.contains(&format!("whenever an {word} you control attacks")))
    {
        return None;
    }
    let rewritten = l.replacen(&format!(" that {word}"), " it", 1);
    if rewritten.contains(&format!(" that {word}")) {
        return None;
    }
    crate::oracle::effects::parse_clause(&rewritten, b)
}

inventory::submit! { EffectPattern { name: "combat 4: that [type] (the attacking trigger creature)", priority: 95, parse: p_that_trigger_noun } }

/// Whether the last instruction of `e` is a delayed triggered ability that triggers at a
/// step of a combat phase ("at the beginning of that player's next declare attackers
/// step").
fn delayed_at_combat_step(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.last().is_some_and(delayed_at_combat_step),
        Effect::DelayedTrigger { trigger, .. } => {
            let s = serde_json::to_string(trigger).unwrap_or_default();
            [
                "\"BeginningOfCombat\"",
                "\"DeclareAttackers\"",
                "\"DeclareBlockers\"",
                "\"CombatDamage\"",
            ]
            .iter()
            .any(|w| s.contains(w))
                && !["\"End\"", "\"Upkeep\"", "\"Draw\"", "Main\"", "\"EndOfCombat\""]
                    .iter()
                    .any(|w| s.contains(w))
        }
        _ => false,
    }
}

/// "Return it to the battlefield ... tapped and attacking at the beginning of that
/// player's next declare attackers step. It can't be blocked that combat." (Noctis, Heir
/// Apparent): "that combat" is the combat of the step the delayed ability triggers in, so
/// the sentence continues the delayed ability with "this combat".
fn f_that_combat_after_delayed(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l.trim());
    if !l.contains(" that combat") || !delayed_at_combat_step(prev) {
        return false;
    }
    let rewritten = l.replace(" that combat", " this combat");
    super::followup_patterns()
        .iter()
        .filter(|p| p.name != "combat 4: that combat after a delayed combat-step ability")
        .any(|p| (p.apply)(&rewritten, prev, b))
}

inventory::submit! { FollowupPattern { name: "combat 4: that combat after a delayed combat-step ability", priority: 45, apply: f_that_combat_after_delayed } }

/// "choose target player and any number of target attacking creatures their opponents
/// control" (Portal Manipulator): the targets "those creatures" and "that player" refer to.
fn p_choose_player_and_attackers(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let r = l.strip_prefix("choose target player and ")?;
    let creatures = r.strip_suffix(" their opponents control")?;
    let (spec, tail) = crate::oracle::phrases::parse_target(creatures)?;
    let TargetKind::Object(f) = spec.what.clone() else {
        return None;
    };
    if !end(tail).is_empty() || matches!(spec.max, Value::Const(1)) {
        return None;
    }
    let (pspec, ptail) = crate::oracle::phrases::parse_target("target player")?;
    if !ptail.trim().is_empty() {
        return None;
    }
    let p = b.add_target(pspec, "target player");
    let mut spec = spec;
    spec.what = TargetKind::Object(Filter::and(vec![
        f,
        Filter::ControllerMatches(Box::new(PlayerFilter::OpponentOf(Box::new(
            PlayerRef::Target(p),
        )))),
    ]));
    let text = format!("{creatures} their opponents control");
    let c = b.add_target(spec, &text);
    b.it_player = PlayerRef::Target(p);
    b.named.push(("those creatures".into(), Sel::Target(c)));
    Some(Effect::Noop)
}

inventory::submit! { EffectPattern { name: "combat 4: choose target player and target attacking creatures their opponents control", priority: 90, parse: p_choose_player_and_attackers } }

/// "Those creatures are now attacking that player." (CR 508.7): each reselects what it's
/// attacking — that player — if it legally can.
fn p_now_attacking(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (who, player) = l.split_once(" are now attacking ")?;
    let (sel, rest) = object_ref(who, b)?;
    let Sel::Target(c) = sel else {
        return None;
    };
    if !rest.trim().is_empty() || player != "that player" {
        return None;
    }
    let PlayerRef::Target(p) = b.it_player else {
        return None;
    };
    Some(Effect::Custom(
        format!("{}{p}:{c}", crate::kw::combat_history_4::NOW_ATTACKING).into(),
    ))
}

inventory::submit! { EffectPattern { name: "combat 4: those creatures are now attacking that player", priority: 90, parse: p_now_attacking } }
