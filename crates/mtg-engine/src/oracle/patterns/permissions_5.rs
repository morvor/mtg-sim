//! Permissions grammar V (round 4): permissions over cards a player exiled (exile
//! provenance, `kw/exile_provenance.rs`), and the draw replacements that feed them.
//!
//! * "You may play lands and cast noncreature spells from among cards you exiled that have
//!   fetch counters on them, ..." (Haldan, Avid Arcanist): the cards you put into exile.
//! * "Once each turn, you may play a card from exile with a collection counter on it if it
//!   was exiled by an ability you controlled, ..." (Evelyn, the Covetous).
//! * "Each player may play lands and cast spells from among cards they exiled with ~ this
//!   turn." (Uba Mask), "Each player may look at cards they exiled with ~, and they may
//!   play lands and cast spells from among those cards." (Shared Fate): each player's
//!   permission for the cards linked to the source that they exiled (CR 607.2a).
//! * "If a player would draw a card, that player exiles that card face up instead." (Uba
//!   Mask: the card they would draw), "If a player would draw a card, that player exiles
//!   the top card of one of their opponents' libraries face down instead." (Shared Fate).
//!
//! Each is read as the "you" form the permission grammar knows, then given the player
//! who exiled the cards.

use super::AbilityPattern;
use crate::ability::*;
use crate::kw::exile_provenance::{EACH_MAY_LOOK_AT_OWN_EXILED, EXILED_BY_THEM, EXILED_BY_YOU};
use crate::oracle::CompileContext;
use std::sync::Arc;

/// Compiles `rewritten` as the block `block` and changes each ability with `f`; the
/// abilities keep the block's text.
fn compile_as(
    rewritten: &str,
    block: &str,
    ctx: &CompileContext,
    f: impl Fn(&mut AbilityDef) -> Option<()>,
) -> Option<Vec<Ability>> {
    let v = crate::oracle::parse_ability(rewritten, ctx)?;
    let mut out = Vec::new();
    for a in v {
        let mut a = (*a).clone();
        f(&mut a)?;
        a.text = block.to_string();
        out.push(Arc::new(a));
    }
    Some(out)
}

/// The play permission of a static ability.
fn permission(a: &mut AbilityDef) -> Option<&mut PlayPermission> {
    match &mut a.kind {
        AbilityKind::Static(s) => match &mut s.effect {
            StaticEffect::PlayPermission(pp) => Some(pp),
            _ => None,
        },
        _ => None,
    }
}

fn add_quality(pp: &mut PlayPermission, q: Filter) {
    let what = std::mem::replace(&mut pp.what, Filter::Any);
    pp.what = Filter::and(vec![what, q]);
}

/// "... from among cards you exiled that have fetch counters on them ..." (Haldan, Avid
/// Arcanist), "... a card from exile with a collection counter on it if it was exiled by
/// an ability you controlled ..." (Evelyn, the Covetous).
fn cards_you_exiled(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if block.contains('\n') {
        return None;
    }
    let (rewritten, owned) = if block.contains(" from among cards you exiled ") {
        (
            block.replacen(" from among cards you exiled ", " from among cards you own in exile ", 1),
            true,
        )
    } else if block.contains(" if it was exiled by an ability you controlled") {
        (block.replacen(" if it was exiled by an ability you controlled", "", 1), false)
    } else {
        return None;
    };
    compile_as(&rewritten, block, ctx, |a| {
        let pp = permission(a)?;
        if pp.zone != ZoneKind::Exile || pp.who != PlayerRel::You {
            return None;
        }
        if owned {
            // Not only cards you own: the cards you exiled.
            let Filter::And(v) = &mut pp.what else {
                return None;
            };
            let i = v
                .iter()
                .position(|f| matches!(f, Filter::OwnedBy(PlayerRel::You)))?;
            v[i] = Filter::Custom(EXILED_BY_YOU.into());
        } else {
            add_quality(pp, Filter::Custom(EXILED_BY_YOU.into()));
        }
        Some(())
    })
}

