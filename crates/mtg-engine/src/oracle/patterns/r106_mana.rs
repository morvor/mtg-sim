//! Mana text (CR 106): restricted mana ("Spend this mana only ..."), mana produced from
//! values ("Add an amount of {G} equal to ~'s power"), "could produce" mana (CR 106.7),
//! mana from a mana cost (CR 106.8–106.11), doubling unspent mana, Drain Power
//! (CR 106.13), and tapped-for-mana triggers and replacement effects (CR 106.12).

use crate::ability::*;
use crate::mana::{ManaRestriction, ManaType};
use crate::oracle::effects::Builder;
use crate::oracle::patterns::{AbilityPattern, EffectPattern, StaticPattern, TriggerPattern};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::*;

/// Parses the restriction in "Spend this mana only ..." / "This mana can't be spent ...".
fn parse_restriction(s: &str) -> Option<ManaRestriction> {
    let s = end(s);
    if let Some(r) = s.strip_prefix("spend this mana only ") {
        return Some(match r {
            "to cast creature spells" | "to cast a creature spell" => {
                ManaRestriction::SpellOfType(CardType::Creature)
            }
            "to cast artifact spells" | "to cast an artifact spell" => {
                ManaRestriction::SpellOfType(CardType::Artifact)
            }
            "to cast enchantment spells" | "to cast an enchantment spell" => {
                ManaRestriction::SpellOfType(CardType::Enchantment)
            }
            "to cast instant or sorcery spells" | "to cast an instant or sorcery spell" => {
                ManaRestriction::InstantOrSorcery
            }
            "to cast noncreature spells" | "to cast a noncreature spell" => {
                ManaRestriction::NoncreatureSpell
            }
            "to cast spells" | "to cast a spell" => ManaRestriction::SpellsOnly,
            "to activate abilities" => ManaRestriction::AbilitiesOnly,
            "on costs that contain {x}" | "on costs that include {x}" => {
                ManaRestriction::XCostsOnly
            }
            "to cast artifact spells or activate abilities of artifacts" => {
                ManaRestriction::ArtifactSpellOrAbility
            }
            _ => return None,
        });
    }
    match s {
        "this mana can't be spent to cast a nonartifact spell"
        | "this mana can't be spent to cast nonartifact spells" => {
            Some(ManaRestriction::NotNonartifactSpell)
        }
        _ => None,
    }
}

fn set_restriction(e: &mut Effect, r: &ManaRestriction) -> bool {
    match e {
        Effect::AddMana { restriction, .. } => {
            *restriction = Some(r.clone());
            true
        }
        Effect::Seq(v) => {
            let mut any = false;
            for x in v.iter_mut() {
                any |= set_restriction(x, r);
            }
            any
        }
        Effect::ChooseOne { options, .. } => {
            let mut any = false;
            for (_, x) in options.iter_mut() {
                any |= set_restriction(x, r);
            }
            any
        }
        Effect::If {
            then, otherwise, ..
        } => set_restriction(then, r) | set_restriction(otherwise, r),
        _ => false,
    }
}

/// "{T}: Add one mana of any color. Spend this mana only to cast a creature spell."
/// (CR 106.6): the restriction becomes part of each AddMana in the ability.
fn restricted_mana(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.to_lowercase();
    let idx = lower
        .find(". spend this mana only ")
        .or_else(|| lower.find(". this mana can't be spent "))?;
    let restriction = parse_restriction(&lower[idx + 2..])?;
    let head = &block[..idx + 1];
    let abilities = crate::oracle::parse_ability(head, ctx)?;
    let mut out = Vec::new();
    for a in abilities {
        let mut kind = a.kind.clone();
        let ok = match &mut kind {
            AbilityKind::Activated(act) => set_restriction(&mut act.body.effect, &restriction),
            AbilityKind::Triggered(t) => set_restriction(&mut t.body.effect, &restriction),
            AbilityKind::Spell(s) => set_restriction(&mut s.body.effect, &restriction),
            _ => false,
        };
        if !ok {
            return None;
        }
        out.push(AbilityDef::new(kind, block));
    }
    Some(out)
}

inventory::submit! { AbilityPattern { name: "r106 restricted mana", priority: 60, parse: restricted_mana } }

/// A single mana symbol "{g}" → type.
fn mana_symbol_type(s: &str) -> Option<(ManaType, &str)> {
    let s = s.trim_start();
    let r = s.strip_prefix('{')?;
    let (inner, rest) = r.split_once('}')?;
    if inner.len() != 1 {
        return None;
    }
    let t = ManaType::from_letter(inner.chars().next()?)?;
    Some((t, rest))
}

