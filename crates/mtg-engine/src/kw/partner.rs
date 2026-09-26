//! CR 702.124 Partner abilities, and the commander tax (CR 903.8) they interact with.
//!
//! * The partner abilities modify the rules for deck construction in the Commander variant
//!   and function before the game begins: each lets a player designate two legendary cards
//!   as their commander rather than one, with its own requirements (CR 702.124a): partner
//!   (CR 702.124h), "partner—[text]" (CR 702.124i), "partner with [name]" (CR 702.124j),
//!   "choose a Background" (CR 702.124k), and "Doctor's companion" (CR 702.124m).
//!   [`commanders_problem`] checks a designation; [`check_commander_deck`] a whole deck
//!   (100 cards including both commanders, CR 702.124b, within their combined color
//!   identity, CR 702.124c).
//! * Partner, "partner—[text]", and "partner with [name]" are the keyword
//!   [`KeywordKind::Partner`] (its text says which); "choose a Background" and "Doctor's
//!   companion" are static abilities ([`CHOOSE_A_BACKGROUND`], [`DOCTORS_COMPANION`]), so
//!   an effect that refers to "partner" doesn't see them (CR 702.124n).
//! * Different partner abilities can't be combined (CR 702.124f); a card with several
//!   may use any one of them, and no combination allows more than two commanders
//!   (CR 702.124g).
//! * "Partner with [name]" also means "When this permanent enters, target player may
//!   search their library for a card named [name], reveal it, put it into their hand,
//!   then shuffle." (CR 702.124j).
//! * In the game, two commanders function independently (CR 702.124d): each has its own
//!   commander tax ([`commander_tax`], CR 903.8) and its own count of combat damage dealt
//!   to each player (CR 903.10a).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::card::CardDef;
use crate::deck::DeckProblem;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::{Characteristics, Zone};
use crate::types::*;
use smol_str::SmolStr;
use std::sync::Arc;

/// `StaticEffect::Custom` name of "Choose a Background" (CR 702.124k).
pub const CHOOSE_A_BACKGROUND: &str = "partner:choose a background";
/// `StaticEffect::Custom` name of "[This card] can be your commander." (CR 903.3a).
pub const CAN_BE_YOUR_COMMANDER: &str = "commander:can be your commander";
/// `Value::Custom` name of "the number of times you've cast your commander from the
/// command zone this game": each of your commanders counts (CR 702.124d, 903.8).
pub const COMMANDER_CASTS: &str = "commander:times cast from the command zone";

/// How many times `p` has cast their commanders from the command zone this game — both
/// of them if they have two (CR 702.124e). Casts count as the spell becomes cast, even if
/// it's later countered.
pub fn times_cast_commanders(g: &Game, p: PlayerId) -> u32 {
    let pl = g.player(p);
    pl.commander_casts
        .iter()
        .filter(|(k, _)| pl.commander_names.iter().any(|n| n == *k))
        .map(|(_, n)| *n)
        .sum()
}
/// `StaticEffect::Custom` name of "Doctor's companion" (CR 702.124m).
pub const DOCTORS_COMPANION: &str = "partner:doctor's companion";

/// One of a card's partner abilities (CR 702.124a).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PartnerAbility {
    /// "Partner" (CR 702.124h).
    Partner,
    /// "Partner—[text]" (CR 702.124i), e.g. "Friends forever".
    Text(String),
    /// "Partner with [name]" (CR 702.124j).
    With(String),
    /// "Choose a Background" (CR 702.124k).
    ChooseABackground,
    /// "Doctor's companion" (CR 702.124m).
    DoctorsCompanion,
}

/// The partner ability a partner keyword instance is.
pub fn partner_keyword(kw: &Keyword) -> PartnerAbility {
    let text = kw.text.as_deref().unwrap_or("Partner").trim();
    let lower = text.to_lowercase();
    if lower == "partner" {
        PartnerAbility::Partner
    } else if let Some(t) = text.strip_prefix("Partner—").or_else(|| text.strip_prefix("partner—")) {
        PartnerAbility::Text(t.trim().to_lowercase())
    } else if lower.starts_with("partner with ") {
        PartnerAbility::With(text["partner with ".len()..].trim().to_string())
    } else {
        // The keyword parser keeps only the name of "partner with [name]".
        PartnerAbility::With(text.to_string())
    }
}

/// The partner abilities of a card with these characteristics.
pub fn partner_abilities(c: &Characteristics) -> Vec<PartnerAbility> {
    let mut out = Vec::new();
    for a in &c.abilities {
        match &a.kind {
            AbilityKind::Keyword(k) if k.kind == KeywordKind::Partner => {
                out.push(partner_keyword(k))
            }
            AbilityKind::Static(s) => match &s.effect {
                StaticEffect::Custom(n) if n == CHOOSE_A_BACKGROUND => {
                    out.push(PartnerAbility::ChooseABackground)
                }
                StaticEffect::Custom(n) if n == DOCTORS_COMPANION => {
                    out.push(PartnerAbility::DoctorsCompanion)
                }
                _ => {}
            },
            _ => {}
        }
    }
    out
}