inventory::submit! { AbilityPattern { name: "permissions 5: cards you exiled", priority: 40, parse: cards_you_exiled } }

/// "Each player may play lands and cast spells from among cards they exiled with ~ this
/// turn.", "Each player may look at cards they exiled with ~, and they may play lands and
/// cast spells from among those cards.": each player's permission for the cards linked to
/// the source that they exiled.
fn each_player_cards_they_exiled(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if block.contains('\n') {
        return None;
    }
    let rewritten = if let Some(r) = block.strip_prefix("Each player may look at cards they exiled with ~, and they may ") {
        format!("You may look at cards exiled with ~, and you may {r}")
    } else {
        let r = block.strip_prefix("Each player may ")?;
        if !r.contains(" from among cards they exiled with ~") {
            return None;
        }
        format!(
            "You may {}",
            r.replacen(" from among cards they exiled with ~", " from among cards exiled with ~", 1)
        )
    };
    compile_as(&rewritten, block, ctx, |a| {
        if let AbilityKind::Static(s) = &mut a.kind {
            if let StaticEffect::Custom(n) = &mut s.effect {
                if n == crate::kw::look_at_exiled_with::MAY_LOOK_AT_LINKED {
                    *n = EACH_MAY_LOOK_AT_OWN_EXILED.into();
                    return Some(());
                }
                return None;
            }
        }
        let pp = permission(a)?;
        if pp.zone != ZoneKind::Exile || pp.who != PlayerRel::You {
            return None;
        }
        // The cards linked to the source (CR 607.2a) that this player exiled.
        if !format!("{:?}", pp.what).contains("In(Linked)") {
            return None;
        }
        pp.who = PlayerRel::Any;
        add_quality(pp, Filter::Custom(EXILED_BY_THEM.into()));
        Some(())
    })
}

inventory::submit! { AbilityPattern { name: "permissions 5: each player, cards they exiled with ~", priority: 40, parse: each_player_cards_they_exiled } }

/// "If a player would draw a card, that player exiles that card face up instead." (Uba
/// Mask): the card they would draw, the top card of their library, exiled by them with
/// the source (CR 614.6, 607.2a).
fn draw_exiles_that_card(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = block.strip_prefix("If a player would draw a card, that player exiles ")?;
    if r != "that card face up instead." && r != "that card instead." {
        return None;
    }
    compile_as(
        "If a player would draw a card, that player exiles the top card of their library instead.",
        block,
        ctx,
        |_| Some(()),
    )
}

inventory::submit! { AbilityPattern { name: "permissions 5: a player exiles the card they would draw instead", priority: 40, parse: draw_exiles_that_card } }

/// "If a player would draw a card, that player exiles the top card of one of their
/// opponents' libraries face down instead." (Shared Fate): the player chooses one of their
/// opponents and exiles that player's top card face down, with the source (CR 406.3,
/// 607.2a).
fn draw_exiles_opponents_top(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if block
        != "If a player would draw a card, that player exiles the top card of one of their opponents' libraries face down instead."
    {
        return None;
    }
    let instead = Effect::AsPlayer {
        who: PlayerRef::TriggerPlayer,
        effect: Box::new(Effect::seq(vec![
            Effect::Choose {
                who: PlayerRef::You,
                kind: ChoiceKind::Opponent,
            },
            Effect::Exile {
                what: Sel::TopOfLibrary(PlayerRef::ChosenOpponent, Value::c(1)),
                face_down: true,
                link: true,
            },
        ])),
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::Draw(PlayerFilter::Any),
                action: ReplacementAction::Instead(Box::new(instead)),
                self_replacement: false,
                optional: false,
            },
        ))),
        block,
    )])
}

inventory::submit! { AbilityPattern { name: "permissions 5: a player exiles an opponent's top card face down instead of drawing", priority: 40, parse: draw_exiles_opponents_top } }

