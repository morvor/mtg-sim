//! Commander (CR 903): color identity (CR 903.4), the deck-construction rules that
//! depend on it (CR 903.5c, 903.5d, 903.12e), a commander's color chosen before the game
//! (CR 903.4b), bringing cards into a Commander game from outside it (CR 903.11), and
//! "your commander" (CR 903.3d, 903.3e).

use crate::ability::*;
use crate::card::CardDef;
use crate::deck::DeckProblem;
use crate::eval::Ctx;
use crate::game::{Game, Variant};
use crate::mana::{ManaSymbol, ManaType};
use crate::object::{ObjKind, Zone};
use crate::replacement::MoveEv;
use crate::types::*;
use std::sync::Arc;

/// `Value::Custom` name: "your commander's mana value" (CR 903.3e).
pub const YOUR_COMMANDER_MANA_VALUE: &str = "your commander's mana value";
/// `Effect::Custom` name prefix: a merged or melded commander that would be put into its
/// owner's hand or library goes to the command zone split up (CR 903.9c); the original
/// destination follows as JSON.
pub const SPLIT_TO_COMMAND_ZONE: &str = "commander split to command zone:";

/// The colors of the mana symbols in `text`, ignoring reminder text (CR 903.4, 903.4c).
fn text_symbol_colors(text: &str) -> ColorSet {
    let mut out = ColorSet::NONE;
    let mut depth = 0u32;
    let mut sym: Option<String> = None;
    for ch in text.chars() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            '{' if depth == 0 => sym = Some(String::new()),
            '}' => {
                if let Some(s) = sym.take() {
                    if let Some(m) = ManaSymbol::parse(&s) {
                        out = out.union(m.colors());
                    }
                }
            }
            c => {
                if let Some(s) = sym.as_mut() {
                    s.push(c);
                }
            }
        }
    }
    out
}

/// A card's color identity computed from its characteristics (CR 903.4): the colors of
/// the mana symbols in its mana costs and rules text — reminder text excluded
/// (CR 903.4c) — and the colors given by its color indicators and characteristic-defining
/// abilities, over every face: the back face of a double-faced card (CR 903.4d) and
/// alternative characteristics such as an adventurer card's Adventure (CR 903.4e).
pub fn computed_color_identity(card: &CardDef) -> ColorSet {
    let mut out = ColorSet::NONE;
    for f in &card.faces {
        let c = &f.chars;
        if let Some(m) = &c.mana_cost {
            out = out.union(m.colors());
        }
        if let Some(ind) = c.color_indicator {
            out = out.union(ind);
        }
        out = out.union(c.colors);
        out = out.union(text_symbol_colors(&c.rules_text));
    }
    out
}

/// A card's color identity (CR 903.4): the one established for the card (Scryfall's, or
/// the colors circled or chosen as the deck was built), together with the one computed
/// from its characteristics.
pub fn color_identity(card: &CardDef) -> ColorSet {
    card.color_identity.union(computed_color_identity(card))
}

/// The color identity of the card `id` represents in a game, as established before the
/// game began (CR 903.4a) — including the color its owner chose for it before the game
/// if it's their commander (CR 903.4b). `None` for an object not represented by a card.
pub fn object_color_identity(g: &Game, id: ObjectId) -> Option<ColorSet> {
    let o = g.obj(id);
    let card = o.card.as_ref()?;
    let mut ci = color_identity(card);
    if o.is_commander && pregame_color_choice(card) {
        if let Some(c) = o.linked_choices.get(&PREGAME_LINK).and_then(|ch| ch.color) {
            ci.insert(c);
        }
    }
    Some(ci)
}

/// Whether a card has "If this card is your commander, choose a color before the game
/// begins" (CR 903.4b, 607.2p).
fn pregame_color_choice(card: &CardDef) -> bool {
    card.front().chars.abilities.iter().any(|a| {
        matches!(&a.kind, AbilityKind::Static(s)
            if matches!(&s.effect, StaticEffect::PregameChoice { kind: ChoiceKind::Color, .. }))
    })
}

/// A commander with a static ability that has its owner choose its color before the game
/// begins, with `color` chosen (CR 903.4b): the choice applies during deck construction,
/// and it's part of the commander's color identity. `None` if the card has no such
/// ability.
pub fn with_chosen_color(card: &CardDef, color: Color) -> Option<CardDef> {
    if !pregame_color_choice(card) {
        return None;
    }
    let mut def = card.clone();
    def.color_identity.insert(color);
    Some(def)
}

