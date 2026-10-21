//! Instructions after a replacement effect's modified event (`ReplacementAction::Then`):
//!
//! * "If a creature an opponent controls would die, exile it instead. When you do, create
//!   a 2/2 green Wolf creature token." (Head of the Hunt, Nemata, Valentin): a reflexive
//!   triggered ability each time the replacement applies (CR 603.12). The creature is
//!   exiled instead of dying, so abilities that trigger when it dies don't trigger
//!   (Head of the Hunt ruling); if another effect is applied instead, this one doesn't
//!   trigger (Nemata ruling).
//! * "If a permanent you control would be put into a graveyard from the battlefield this
//!   turn, exile it instead. Return it to the battlefield under its owner's control at the
//!   beginning of the next end step." (Cosmic Intervention): each permanent exiled this
//!   way creates its own delayed triggered ability to return it (its ruling, CR 603.7).

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::CompileContext;

/// The replacement effect the ability creates or is, if it's one.
fn replacement_of(a: &mut Ability) -> Option<&mut ReplacementDef> {
    let a = std::sync::Arc::make_mut(a);
    match &mut a.kind {
        AbilityKind::Static(s) => match &mut s.effect {
            StaticEffect::Replacement(def) => Some(def),
            _ => None,
        },
        AbilityKind::Spell(sp) => match &mut sp.body.effect {
            Effect::AddReplacement { def, .. } => Some(def),
            _ => None,
        },
        _ => None,
    }
}

fn then_instructions(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    if text.contains('\n') || !text.starts_with("If ") {
        return None;
    }
    let (head, tail, reflexive) = match text.split_once(" instead. When you do, ") {
        Some((h, t)) => (format!("{h} instead."), t, true),
        None => {
            let (h, _) = text.split_once(" instead. Return it ")?;
            (
                format!("{h} instead."),
                &text[h.len() + " instead. ".len()..],
                false,
            )
        }
    };
    let mut abilities = crate::oracle::parse_ability(&head, ctx)?;
    let [a] = abilities.as_mut_slice() else {
        return None;
    };
    let def = replacement_of(a)?;
    if !matches!(def.action, ReplacementAction::MoveInstead(_)) {
        return None;
    }
    let mut b = Builder::new(ctx);
    b.in_trigger = reflexive;
    b.it = Sel::Var(vars::IT);
    let effect = parse_effect_text(tail, &mut b)?;
    let then = if reflexive {
        Effect::Reflexive {
            body: Box::new(Body {
                targets: b.targets,
                effect,
                modal: None,
            }),
        }
    } else {
        // A delayed triggered ability: no targets of its own now.
        if !b.targets.is_empty() || !is_delayed(&effect) {
            return None;
        }
        effect
    };
    let action = std::mem::replace(&mut def.action, ReplacementAction::Prevent);
    def.action = ReplacementAction::Then(Box::new(action), Box::new(then));
    let a = std::sync::Arc::make_mut(a);
    a.text = text.to_string();
    Some(abilities)
}

/// Whether the instruction is a delayed triggered ability ("at the beginning of the next
/// end step").
fn is_delayed(e: &Effect) -> bool {
    matches!(e, Effect::AtNext { .. } | Effect::DelayedTrigger { .. })
}

inventory::submit! { AbilityPattern { name: "replacement bodies: [replacement] instead. When you do / Return it ...", priority: 70, parse: then_instructions } }

// ---------------------------------------------------------------------------
// Draw replacements with several instructions (CR 614.6, 614.11)
// ---------------------------------------------------------------------------

/// The static draw replacement of a parsed ability, with its replacing instructions.
fn draw_instead(abilities: &mut [Ability]) -> Option<&mut Effect> {
    let [a] = abilities else {
        return None;
    };
    let a = std::sync::Arc::make_mut(a);
    let AbilityKind::Static(s) = &mut a.kind else {
        return None;
    };
    let StaticEffect::Replacement(ReplacementDef {
        event: ReplacementEvent::Draw(_),
        action: ReplacementAction::Instead(e),
        ..
    }) = &mut s.effect
    else {
        return None;
    };
    Some(e)
}