/// The triggered ability's body effect.
fn trigger_body(a: &mut AbilityDef) -> Option<&mut Effect> {
    match &mut a.kind {
        AbilityKind::Triggered(t) => Some(&mut t.body.effect),
        _ => None,
    }
}

/// "Whenever a creature deals combat damage to one of your opponents, its controller looks
/// at the top card of that opponent's library and exiles it face down. They may play that
/// card for as long as it remains exiled. Mana of any type can be spent to cast a spell
/// this way." (Gonti, Night Minister): the creature's controller does it all as if they
/// were "you" (they look, exile, and get the permission and its terms); the card is
/// exiled by them.
fn its_controller_looks_and_exiles(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (trigger, rest) = block.split_once(", its controller looks at ")?;
    if !trigger.starts_with("Whenever ") || block.contains('\n') {
        return None;
    }
    let (first, others) = rest.split_once(". ")?;
    let (what, how) = first.split_once(" and exiles it")?;
    let what = what.replace("that opponent's library", "that player's library");
    let others = others.strip_prefix("They may ")?;
    let rewritten = format!(
        "{trigger}, look at {what}, then exile it{how}. You may {others}"
    );
    // Who "its controller" is, as the trigger's text reads it.
    let probe = crate::oracle::parse_ability(
        &format!("{trigger}, its controller draws a card."),
        ctx,
    )?;
    let mut probe = (*probe.first()?.clone()).clone();
    let Effect::Draw { who, .. } = trigger_body(&mut probe)? else {
        return None;
    };
    let who = who.clone();
    if !matches!(who, PlayerRef::ControllerOf(_)) {
        return None;
    }
    compile_as(&rewritten, block, ctx, |a| {
        let body = trigger_body(a)?;
        let e = std::mem::replace(body, Effect::Noop);
        *body = Effect::AsPlayer {
            who: who.clone(),
            effect: Box::new(e),
        };
        Some(())
    })
}

inventory::submit! { AbilityPattern { name: "permissions 5: its controller looks at and exiles a card; they may play it", priority: 40, parse: its_controller_looks_and_exiles } }

/// Replaces `from` (serialized) with `to` in the ability's kind; false if `from` isn't
/// there exactly `times` times.
fn replace_in_kind<T: serde::Serialize>(a: &mut AbilityDef, from: &T, to: &T, times: usize) -> Option<()> {
    let json = serde_json::to_string(&a.kind).ok()?;
    let from = serde_json::to_string(from).ok()?;
    let to = serde_json::to_string(to).ok()?;
    if json.matches(&from).count() != times {
        return None;
    }
    a.kind = serde_json::from_str(&json.replace(&from, &to)).ok()?;
    Some(())
}

/// "If the player does, they may cast a spell from among other cards exiled with ~
/// without paying its mana cost." (Knowledge Pool): the cards exiled with the source but
/// the one just exiled (CR 607.2a).
fn other_cards_exiled_with(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    const OTHER: &str = " from among other cards exiled with ~";
    if !block.contains(OTHER) || block.contains('\n') {
        return None;
    }
    let rewritten = block.replacen(OTHER, " from among cards exiled with ~", 1);
    compile_as(&rewritten, block, ctx, |a| {
        let linked = Filter::In(Box::new(Sel::Linked));
        let other = Filter::And(vec![
            Filter::In(Box::new(Sel::Linked)),
            Filter::not(Filter::In(Box::new(Sel::Var(vars::IT)))),
        ]);
        // The instruction just before it exiled the card ("that player exiles it").
        if !serde_json::to_string(&a.kind).ok()?.contains("\"Exile\"") {
            return None;
        }
        replace_in_kind(a, &linked, &other, 1)
    })
}

inventory::submit! { AbilityPattern { name: "permissions 5: from among other cards exiled with ~", priority: 40, parse: other_cards_exiled_with } }