/// The combined color identity of the commanders `p` owns in the game (CR 903.4,
/// 702.124c), as established before the game began (CR 903.4a, 903.4b). `None` if they
/// have no commander (CR 903.4f).
pub fn commanders_identity(g: &Game, p: PlayerId) -> Option<ColorSet> {
    let mut out: Option<ColorSet> = None;
    for o in &g.objects {
        if o.is_commander && o.owner == p && g.is_live(o.id) {
            if let Some(ci) = object_color_identity(g, o.id) {
                out = Some(out.unwrap_or(ColorSet::NONE).union(ci));
            }
        }
    }
    out
}

/// The mana types of `p`'s commanders' color identity (see [`commanders_identity`]).
pub fn commander_identity_types(g: &Game, p: PlayerId) -> Vec<ManaType> {
    commanders_identity(g, p)
        .unwrap_or(ColorSet::NONE)
        .iter()
        .map(ManaType::from_color)
        .collect()
}

// --- Deck construction ----------------------------------------------------------------

const BASIC_TYPES: [(&str, Color); 5] = [
    ("Plains", Color::White),
    ("Island", Color::Blue),
    ("Swamp", Color::Black),
    ("Mountain", Color::Red),
    ("Forest", Color::Green),
];

/// The basic land types a card has (on any face).
fn basic_land_types(card: &CardDef) -> Vec<&'static str> {
    BASIC_TYPES
        .iter()
        .filter(|(t, _)| card.faces.iter().any(|f| f.chars.has_subtype(t)))
        .map(|(t, _)| *t)
        .collect()
}

/// The colors of mana a card with a basic land type could produce: those of its basic
/// land types' intrinsic mana abilities (CR 305.6) and of its other mana abilities.
fn producible_colors(card: &CardDef) -> ColorSet {
    let mut out = ColorSet::NONE;
    for (t, c) in BASIC_TYPES {
        if card.faces.iter().any(|f| f.chars.has_subtype(t)) {
            out.insert(c);
        }
    }
    for m in &card.produced_mana {
        if let Some(c) = m.color() {
            out.insert(c);
        }
    }
    out
}

fn is_basic_land(card: &CardDef) -> bool {
    let c = &card.front().chars;
    c.supertypes.contains(Supertype::Basic) && c.card_types.contains(CardType::Land)
}

/// Brawl: if the commander's color identity has no colors, the deck may contain any
/// number of basic lands of one basic land type (CR 903.12e) — they're exempt from the
/// color identity rules. Whether `card` is such a basic land.
pub fn brawl_basic_exception(card: &CardDef, commander_identity: &ColorSet, brawl: bool) -> bool {
    brawl
        && commander_identity.is_colorless()
        && is_basic_land(card)
        && !basic_land_types(card).is_empty()
}

/// CR 903.5d: a card with a basic land type may be included only if each color of mana it
/// could produce is in the commander's color identity. With the Brawl option and a
/// colorless commander, the deck may instead contain basic lands of one basic land type
/// (CR 903.12e).
pub fn basic_land_type_problems(
    cards: &[&Arc<CardDef>],
    commander: &CardDef,
    brawl: bool,
) -> Vec<DeckProblem> {
    let identity = color_identity(commander);
    let mut problems = Vec::new();
    let mut brawl_types: Vec<String> = Vec::new();
    for c in cards {
        if basic_land_types(c).is_empty() {
            continue;
        }
        if brawl_basic_exception(c, &identity, brawl) {
            for t in basic_land_types(c) {
                if !brawl_types.iter().any(|x| x == t) {
                    brawl_types.push(t.to_string());
                }
            }
            continue;
        }
        let colors = producible_colors(c);
        if colors.union(identity) != identity
            && !problems.iter().any(
                |p| matches!(p, DeckProblem::ManaOutsideColorIdentity { name } if *name == c.name),
            )
        {
            problems.push(DeckProblem::ManaOutsideColorIdentity {
                name: c.name.to_string(),
            });
        }
    }
    if brawl_types.len() > 1 {
        problems.push(DeckProblem::BasicLandTypes { types: brawl_types });
    }
    problems
}

// --- Cards from outside the game (CR 903.11) ----------------------------------------