/// "If you would draw a card, reveal the top card of your library instead. If it's a
/// creature card, put it into your graveyard. Otherwise, draw a card." (Enduring Renewal):
/// the revealed card is checked; otherwise the draw happens after all (and is the card
/// revealed, its ruling). The draw the instructions perform isn't replaced again by this
/// effect (CR 614.5).
fn reveal_top_instead(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let (head, rest) = text.split_once(" instead. If it's ")?;
    let (quality, rest) = rest.split_once(", put it into ")?;
    let (dest, rest) = rest.split_once(". ")?;
    if rest != "Otherwise, draw a card." {
        return None;
    }
    let to = match dest {
        "your graveyard" => Destination::zone(ZoneKind::Graveyard),
        "your hand" => Destination::zone(ZoneKind::Hand),
        _ => return None,
    };
    if !head.ends_with("reveal the top card of your library") {
        return None;
    }
    let quality = quality.to_lowercase();
    let quality = quality
        .strip_prefix("a ")
        .or_else(|| quality.strip_prefix("an "))?;
    let (f, _, tail) = crate::oracle::phrases::parse_object_phrase(quality)?;
    if !crate::oracle::phrases::end(tail).is_empty() {
        return None;
    }
    let mut abilities = crate::oracle::parse_ability(&format!("{head} instead."), ctx)?;
    let e = draw_instead(&mut abilities)?;
    let top = Sel::TopOfLibrary(PlayerRef::You, Value::c(1));
    *e = Effect::seq(vec![
        e.clone(),
        Effect::If {
            cond: Condition::SelMatches(top.clone(), f),
            then: Box::new(Effect::Move { what: top, to }),
            otherwise: Box::new(Effect::Draw {
                who: PlayerRef::You,
                n: Value::c(1),
            }),
        },
    ]);
    let a = std::sync::Arc::make_mut(&mut abilities[0]);
    a.text = text.to_string();
    Some(abilities)
}

inventory::submit! { AbilityPattern { name: "replacement bodies: reveal the top card instead; if it's a [card], put it ...; otherwise, draw", priority: 70, parse: reveal_top_instead } }

/// "If a player would draw a card, instead they draw a card and reveal it. If it's a
/// creature card, that player discards it unless they pay 3 life." (Breathstealer's
/// Crypt): the card drawn is checked and that player chooses (CR 118.12a).
fn draw_reveal_discard_unless(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let (head, rest) = text.split_once(" instead they draw a card and reveal it. If it's ")?;
    let (quality, cost) = rest.split_once(", that player discards it unless they pay ")?;
    let life = cost.strip_suffix(" life.")?;
    let (n, tail) = crate::oracle::phrases::parse_number(life)?;
    if !tail.trim().is_empty() {
        return None;
    }
    let quality = quality.to_lowercase();
    let quality = quality
        .strip_prefix("a ")
        .or_else(|| quality.strip_prefix("an "))?;
    let (f, _, tail) = crate::oracle::phrases::parse_object_phrase(quality)?;
    if !crate::oracle::phrases::end(tail).is_empty() {
        return None;
    }
    let mut abilities =
        crate::oracle::parse_ability(&format!("{head} instead they draw a card and reveal it."), ctx)?;
    let e = draw_instead(&mut abilities)?;
    // Performed by the player drawing (`AsPlayer`): "you" in it is that player.
    let Effect::AsPlayer { effect, .. } = e else {
        return None;
    };
    let drawn = Sel::Var(vars::IT);
    let check = Effect::If {
        cond: Condition::SelMatches(drawn.clone(), f),
        then: Box::new(Effect::PayOptional {
            who: PlayerRef::You,
            cost: Cost::free().with(CostPart::PayLife(n)),
            then: Box::new(Effect::Noop),
            otherwise: Box::new(Effect::Discard {
                who: PlayerRef::You,
                n: Value::c(1),
                random: false,
                filter: Filter::and(vec![Filter::Card, Filter::In(Box::new(drawn))]),
            }),
        }),
        otherwise: Box::new(Effect::Noop),
    };
    **effect = Effect::seq(vec![(**effect).clone(), check]);
    let a = std::sync::Arc::make_mut(&mut abilities[0]);
    a.text = text.to_string();
    Some(abilities)
}

