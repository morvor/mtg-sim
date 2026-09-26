//! Adventurer cards (CR 715) and omen cards (CR 720): cards whose inset frame gives
//! alternative characteristics the object may have as a spell — an Adventure or an Omen
//! (Scryfall's "adventure" layout for both; the inset's subtype tells which).
//!
//! * A player casting the card chooses to cast it normally or as its inset spell
//!   (`CastMethod::Half(1)`, CR 715.3, 720.3); only the inset's characteristics are
//!   evaluated and exist on the stack (CR 715.3a–b, 720.3a–b), a copy of such a spell is
//!   one too (CR 715.3c, 720.3c), and elsewhere the card has only its normal
//!   characteristics (CR 715.4, 720.4).
//! * As a spell cast as an Adventure resolves, its controller exiles it instead of
//!   putting it into its owner's graveyard, and may play it for as long as it remains
//!   exiled — but not as an Adventure (CR 715.3d). An Omen spell is shuffled into its
//!   owner's library instead (CR 720.3d).
//! * "Has an Adventure" / "has an Omen" (CR 715.2a, 720.2a): the card's inset is part of
//!   the copiable values (CR 715.2b, 720.2b, [`Characteristics::printed`]).

use crate::ability::LibraryPosition;
use crate::card::{CardDef, Layout};
use crate::casting::CastOption;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::*;
use crate::types::*;

/// The kind of inset spell of an adventurer or omen card.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inset {
    Adventure,
    Omen,
}

/// `Filter::Custom`: "that has an Adventure" (CR 715.2a).
pub const HAS_ADVENTURE: &str = "has an adventure";
/// `Filter::Custom`: "that has an Omen" (CR 720.2a).
pub const HAS_OMEN: &str = "has an omen";

/// The inset spell a card has, if it's an adventurer or omen card.
pub fn inset_of(card: &CardDef) -> Option<Inset> {
    if card.layout != Layout::Adventure || card.faces.len() < 2 {
        return None;
    }
    let f = &card.faces[1].chars;
    if f.has_subtype("Omen") {
        Some(Inset::Omen)
    } else if f.has_subtype("Adventure") {
        Some(Inset::Adventure)
    } else {
        None
    }
}

/// The inset spell the object `id` has — even if it doesn't currently use those
/// characteristics (CR 715.2a, 720.2a) — from its copiable values (CR 715.2b, 720.2b).
pub fn inset(g: &Game, id: ObjectId) -> Option<Inset> {
    let o = g.obj(id);
    if o.face_down {
        return None;
    }
    let printed = o.chars.printed.as_ref().map(|p| &p.0);
    // Cards whose characteristics aren't computed (in a library) are their own card.
    let uncomputed = matches!(o.zone, Zone::Library(_) | Zone::Nowhere);
    let card = printed.or(o.card.as_ref().filter(|_| uncomputed))?;
    inset_of(card)
}

/// The inset spell `id` is on the stack as (cast as an Adventure or Omen, or a copy of
/// such a spell, CR 715.3c, 720.3c).
pub fn on_stack_as(g: &Game, id: ObjectId) -> Option<Inset> {
    let o = g.obj(id);
    if o.zone != Zone::Stack || o.face != FaceState::Half(1) {
        return None;
    }
    o.card.as_deref().and_then(inset_of)
}

/// Where a resolving spell cast as an Adventure or Omen goes instead of its owner's
/// graveyard (CR 715.3d, 720.3d).
pub fn resolved_destination(g: &Game, id: ObjectId) -> Option<(Zone, LibraryPosition)> {
    match on_stack_as(g, id)? {
        Inset::Adventure => Some((Zone::Exile, LibraryPosition::Top)),
        Inset::Omen => Some((Zone::Library(g.obj(id).owner), LibraryPosition::Shuffled)),
    }
}

/// Cards exiled as Adventure spells resolved, with the player who may play them.
#[derive(Clone, Debug, Default)]
pub struct AdventureState {
    pub exiled: Vec<(ObjectId, PlayerId)>,
}

/// After a spell cast as an Adventure resolved and was exiled as `new`: its controller
/// may play that card for as long as it remains exiled (CR 715.3d).
pub fn after_resolved(g: &mut Game, spell: ObjectId, new: ObjectId) {
    if on_stack_as(g, spell) != Some(Inset::Adventure) {
        return;
    }
    let o = g.obj(new);
    if o.zone != Zone::Exile || o.kind != ObjKind::Card {
        return;
    }
    let p = g.obj(spell).controller;
    g.special.adventures.exiled.push((new, p));
}

/// Whether `p` may play the exiled card `card` because it resolved as an Adventure spell
/// they controlled (CR 715.3d).
pub fn on_an_adventure_for(g: &Game, card: ObjectId, p: PlayerId) -> bool {
    g.is_live(card)
        && g.obj(card).zone == Zone::Exile
        && g.special
            .adventures
            .exiled
            .iter()
            .any(|(c, q)| *c == card && *q == p)
}

/// Hooks for adventurer and omen cards in the keyword registry (they aren't keywords).
pub struct AdventureRules;

impl KeywordRules for AdventureRules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    /// CR 715.3d: the player may cast the exiled card — normally, not as an Adventure.
    fn global_cast_options(&self, g: &Game, p: PlayerId, card: ObjectId) -> Vec<CastOption> {
        if on_an_adventure_for(g, card, p) {
            vec![CastOption::normal(FaceState::Front)]
        } else {
            vec![]
        }
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        match name {
            HAS_ADVENTURE => Some(inset(g, id) == Some(Inset::Adventure)),
            HAS_OMEN => Some(inset(g, id) == Some(Inset::Omen)),
            _ => None,
        }
    }
}

inventory::submit! { KeywordRegistration(&AdventureRules) }
