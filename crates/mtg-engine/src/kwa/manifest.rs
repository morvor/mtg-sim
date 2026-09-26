//! CR 701.40: manifest, CR 701.58: cloak, and CR 701.62: manifest dread.
//!
//! * To manifest a card, turn it face down; it becomes a 2/2 face-down creature card with
//!   no text, no name, no subtypes, and no mana cost, and is put onto the battlefield face
//!   down (CR 701.40a). To cloak a card is the same, except the face-down creature also has
//!   ward {2} (CR 701.58a). The face-down status is recorded as "Manifest" / "Cloak" (see
//!   `facedown.rs`); the permanent is a manifested / cloaked permanent for as long as it
//!   remains face down.
//! * Turning a manifested or cloaked permanent face up by paying the mana cost of a
//!   creature card is a special action (CR 701.40b, 701.58b; `special_actions.rs`); one
//!   that would have morph or disguise face up may use either procedure (CR 701.40c–d,
//!   701.58c–d).
//! * Several cards from a library are manifested or cloaked one at a time (CR 701.40e,
//!   701.58e).
//! * A card that a rule or effect prohibits from entering the battlefield face down isn't
//!   manifested or cloaked: it stays where it was, unchanged (CR 701.40f, 701.58f).
//! * A manifested or cloaked permanent represented by an instant or sorcery card that
//!   would turn face up is revealed and stays face down (CR 701.40g, 701.58g;
//!   `facedown::turn_face_up`).
//! * Manifest dread: look at the top two cards of your library, manifest one of them, and
//!   put the rest into your graveyard (CR 701.62a). "Whenever you manifest dread"
//!   triggers once the process is complete (CR 701.62b): a `"manifest dread"` event.

use super::*;
use crate::events::MoveCause;
use crate::keywords::KeywordKind;
use crate::replacement::{EtbInfo, MoveEv};

/// The face-down kinds recorded for manifested and cloaked permanents.
pub const MANIFESTED: &str = "Manifest";
pub const CLOAKED: &str = "Cloak";
/// `Event::Custom` name reported when a player manifests dread.
pub const MANIFESTED_DREAD: &str = "manifest dread";

/// Puts `card` onto the battlefield face down under `p`'s control as a manifested
/// (`kind` = [`MANIFESTED`]) or cloaked ([`CLOAKED`]) permanent. Returns the permanent, or
/// None if it couldn't enter (it stays where it was).
pub fn put_face_down(
    g: &mut Game,
    card: ObjectId,
    p: PlayerId,
    kind: &str,
    source: Option<ObjectId>,
) -> Option<ObjectId> {
    if !g.is_live(card) || g.obj(card).zone == Zone::Battlefield {
        return None;
    }
    let new = g.move_object_ev(MoveEv {
        obj: card,
        to: Zone::Battlefield,
        pos: LibraryPosition::Top,
        cause: MoveCause::Effect,
        by: Some(p),
        etb: EtbInfo {
            controller: Some(p),
            // Enters with the plain face-down characteristics (CR 708.3); its kind is
            // recorded right after.
            face_down: Some(KeywordKind::Morph),
            ..Default::default()
        },
        source,
    })?;
    if g.obj(new).zone != Zone::Battlefield || !g.obj(new).face_down {
        return None;
    }
    g.objects[new.0 as usize].choices.text = Some(SmolStr::new(kind));
    g.dirty = true;
    g.recompute();
    Some(new)
}

/// `p` manifests (or cloaks) the top `n` cards of their library, one at a time
/// (CR 701.40e, 701.58e). Returns the permanents.
pub fn from_top(
    g: &mut Game,
    p: PlayerId,
    n: u32,
    kind: &str,
    source: Option<ObjectId>,
) -> Vec<ObjectId> {
    let mut out = Vec::new();
    for _ in 0..n {
        let Some(top) = g.library_top(p) else {
            break;
        };
        match put_face_down(g, top, p, kind, source) {
            Some(new) => out.push(new),
            // It couldn't be manifested: it's still on top (CR 701.40f).
            None => break,
        }
    }
    out
}

/// `p` manifests dread (CR 701.62a). Returns the manifested permanent.
pub fn manifest_dread(g: &mut Game, p: PlayerId, source: Option<ObjectId>) -> Option<ObjectId> {
    let cards = crate::library::top_cards(g, p, 2);
    let mut manifested = None;
    if let Some(first) = cards.first().copied() {
        let pick = if cards.len() == 1 {
            first
        } else {
            g.ask_objects(
                p,
                source,
                "Manifest dread: choose a card to manifest",
                cards.clone(),
                1,
                1,
            )
            .first()
            .copied()
            .unwrap_or(first)
        };
        manifested = put_face_down(g, pick, p, MANIFESTED, source);
        let rest: Vec<MoveEv> = cards
            .iter()
            .copied()
            // The cards looked at that weren't manifested.
            .filter(|c| g.is_live(*c) && g.obj(*c).zone == Zone::Library(p))
            .map(|c| MoveEv {
                obj: c,
                to: Zone::Graveyard(g.obj(c).owner),
                pos: LibraryPosition::Top,
                cause: MoveCause::Effect,
                by: Some(p),
                etb: EtbInfo::default(),
                source,
            })
            .collect();
        g.move_objects(rest);
    }
    emit(g, MANIFESTED_DREAD, p, manifested, 0);
    manifested
}

pub struct Manifest;

impl KeywordActionRules for Manifest {
    fn actions(&self) -> &'static [KeywordAction] {
        &[
            KeywordAction::Manifest,
            KeywordAction::Cloak,
            KeywordAction::ManifestDread,
        ]
    }

    /// "[Player] manifests / cloaks the top N cards of their library" (`what` is
    /// `Sel::None`), "manifest / cloak [cards]", and "manifest dread [N times]".
    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let kind = match a.action {
            KeywordAction::Cloak => CLOAKED,
            _ => MANIFESTED,
        };
        let n = number(g, a.n, ctx);
        let mut out = Vec::new();
        for p in g.eval_players(a.who, ctx) {
            match (a.action, a.what) {
                (KeywordAction::ManifestDread, _) => {
                    for _ in 0..n {
                        out.extend(manifest_dread(g, p, ctx.source));
                    }
                }
                (_, Sel::None) => out.extend(from_top(g, p, n, kind, ctx.source)),
                (_, what) => {
                    for c in g.resolve_objects(what, ctx) {
                        out.extend(put_face_down(g, c, p, kind, ctx.source));
                    }
                }
            }
        }
        ctx.prev_happened = !out.is_empty();
        let out: Vec<Entity> = out.into_iter().map(Entity::Object).collect();
        ctx.set_var(kvars::MANIFESTED, out.clone());
        ctx.set_var(vars::IT, out);
    }
}

inventory::submit! { KeywordActionRegistration(&Manifest) }