inventory::submit! { AbilityPattern { name: "replacement bodies: instead draw and reveal; if it's a [card], discard it unless they pay life", priority: 70, parse: draw_reveal_discard_unless } }

/// "the next time they would draw a card this turn, instead they exile the top card of
/// their library" (Urabrask, Heretic Praetor, on an opponent's upkeep): a one-use
/// replacement of that player's next draw this turn, the player locked in as it's created
/// (CR 611.2c).
fn their_next_draw_instead(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = crate::oracle::phrases::end(l)
        .strip_prefix("the next time they would draw a card this turn, instead ")?;
    let e = super::replacement_grammar::instructions(r, b.ctx, Sel::TriggerObject)?;
    Some(Effect::AddReplacement {
        def: ReplacementDef {
            event: ReplacementEvent::Draw(PlayerFilter::Ref(Box::new(b.it_player.clone()))),
            action: ReplacementAction::Instead(Box::new(e)),
            self_replacement: false,
            optional: false,
        },
        duration: Duration::EndOfTurn,
        uses: Some(1),
    })
}

inventory::submit! { super::EffectPattern { name: "replacement bodies: the next time they would draw a card this turn, instead ...", priority: 70, parse: their_next_draw_instead } }

/// "They may play it this turn." after [`their_next_draw_instead`] exiled a card: the
/// player drawing may play the card exiled instead.
fn they_may_play_it(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if l != "they may play it this turn" {
        return false;
    }
    let Some(Effect::AddReplacement {
        def:
            ReplacementDef {
                event: ReplacementEvent::Draw(_),
                action: ReplacementAction::Instead(e),
                ..
            },
        uses: Some(1),
        ..
    }) = last_of(prev)
    else {
        return false;
    };
    let moved = |e: &Effect| matches!(e, Effect::Exile { .. } | Effect::Move { .. });
    let ok = match e.as_ref() {
        Effect::AsPlayer { effect, .. } => moved(effect),
        other => moved(other),
    };
    if !ok {
        return false;
    }
    let grant = Effect::GrantPlayPermission {
        who: PlayerRef::TriggerPlayer,
        what: Sel::Var(vars::IT),
        duration: Duration::EndOfTurn,
        free: false,
    };
    **e = Effect::seq(vec![(**e).clone(), grant]);
    true
}

/// The last instruction of an effect (in execution order), mutably.
fn last_of(e: &mut Effect) -> Option<&mut Effect> {
    match e {
        Effect::Seq(v) => v.last_mut().and_then(last_of),
        other => Some(other),
    }
}

inventory::submit! { super::FollowupPattern { name: "replacement bodies: they may play it this turn (after a draw replaced by exile)", priority: 70, apply: they_may_play_it } }

// ---------------------------------------------------------------------------
// A life total that would be reduced to N or less (CR 614.1a, 119.3)
// ---------------------------------------------------------------------------