/// "{r}{r}" → types.
fn fixed_mana(s: &str) -> Option<Vec<ManaType>> {
    let mut out = Vec::new();
    let mut rest = end(s);
    while !rest.is_empty() {
        let (t, r) = mana_symbol_type(rest)?;
        out.push(t);
        rest = r.trim_start();
    }
    (!out.is_empty()).then_some(out)
}

/// Mana production phrases after "add " (lowercase, no trailing period).
fn production(r: &str, b: &mut Builder) -> Option<ManaProduction> {
    let r = end(r);
    // "an amount of {g} equal to ~'s power" (CR 107.1b: a negative amount adds nothing).
    if let Some(x) = r.strip_prefix("an amount of ") {
        let (t, rest) = mana_symbol_type(x)?;
        let rest = rest.trim_start().strip_prefix("equal to ")?;
        let (v, tail) = crate::oracle::statics::parse_value_phrase(rest, b)?;
        if !end(&tail).is_empty() {
            return None;
        }
        return Some(ManaProduction::Amount(t, v));
    }
    // "one mana of any color that a land an opponent controls could produce",
    // "one mana of any type that a land you control could produce" (CR 106.7).
    for (p, colors_only) in [
        ("one mana of any color that ", true),
        ("one mana of any type that ", false),
    ] {
        if let Some(x) = r.strip_prefix(p) {
            let x = x
                .strip_prefix("a ")
                .or_else(|| x.strip_prefix("an "))
                .unwrap_or(x);
            let Some((f, _, tail)) = parse_object_phrase(x) else {
                continue;
            };
            if end(tail) != "could produce" {
                continue;
            }
            return Some(if colors_only {
                ManaProduction::CouldProduceColor(f)
            } else {
                ManaProduction::CouldProduce(f)
            });
        }
    }
    // "one mana of any type that land produced" / "that permanent produced".
    if matches!(
        r,
        "one mana of any type that land produced"
            | "one mana of any type that permanent produced"
            | "one additional mana of any type that land produced"
    ) {
        return Some(ManaProduction::AnyTypeProduced);
    }
    // "mana equal to enchanted permanent's mana cost" (CR 106.8–106.11).
    if let Some(x) = r.strip_prefix("mana equal to ") {
        let sel = match x {
            "enchanted permanent's mana cost" | "enchanted creature's mana cost" => Sel::AttachedTo,
            "its mana cost" | "that card's mana cost" => b.it.clone(),
            "~'s mana cost" => Sel::This,
            _ => return None,
        };
        return Some(ManaProduction::ManaCostOf(sel));
    }
    if let Some(x) = r
        .strip_prefix("one mana of any color")
        .or_else(|| r.strip_prefix("one additional mana of any color"))
    {
        if end(x).is_empty() {
            return Some(ManaProduction::AnyOneColor(Value::c(1)));
        }
        return None;
    }
    if let Some(x) = r.strip_prefix("one mana of the chosen color") {
        if end(x).is_empty() {
            return Some(ManaProduction::ChosenColor(Value::c(1)));
        }
        return None;
    }
    if r.contains(" or ") {
        let opts: Vec<ManaType> = r
            .split([',', ' '])
            .filter(|w| !w.is_empty() && *w != "or")
            .map(mana_symbol_type)
            .collect::<Option<Vec<_>>>()?
            .into_iter()
            .map(|(t, _)| t)
            .collect();
        return (opts.len() >= 2).then_some(ManaProduction::OneOf(opts));
    }
    fixed_mana(r).map(ManaProduction::Fixed)
}

/// Mana effects the core compiler doesn't know: "add an amount of ...", "could produce",
/// "add mana equal to ...'s mana cost", "[player] adds an additional {G}", "double the
/// amount of each type of unspent mana you have".
fn mana_effects(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if l == "double the amount of each type of unspent mana you have" {
        return Some(Effect::AddMana {
            who: PlayerRef::You,
            mana: ManaProduction::DoubleUnspent,
            restriction: None,
        });
    }
    // "add ...", "add an additional ..."
    if let Some(r) = l.strip_prefix("add ") {
        let r = r.strip_prefix("an additional ").unwrap_or(r);
        let mana = production(r, b)?;
        return Some(Effect::AddMana {
            who: PlayerRef::You,
            mana,
            restriction: None,
        });
    }
    // "its controller adds an additional {g}", "that player adds one mana of any type
    // that land produced".
    let (who, r) = if let Some(r) = l.strip_prefix("its controller adds ") {
        (PlayerRef::ControllerOf(Box::new(b.it.clone())), r)
    } else if let Some(r) = l.strip_prefix("that player adds ") {
        (b.it_player.clone(), r)
    } else {
        return None;
    };
    let r = r.strip_prefix("an additional ").unwrap_or(r);
    let mana = production(r, b)?;
    Some(Effect::AddMana {
        who,
        mana,
        restriction: None,
    })
}

inventory::submit! { EffectPattern { name: "r106 mana effects", priority: 60, parse: mana_effects } }

