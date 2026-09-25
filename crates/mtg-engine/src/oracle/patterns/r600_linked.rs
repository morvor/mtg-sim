//! Oracle patterns for linked abilities (CR 607): static abilities linked to triggered
//! abilities printed in the same paragraph (CR 603.11, 607.2h: "You may exert this
//! creature as it attacks. When you do, [effect]."), choices and "the chosen [value]"
//! (CR 607.2d), and "the exiled card" / "cards exiled with ~" (CR 607.2a).

use super::{AbilityPattern, EffectPattern, StaticPattern, TriggerPattern};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::kw::exert::{EXERTED, EXERT_AS_ATTACKS};
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::CompileContext;

/// The link shared by an exert static ability and its "when you do" triggers.
const EXERT_LINK: u16 = 0x4001;

fn exert_paragraph(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    let r = lower
        .strip_prefix("you may exert ~ as it attacks.")
        .or_else(|| lower.strip_prefix("you may exert ~ as he attacks."))
        .or_else(|| lower.strip_prefix("you may exert ~ as she attacks."))?
        .trim();
    let mut out = vec![AbilityDef::with_link(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(
            EXERT_AS_ATTACKS.into(),
        ))),
        t,
        EXERT_LINK,
    )];
    if r.is_empty() {
        return Some(out);
    }
    let effect_text = r.strip_prefix("when you do, ")?;
    let mut b = Builder::new(ctx);
    b.in_trigger = true;
    let effect = parse_effect_text(effect_text, &mut b)?;
    let body = Body {
        targets: b.targets,
        effect,
        modal: None,
    };
    out.push(AbilityDef::with_link(
        AbilityKind::Triggered(TriggeredAbility::new(
            TriggerCond::Custom(EXERTED.into()),
            body,
        )),
        t,
        EXERT_LINK,
    ));
    Some(out)
}

inventory::submit! { AbilityPattern { name: "exert as it attacks", priority: 0, parse: exert_paragraph } }

/// "As ~ enters, choose a color." and similar: the choice is stored for the abilities
/// linked to this one (CR 607.2d).
fn as_enters_choose(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = l.strip_prefix("as ~ enters, choose ")?;
    let kind = match r {
        "a color" => ChoiceKind::Color,
        "a creature type" => ChoiceKind::CreatureType,
        "a card type" => ChoiceKind::CardType,
        "a basic land type" => ChoiceKind::BasicLandType,
        "an opponent" => ChoiceKind::Opponent,
        "a player" => ChoiceKind::Player,
        "odd or even" => ChoiceKind::OddOrEven,
        _ => return None,
    };
    let s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
        event: ReplacementEvent::EntersBattlefield(Filter::Source),
        action: ReplacementAction::AsEnters(Box::new(Effect::Choose {
            who: PlayerRef::You,
            kind,
        })),
        self_replacement: true,
        optional: false,
    }));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

