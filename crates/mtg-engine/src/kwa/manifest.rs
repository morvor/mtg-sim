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
/// `Event::Custom` name reported, before [`MANIFESTED_DREAD`], for each card a manifest
/// dread put into a graveyard (the card there).
pub const DREAD_TO_GRAVEYARD: &str = "manifest dread: put into a graveyard";
/// `Filter::Custom`: a card the manifest dread the ability triggered on put into a
/// graveyard (still there: CR 400.7).
pub const PUT_IN_GRAVEYARD_BY_THAT_DREAD: &str = "put into a graveyard by that manifest dread";

/// The cards the manifest dread of the triggering event (`ctx.event`) put into a
/// graveyard: the [`DREAD_TO_GRAVEYARD`] events just before it.
pub fn dread_graveyard_cards(g: &Game, ctx: &crate::eval::Ctx) -> Vec<ObjectId> {
    let Some(ev) = ctx.event.as_ref() else {
        return vec![];
    };
    let events: Vec<&crate::events::Event> = g.turn_events.iter().chain(g.events.iter()).collect();
    let mut out = Vec::new();
    for (i, e) in events.iter().enumerate() {
        let crate::events::Event::Custom {
            name, player, obj, ..
        } = e
        else {
            continue;
        };
        if name.as_str() != MANIFESTED_DREAD || *player != ev.player || *obj != ev.object {
            continue;
        }
        out.clear();
        for prev in events[..i].iter().rev() {
            match prev {
                crate::events::Event::Custom {
                    name, obj: Some(c), ..
                } if name.as_str() == DREAD_TO_GRAVEYARD => out.push(*c),
                _ => break,
            }
        }
    }
    out
}

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
    put_face_down_entering(g, card, p, kind, source, false)
}

/// [`put_face_down`], entering tapped if `tapped`.
pub fn put_face_down_entering(
    g: &mut Game,
    card: ObjectId,
    p: PlayerId,
    kind: &str,
    source: Option<ObjectId>,
    tapped: bool,
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
            tapped,
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
        // Each card is manifested by its own event: "whenever one or more creatures
        // enter" triggers once per card (CR 701.40e, 603.2c).
        g.end_event_batch();
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
        let moved = g.move_objects(rest);
        // The cards put into a graveyard this way ("a card you put into your graveyard
        // this way", Paranormal Analyst), reported just before the manifest dread event.
        for c in moved.into_iter().flatten() {
            if matches!(g.obj(c).zone, Zone::Graveyard(_)) {
                emit(g, DREAD_TO_GRAVEYARD, p, Some(c), 0);
            }
        }
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
        let tapped = a.spec.is_some_and(|s| s.tapped);
        let mut out = Vec::new();
        for p in g.eval_players(a.who, ctx) {
            match (a.action, a.what) {
                (KeywordAction::ManifestDread, _) => {
                    for _ in 0..n {
                        // "Manifest dread X times": one event each (CR 701.40e, 603.2c).
                        g.end_event_batch();
                        out.extend(manifest_dread(g, p, ctx.source));
                    }
                }
                (_, Sel::None) => out.extend(from_top(g, p, n, kind, ctx.source)),
                (_, what) => {
                    let mut cards = g.resolve_objects(what, ctx);
                    // A face-down pile of exiled cards no one may look at ("exile ... in a
                    // face-down pile, shuffle that pile, then manifest those cards"): which
                    // is manifested when isn't known to anyone (CR 406.3).
                    if cards.len() > 1
                        && cards.iter().all(|c| {
                            let o = g.obj(*c);
                            o.zone == Zone::Exile
                                && o.face_down
                                && !crate::zones::may_look(g, p, *c)
                        })
                    {
                        use rand::seq::SliceRandom;
                        cards.shuffle(&mut g.rng);
                    }
                    for c in cards {
                        // One at a time (CR 701.40e, 701.58e).
                        g.end_event_batch();
                        out.extend(put_face_down_entering(g, c, p, kind, ctx.source, tapped));
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
