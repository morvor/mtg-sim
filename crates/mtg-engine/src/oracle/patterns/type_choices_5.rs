//! Type choices (cards5-choices-players): "each player chooses artifact, creature, or land
//! during their untap step. That player can untap only permanents of the chosen type this
//! step." (Storage Matrix; see `untap_limits.rs`, CR 502.3).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;
use crate::types::CardType;

/// "artifact, creature, or land": two or more card type words.
fn card_type_list(s: &str) -> Option<Vec<CardType>> {
    let s = s.replace(", or ", ", ").replace(" or ", ", ");
    let v: Vec<CardType> = s
        .split(", ")
        .map(|w| CardType::from_word(w.trim()))
        .collect::<Option<_>>()?;
    let mut d = v.clone();
    d.dedup();
    (v.len() >= 2 && d.len() == v.len()).then_some(v)
}

fn untap_only_chosen_type(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (cond, rest) = match l.strip_prefix("as long as ") {
        Some(r) => {
            let (c, rest) = r.split_once(", ")?;
            let (cond, _) =
                super::statics_conditions::parse_static_condition(c, Some(&Sel::This), ctx)?;
            (Some(cond), rest)
        }
        None => (None, l),
    };
    let list = rest
        .strip_prefix("each player chooses ")?
        .strip_suffix(
            " during their untap step. that player can untap only permanents of the chosen type this step",
        )?;
    let types = card_type_list(list)?;
    let name = format!(
        "{}{}",
        crate::untap_limits::UNTAP_ONLY_CHOSEN_TYPE,
        types.iter().map(|t| t.word()).collect::<Vec<_>>().join(",")
    );
    let mut s = StaticAbility::new(StaticEffect::Restriction(Restriction::Custom(name.into())));
    s.condition = cond;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "each player chooses [card types] during their untap step; untap only the chosen type", priority: 100, parse: untap_only_chosen_type } }

/// The choice kinds of "choose a creature card name and a creature type" (each item one
/// choice), other than a color alone (`choice_grammar_multi`).
fn attach_choice_kinds(s: &str) -> Option<Vec<ChoiceKind>> {
    let mut out = Vec::new();
    for part in s.split(" and ") {
        out.push(match part.trim() {
            "a color" => ChoiceKind::Color,
            "a creature type" => ChoiceKind::CreatureType,
            "a card name" => ChoiceKind::CardName,
            "a creature card name" => ChoiceKind::CardNameFiltered("creature".into()),
            _ => return None,
        });
    }
    (!matches!(out.as_slice(), [ChoiceKind::Color])).then_some(out)
}

/// "As ~ becomes attached to a creature, choose a creature card name and a creature
/// type." (Psychic Paper): see `kw/attach_choices_5.rs`.
fn as_attached_choose(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("as ~ becomes attached to a creature, choose ")?;
    let kinds = attach_choice_kinds(&format!("a {}", r.strip_prefix("a ")?))?;
    let e = crate::kw::attach_choices_5::as_attached_choose(&kinds)?;
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(e)),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "as ~ becomes attached to a creature, choose [name, type, ...]", priority: 60, parse: as_attached_choose } }

/// "Equipped creature has ward {1}, it can't be blocked, and its name and creature type
/// are the last chosen name and creature type." (Psychic Paper): the abilities it grants
/// and the name and creature type it sets, each its own static ability of the Equipment.
fn equipped_last_chosen_name_and_type(
    l: &str,
    text: &str,
    ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    use crate::kw::attach_choices_5::{LAST_CHOSEN_CREATURE_TYPE, LAST_CHOSEN_NAME};
    let r = end(l).strip_prefix("equipped creature ")?;
    let (before, sets) = r.split_once("its name and creature type are the last chosen name and creature type")?;
    if !sets.trim().is_empty() {
        return None;
    }
    let mut out = Vec::new();
    // "has ward {1}, it can't be blocked, and ".
    let before = before.trim_end().strip_suffix(" and").unwrap_or(before.trim_end());
    let before = before.trim_end_matches(',');
    for (i, part) in before.split(", ").filter(|p| !p.is_empty()).enumerate() {
        let part = if i == 0 {
            part.to_string()
        } else {
            part.strip_prefix("it ")?.to_string()
        };
        let a = crate::oracle::parse_ability(&format!("Equipped creature {part}."), ctx)?;
        for x in a {
            let mut x = (*x).clone();
            x.text = text.to_string();
            out.push(std::sync::Arc::new(x));
        }
    }
    let mods = vec![
        Modification::Custom {
            name: LAST_CHOSEN_NAME.into(),
            layer: Layer::L3Text,
        },
        Modification::Custom {
            name: LAST_CHOSEN_CREATURE_TYPE.into(),
            layer: Layer::L4Type,
        },
    ];
    out.push(AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::AttachedToSource,
            mods,
        })),
        text,
    ));
    Some(out)
}

inventory::submit! { StaticPattern { name: "equipped creature has ..., and its name and creature type are the last chosen name and creature type", priority: 60, parse: equipped_last_chosen_name_and_type } }

/// "As ~ becomes attached to a creature, choose an exiled creature card used to craft ~."
/// (Paleontologist's Pick-Axe // Dinosaur Headdress): one of the cards the craft cost
/// exiled (CR 702.167c), noted for "the last chosen card" (CR 607.2d), chosen again each
/// time it becomes attached to a creature.
fn as_attached_choose_craft_material(
    l: &str,
    text: &str,
    _ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("as ~ becomes attached to a creature, choose an exiled ")?;
    let noun = r
        .strip_suffix(" used to craft ~")
        .or_else(|| r.strip_suffix(" used to craft it"))?;
    let (f, false, tail) = crate::oracle::phrases::parse_object_phrase(noun)? else {
        return None;
    };
    if !end(tail).is_empty() || f.zone().is_some() {
        return None;
    }
    let choose = Effect::NoteLinked {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![f, crate::kw::craft::used_to_craft_filter()]),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
        replace: true,
    };
    let e = crate::kw::attach_choices_5::as_attached_do(&choose)?;
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(e)),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "as ~ becomes attached to a creature, choose an exiled card used to craft ~", priority: 60, parse: as_attached_choose_craft_material } }
