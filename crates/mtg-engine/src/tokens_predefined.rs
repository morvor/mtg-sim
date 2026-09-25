//! Predefined tokens (CR 111.10).

use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::mana::{ManaCost, ManaRestriction, ManaType};
use crate::types::*;
use smol_str::SmolStr;

fn artifact(name: &str, subtype: &str, abilities: Vec<Ability>) -> TokenSpec {
    TokenSpec {
        name: SmolStr::new(name),
        colors: ColorSet::NONE,
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: vec![SmolStr::new(subtype)],
        power: None,
        toughness: None,
        abilities,
        scryfall_name: Some(SmolStr::new(name)),
    }
}

fn activated(
    cost: Cost,
    effect: Effect,
    targets: Vec<TargetSpec>,
    mana: bool,
    sorcery: bool,
    text: &str,
) -> Ability {
    let mut a = ActivatedAbility::new(cost, Body::simple(targets, effect));
    a.is_mana_ability = mana;
    if sorcery {
        a.timing = ActivationTiming::Sorcery;
    }
    AbilityDef::new(AbilityKind::Activated(a), text)
}

fn mc(s: &str) -> Option<ManaCost> {
    ManaCost::parse(s)
}

fn role(name: &str, mods: Vec<Modification>, extra: Vec<Ability>) -> TokenSpec {
    let mut abilities = vec![AbilityDef::new(
        AbilityKind::Keyword(Keyword::with_filter(
            KeywordKind::Enchant,
            Filter::creature(),
        )),
        "Enchant creature",
    )];
    if !mods.is_empty() {
        abilities.push(AbilityDef::new(
            AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
                affected: Filter::AttachedToSource,
                mods,
            })),
            format!("{name} role"),
        ));
    }
    abilities.extend(extra);
    TokenSpec {
        name: SmolStr::new(name),
        colors: ColorSet::NONE,
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: vec![SmolStr::new("Aura"), SmolStr::new("Role")],
        power: None,
        toughness: None,
        abilities,
        scryfall_name: None,
    }
}

