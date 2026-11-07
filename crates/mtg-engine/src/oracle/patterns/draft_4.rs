//! Conspiracy draft-matters cards (CR 905.2), round 3, on top of `r905_draft.rs`:
//!
//! - abilities that function during the draft (`crate::draft` applies them): "Reveal ~ as
//!   you draft it. Reveal the next card you draft and note its name.", "As you draft a
//!   card, you may remove it from the draft face up/down.", "As you draft a creature card,
//!   you may reveal it, note its name (its creature types), then turn ~ face down.",
//!   "... The player to your right chooses a color, you choose another color, then the
//!   player to your left chooses a third color.", "As you draft a card, you may draft an
//!   additional card from that booster pack. If you do, put ~ into that booster pack / turn
//!   ~ face down, then pass the next booster pack without drafting a card from it.";
//! - what the game reads of them (CR 905.2b): "a card with a name you noted for cards
//!   named ~", "of a type you noted for cards named ~", "one or more of the colors chosen
//!   as you drafted cards named ~", "the number of cards you removed from the draft with
//!   cards named ~", "If you removed a creature card with flying from the draft with cards
//!   named ~, ~ has flying. The same is true for ...", "a player you noted for cards named
//!   ~", "one mana of any color chosen as you drafted cards named ~".
//!
//! The behaviors are in `crate::draft` and `kw/draft_4.rs`.

use super::{AbilityPattern, ConditionPattern, EffectPattern, FilterSuffixPattern, StaticPattern};
use crate::ability::*;
use crate::draft::*;
use crate::kw::draft_4::{NOTED_PLAYER, REMOVED_KEYWORDS};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use smol_str::SmolStr;

fn draft_static(name: &str, text: &str) -> Ability {
    let mut s = StaticAbility::new(StaticEffect::Custom(SmolStr::new(name)));
    s.zone = FunctionZone::Anywhere;
    AbilityDef::new(AbilityKind::Static(s), text)
}

/// Abilities that function during the draft (CR 905.2): markers `crate::draft` reads as
/// the card is drafted. They do nothing during the game.
fn draft_abilities(text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = text.trim().to_lowercase();
    let name = match end(&l) {
        "reveal ~ as you draft it. reveal the next card you draft and note its name" => {
            DRAFT_NOTE_NEXT_NAME.to_string()
        }
        "as you draft a card, you may remove it from the draft face up" => {
            DRAFT_REMOVE_FACE_UP.to_string()
        }
        "as you draft a card, you may remove it from the draft face down" => {
            DRAFT_REMOVE_FACE_DOWN.to_string()
        }
        "as you draft a card, you may reveal it, note its name, then turn ~ face down" => {
            DRAFT_NOTE_NAME.to_string()
        }
        "as you draft a creature card, you may reveal it, note its name, then turn ~ face down" => {
            format!("{DRAFT_NOTE_NAME}:creature")
        }
        "as you draft a creature card, you may reveal it, note its creature types, then turn ~ face down" => {
            DRAFT_NOTE_TYPES.to_string()
        }
        "reveal ~ as you draft it. the player to your right chooses a color, you choose another color, then the player to your left chooses a third color" => {
            DRAFT_THREE_COLORS.to_string()
        }
        "as you draft a card, you may draft an additional card from that booster pack. if you do, put ~ into that booster pack" => {
            DRAFT_ADDITIONAL_PUT_BACK.to_string()
        }
        "as you draft a card, you may draft an additional card from that booster pack. if you do, turn ~ face down, then pass the next booster pack without drafting a card from it" => {
            DRAFT_ADDITIONAL_SKIP_NEXT.to_string()
        }
        "reveal ~ as you draft it. the next time a player drafts a card from this booster pack, guess that card's name. then that player reveals the drafted card" => {
            DRAFT_GUESS_NEXT.to_string()
        }
        "reveal ~ as you draft it. you may look at the next card drafted from this booster pack" => {
            DRAFT_LOOK_NEXT.to_string()
        }
        _ => return None,
    };
    Some(vec![draft_static(&name, text)])
}

inventory::submit! { AbilityPattern { name: "draft 4: abilities that function during the draft", priority: 100, parse: draft_abilities } }

/// "with a name you noted for cards named ~", "with a name noted as you drafted cards
/// named ~", "of a type you noted for cards named ~", "that's one or more of the colors
/// chosen as you drafted cards named ~": what the filter's player noted for cards with the
/// source's name (CR 905.2b).
fn noted_suffix<'a>(s: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let t = s.trim_start();
    for (p, name) in [
        ("with a name you noted for cards named ~", NOTED_NAME),
        ("with a name noted as you drafted cards named ~", NOTED_NAME),
        ("of a type you noted for cards named ~", NOTED_TYPE),
        (
            "that's one or more of the colors chosen as you drafted cards named ~",
            CHOSEN_COLOR,
        ),
    ] {
        if let Some(rest) = t.strip_prefix(p) {
            return Some((Filter::Custom(SmolStr::new(name)), rest));
        }
    }
    None
}

inventory::submit! { FilterSuffixPattern { name: "draft 4: noted for cards named ~", priority: 100, parse: noted_suffix } }

/// "the number of cards you removed from the draft with cards named ~" (Cogwork Grinder),
/// hooked into the value grammar by `dice_coin_ext::value`.
pub fn value(s: &str) -> Option<(Value, String)> {
    let rest =
        s.strip_prefix("the number of cards you removed from the draft with cards named ~")?;
    Some((Value::Custom(SmolStr::new(REMOVED_COUNT)), rest.to_string()))
}