/// Drain Power (CR 106.13): "target player activates a mana ability of each land they
/// control" and "then that player loses all unspent mana and you add the mana lost this
/// way".
fn drain_power(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    if let Some(r) = l.strip_suffix(" activates a mana ability of each land they control") {
        let who = match r {
            "target player" => {
                let slot = b.add_target(
                    TargetSpec::player(PlayerFilter::Any, "target player"),
                    "target player",
                );
                b.it_player = PlayerRef::Target(slot);
                PlayerRef::Target(slot)
            }
            "each player" => PlayerRef::EachPlayer,
            "you" => PlayerRef::You,
            _ => return None,
        };
        return Some(Effect::ActivateManaAbilities {
            who,
            filter: Filter::Type(CardType::Land),
        });
    }
    let who = if l.starts_with("that player ") {
        b.it_player.clone()
    } else if l.starts_with("each player ") {
        PlayerRef::EachPlayer
    } else {
        return None;
    };
    let r = l.split_once(' ').map(|x| x.1)?;
    let r = r.strip_prefix("player ").unwrap_or(r);
    match r {
        "loses all unspent mana and you add the mana lost this way" => {
            Some(Effect::LoseUnspentMana {
                who,
                to: Some(PlayerRef::You),
            })
        }
        "loses all unspent mana" => Some(Effect::LoseUnspentMana { who, to: None }),
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "r106 drain power", priority: 60, parse: drain_power } }

/// "{T}: Choose a color of a permanent you control. Add one mana of that color."
/// (Meteor Crater, CR 106.5).
fn color_of_a_permanent(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (cost_s, eff_orig) = crate::oracle::split_cost(block.trim())?;
    let eff = eff_orig.to_lowercase();
    let r = eff.strip_prefix("choose a color of ")?;
    let (head, tail) = r.split_once(". ")?;
    if end(tail) != "add one mana of that color" {
        return None;
    }
    let head = head
        .strip_prefix("a ")
        .or_else(|| head.strip_prefix("an "))
        .unwrap_or(head);
    let (f, _, rest) = parse_object_phrase(head)?;
    if !end(rest).is_empty() {
        return None;
    }
    let (cost, _) = crate::oracle::costs::parse_cost(cost_s)?;
    let mut act = ActivatedAbility::new(
        cost,
        Body::effect(Effect::AddMana {
            who: PlayerRef::You,
            mana: ManaProduction::AnyColorAmong(f),
            restriction: None,
        }),
    );
    act.is_mana_ability = true;
    Some(vec![AbilityDef::new(AbilityKind::Activated(act), block)])
}

inventory::submit! { AbilityPattern { name: "r106 color of a permanent", priority: 60, parse: color_of_a_permanent } }

/// "for mana" / "for {c}" suffix of a tapped-for-mana trigger.
fn for_mana(s: &str) -> Option<Option<ManaType>> {
    let s = end(s);
    if s == "for mana" {
        return Some(None);
    }
    let r = s.strip_prefix("for ")?;
    let (t, rest) = mana_symbol_type(r)?;
    end(rest).is_empty().then_some(Some(t))
}

/// The trigger for "[filter] tapped for [a type of mana]". Plain "for mana" triggers are
/// parsed by the general tapped-for-mana pattern (`triggers_mana.rs`).
fn tapped_cond(filter: Filter, mana: Option<ManaType>) -> Option<TriggerCond> {
    mana.map(|t| TriggerCond::TappedForManaOfType { filter, mana: t })
}