fn legendary(c: &Characteristics) -> bool {
    c.supertypes.contains(Supertype::Legendary)
}

/// A legendary Background enchantment card (CR 702.124k).
fn is_background(c: &Characteristics) -> bool {
    legendary(c) && c.is(CardType::Enchantment) && c.has_subtype("Background")
}

/// A legendary Time Lord Doctor creature card that has no other creature types
/// (CR 702.124m); a changeling is every creature type.
fn is_lone_doctor(c: &Characteristics) -> bool {
    if !legendary(c) || !c.is(CardType::Creature) || super::changeling::every_creature_type(c) {
        return false;
    }
    let types: Vec<&str> = c
        .subtypes
        .iter()
        .filter(|s| is_creature_type(s))
        .map(|s| s.as_str())
        .collect();
    types.len() == 2 && types.contains(&"Time Lord") && types.contains(&"Doctor")
}

/// Whether `a`'s ability `pa` lets `a` and `b` be a player's two commanders (b being the
/// other card, with its own partner abilities `pb`).
fn pair_allowed(a: &Characteristics, pa: &PartnerAbility, b: &Characteristics) -> bool {
    let pb = partner_abilities(b);
    match pa {
        // CR 702.124h: each of them has partner.
        PartnerAbility::Partner => {
            legendary(a) && legendary(b) && pb.contains(&PartnerAbility::Partner)
        }
        // CR 702.124i: each has the same "partner—[text]" ability.
        PartnerAbility::Text(t) => {
            legendary(a) && legendary(b) && pb.contains(&PartnerAbility::Text(t.clone()))
        }
        // CR 702.124j: each has a "partner with [name]" ability with the other's name.
        PartnerAbility::With(name) => {
            legendary(a)
                && legendary(b)
                && b.name.eq_ignore_ascii_case(name)
                && pb.iter().any(
                    |x| matches!(x, PartnerAbility::With(n) if a.name.eq_ignore_ascii_case(n)),
                )
        }
        // CR 702.124k: the other is a legendary Background enchantment card.
        PartnerAbility::ChooseABackground => is_background(b),
        // CR 702.124m: both legendary creature cards, the other a Time Lord Doctor with no
        // other creature types.
        PartnerAbility::DoctorsCompanion => {
            legendary(a) && a.is(CardType::Creature) && is_lone_doctor(b)
        }
    }
}

/// Why the cards can't be designated as a player's commander together, or `None` if they
/// can (CR 702.124, 903.3). One card is always allowed as far as partner abilities go,
/// except a Background, which needs a commander with "choose a Background" (CR 702.124k).
pub fn commanders_problem(commanders: &[&CardDef]) -> Option<String> {
    let chars: Vec<&Characteristics> = commanders.iter().map(|c| &c.front().chars).collect();
    match chars.as_slice() {
        [] => Some("no commander".into()),
        [one] => is_background(one).then(|| {
            format!(
                "{}: a Background can be a commander only alongside a commander with choose a Background",
                one.name
            )
        }),
        [a, b] => {
            // CR 702.124f–g: one partner ability of one card must allow the pair; abilities
            // of different kinds can't be combined.
            let ok = partner_abilities(a).iter().any(|pa| pair_allowed(a, pa, b))
                || partner_abilities(b).iter().any(|pb| pair_allowed(b, pb, a));
            (!ok).then(|| {
                format!(
                    "{} and {} can't both be commanders: no partner ability allows it",
                    a.name, b.name
                )
            })
        }
        // CR 702.124g: never more than two commanders.
        _ => Some("more than two commanders".into()),
    }
}

/// Whether a card may be designated as a commander on its own terms (CR 903.3): a
/// legendary creature card, Vehicle card, or Spacecraft card with a power/toughness box —
/// in Brawl also a legendary planeswalker card (CR 903.12c) — or a card whose own ability
/// says it can be your commander (CR 903.3a; that ability modifies the deck construction
/// rules, CR 113.6n). A Background is a commander only through a "choose a Background"
/// partner (CR 702.124k, see [`commanders_problem`]).
pub fn can_be_commander(card: &CardDef, brawl: bool) -> bool {
    let c = &card.front().chars;
    let says_so = c.abilities.iter().any(|a| {
        matches!(&a.kind, AbilityKind::Static(s)
            if matches!(&s.effect, StaticEffect::Custom(n) if n == CAN_BE_YOUR_COMMANDER))
    });
    if says_so {
        return true;
    }
    legendary(c)
        && (c.is(CardType::Creature)
            || c.has_subtype("Vehicle")
            || (c.has_subtype("Spacecraft") && c.power.is_some() && c.toughness.is_some())
            || (brawl && c.is(CardType::Planeswalker)))
}

