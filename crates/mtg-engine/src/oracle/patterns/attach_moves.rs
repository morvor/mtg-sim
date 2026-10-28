//! Attaching as cards move (round 4): Auras and Equipment entering or returning attached
//! to a host chosen as they move (CR 303.4f, 301.5b, 701.3).
//!
//! * "When enchanted creature dies, choose a creature at random ~ can enchant. Return ~ to
//!   the battlefield attached to that creature." (Infectious Rage): the random choice is
//!   among the creatures the Aura could enchant (`attach::SOURCE_CAN_ATTACH`).

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::CompileContext;
use std::sync::Arc;

/// "choose a creature at random ~ can enchant. Return ~ to the battlefield attached to
/// that creature."
fn random_host_it_can_enchant(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    const CHOOSE: &str = "choose a creature at random ~ can enchant. Return ~ to the battlefield attached to that creature.";
    let head = block.strip_suffix(CHOOSE)?;
    if block.contains('\n') {
        return None;
    }
    let rewritten = format!("{head}return ~ to the battlefield attached to a creature chosen at random.");
    let v = crate::oracle::parse_ability(&rewritten, ctx)?;
    let mut out = Vec::new();
    for a in v {
        let mut a = (*a).clone();
        let json = serde_json::to_string(&a.kind).ok()?;
        let from = serde_json::to_string(&Sel::AtRandom {
            filter: Filter::and(vec![
                Filter::Type(crate::types::CardType::Creature),
                Filter::InZone(ZoneKind::Battlefield),
            ]),
            count: Value::c(1),
            store: None,
        })
        .ok()?;
        let to = serde_json::to_string(&Sel::AtRandom {
            filter: Filter::and(vec![
                Filter::Type(crate::types::CardType::Creature),
                Filter::InZone(ZoneKind::Battlefield),
                Filter::Custom(crate::attach::SOURCE_CAN_ATTACH.into()),
            ]),
            count: Value::c(1),
            store: None,
        })
        .ok()?;
        if json.matches(&from).count() != 1 {
            return None;
        }
        a.kind = serde_json::from_str(&json.replace(&from, &to)).ok()?;
        a.text = block.to_string();
        out.push(Arc::new(a));
    }
    Some(out)
}

inventory::submit! { AbilityPattern { name: "attach moves: a creature at random ~ can enchant", priority: 40, parse: random_host_it_can_enchant } }

/// The player chosen at random to attach an Aura to.
const RANDOM_HOST: Var = vars::USER + 52020;

/// "Then attach ~ to another one of your opponents chosen at random." (Maddening Hex): a
/// Curse moved to one of the opponents other than the player it enchants, chosen at
/// random; with none, it stays (CR 701.3b).
fn attach_to_random_other_opponent(
    l: &str,
    _b: &mut crate::oracle::effects::Builder,
) -> Option<Effect> {
    let l = crate::oracle::phrases::end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    if l != "attach ~ to another one of your opponents chosen at random" {
        return None;
    }
    let enchanted = PlayerRef::ControllerOf(Box::new(Sel::AttachedTo));
    let others = PlayerRef::Each(PlayerFilter::And(vec![
        PlayerFilter::Opponent,
        PlayerFilter::Not(Box::new(PlayerFilter::Ref(Box::new(enchanted)))),
    ]));
    Some(Effect::seq(vec![
        crate::kw::choice_grammar::random_player_effect(RANDOM_HOST, others)?,
        Effect::Attach {
            what: Sel::This,
            to: Sel::Players(PlayerRef::Var(RANDOM_HOST)),
        },
    ]))
}

inventory::submit! { super::EffectPattern { name: "attach moves: attach ~ to another one of your opponents chosen at random", priority: 90, parse: attach_to_random_other_opponent } }

/// "Attach target Aura attached to a creature or land to another permanent of that type."
/// (Enchantment Alteration): to another creature if its host is a creature, else to
/// another land (CR 701.3a); a host that's both is treated as a creature, the first type
/// named.
fn to_another_permanent_of_that_type(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let head = block.strip_suffix(" to another permanent of that type.")?;
    if !ctx.is_spell() || block.contains('\n') {
        return None;
    }
    let types = head.strip_prefix("Attach target Aura attached to a ")?;
    let (t1, t2) = types.split_once(" or ")?;
    let mut branches = Vec::new();
    let mut targets = None;
    for t in [t1, t2] {
        let ty = crate::types::CardType::from_word(t)?;
        let v = crate::oracle::parse_ability(&format!("{head} to another {t}."), ctx)?;
        let [a] = v.as_slice() else {
            return None;
        };
        let AbilityKind::Spell(s) = &a.kind else {
            return None;
        };
        if !matches!(s.body.effect, Effect::Attach { .. }) {
            return None;
        }
        targets = Some(s.body.targets.clone());
        branches.push((ty, s.body.effect.clone()));
    }
    let [(ty1, e1), (_, e2)] = <[_; 2]>::try_from(branches).ok()?;
    let host = Sel::HostOf(Box::new(Sel::Target(0)));
    let effect = Effect::If {
        cond: Condition::SelMatches(host, Filter::Type(ty1)),
        then: Box::new(e1),
        otherwise: Box::new(e2),
    };
    let mut body = Body::effect(effect);
    body.targets = targets?;
    Some(vec![AbilityDef::new(
        AbilityKind::Spell(SpellAbility { body }),
        block,
    )])
}

