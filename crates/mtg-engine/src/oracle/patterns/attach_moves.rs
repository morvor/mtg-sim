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

const FLICKER_AURAS: Var = vars::USER + 52022;
const FLICKER_CREATURE: Var = vars::USER + 52023;
const FLICKER_OTHERS: Var = vars::USER + 52024;
const FLICKER_HOST: Var = vars::USER + 52029;

/// "Exile enchanted creature and all Auras attached to it. At the beginning of the next
/// end step, return that card to the battlefield under its owner's control. If you do,
/// return the other cards exiled this way to the battlefield under their owners' control
/// attached to that creature." (Flickerform): "that card" is the creature card; the Aura
/// cards return attached to the permanent it became (CR 400.7, 303.4f). The creature and
/// its Auras are exiled at the same time.
fn exile_with_auras_and_return_attached(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (cost, effect) = crate::oracle::split_cost(block)?;
    if effect.trim()
        != "Exile enchanted creature and all Auras attached to it. At the beginning of the next end step, return that card to the battlefield under its owner's control. If you do, return the other cards exiled this way to the battlefield under their owners' control attached to that creature."
    {
        return None;
    }
    // The ability itself, with a placeholder effect, for its cost and timing.
    let v = crate::oracle::parse_ability(&format!("{cost}: Draw a card."), ctx)?;
    let [a] = v.as_slice() else {
        return None;
    };
    let mut a = (**a).clone();
    let AbilityKind::Activated(act) = &mut a.kind else {
        return None;
    };
    let owner_control = |s: Sel| {
        let mut d = Destination::battlefield();
        d.controller = Some(PlayerRef::OwnerOf(Box::new(s)));
        d
    };
    let mut attached = owner_control(Sel::Var(FLICKER_OTHERS));
    attached.attached_to = Some(Sel::Var(vars::IT));
    let exiled = |v: Var| {
        Sel::All(Filter::and(vec![
            Filter::In(Box::new(Sel::Var(v))),
            Filter::InZone(ZoneKind::Exile),
        ]))
    };
    let delayed = Effect::DelayedTrigger {
        trigger: TriggerCond::BeginningOf {
            step: TriggerStep::End,
            whose: PlayerRel::Any,
        },
        body: Box::new(Body::effect(Effect::seq(vec![
            Effect::Move {
                what: exiled(FLICKER_CREATURE),
                to: owner_control(Sel::Var(FLICKER_CREATURE)),
            },
            Effect::If {
                cond: Condition::PrevHappened,
                then: Box::new(Effect::Move {
                    what: exiled(FLICKER_OTHERS),
                    to: attached,
                }),
                otherwise: Box::new(Effect::Noop),
            },
        ]))),
        once: true,
    };
    let was_host = crate::kw::attached_before::was_in_var(FLICKER_HOST);
    act.body = Body::effect(Effect::seq(vec![
        Effect::Store {
            var: FLICKER_HOST,
            sel: Sel::AttachedTo,
        },
        Effect::Store {
            var: FLICKER_AURAS,
            sel: Sel::All(Filter::and(vec![
                Filter::Subtype("Aura".into()),
                Filter::AttachedToAnyOf(Box::new(Sel::Var(FLICKER_HOST))),
                Filter::InZone(ZoneKind::Battlefield),
            ])),
        },
        Effect::Exile {
            what: Sel::Union(vec![Sel::Var(FLICKER_HOST), Sel::Var(FLICKER_AURAS)]),
            face_down: false,
            link: false,
        },
        Effect::Store {
            var: FLICKER_CREATURE,
            sel: Sel::Matching(Box::new(Sel::Var(vars::IT)), was_host.clone()),
        },
        Effect::Store {
            var: FLICKER_OTHERS,
            sel: Sel::Matching(Box::new(Sel::Var(vars::IT)), Filter::not(was_host)),
        },
        delayed,
    ]));
    a.text = block.to_string();
    Some(vec![Arc::new(a)])
}

inventory::submit! { AbilityPattern { name: "attach moves: exile enchanted creature and its Auras, return them attached", priority: 40, parse: exile_with_auras_and_return_attached } }