/// "If your life total would be reduced to 0 or less, instead transform ~ and your life
/// total becomes 3. Then if ~ didn't transform this way, you lose the game." (Enduring
/// Angel): a replacement of the life loss (from damage or otherwise) that would bring the
/// life total to N or less; it doesn't use the stack (its ruling). A permanent that can't
/// transform (a token, or a copy) doesn't, and its controller loses (CR 701.27d).
fn life_reduced_to_or_less(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let r = text.strip_prefix("If your life total would be reduced to ")?;
    let (n, r) = crate::oracle::phrases::parse_number(r)?;
    let Value::Const(_) = n else {
        return None;
    };
    let r = r
        .trim_start()
        .strip_prefix("or less, instead transform ~ and your life total becomes ")?;
    let m = r.strip_suffix(". Then if ~ didn't transform this way, you lose the game.")?;
    let m = Value::c(m.trim().parse().ok()?);
    let transformed = Condition::SelMatches(
        Sel::This,
        Filter::Custom(crate::transform_rules::TRANSFORMED.into()),
    );
    let effect = Effect::seq(vec![
        Effect::Transform { what: Sel::This },
        Effect::SetLife {
            who: PlayerRef::You,
            n: m,
        },
        Effect::If {
            cond: Condition::Not(Box::new(transformed)),
            then: Box::new(Effect::LoseGame {
                who: PlayerRef::You,
            }),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    let def = ReplacementDef {
        event: ReplacementEvent::Where {
            event: Box::new(ReplacementEvent::LoseLife(PlayerFilter::You)),
            cond: Condition::Compare(
                Value::Diff(
                    Box::new(Value::LifeTotal(PlayerRef::You)),
                    Box::new(Value::EventAmount),
                ),
                Cmp::Le,
                n,
            ),
        },
        action: ReplacementAction::Instead(Box::new(effect)),
        self_replacement: false,
        optional: false,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(def))),
        text,
    )])
}

inventory::submit! { AbilityPattern { name: "replacement bodies: if your life total would be reduced to N or less, instead transform ~", priority: 70, parse: life_reduced_to_or_less } }

// ---------------------------------------------------------------------------
// Entering that requires a cost (CR 614.1c, 614.12)
// ---------------------------------------------------------------------------

/// "If ~ would enter, sacrifice two untapped lands instead. If you do, put ~ onto the
/// battlefield. If you don't, put it into its owner's graveyard." (Lotus Vale): the
/// permanent enters only if the cost is paid; otherwise it never enters (its ruling). The
/// replacement applies once to the event (CR 614.5): putting it onto the battlefield isn't
/// replaced again.
fn enter_only_if_paid(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let lower = text.to_lowercase();
    let r = lower.strip_prefix("if ~ would enter, ")?;
    let (cost_s, rest) = r.split_once(" instead. ")?;
    if rest != "if you do, put ~ onto the battlefield. if you don't, put it into its owner's graveyard." {
        return None;
    }
    let (cost, false) = crate::oracle::costs::parse_cost(cost_s)? else {
        return None;
    };
    if cost.mana.is_some()
        || !cost
            .parts
            .iter()
            .all(|p| matches!(p, CostPart::Sacrifice { .. } | CostPart::TapUntapped { .. }))
    {
        return None;
    }
    let effect = Effect::seq(vec![
        Effect::PayOptional {
            who: PlayerRef::You,
            cost,
            then: Box::new(Effect::Noop),
            otherwise: Box::new(Effect::Noop),
        },
        Effect::If {
            cond: Condition::PrevHappened,
            then: Box::new(Effect::Move {
                what: Sel::This,
                to: Destination::battlefield(),
            }),
            otherwise: Box::new(Effect::Move {
                what: Sel::This,
                to: Destination::zone(ZoneKind::Graveyard),
            }),
        },
    ]);
    let def = ReplacementDef {
        event: ReplacementEvent::EntersBattlefield(Filter::Source),
        action: ReplacementAction::Instead(Box::new(effect)),
        self_replacement: false,
        optional: false,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(def))),
        text,
    )])
}

inventory::submit! { AbilityPattern { name: "replacement bodies: if ~ would enter, [cost] instead; if you do, it enters", priority: 70, parse: enter_only_if_paid } }