/// Why one of the cards can't be a commander at all (CR 903.3, 903.3a, 903.12c): each
/// must be allowed by [`can_be_commander`], except a Background alongside a commander
/// with "choose a Background" (CR 702.124k).
pub fn ineligible_commander(commanders: &[&CardDef], brawl: bool) -> Option<String> {
    let chooses_background = commanders.iter().any(|c| {
        partner_abilities(&c.front().chars).contains(&PartnerAbility::ChooseABackground)
    });
    commanders
        .iter()
        .find(|c| {
            !can_be_commander(c, brawl)
                && !(chooses_background && is_background(&c.front().chars))
        })
        .map(|c| {
            format!(
                "{} can't be a commander: it isn't a legendary creature, Vehicle, or Spacecraft card{}",
                c.name,
                if brawl { " or planeswalker card" } else { "" }
            )
        })
}

/// Checks a Commander deck led by one or two commanders (CR 903.5, 702.124b–c): the
/// designation must be allowed (see [`commanders_problem`]), the deck must contain
/// exactly 100 cards including its commanders, and each card's color identity must be
/// within the commanders' combined color identity.
pub fn check_commander_deck(
    deck: &[Arc<CardDef>],
    commanders: &[Arc<CardDef>],
    sideboard: &[Arc<CardDef>],
    brawl: bool,
) -> Vec<DeckProblem> {
    let refs: Vec<&CardDef> = commanders.iter().map(|c| c.as_ref()).collect();
    let mut problems: Vec<DeckProblem> = commanders_problem(&refs)
        .into_iter()
        .chain(ineligible_commander(&refs, brawl))
        .map(|reason| DeckProblem::InvalidCommanders { reason })
        .collect();
    let Some(first) = commanders.first() else {
        return problems;
    };
    // CR 702.124c: the combined color identity.
    let mut combined = (**first).clone();
    for c in commanders.iter().skip(1) {
        combined.color_identity = combined.color_identity.union(c.color_identity);
    }
    problems.extend(crate::deck::check_commander_cards(
        deck, &combined, sideboard, brawl,
    ));
    problems
}

/// CR 903.8: a commander cast from the command zone costs an additional {2} for each
/// previous time the player casting it has cast it from the command zone that game. Each
/// of two commanders is counted separately (CR 702.124d). `card` is the card being
/// considered for casting (in the command zone) or the spell being cast from there.
pub fn commander_tax(g: &Game, p: PlayerId, card: ObjectId) -> u32 {
    let o = g.obj(card);
    if !o.is_commander {
        return 0;
    }
    let from_command = match o.zone {
        Zone::Command => true,
        Zone::Stack => o
            .stack
            .as_deref()
            .is_some_and(|s| s.cast.from == Some(ZoneKind::Command)),
        _ => false,
    };
    if !from_command {
        return 0;
    }
    let casts = g
        .player(p)
        .commander_casts
        .get(&commander_key(g, card))
        .copied()
        .unwrap_or(0);
    2 * casts
}

/// The key under which casts of the commander `id` from the command zone are counted
/// (`Player::commander_casts`, see `Game::cast_spell`): the card's name, whichever face
/// was cast (a modal double-faced commander is one commander, CR 903.8).
pub fn commander_key(g: &Game, id: ObjectId) -> SmolStr {
    let o = g.obj(id);
    o.card
        .as_ref()
        .map_or_else(|| o.chars.name.clone(), |c| c.name.clone())
}

/// The commanders `p` owns in the command zone, which they may cast from there (CR 903.8).
pub fn castable_commanders(g: &Game, p: PlayerId) -> Vec<ObjectId> {
    g.command
        .iter()
        .copied()
        .filter(|c| {
            let o = g.obj(*c);
            o.is_commander && o.owner == p && !o.face_down
        })
        .collect()
}

/// "When this permanent enters, target player may search their library for a card named
/// [name], reveal it, put it into their hand, then shuffle." (CR 702.124j)
fn partner_with_trigger(name: &str) -> Ability {
    let search = Effect::Search {
        who: PlayerRef::Target(0),
        whose: PlayerRef::Target(0),
        filter: Filter::Named(SmolStr::new(name)),
        count: Value::c(1),
        to: Destination::zone(ZoneKind::Hand),
        reveal: true,
        shuffle: true,
    };
    AbilityDef::new(
        AbilityKind::Triggered(TriggeredAbility::new(
            TriggerCond::EntersBattlefield(Filter::Source),
            Body::simple(
                vec![TargetSpec::player(PlayerFilter::Any, "target player")],
                Effect::May {
                    who: PlayerRef::Target(0),
                    effect: Box::new(search),
                },
            ),
        )),
        format!("Partner with {name}"),
    )
}

pub struct Partner;

impl KeywordRules for Partner {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Partner]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        match partner_keyword(kw) {
            PartnerAbility::With(name) => Some(vec![partner_with_trigger(&name)]),
            _ => Some(vec![]),
        }
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &crate::eval::Ctx) -> Option<i64> {
        (name == COMMANDER_CASTS).then(|| times_cast_commanders(g, ctx.controller) as i64)
    }
}

inventory::submit! { KeywordRegistration(&Partner) }