/// "[trigger], you may attach to it any number of Auras on the battlefield and you may put
/// onto the battlefield attached to it any number of Aura cards that could enchant it from
/// your graveyard and/or hand." (Bruna, Light of Alabaster): Auras chosen (not targeted)
/// are attached to the source (CR 701.3a: one that can't legally be stays where it is);
/// Aura cards that could enchant it are put onto the battlefield attached to it (CR
/// 303.4f).
fn attach_and_put_auras_onto_it(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    const BODY: &str = ", you may attach to it any number of Auras on the battlefield and you may put onto the battlefield attached to it any number of Aura cards that could enchant it from your graveyard and/or hand.";
    let trigger = block.strip_suffix(BODY)?;
    if block.contains('\n') || !trigger.starts_with("Whenever ~ ") {
        return None;
    }
    let v = crate::oracle::parse_ability(&format!("{trigger}, draw a card."), ctx)?;
    let [a] = v.as_slice() else {
        return None;
    };
    let mut a = (**a).clone();
    let AbilityKind::Triggered(t) = &mut a.kind else {
        return None;
    };
    let auras = Filter::and(vec![
        Filter::Subtype("Aura".into()),
        Filter::InZone(ZoneKind::Battlefield),
        Filter::not(Filter::AttachedTo(Box::new(Sel::This))),
    ]);
    let cards = Filter::and(vec![
        Filter::Subtype("Aura".into()),
        Filter::Card,
        Filter::Or(vec![
            Filter::InZone(ZoneKind::Graveyard),
            Filter::InZone(ZoneKind::Hand),
        ]),
        Filter::OwnedBy(PlayerRel::You),
        Filter::CouldAttachTo(Box::new(Sel::This)),
    ]);
    let any = |f: Filter| Sel::Choose {
        chooser: PlayerRef::You,
        count: Value::Count(f.clone()),
        filter: f,
        up_to: true,
        store: None,
    };
    let mut onto = Destination::battlefield();
    onto.attached_to = Some(Sel::This);
    t.body = Body::effect(Effect::seq(vec![
        Effect::Attach {
            what: any(auras),
            to: Sel::This,
        },
        Effect::Move {
            what: any(cards),
            to: onto,
        },
    ]));
    a.text = block.to_string();
    Some(vec![Arc::new(a)])
}

inventory::submit! { AbilityPattern { name: "attach moves: attach to it any number of Auras and put Aura cards onto the battlefield attached to it", priority: 40, parse: attach_and_put_auras_onto_it } }

/// "Whenever ~ or another creature you control dies, if it was enchanted or equipped,
/// return any number of Aura cards that were attached to it from your graveyard to the
/// battlefield attached to target creature, then attach any number of Equipment that were
/// attached to it to that creature." (Cass, Hand of Vengeance): the Auras and Equipment
/// attached to the creature as it last existed on the battlefield (CR 608.2h).
fn return_auras_and_equipment_that_were_attached(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    const BODY: &str = ", return any number of Aura cards that were attached to it from your graveyard to the battlefield attached to target creature, then attach any number of Equipment that were attached to it to that creature.";
    let trigger = block.strip_suffix(BODY)?;
    if block.contains('\n') {
        return None;
    }
    let v = crate::oracle::parse_ability(&format!("{trigger}, destroy target creature."), ctx)?;
    let [a] = v.as_slice() else {
        return None;
    };
    let mut a = (**a).clone();
    let AbilityKind::Triggered(t) = &mut a.kind else {
        return None;
    };
    if t.body.targets.len() != 1 || !matches!(t.body.effect, Effect::Destroy { .. }) {
        return None;
    }
    let any = |f: Filter| Sel::Choose {
        chooser: PlayerRef::You,
        count: Value::Count(f.clone()),
        filter: f,
        up_to: true,
        store: None,
    };
    let auras = Filter::and(vec![
        Filter::Subtype("Aura".into()),
        Filter::Card,
        Filter::InZone(ZoneKind::Graveyard),
        Filter::OwnedBy(PlayerRel::You),
        Filter::Custom(crate::kw::attached_before::WAS_ATTACHED_TO_TRIGGER_OBJECT.into()),
    ]);
    let equipment = Filter::and(vec![
        Filter::Subtype("Equipment".into()),
        Filter::InZone(ZoneKind::Battlefield),
        Filter::AttachedToAnyOf(Box::new(Sel::TriggerLki)),
    ]);
    let mut onto = Destination::battlefield();
    onto.attached_to = Some(Sel::Target(0));
    t.body.effect = Effect::seq(vec![
        Effect::Move {
            what: any(auras),
            to: onto,
        },
        Effect::Attach {
            what: any(equipment),
            to: Sel::Target(0),
        },
    ]);
    a.text = block.to_string();
    Some(vec![Arc::new(a)])
}

inventory::submit! { AbilityPattern { name: "attach moves: return Aura cards and attach Equipment that were attached to it", priority: 40, parse: return_auras_and_equipment_that_were_attached } }

const HERALD_RETURNED: Var = vars::USER + 52028;