/// "~ has protection from the chosen color", "enchanted creature has protection from the
/// chosen color", "creatures you control have protection from the chosen color".
fn protection_from_chosen(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let affected = if l == "~ has protection from the chosen color" {
        Filter::Source
    } else if l == "enchanted creature has protection from the chosen color" {
        Filter::AttachedToSource
    } else if l == "equipped creature has protection from the chosen color" {
        Filter::AttachedToSource
    } else if l == "creatures you control have protection from the chosen color" {
        Filter::creature().you_control()
    } else {
        return None;
    };
    let s = StaticAbility::new(StaticEffect::Continuous {
        affected,
        mods: vec![Modification::AddKeyword(Keyword::with_filter(
            KeywordKind::Protection,
            Filter::ChosenColor,
        ))],
    });
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

/// "Whenever a player casts a spell of the chosen color".
fn casts_chosen_color(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let (who, rest) = if let Some(x) = r.strip_prefix("you cast ") {
        (PlayerRel::You, x)
    } else if let Some(x) = r.strip_prefix("an opponent casts ") {
        (PlayerRel::Opponent, x)
    } else if let Some(x) = r.strip_prefix("a player casts ") {
        (PlayerRel::Any, x)
    } else {
        return None;
    };
    let filter = match rest {
        "a spell of the chosen color" => Filter::ChosenColor,
        "a creature spell of the chosen type" => Filter::And(vec![
            Filter::Type(crate::types::CardType::Creature),
            Filter::ChosenCreatureType,
        ]),
        _ => return None,
    };
    Some((
        TriggerCond::CastSpell { who, filter },
        Sel::TriggerSpell,
        PlayerRef::TriggerPlayer,
    ))
}

/// "return the exiled card(s) to the battlefield under its/their owner's control", "return
/// the exiled card(s) to its/their owner's hand(s)", "put a creature card exiled with ~ onto
/// the battlefield under your control" (CR 607.2a, 607.3).
fn linked_exile_effects(l: &str, _b: &mut Builder) -> Option<Effect> {
    const V: Var = vars::USER + 80;
    let each = |to: Destination| Effect::ForEach {
        sel: Sel::Linked,
        var: V,
        effect: Box::new(Effect::Move {
            what: Sel::Var(V),
            to,
        }),
    };
    let owner_bf = || {
        let mut d = Destination::battlefield();
        d.controller = Some(PlayerRef::OwnerOf(Box::new(Sel::Var(V))));
        d
    };
    match l {
        "return the exiled card to the battlefield under its owner's control"
        | "return the exiled cards to the battlefield under their owners' control"
        | "return the exiled permanent to the battlefield under its owner's control"
        | "return all cards exiled with ~ to the battlefield under their owners' control" => {
            Some(each(owner_bf()))
        }
        "return the exiled card to its owner's hand"
        | "return the exiled cards to their owners' hands"
        | "return all cards exiled with ~ to their owners' hands"
        | "put each card exiled with it into its owner's hand"
        | "put each card exiled with ~ into its owner's hand"
        | "put all cards exiled with ~ into their owners' hands" => {
            Some(each(Destination::zone(ZoneKind::Hand)))
        }
        "put a creature card exiled with ~ onto the battlefield under your control"
        | "put a card exiled with ~ onto the battlefield under your control" => {
            let filter = if l.starts_with("put a creature") {
                Filter::And(vec![
                    Filter::In(Box::new(Sel::Linked)),
                    Filter::InZone(ZoneKind::Exile),
                    Filter::Type(crate::types::CardType::Creature),
                ])
            } else {
                Filter::And(vec![
                    Filter::In(Box::new(Sel::Linked)),
                    Filter::InZone(ZoneKind::Exile),
                ])
            };
            Some(Effect::Move {
                what: Sel::Choose {
                    chooser: PlayerRef::You,
                    filter,
                    count: Value::c(1),
                    up_to: false,
                    store: None,
                },
                to: Destination::battlefield().under_your_control(),
            })
        }
        _ => None,
    }
}

/// Block-level forms of the above (so they're tried before other parsers).
fn chosen_value_block(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    let l = lower.strip_suffix('.')?;
    if let Some(v) = protection_from_chosen(l, t, ctx) {
        return Some(v);
    }
    let r = l.strip_prefix("whenever ")?;
    let (cond, eff) = r.split_once(", ")?;
    let (trigger, _, _) = casts_chosen_color(cond)?;
    let mut b = Builder::new(ctx);
    b.in_trigger = true;
    b.it = Sel::TriggerSpell;
    let effect = parse_effect_text(eff, &mut b)?;
    let body = Body {
        targets: b.targets,
        effect,
        modal: None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Triggered(TriggeredAbility::new(trigger, body)),
        t,
    )])
}

inventory::submit! { StaticPattern { name: "as enters, choose", priority: 0, parse: as_enters_choose } }
inventory::submit! { StaticPattern { name: "protection from the chosen color", priority: 0, parse: protection_from_chosen } }
inventory::submit! { TriggerPattern { name: "casts a spell of the chosen color", priority: 0, parse: casts_chosen_color } }
inventory::submit! { AbilityPattern { name: "chosen value abilities", priority: 0, parse: chosen_value_block } }
inventory::submit! { EffectPattern { name: "linked exile references", priority: 0, parse: linked_exile_effects } }