/// CR 903.11, 903.11a: whether moving a card from outside a Commander game into it is
/// forbidden. Traditional cards can be brought in only by rules and special actions (a
/// companion, CR 702.139a) — not by other effects (wishes); and never a card with the same
/// name as a card in the player's starting deck or a card they own in the game, or with a
/// color outside their commander's color identity.
pub fn outside_game_move_forbidden(g: &Game, mv: &MoveEv) -> bool {
    if g.config.variant != Variant::Commander {
        return false;
    }
    let o = g.obj(mv.obj);
    let Zone::Outside(p) = o.zone else {
        return false;
    };
    if matches!(mv.to, Zone::Outside(_) | Zone::Nowhere) || o.kind != ObjKind::Card {
        return false;
    }
    let Some(card) = o.card.as_ref() else {
        return false;
    };
    if crate::variants::is_nontraditional(card) {
        return false;
    }
    // Brought in by an effect of a spell or ability.
    if mv.source.is_some() {
        return true;
    }
    let name = card.name.as_str();
    let in_starting_deck = g
        .start
        .starting_decks
        .get(&p)
        .is_some_and(|d| d.iter().any(|id| g.obj(*id).base.name == name));
    let owned_in_game = g.objects.iter().any(|x| {
        x.id != o.id
            && x.owner == p
            && x.next.is_none()
            && !matches!(x.zone, Zone::Outside(_) | Zone::Nowhere)
            && x.card.as_ref().is_some_and(|c| c.name == name)
    });
    let outside_identity =
        commanders_identity(g, p).is_some_and(|ci| color_identity(card).union(ci) != ci);
    in_starting_deck || owned_in_game || outside_identity
}

/// Whether `card` (outside the game) may be put into `to` in a Commander game by a rule or
/// special action (a companion, CR 702.139a): see [`outside_game_move_forbidden`].
pub fn may_bring_in(g: &Game, card: ObjectId, to: Zone) -> bool {
    !outside_game_move_forbidden(
        g,
        &MoveEv {
            obj: card,
            to,
            pos: LibraryPosition::Top,
            cause: crate::events::MoveCause::Effect,
            by: None,
            etb: Default::default(),
            source: None,
        },
    )
}

/// The commander a source of combat damage is, for counting commander damage
/// (CR 903.10a): the card that is the commander — the commander component of a merged or
/// melded commander (CR 903.3b, 903.3c) — whatever the permanent's name is now.
pub fn commander_damage_key(g: &Game, id: ObjectId) -> smol_str::SmolStr {
    crate::merge::physical_components(g, id)
        .into_iter()
        .find(|c| g.obj(*c).is_commander)
        .map(|c| crate::kw::partner::commander_key(g, c))
        .unwrap_or_else(|| crate::kw::partner::commander_key(g, id))
}

// --- "Your commander" (CR 903.3d, 903.3e) --------------------------------------------

/// The commanders `p` owns, in any zone (CR 903.3e: including the library and hand).
pub fn commanders_of(g: &Game, p: PlayerId) -> Vec<ObjectId> {
    g.objects
        .iter()
        .filter(|o| {
            o.is_commander
                && o.owner == p
                && g.is_live(o.id)
                && !matches!(o.zone, Zone::Nowhere | Zone::Outside(_))
        })
        .map(|o| o.id)
        .collect()
}

/// Values referring to "your commander" (CR 903.3e): its current characteristics, as
/// modified by continuous effects, in whatever zone it's in. With two commanders, the
/// greater value; 0 if the player has no commander.
pub fn custom_value(g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
    if name != YOUR_COMMANDER_MANA_VALUE {
        return None;
    }
    Some(
        commanders_of(g, ctx.controller)
            .into_iter()
            .map(|c| g.mana_value_of(c) as i64)
            .max()
            .unwrap_or(0),
    )
}

// --- Returning to the command zone (CR 903.9b, 903.9c) ------------------------------

/// The action of the commander replacement effect (CR 903.9b) for a move: the command zone
/// instead; for a merged or melded commander, its commander card goes there and its other
/// components go where the permanent was going (CR 903.9c).
pub fn command_zone_instead(g: &Game, m: &MoveEv) -> ReplacementAction {
    if !crate::merge::is_merged(g, m.obj) {
        return ReplacementAction::MoveInstead(Destination::zone(ZoneKind::Command));
    }
    let dest = serde_json::to_string(&(m.to, m.pos, m.cause, m.by)).unwrap_or_default();
    ReplacementAction::Instead(Box::new(Effect::Custom(
        format!("{SPLIT_TO_COMMAND_ZONE}{dest}").into(),
    )))
}

/// Performs [`command_zone_instead`] for a merged or melded commander: it leaves for the
/// command zone with a commander card on top, and its other components go to the original
/// destination.
pub fn custom_effect(g: &mut Game, name: &str, ctx: &Ctx) -> bool {
    let Some(json) = name.strip_prefix(SPLIT_TO_COMMAND_ZONE) else {
        return false;
    };
    let Ok((to, pos, cause, by)) = serde_json::from_str::<(
        Zone,
        LibraryPosition,
        crate::events::MoveCause,
        Option<PlayerId>,
    )>(json) else {
        return true;
    };
    let Some(obj) = ctx.event.as_ref().and_then(|e| e.object) else {
        return true;
    };
    if !g.is_live(obj) {
        return true;
    }
    // A commander card represents the permanent as it goes to the command zone.
    let comps = g.obj(obj).merged_with.clone();
    let top = comps.iter().position(|c| {
        crate::merge::physical_components(g, *c)
            .first()
            .copied()
            .or(Some(*c))
            .is_some_and(|x| g.obj(x).is_commander)
    });
    if let Some(i) = top.filter(|i| *i > 0) {
        let mut v = comps;
        let c = v.remove(i);
        v.insert(0, c);
        g.objects[obj.0 as usize].merged_with = v;
    }
    g.merges.commander_split = Some((to, pos));
    g.move_object_ev(MoveEv {
        obj,
        to: Zone::Command,
        pos,
        cause,
        by,
        etb: Default::default(),
        source: None,
    });
    g.merges.commander_split = None;
    true
}

