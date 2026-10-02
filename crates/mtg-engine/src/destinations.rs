//! Where an effect puts an object ([`Destination`]): the zone and library position, and
//! how a permanent enters — tapped, under whose control (CR 110.2), attacking (CR 508.4),
//! transformed, attached to something (CR 303.4f–i), with counters (CR 122.6) or with a
//! characteristic the effect gives it (CR 611.2e); a card exiled face down (CR 406.3).
//!
//! The same description is used by an effect that moves objects
//! ([`Game::move_to_destination`]) and by a replacement effect that has an object go
//! somewhere else "instead" (CR 614.1a, 614.6): the modified event moves the object to
//! the whole destination, not only to its zone ("exile it with three time counters on
//! it instead", Delay; "put that card onto the battlefield under your control instead",
//! Desertion; "on your choice of the top or bottom of its owner's library instead",
//! Hinder).
//!
//! Counters an object is given as it moves to a zone other than the battlefield ("exile it
//! with three time counters on it") are put on it as part of that move (see
//! `Game::perform_move`).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;

/// A [`Destination`] evaluated for one effect: everything that doesn't depend on the
/// object moved.
pub(crate) struct PreparedDestination {
    zone: ZoneKind,
    pos: LibraryPosition,
    tapped: bool,
    /// "Under [player]'s control".
    controller: Option<PlayerId>,
    /// "Under its owner's control" / "under their owners' control".
    owners_control: bool,
    /// The player putting it onto the battlefield, who controls it by default (CR 110.2a).
    putter: PlayerId,
    counters: Vec<(CounterKind, u32)>,
    face_down: bool,
    transformed: bool,
    attacking: Option<Entity>,
    with_mods: Option<(Option<ObjectId>, PlayerId, Vec<Modification>)>,
    /// `Some(None)`: attached to something undefined (CR 303.4i).
    attach_to: Option<Option<Entity>>,
}

/// How a library position is offered.
fn position_label(pos: LibraryPosition) -> String {
    match pos {
        LibraryPosition::Top => "Top of library".into(),
        LibraryPosition::Bottom | LibraryPosition::BottomRandom => "Bottom of library".into(),
        LibraryPosition::FromTop(n) => format!("{} from the top of library", n + 1),
        LibraryPosition::Shuffled => "Shuffled into library".into(),
    }
}

impl Game {
    /// Evaluates `to` for an effect whose context is `ctx` (the controller of the effect
    /// makes its choices, such as "your choice of the top or bottom").
    pub(crate) fn prepare_destination(
        &mut self,
        to: &Destination,
        ctx: &mut Ctx,
    ) -> PreparedDestination {
        let controller = to
            .controller
            .as_ref()
            .and_then(|r| self.eval_player(r, ctx));
        let owners_control = matches!(to.controller, Some(PlayerRef::OwnerOf(_)));
        let mut counters: Vec<(CounterKind, u32)> = Vec::new();
        for (k, v) in &to.with_counters {
            counters.push((k.clone(), self.eval_value(v, ctx).max(0) as u32));
        }
        let battlefield = to.zone == ZoneKind::Battlefield;
        let attacking = if to.attacking {
            self.attack_target_for_new_attacker(ctx)
        } else {
            None
        };
        let with_mods = (battlefield && !to.with_mods.is_empty()).then(|| {
            (
                ctx.source,
                ctx.controller,
                self.fix_mods(&to.with_mods, ctx),
            )
        });
        let attach_to = match (&to.attached_to, battlefield) {
            (Some(sel), true) => Some(self.resolve_sel(sel, ctx).first().copied()),
            _ => None,
        };
        let pos = if to.position_choice.len() > 1 && to.zone == ZoneKind::Library {
            let labels = to
                .position_choice
                .iter()
                .map(|p| position_label(*p))
                .collect();
            let i = self.ask_option(ctx.controller, ctx.source, "Choose where to put it", labels);
            to.position_choice
                .get(i)
                .copied()
                .unwrap_or(to.position_choice[0])
        } else {
            to.position_choice.first().copied().unwrap_or(to.position)
        };
        PreparedDestination {
            zone: to.zone,
            pos,
            tapped: to.tapped,
            controller,
            owners_control,
            putter: ctx.controller,
            counters,
            face_down: to.face_down,
            transformed: to.transformed,
            attacking,
            with_mods,
            attach_to,
        }
    }
}

impl PreparedDestination {
    /// How an object owned by `owner` moves there.
    pub(crate) fn etb(&self, owner: PlayerId) -> EtbInfo {
        let mut etb = EtbInfo::default();
        self.apply_etb(&mut etb, owner);
        etb
    }

    fn apply_etb(&self, etb: &mut EtbInfo, owner: PlayerId) {
        let battlefield = self.zone == ZoneKind::Battlefield;
        if battlefield {
            etb.tapped |= self.tapped;
            etb.controller = Some(if self.owners_control {
                owner
            } else {
                self.controller.unwrap_or(self.putter)
            });
            if self.with_mods.is_some() {
                etb.with_mods = self.with_mods.clone();
            }
            if let Some(a) = self.attach_to {
                etb.attach_to = a;
                etb.attach_specified = true;
            }
        }
        etb.transformed |= self.transformed;
        if self.attacking.is_some() {
            etb.attacking = self.attacking;
        }
        etb.counters.extend(self.counters.iter().cloned());
        if self.face_down {
            etb.face_down = Some(KeywordKind::Morph);
        }
    }

    pub(crate) fn zone(&self, owner: PlayerId) -> Zone {
        Zone::of_kind(self.zone, owner)
    }

    pub(crate) fn position(&self) -> LibraryPosition {
        self.pos
    }

    /// Has a proposed zone change go here instead (CR 614.6): the modified event moves the
    /// object to this zone and position, entering the way the destination says.
    pub(crate) fn redirect(&self, m: &mut MoveEv, owner: PlayerId) {
        m.to = self.zone(owner);
        m.pos = self.pos;
        self.apply_etb(&mut m.etb, owner);
    }
}