/// "For as long as you control ~, you may play one of the exiled cards, ..." (Kotose, the
/// Silent Spider) after several instructions that exiled cards: one of all the cards the
/// ability exiled (with its source, CR 607.2a), not only the last instruction's.
fn one_of_the_exiled_cards(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    const LEAD: &str = "For as long as you control ~, you may play one of the exiled cards";
    let (before, after) = block.split_once(LEAD)?;
    if block.contains('\n') || before.matches("exile").count() < 2 {
        return None;
    }
    let rewritten = format!("{before}You may play one of the exiled cards for as long as you control ~{after}");
    compile_as(&rewritten, block, ctx, |a| {
        let json = serde_json::to_string(&a.kind).ok()?;
        let grant = "\"GrantPlayPermission\":{\"who\":\"You\",\"what\":{\"Var\":0}";
        if json.matches(grant).count() != 1 || json.matches("\"Exile\"").count() < 2 {
            return None;
        }
        let all = serde_json::to_string(&Sel::All(Filter::and(vec![
            Filter::In(Box::new(Sel::Linked)),
            Filter::InZone(ZoneKind::Exile),
        ])))
        .ok()?;
        let new = format!("\"GrantPlayPermission\":{{\"who\":\"You\",\"what\":{all}");
        a.kind = serde_json::from_str(&json.replace(grant, &new)).ok()?;
        Some(())
    })
}

inventory::submit! { AbilityPattern { name: "permissions 5: for as long as you control ~, you may play one of the exiled cards", priority: 40, parse: one_of_the_exiled_cards } }

/// "During each player's turn, that player may play a land or cast a spell from among cards
/// exiled with ~, ... When they do, exile the top card of their library." (Share the
/// Spoils): a triggered ability of the source that triggers when any player plays a card
/// with its permission (CR 603.2, 601.3); "they" is that player.
fn when_they_do(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    use crate::oracle::effects::{parse_effect_text, Builder};
    let (perm, then) = block.split_once(". When they do, ")?;
    if block.contains('\n') || !perm.starts_with("During each player's turn, that player may ") {
        return None;
    }
    let mut v = crate::oracle::parse_ability(&format!("{perm}."), ctx)?;
    let (lands, spells) = match v.as_slice() {
        [a] => match &a.kind {
            AbilityKind::Static(s) => match &s.effect {
                StaticEffect::PlayPermission(pp) if pp.who == PlayerRel::Active => {
                    (pp.lands, pp.spells)
                }
                _ => return None,
            },
            _ => return None,
        },
        _ => return None,
    };
    let mut b = Builder::new(ctx);
    b.in_trigger = true;
    b.it = Sel::TriggerObject;
    b.it_player = PlayerRef::TriggerPlayer;
    let e = parse_effect_text(then, &mut b)?;
    if !b.targets.is_empty() {
        return None;
    }
    let this_way = Filter::Custom(crate::permissions::PLAYED_THIS_WAY.into());
    let mut conds = Vec::new();
    if spells {
        conds.push(TriggerCond::CastSpell {
            who: PlayerRel::Any,
            filter: this_way.clone(),
        });
    }
    if lands {
        conds.push(TriggerCond::LandPlayed {
            who: PlayerRel::Any,
            filter: this_way,
        });
    }
    let trigger = match conds.len() {
        1 => conds.pop()?,
        _ => TriggerCond::AnyOf(conds),
    };
    let tr = TriggeredAbility::new(trigger, Body::effect(e));
    let mut out: Vec<Ability> = v
        .drain(..)
        .map(|a| {
            let mut a = (*a).clone();
            a.text = block.to_string();
            Arc::new(a)
        })
        .collect();
    out.push(AbilityDef::new(AbilityKind::Triggered(tr), block));
    Some(out)
}

inventory::submit! { AbilityPattern { name: "permissions 5: [each player's permission]. when they do, ...", priority: 40, parse: when_they_do } }