// --- Commander Draft (CR 903.13) ------------------------------------------------------

/// Set codes of the draft boosters that give Commander Draft players extra options
/// (CR 903.13e, 903.13f).
pub const COMMANDER_LEGENDS: &str = "cmr";
pub const COMMANDER_MASTERS: &str = "cmm";
pub const BALDURS_GATE: &str = "clb";

/// Checks a Commander Draft deck (CR 903.13f): the Commander deck construction rules
/// (CR 903.5) except that the deck must contain at least 60 cards with no maximum, it may
/// include any number of cards from the player's card pool with the same name, and with
/// Commander Masters boosters any card that can be a commander by itself and whose color
/// identity has one or fewer colors is considered to have partner. `deck` includes the
/// commanders; `pool` is the player's card pool; `sets` are the codes of the draft
/// boosters used. A player may add up to two cards named The Prismatic Piper (with
/// Commander Legends or Commander Masters boosters) or Faceless One (with Battle for
/// Baldur's Gate boosters) to their pool, but only as their commanders (CR 903.13e).
pub fn check_commander_draft_deck(
    deck: &[Arc<CardDef>],
    commanders: &[Arc<CardDef>],
    pool: &[Arc<CardDef>],
    sets: &[&str],
) -> Vec<DeckProblem> {
    use crate::kw::partner::{can_be_commander, commanders_problem, ineligible_commander};
    let mut problems = Vec::new();
    let refs: Vec<&CardDef> = commanders.iter().map(|c| c.as_ref()).collect();
    let masters_partners = sets.contains(&COMMANDER_MASTERS)
        && refs.len() == 2
        && refs
            .iter()
            .all(|c| can_be_commander(c, false) && color_identity(c).count() <= 1);
    if !masters_partners {
        if let Some(reason) = commanders_problem(&refs) {
            problems.push(DeckProblem::InvalidCommanders { reason });
        }
    }
    if let Some(reason) = ineligible_commander(&refs, false) {
        problems.push(DeckProblem::InvalidCommanders { reason });
    }
    let cards: Vec<&Arc<CardDef>> = deck
        .iter()
        .filter(|c| !crate::variants::is_nontraditional(c))
        .collect();
    if cards.len() < 60 {
        problems.push(DeckProblem::TooFewCards {
            have: cards.len(),
            min: 60,
        });
    }
    // The cards must come from the pool (basic lands may be added, CR 100.2b).
    let extra_allowed = |name: &str| -> usize {
        let named = |sets_ok: bool, n: &str| {
            if sets_ok && name == n {
                commanders.iter().filter(|c| c.name == n).count().min(2)
            } else {
                0
            }
        };
        named(
            sets.contains(&COMMANDER_LEGENDS) || sets.contains(&COMMANDER_MASTERS),
            "The Prismatic Piper",
        ) + named(sets.contains(&BALDURS_GATE), "Faceless One")
    };
    let mut names: Vec<&str> = cards.iter().map(|c| c.name.as_str()).collect();
    names.sort_unstable();
    names.dedup();
    for name in names {
        let card = cards.iter().find(|c| c.name == name).expect("named card");
        if is_basic_land(card) {
            continue;
        }
        let have = cards.iter().filter(|c| c.name == name).count();
        let available = pool.iter().filter(|c| c.name == name).count() + extra_allowed(name);
        if have > available {
            problems.push(DeckProblem::NotInCardPool {
                name: name.to_string(),
            });
        }
    }
    // Color identity (CR 903.5c, 903.5d).
    let Some(first) = commanders.first() else {
        return problems;
    };
    let mut combined = (**first).clone();
    for c in commanders {
        combined.color_identity = combined.color_identity.union(color_identity(c));
    }
    let identity = color_identity(&combined);
    for c in &cards {
        if color_identity(c).union(identity) != identity {
            problems.push(DeckProblem::OutsideColorIdentity {
                name: c.name.to_string(),
            });
        }
    }
    problems.extend(basic_land_type_problems(&cards, &combined, false));
    problems
}