/// "When ~ enters, return any number of Aura cards from your graveyard to the battlefield
/// attached to creatures you control. Exile those Auras at the beginning of your next end
/// step. If those Auras would leave the battlefield, exile them instead of putting them
/// anywhere else." (Storm Herald): for each Aura card chosen, a creature you control it
/// could enchant is chosen (CR 303.4f, not targeted), and they're all returned at the same
/// time, each attached to its creature (`kw/auras_to_chosen_hosts.rs`); an Aura with no
/// such creature stays in the graveyard (CR 303.4g).
fn return_auras_attached_to_creatures_you_control(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    use crate::kw::auras_to_chosen_hosts::{AURAS, HOSTS, PUT_AURAS_ONTO_CHOSEN_HOSTS};
    const BODY: &str = ", return any number of Aura cards from your graveyard to the battlefield attached to creatures you control. Exile those Auras at the beginning of your next end step. If those Auras would leave the battlefield, exile them instead of putting them anywhere else.";
    let trigger = block.strip_suffix(BODY)?;
    if block.contains('\n') {
        return None;
    }
    let v = crate::oracle::parse_ability(&format!("{trigger}, draw a card."), ctx)?;
    let [a] = v.as_slice() else {
        return None;
    };
    let mut a = (**a).clone();
    let AbilityKind::Triggered(t) = &mut a.kind else {
        return None;
    };
    let yours = Filter::and(vec![
        Filter::Type(crate::types::CardType::Creature),
        Filter::InZone(ZoneKind::Battlefield),
        Filter::ControlledBy(PlayerRel::You),
    ]);
    let auras = Filter::and(vec![
        Filter::Subtype("Aura".into()),
        Filter::Card,
        Filter::InZone(ZoneKind::Graveyard),
        Filter::OwnedBy(PlayerRel::You),
    ]);
    let returned = Filter::In(Box::new(Sel::Var(HERALD_RETURNED)));
    t.body = Body::effect(Effect::seq(vec![
        Effect::Store {
            var: AURAS,
            sel: Sel::Choose {
                chooser: PlayerRef::You,
                count: Value::Count(auras.clone()),
                filter: auras,
                up_to: true,
                store: None,
            },
        },
        Effect::Store {
            var: HOSTS,
            sel: Sel::All(yours),
        },
        Effect::Custom(PUT_AURAS_ONTO_CHOSEN_HOSTS.into()),
        Effect::Store {
            var: HERALD_RETURNED,
            sel: Sel::Var(vars::IT),
        },
        Effect::DelayedTrigger {
            trigger: TriggerCond::BeginningOf {
                step: TriggerStep::End,
                whose: PlayerRel::You,
            },
            body: Box::new(Body::effect(Effect::Exile {
                what: Sel::All(Filter::and(vec![
                    returned.clone(),
                    Filter::InZone(ZoneKind::Battlefield),
                ])),
                face_down: false,
                link: false,
            })),
            once: true,
        },
        Effect::AddReplacement {
            def: ReplacementDef {
                event: ReplacementEvent::ZoneChange {
                    filter: returned,
                    from: Some(ZoneKind::Battlefield),
                    to: None,
                },
                action: ReplacementAction::MoveInstead(Destination::zone(ZoneKind::Exile)),
                self_replacement: false,
                optional: false,
            },
            duration: Duration::Permanent,
            uses: None,
        },
    ]));
    a.text = block.to_string();
    Some(vec![Arc::new(a)])
}

inventory::submit! { AbilityPattern { name: "attach moves: return Aura cards attached to creatures you control", priority: 40, parse: return_auras_attached_to_creatures_you_control } }

/// "[trigger], you may pay {1}{G} if ~ is attached to a creature you control. If you do,
/// create a token that's a copy of that creature. If you didn't create a token this way,
/// create a 1/1 green Insect creature token." (Springheart Nantuko): the payment is
/// offered only while it's attached to a creature you control; "that creature" is the
/// enchanted creature; otherwise (no payment, or no token created) the Insect.
fn pay_if_attached_then_copy(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (trigger, rest) = block.split_once(", you may pay ")?;
    let (cost, rest) = rest.split_once(" if ~ is attached to a creature you control. If you do, ")?;
    let (copy, other) = rest.split_once(". If you didn't create a token this way, ")?;
    if block.contains('\n') || copy != "create a token that's a copy of that creature" {
        return None;
    }
    let paid = crate::oracle::parse_ability(
        &format!("{trigger}, you may pay {cost}. If you do, create a token that's a copy of enchanted creature."),
        ctx,
    )?;
    let fallback = crate::oracle::parse_ability(&format!("{trigger}, {other}"), ctx)?;
    let ([p], [f]) = (paid.as_slice(), fallback.as_slice()) else {
        return None;
    };
    let (AbilityKind::Triggered(pt), AbilityKind::Triggered(ft)) = (&p.kind, &f.kind) else {
        return None;
    };
    let pay = pt.body.effect.clone();
    if !matches!(pay, Effect::PayOptional { .. }) || !matches!(ft.body.effect, Effect::CreateToken { .. }) {
        return None;
    }
    let attached = Condition::SelMatches(
        Sel::AttachedTo,
        Filter::and(vec![
            Filter::Type(crate::types::CardType::Creature),
            Filter::ControlledBy(PlayerRel::You),
        ]),
    );
    let mut a = (**p).clone();
    let AbilityKind::Triggered(t) = &mut a.kind else {
        return None;
    };
    t.body.effect = Effect::seq(vec![
        Effect::Store {
            var: vars::CREATED,
            sel: Sel::Union(vec![]),
        },
        Effect::If {
            cond: attached,
            then: Box::new(pay),
            otherwise: Box::new(Effect::Noop),
        },
        Effect::If {
            cond: Condition::Not(Box::new(Condition::SelNonEmpty(Sel::Var(vars::CREATED)))),
            then: Box::new(ft.body.effect.clone()),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    a.text = block.to_string();
    Some(vec![Arc::new(a)])
}

inventory::submit! { AbilityPattern { name: "attach moves: you may pay if ~ is attached to a creature you control", priority: 40, parse: pay_if_attached_then_copy } }