/// "Until end of turn, for each nonland card type, you may cast a spell of that type from
/// among the exiled cards without paying its mana cost." (Aminatou's Augury): one
/// permission for each nonland card type, each usable once; a card of several types may be
/// cast with any one of its types' permissions (its ruling; CR 601.3: the player chooses
/// which permission they use).
fn for_each_card_type(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    const EACH: &str = "for each nonland card type, you may cast a spell of that type from among";
    if !block.contains(EACH) || block.contains('\n') {
        return None;
    }
    let rewritten = block.replacen(EACH, "you may cast a spell from among", 1);
    compile_as(&rewritten, block, ctx, |a| {
        let AbilityKind::Spell(s) = &mut a.kind else {
            return None;
        };
        let Effect::Seq(v) = &mut s.body.effect else {
            return None;
        };
        // "the exiled cards": those the first instruction exiled from the top of the
        // library (`vars::DUG`) that are still there, not what a later instruction took.
        if !matches!(v.first(), Some(Effect::Exile { what: Sel::TopOfLibrary(..), .. })) {
            return None;
        }
        let last = v.last_mut()?;
        let Effect::WithPlayTerms { terms, effect } = last else {
            return None;
        };
        if terms.limit != Some(1) {
            return None;
        }
        let Effect::GrantPlayPermission {
            who,
            what: Sel::Var(vars::IT),
            duration,
            free,
        } = effect.as_ref()
        else {
            return None;
        };
        let effect = Box::new(Effect::GrantPlayPermission {
            who: who.clone(),
            what: Sel::All(Filter::and(vec![
                Filter::In(Box::new(Sel::Var(vars::DUG))),
                Filter::InZone(ZoneKind::Exile),
            ])),
            duration: duration.clone(),
            free: *free,
        });
        let mut per_type = Vec::new();
        for t in crate::types::CardType::ALL {
            use crate::types::CardType::*;
            if !matches!(
                t,
                Artifact | Battle | Creature | Enchantment | Instant | Kindred | Planeswalker | Sorcery
            ) {
                continue;
            }
            let mut terms = terms.clone();
            terms.what = Some(match terms.what.take() {
                Some(w) => Filter::and(vec![w, Filter::Type(t)]),
                None => Filter::Type(t),
            });
            per_type.push(Effect::WithPlayTerms {
                terms,
                effect: effect.clone(),
            });
        }
        *last = Effect::Seq(per_type);
        Some(())
    })
}

inventory::submit! { AbilityPattern { name: "permissions 5: for each nonland card type, a spell of that type", priority: 40, parse: for_each_card_type } }

/// "Spell mastery — If there are two or more instant and/or sorcery cards in your
/// graveyard, you may cast up to two instant and/or sorcery spells from among the revealed
/// cards instead of one." (Talent of the Telepath): a paragraph that changes how many
/// cards the earlier paragraph's permission casts (CR 113.3a, 207.2c: the spell's text is
/// one set of instructions). Joined to the paragraph before it (see `cast_up_to_instead`).
fn join_instead_of_one(blocks: Vec<String>, ctx: &CompileContext) -> Vec<String> {
    if !ctx.is_spell() {
        return blocks;
    }
    let mut out: Vec<String> = Vec::new();
    for b in blocks {
        let rest = crate::oracle::strip_ability_word(&b);
        let joins = rest.len() < b.len()
            && rest.starts_with("If ")
            && rest.ends_with(" instead of one.")
            && rest.contains(", you may cast up to ")
            && out.last().is_some_and(|p| !p.contains('\n') && p.ends_with('.'));
        if joins {
            let prev = out.pop().unwrap_or_default();
            out.push(format!("{prev} {rest}"));
        } else {
            out.push(b);
        }
    }
    out
}

inventory::submit! { super::BlockGroupPattern { name: "permissions 5: ... cast up to N ... instead of one", priority: 85, group: join_instead_of_one } }