pub fn predefined(name: &str) -> Option<TokenSpec> {
    let sac = CostPart::SacrificeSelf;
    Some(match name.to_ascii_lowercase().as_str() {
        // 111.10a
        "treasure" => artifact(
            "Treasure",
            "Treasure",
            vec![activated(
                Cost { mana: None, parts: vec![CostPart::Tap, sac] },
                Effect::AddMana { who: PlayerRef::You, mana: ManaProduction::AnyOneColor(Value::c(1)), restriction: None },
                vec![],
                true,
                false,
                "{T}, Sacrifice this token: Add one mana of any color.",
            )],
        ),
        // 111.10b
        "food" => artifact(
            "Food",
            "Food",
            vec![activated(
                Cost { mana: mc("{2}"), parts: vec![CostPart::Tap, sac] },
                Effect::GainLife { who: PlayerRef::You, n: Value::c(3) },
                vec![],
                false,
                false,
                "{2}, {T}, Sacrifice this token: You gain 3 life.",
            )],
        ),
        // 111.10c
        "gold" => artifact(
            "Gold",
            "Gold",
            vec![activated(
                Cost { mana: None, parts: vec![sac] },
                Effect::AddMana { who: PlayerRef::You, mana: ManaProduction::AnyOneColor(Value::c(1)), restriction: None },
                vec![],
                true,
                false,
                "Sacrifice this token: Add one mana of any color.",
            )],
        ),
        // 111.10d
        "walker" => TokenSpec {
            name: SmolStr::new("Walker"),
            colors: ColorSet::single(Color::Black),
            supertypes: vec![],
            card_types: vec![CardType::Creature],
            subtypes: vec![SmolStr::new("Zombie")],
            power: Some(2),
            toughness: Some(2),
            abilities: vec![],
            scryfall_name: Some(SmolStr::new("Walker")),
        },
        // 111.10e
        "shard" => {
            let mut t = artifact(
                "Shard",
                "Shard",
                vec![activated(
                    Cost { mana: mc("{2}"), parts: vec![sac] },
                    Effect::seq(vec![
                        Effect::Scry { who: PlayerRef::You, n: Value::c(1) },
                        Effect::Draw { who: PlayerRef::You, n: Value::c(1) },
                    ]),
                    vec![],
                    false,
                    false,
                    "{2}, Sacrifice this token: Scry 1, then draw a card.",
                )],
            );
            t.card_types = vec![CardType::Enchantment];
            t
        }
        // 111.10f
        "clue" => artifact(
            "Clue",
            "Clue",
            vec![activated(
                Cost { mana: mc("{2}"), parts: vec![sac] },
                Effect::Draw { who: PlayerRef::You, n: Value::c(1) },
                vec![],
                false,
                false,
                "{2}, Sacrifice this token: Draw a card.",
            )],
        ),
        // 111.10g
        "blood" => artifact(
            "Blood",
            "Blood",
            vec![activated(
                Cost {
                    mana: mc("{1}"),
                    parts: vec![CostPart::Tap, CostPart::Discard { filter: Filter::Any, count: Value::c(1), random: false }, sac],
                },
                Effect::Draw { who: PlayerRef::You, n: Value::c(1) },
                vec![],
                false,
                false,
                "{1}, {T}, Discard a card, Sacrifice this token: Draw a card.",
            )],
        ),
        // 111.10h
        "powerstone" => artifact(
            "Powerstone",
            "Powerstone",
            vec![activated(
                Cost::tap(),
                Effect::AddMana {
                    who: PlayerRef::You,
                    mana: ManaProduction::Fixed(vec![ManaType::C]),
                    restriction: Some(ManaRestriction::NotNonartifactSpell),
                },
                vec![],
                true,
                false,
                "{T}: Add {C}. This mana can't be spent to cast a nonartifact spell.",
            )],
        ),
        // 111.10k
        "monster" => role(
            "Monster",
            vec![Modification::ModifyPT(Value::c(1), Value::c(1)), Modification::AddKeyword(Keyword::new(KeywordKind::Trample))],
            vec![],
        ),
        // 111.10j
        "cursed" => role("Cursed", vec![Modification::SetPT(Some(Value::c(1)), Some(Value::c(1)))], vec![]),
        // 111.10m
        "royal" => role(
            "Royal",
            vec![
                Modification::ModifyPT(Value::c(1), Value::c(1)),
                Modification::AddKeyword(Keyword::with_cost(KeywordKind::Ward, Cost::mana(ManaCost::parse("{1}").unwrap()))),
            ],
            vec![],
        ),
        // 111.10p
        "virtuous" => role(
            "Virtuous",
            vec![Modification::ModifyPT(
                Value::Count(Filter::Type(CardType::Enchantment).you_control()),
                Value::Count(Filter::Type(CardType::Enchantment).you_control()),
            )],
            vec![],
        ),
        // 111.10q
        "wicked" => role(
            "Wicked",
            vec![Modification::ModifyPT(Value::c(1), Value::c(1))],
            vec![AbilityDef::new(
                AbilityKind::Triggered(TriggeredAbility::new(
                    TriggerCond::Dies(Filter::Source),
                    Body::effect(Effect::LoseLife { who: PlayerRef::EachOpponent, n: Value::c(1) }),
                )),
                "When this token is put into a graveyard from the battlefield, each opponent loses 1 life.",
            )],
        ),
        // 111.10s
        "map" => artifact(
            "Map",
            "Map",
            vec![activated(
                Cost { mana: mc("{1}"), parts: vec![CostPart::Tap, sac] },
                Effect::KeywordAction {
                    action: KeywordAction::Explore,
                    who: PlayerRef::You,
                    what: Sel::Target(0),
                    n: Value::c(1),
                },
                vec![TargetSpec::object(Filter::creature().you_control(), "target creature you control")],
                false,
                true,
                "{1}, {T}, Sacrifice this token: Target creature you control explores. Activate only as a sorcery.",
            )],
        ),
        // 111.10t
        "junk" => artifact(
            "Junk",
            "Junk",
            vec![activated(
                Cost { mana: None, parts: vec![CostPart::Tap, sac] },
                Effect::seq(vec![
                    Effect::Dig {
                        who: PlayerRef::You,
                        n: Value::c(1),
                        reveal: false,
                        filter: Filter::Any,
                        take: Value::c(1),
                        take_up_to: false,
                        take_to: Destination::zone(ZoneKind::Exile),
                        rest_to: Destination::library_bottom(),
                    },
                    Effect::GrantPlayPermission { who: PlayerRef::You, what: Sel::Var(vars::IT), duration: Duration::EndOfTurn, free: false },
                ]),
                vec![],
                false,
                true,
                "{T}, Sacrifice this token: Exile the top card of your library. You may play that card this turn. Activate only as a sorcery.",
            )],
        ),
        // 111.10u
        "lander" => artifact(
            "Lander",
            "Lander",
            vec![activated(
                Cost { mana: mc("{2}"), parts: vec![CostPart::Tap, sac] },
                Effect::Search {
                    who: PlayerRef::You,
                    whose: PlayerRef::You,
                    filter: Filter::and(vec![Filter::Type(CardType::Land), Filter::Supertype(Supertype::Basic)]),
                    count: Value::c(1),
                    to: Destination::battlefield().tapped(),
                    reveal: false,
                    shuffle: true,
                },
                vec![],
                false,
                false,
                "{2}, {T}, Sacrifice this token: Search your library for a basic land card, put it onto the battlefield tapped, then shuffle.",
            )],
        ),
        // 111.10v
        "mutagen" => artifact(
            "Mutagen",
            "Mutagen",
            vec![activated(
                Cost { mana: mc("{1}"), parts: vec![CostPart::Tap, sac] },
                Effect::AddCounters { what: Sel::Target(0), kind: counters::PLUS1.into(), n: Value::c(1) },
                vec![TargetSpec::object(Filter::creature(), "target creature")],
                false,
                true,
                "{1}, {T}, Sacrifice this token: Put a +1/+1 counter on target creature. Activate only as a sorcery.",
            )],
        ),
        // 111.10x
        "heartwood" => {
            let mut t = artifact(
                "Heartwood",
                "Heartwood",
                vec![activated(
                    Cost::tap(),
                    Effect::AddMana { who: PlayerRef::You, mana: ManaProduction::OneOf(vec![ManaType::R, ManaType::G]), restriction: None },
                    vec![],
                    true,
                    false,
                    "{T}: Add {R} or {G}.",
                )],
            );
            t.colors = [Color::Red, Color::Green].into_iter().collect();
            t
        }
        _ => return None,
    })
}