/// "If you removed a creature card with flying from the draft with cards named ~, ~ has
/// flying. The same is true for first strike, double strike, ..." (Animus of Predation):
/// for each keyword, the keyword abilities of that kind of the creature cards removed
/// that way (variants included, its rulings).
fn removed_creature_keywords(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let r = l.strip_prefix("if you removed a creature card with ")?;
    let (first, r) = r.split_once(" from the draft with cards named ~, ~ has ")?;
    let (again, more) = r.split_once(". the same is true for ")?;
    if first != again {
        return None;
    }
    let more = more.replace(", and ", ", ");
    let mut names: Vec<&str> = vec![first];
    names.extend(more.split(", ").map(str::trim));
    let mut out = Vec::new();
    for n in names {
        let (k, rest) = parse_keyword_word(n)?;
        if !rest.is_empty() {
            return None;
        }
        let st = StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::Source,
            mods: vec![Modification::Custom {
                name: SmolStr::new(format!("{REMOVED_KEYWORDS}:{}", k.name())),
                layer: Layer::L6Ability,
            }],
        });
        out.push(AbilityDef::new(AbilityKind::Static(st), text));
    }
    Some(out)
}

/// A keyword without parameters named by `s` ("first strike", "lifelink").
fn parse_keyword_word(s: &str) -> Option<(crate::keywords::KeywordKind, &str)> {
    let k = crate::keywords::KeywordKind::from_name(s.trim())?;
    Some((k, ""))
}

inventory::submit! { StaticPattern { name: "draft 4: removed a creature card with [keyword] from the draft", priority: 100, parse: removed_creature_keywords } }

/// "As long as you control one or more creatures with a name you noted for cards named ~,
/// ~ and those creatures get +1/+1 and have lifelink." (Noble Banneret).
fn noted_creatures_and_this(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let r = l.strip_prefix("as long as you control one or more creatures ")?;
    let (suffix, pred) = r.split_once(", ~ and those creatures ")?;
    let (extra, rest) = noted_suffix(suffix, &Filter::Any)?;
    if !rest.trim().is_empty() {
        return None;
    }
    let those = Filter::and(vec![
        Filter::Type(crate::types::CardType::Creature),
        Filter::ControlledBy(PlayerRel::You),
        extra,
    ]);
    let parsed = crate::oracle::statics::parse_static(
        &format!(
            "~ {}",
            pred.replacen("get ", "gets ", 1)
                .replacen(" and have ", " and has ", 1)
        ),
        ctx,
    )?;
    let [a] = parsed.as_slice() else {
        return None;
    };
    let AbilityKind::Static(s) = &a.kind else {
        return None;
    };
    let StaticEffect::Continuous { mods, .. } = &s.effect else {
        return None;
    };
    let mut st = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::Or(vec![
            Filter::Source,
            Filter::and(vec![those.clone(), Filter::InZone(ZoneKind::Battlefield)]),
        ]),
        mods: mods.clone(),
    });
    st.condition = Some(Condition::Exists(Filter::and(vec![
        those,
        Filter::InZone(ZoneKind::Battlefield),
    ])));
    Some(vec![AbilityDef::new(AbilityKind::Static(st), text)])
}

inventory::submit! { StaticPattern { name: "draft 4: ~ and creatures with a noted name get", priority: 100, parse: noted_creatures_and_this } }

/// "~ attacks a player you noted for cards named ~ each combat if able." (Cogwork
/// Tracker): a requirement to attack one of those players (CR 508.1d); none if no noted
/// player is in the game (its rulings).
fn attacks_noted_player(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if end(l) != "~ attacks a player you noted for cards named ~ each combat if able" {
        return None;
    }
    let st = StaticAbility::new(StaticEffect::Restriction(Restriction::MustAttackPlayer {
        attackers: Filter::Source,
        defender: PlayerFilter::History(Box::new(Condition::Custom(SmolStr::new(NOTED_PLAYER)))),
    }));
    Some(vec![AbilityDef::new(AbilityKind::Static(st), text)])
}

inventory::submit! { StaticPattern { name: "draft 4: attacks a player you noted each combat if able", priority: 100, parse: attacks_noted_player } }

/// "copy a card you exiled with cards named ~", "choose a card at random you exiled with
/// cards named ~": the cards its controller exiled before the game with abilities of cards
/// with this name (CR 607.2n), read like "a card exiled with ~" with that set of cards.
fn exiled_with_cards_named(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains(" you exiled with cards named ~") || b.ctx.card_name.is_empty() {
        return None;
    }
    let e = crate::oracle::effects::parse_clause(
        &l.replace(" you exiled with cards named ~", " exiled with ~"),
        b,
    )?;
    let json = serde_json::to_string(&e).ok()?;
    if !json.contains("\"Linked\"") {
        return None;
    }
    let named =
        serde_json::to_string(&Sel::ExiledWithCardsNamed(SmolStr::new(b.ctx.card_name))).ok()?;
    serde_json::from_str(&json.replace("\"Linked\"", &named)).ok()
}

inventory::submit! { EffectPattern { name: "draft 4: a card you exiled with cards named ~", priority: 100, parse: exiled_with_cards_named } }

/// "you guessed correctly for a card named ~" (Spire Phantasm, CR 905.2b).
fn guessed_correctly(c: &str) -> Option<Condition> {
    (end(c) == "you guessed correctly for a card named ~")
        .then(|| Condition::Custom(SmolStr::new(GUESSED_CORRECTLY)))
}

inventory::submit! { ConditionPattern { name: "draft 4: you guessed correctly for a card named ~", priority: 100, parse: guessed_correctly } }