/// How many resolution-time casts an effect has.
fn count_casts(e: &Effect) -> usize {
    match e {
        Effect::CastCard { .. } => 1,
        Effect::Seq(v) => v.iter().map(count_casts).sum(),
        _ => 0,
    }
}

/// The first resolution-time cast of an effect.
fn first_cast(e: &mut Effect) -> Option<&mut Effect> {
    match e {
        Effect::CastCard { .. } => Some(e),
        Effect::Seq(v) => v.iter_mut().find_map(first_cast),
        _ => None,
    }
}

/// "[instructions with a cast]. If [condition], you may cast up to two [spells] from among
/// the revealed cards instead of one.": the earlier "you may cast a [spell] from among
/// them" casts up to that many if the condition holds as the spell resolves (Talent of the
/// Telepath's ruling).
fn cast_up_to_instead(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    use crate::oracle::phrases::parse_number;
    if !ctx.is_spell() || block.contains('\n') {
        return None;
    }
    let (first, last) = block.rsplit_once(". If ")?;
    let (c, r) = last.split_once(", you may cast up to ")?;
    let r = r.strip_suffix(" from among the revealed cards instead of one.")?;
    let (n, what) = parse_number(r)?;
    if n.as_const().is_none_or(|n| n < 2) {
        return None;
    }
    let cond = crate::oracle::statics::parse_condition(&c.to_lowercase(), ctx)?;
    compile_as(&format!("{first}."), block, ctx, |a| {
        let AbilityKind::Spell(s) = &mut a.kind else {
            return None;
        };
        let prev = &mut s.body.effect;
        if count_casts(prev) != 1 {
            return None;
        }
        let one = first_cast(prev)?;
        let Effect::CastCard {
            what:
                Sel::Choose {
                    filter,
                    count: Value::Const(1),
                    up_to: true,
                    ..
                },
            ..
        } = &*one
        else {
            return None;
        };
        // The spells are of the kind the earlier permission named ("instant and/or
        // sorcery spells" for "an instant or sorcery spell").
        let f = format!("{filter:?}");
        let kinds_ok = what
            .trim()
            .trim_end_matches(" spells")
            .split(" and/or ")
            .all(|k| {
                crate::types::CardType::from_word(k)
                    .is_some_and(|t| f.contains(&format!("Type({t:?})")))
            });
        if !kinds_ok {
            return None;
        }
        let mut more = one.clone();
        if let Effect::CastCard {
            what: Sel::Choose { count, .. },
            ..
        } = &mut more
        {
            *count = n.clone();
        }
        let orig = std::mem::replace(one, Effect::Noop);
        *one = Effect::If {
            cond: cond.clone(),
            then: Box::new(more),
            otherwise: Box::new(orig),
        };
        Some(())
    })
}

inventory::submit! { AbilityPattern { name: "permissions 5: if [condition], you may cast up to N ... instead of one", priority: 40, parse: cast_up_to_instead } }

/// "Each opponent reveals their hand. For each opponent, exile a creature card they
/// revealed this way until ~ leaves the battlefield." (Valki, God of Lies): the cards each
/// opponent revealed are the cards in that player's hand (CR 701.20a), so the choice is a
/// card from that player's hand.
fn card_they_revealed_this_way(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    const REVEAL: &str = "each opponent reveals their hand. For each opponent, ";
    if block.contains('\n') || !block.contains(REVEAL) || !block.contains(" card they revealed this way") {
        return None;
    }
    let (_, after) = block.split_once(REVEAL)?;
    // Only one reference, in the sentence right after the reveal.
    if after.matches(" they revealed this way").count() != 1 {
        return None;
    }
    let rewritten = block.replacen(" card they revealed this way", " card from that player's hand", 1);
    compile_as(&rewritten, block, ctx, |_| Some(()))
}

inventory::submit! { AbilityPattern { name: "permissions 5: a card they revealed this way (their hand)", priority: 40, parse: card_they_revealed_this_way } }