/// "As an additional cost to cast ~, [cost]" on a permanent card (on instants and sorceries
/// it's a spell static already). Cards it exiles are exiled with the permanent
/// (CR 607.2q).
fn permanent_additional_cost(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    if !t
        .to_lowercase()
        .starts_with("as an additional cost to cast ~, ")
        || ctx.is_spell()
    {
        return None;
    }
    if let Some(a) = crate::oracle::statics::parse_spell_static(t, ctx) {
        return Some(vec![a]);
    }
    let lower = t.to_lowercase();
    let r = lower.strip_prefix("as an additional cost to cast ~, ")?;
    let filter = match r.strip_suffix('.')? {
        "exile a creature you control" => Filter::creature().you_control(),
        "exile an artifact you control" => {
            Filter::Type(crate::types::CardType::Artifact).you_control()
        }
        "exile a creature card from your graveyard" => Filter::creature(),
        _ => return None,
    };
    let zone = if r.contains("graveyard") {
        ZoneKind::Graveyard
    } else {
        ZoneKind::Battlefield
    };
    let cost = Cost::default().with(CostPart::Exile {
        filter,
        zone,
        count: Value::c(1),
    });
    let mut s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::ThisSpell,
        who: PlayerRel::You,
        change: CostChange::AdditionalCost(cost),
    }));
    s.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), t)])
}

inventory::submit! { AbilityPattern { name: "permanent additional cost", priority: 0, parse: permanent_additional_cost } }

/// Anchor words (CR 607.2m, 614.12b): "As ~ enters, choose Khans or Dragons." followed by
/// "• Khans — [ability]" and "• Dragons — [ability]". Each anchored ability functions only
/// if its word was chosen by the linked "choose" ability.
fn anchor_words(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let mut lines = t.lines();
    let first = lines.next()?.trim();
    let r = first
        .strip_prefix("As ~ enters, choose ")?
        .strip_suffix('.')?;
    let words: Vec<String> = r
        .replace(", or ", ", ")
        .replace(" or ", ", ")
        .split(", ")
        .map(|w| w.trim().to_string())
        .filter(|w| !w.is_empty())
        .collect();
    if words.len() < 2 {
        return None;
    }
    let mut out = vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::EntersBattlefield(Filter::Source),
                action: ReplacementAction::AsEnters(Box::new(Effect::Choose {
                    who: PlayerRef::You,
                    kind: ChoiceKind::Word(words.clone()),
                })),
                self_replacement: true,
                optional: false,
            },
        ))),
        first,
    )];
    let mut anchored = 0;
    for line in lines {
        let line = line.trim().trim_start_matches('•').trim();
        let (word, text) = line.split_once(" — ")?;
        if !words.iter().any(|w| w == word) {
            return None;
        }
        let cond = Condition::ChosenWord(word.to_string());
        for a in crate::oracle::parse_ability(text, ctx)? {
            let kind = match a.kind.clone() {
                AbilityKind::Triggered(mut tr) => {
                    tr.intervening_if = Some(match tr.intervening_if {
                        Some(c) => Condition::And(vec![cond.clone(), c]),
                        None => cond.clone(),
                    });
                    AbilityKind::Triggered(tr)
                }
                AbilityKind::Static(mut st) => {
                    st.condition = Some(match st.condition {
                        Some(c) => Condition::And(vec![cond.clone(), c]),
                        None => cond.clone(),
                    });
                    AbilityKind::Static(st)
                }
                AbilityKind::Activated(mut ac) => {
                    ac.condition = Some(match ac.condition {
                        Some(c) => Condition::And(vec![cond.clone(), c]),
                        None => cond.clone(),
                    });
                    AbilityKind::Activated(ac)
                }
                _ => return None,
            };
            out.push(AbilityDef::with_link(kind, line, a.link));
            anchored += 1;
        }
    }
    (anchored > 0).then_some(out)
}

inventory::submit! { AbilityPattern { name: "anchor words", priority: 0, parse: anchor_words } }