inventory::submit! { AbilityPattern { name: "attach moves: attach target Aura to another permanent of that type", priority: 40, parse: to_another_permanent_of_that_type } }

/// The card a search found, for the instructions after it.
const FOUND: Var = vars::USER + 52021;

/// "When ~ enters, search your library for an Aura card that could enchant it. If ~ is
/// still on the battlefield, put that Aura card onto the battlefield attached to it.
/// Otherwise, reveal the Aura card and put it into your hand. Then shuffle." (Auratouched
/// Mage): whether the source is still the same permanent is checked after the search
/// (CR 400.7).
fn if_still_on_battlefield_attach(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    use crate::oracle::effects::{parse_clause, Builder};
    let (search, rest) = block.split_once(". If ~ is still on the battlefield, ")?;
    let (then, rest) = rest.split_once(". Otherwise, ")?;
    let (otherwise, tail) = rest.split_once(". ")?;
    if tail != "Then shuffle." || block.contains('\n') {
        return None;
    }
    if then != "put that Aura card onto the battlefield attached to it" {
        return None;
    }
    let v = crate::oracle::parse_ability(&format!("{search}."), ctx)?;
    let [a] = v.as_slice() else {
        return None;
    };
    let mut a = (**a).clone();
    let AbilityKind::Triggered(t) = &mut a.kind else {
        return None;
    };
    let Effect::SearchCards(spec) = &t.body.effect else {
        return None;
    };
    if !spec.dests.is_empty() || spec.shuffle != SearchShuffle::No {
        return None;
    }
    let mut b = Builder::new(ctx);
    b.in_trigger = true;
    b.it = Sel::Var(FOUND);
    let otherwise = otherwise
        .to_lowercase()
        .replace("the aura card", "it");
    let other = parse_clause(&otherwise, &mut b)?;
    if !b.targets.is_empty() {
        return None;
    }
    let mut to = Destination::battlefield();
    to.attached_to = Some(Sel::This);
    let search = t.body.effect.clone();
    t.body.effect = Effect::seq(vec![
        search,
        Effect::Store {
            var: FOUND,
            sel: Sel::Var(vars::IT),
        },
        Effect::If {
            cond: Condition::Custom(
                crate::kw::delayed_conditions::SOURCE_STILL_ON_BATTLEFIELD.into(),
            ),
            then: Box::new(Effect::Move {
                what: Sel::Var(FOUND),
                to,
            }),
            otherwise: Box::new(other),
        },
        Effect::Shuffle {
            who: PlayerRef::You,
        },
    ]);
    a.text = block.to_string();
    Some(vec![Arc::new(a)])
}

inventory::submit! { AbilityPattern { name: "attach moves: if ~ is still on the battlefield, put that Aura card onto the battlefield attached to it", priority: 40, parse: if_still_on_battlefield_attach } }

/// The search instruction of an ability's body: its filter and destination.
fn search_of(e: &mut Effect) -> Option<(&mut Filter, &mut Destination)> {
    match e {
        Effect::Search { filter, to, .. } => Some((filter, to)),
        Effect::Seq(v) => v.iter_mut().find_map(search_of),
        Effect::May { effect, .. } => search_of(effect),
        _ => None,
    }
}

/// "At the beginning of your upkeep, you may search your library for a Curse card that
/// doesn't have the same name as a Curse attached to enchanted player, put it onto the
/// battlefield attached to that player, then shuffle." (Curse of Misfortunes): a Curse put
/// onto the battlefield attached to a player (CR 303.4f), not one with the name of a Curse
/// already on them.
fn search_curse_attached_to_player(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    const NAME: &str = " that doesn't have the same name as a Curse attached to enchanted player";
    const ONTO: &str = "put it onto the battlefield attached to that player";
    if !block.contains(NAME) || !block.contains(ONTO) || block.contains('\n') {
        return None;
    }
    let rewritten = block
        .replacen(NAME, "", 1)
        .replacen(ONTO, "put it onto the battlefield", 1);
    let v = crate::oracle::parse_ability(&rewritten, ctx)?;
    let [a] = v.as_slice() else {
        return None;
    };
    let mut a = (**a).clone();
    let AbilityKind::Triggered(t) = &mut a.kind else {
        return None;
    };
    let (filter, to) = search_of(&mut t.body.effect)?;
    if to.zone != ZoneKind::Battlefield {
        return None;
    }
    let enchanted = PlayerRef::ControllerOf(Box::new(Sel::AttachedTo));
    let curses_there = Sel::All(Filter::and(vec![
        Filter::Subtype("Curse".into()),
        Filter::InZone(ZoneKind::Battlefield),
        Filter::AttachedTo(Box::new(Sel::Players(enchanted.clone()))),
    ]));
    *filter = Filter::and(vec![
        filter.clone(),
        Filter::not(Filter::SameNameAs(Box::new(curses_there))),
    ]);
    to.attached_to = Some(Sel::Players(enchanted));
    a.text = block.to_string();
    Some(vec![Arc::new(a)])
}

inventory::submit! { AbilityPattern { name: "attach moves: search for a Curse and put it onto the battlefield attached to that player", priority: 40, parse: search_curse_attached_to_player } }