/// Tapped-for-mana-of-a-type trigger conditions (CR 106.12a): "you tap a permanent for
/// {c}", "a land is tapped for {g}". The player who taps a permanent for mana is its
/// controller, so "you tap" / "an opponent taps" restrict the permanent's controller.
fn tapped_for_mana_trigger(l: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let l = end(l);
    // "[player] taps [object] for mana"
    for (p, who) in [
        ("a player taps ", PlayerRel::Any),
        ("you tap ", PlayerRel::You),
        ("an opponent taps ", PlayerRel::Opponent),
    ] {
        if let Some(r) = l.strip_prefix(p) {
            let (filter, rest) = if let Some(rest) = r.strip_prefix("~ ") {
                (Filter::Source, rest)
            } else {
                let r = r
                    .strip_prefix("a ")
                    .or_else(|| r.strip_prefix("an "))
                    .unwrap_or(r);
                let (f, _, rest) = parse_object_phrase(r)?;
                (f, rest)
            };
            let mana = for_mana(rest)?;
            let filter = match who {
                PlayerRel::Any => filter,
                rel => Filter::and(vec![filter, Filter::ControlledBy(rel)]),
            };
            return Some((
                tapped_cond(filter, mana)?,
                Sel::TriggerObject,
                PlayerRef::TriggerPlayer,
            ));
        }
    }
    // "[object] is tapped for mana"
    let (subject, rest) = l.split_once(" is tapped ")?;
    let mana = for_mana(rest)?;
    let filter = if let Some(r) = subject.strip_prefix("enchanted ") {
        let (f, _, tail) = parse_object_phrase(r)?;
        if !end(tail).is_empty() {
            return None;
        }
        Filter::and(vec![Filter::AttachedToSource, f])
    } else if subject == "~" {
        Filter::Source
    } else {
        let s = subject
            .strip_prefix("a ")
            .or_else(|| subject.strip_prefix("an "))
            .unwrap_or(subject);
        let (f, _, tail) = parse_object_phrase(s)?;
        if !end(tail).is_empty() {
            return None;
        }
        f
    };
    Some((
        tapped_cond(filter, mana)?,
        Sel::TriggerObject,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "r106 tapped for mana", priority: 60, parse: tapped_for_mana_trigger } }

/// Mana-production replacement effects (CR 106.12b): "If you tap a permanent for mana,
/// it produces twice as much of that mana instead." / "If a land is tapped for mana, it
/// produces {B} instead of any other type and amount."
fn produce_mana_replacement(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = l.strip_prefix("if ")?;
    let (cond, action) = r.split_once(", it produces ")?;
    let filter = if let Some(x) = cond.strip_prefix("you tap ") {
        let x = x
            .strip_prefix("a ")
            .or_else(|| x.strip_prefix("an "))
            .unwrap_or(x);
        let (f, _, tail) = parse_object_phrase(x)?;
        if end(tail) != "for mana" {
            return None;
        }
        f.you_control()
    } else {
        let (subject, tail) = cond.split_once(" is tapped ")?;
        if end(tail) != "for mana" {
            return None;
        }
        let s = subject
            .strip_prefix("a ")
            .or_else(|| subject.strip_prefix("an "))
            .unwrap_or(subject);
        let (f, _, t2) = parse_object_phrase(s)?;
        if !end(t2).is_empty() {
            return None;
        }
        f
    };
    let action = end(action);
    let act = match action {
        "twice as much of that mana instead" => ReplacementAction::Multiply(2),
        "three times as much of that mana instead" => ReplacementAction::Multiply(3),
        _ => {
            let x = action.strip_suffix(" instead of any other type and amount")?;
            let v = fixed_mana(x)?;
            ReplacementAction::Instead(Box::new(Effect::AddMana {
                who: PlayerRef::You,
                mana: ManaProduction::Fixed(v),
                restriction: None,
            }))
        }
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::ProduceMana(filter),
                action: act,
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "r106 produce mana replacement", priority: 60, parse: produce_mana_replacement } }

/// "{T}: Add {R}. When that mana is spent to cast a red instant or sorcery spell, copy
/// that spell and you may choose new targets for the copy." (CR 106.6): the mana carries
/// a delayed triggered ability.
fn mana_spent_trigger(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.to_lowercase();
    let marker = ". when that mana is spent to cast ";
    let idx = lower.find(marker)?;
    let head = &block[..idx + 1];
    let tail = &lower[idx + marker.len()..];
    let (spell_s, eff_s) = tail.split_once(", ")?;
    let spell_s = spell_s
        .strip_prefix("a ")
        .or_else(|| spell_s.strip_prefix("an "))
        .unwrap_or(spell_s);
    let (f, _, rest) = parse_object_phrase(spell_s)?;
    if !end(rest).is_empty() {
        return None;
    }
    let spell_filter = Filter::and(vec![f, Filter::Spell]);
    let body = match end(eff_s) {
        "copy that spell and you may choose new targets for the copy" => {
            Body::effect(Effect::CopySpell {
                what: Sel::TriggerSpell,
                count: Value::c(1),
                new_targets: true,
            })
        }
        other => crate::oracle::effects::parse_trigger_body(
            other,
            ctx,
            Sel::TriggerSpell,
            PlayerRef::You,
        )?,
    };
    let abilities = crate::oracle::parse_ability(head, ctx)?;
    let mut out = Vec::new();
    for a in abilities {
        let AbilityKind::Activated(act) = &a.kind else {
            return None;
        };
        if !matches!(act.body.effect, Effect::AddMana { .. }) {
            return None;
        }
        let mut act = act.clone();
        act.body.effect = Effect::AddManaWithSpentTrigger {
            add: Box::new(act.body.effect.clone()),
            spell_filter: spell_filter.clone(),
            body: Box::new(body.clone()),
        };
        act.is_mana_ability = act.body.targets.is_empty();
        out.push(AbilityDef::new(AbilityKind::Activated(act), block));
    }
    Some(out)
}

inventory::submit! { AbilityPattern { name: "r106 mana spent trigger", priority: 60, parse: mana_spent_trigger } }
