//! CR 702.55 Haunt. "Haunt" on a permanent means "When this permanent is put into a
//! graveyard from the battlefield, exile it haunting target creature." On an instant or
//! sorcery spell it means "When this spell is put into a graveyard during its
//! resolution, exile it haunting target creature." (CR 702.55a).
//!
//! The card exiled this way "haunts" the object targeted by that ability, whether or not
//! it's still a creature (CR 702.55b): the relation is kept on the card in exile, as the
//! object it haunts (linked under [`HAUNT_LINK`]). Triggered abilities of the card that
//! refer to the creature it haunts ("When the creature ~ haunts dies, ...") function in
//! exile (CR 702.55c); they use [`HAUNTED`] as their filter. The relation ends when either
//! object changes zones (CR 400.7).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::{Event, MoveCause};
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;
use smol_str::SmolStr;

/// The link (in `GameObject::linked`) of an exiled card with haunt to the object it
/// haunts.
pub const HAUNT_LINK: u16 = 0x7ffe;

/// `Filter::Custom`: the object the ability's source (a card with haunt in exile) haunts
/// ("the creature it haunts").
pub const HAUNTED: &str = "haunt:the creature it haunts";

/// `TriggerCond::Custom`: this spell card was put into a graveyard as it resolved.
const SPELL_RESOLVED: &str = "haunt:this spell was put into a graveyard during its resolution";

/// `Effect::Custom`: "exile it haunting target creature".
const EXILE_HAUNTING: &str = "haunt:exile it haunting target creature";

pub struct Haunt;

fn haunt_body() -> Body {
    Body::simple(
        vec![TargetSpec::object(Filter::creature(), "target creature")],
        Effect::Custom(SmolStr::new(EXILE_HAUNTING)),
    )
}

impl KeywordRules for Haunt {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Haunt]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let text = KeywordKind::Haunt.name();
        // A permanent put into a graveyard from the battlefield (a leaves-the-battlefield
        // ability: it looks back in time, CR 603.10a).
        let permanent = TriggeredAbility::new(TriggerCond::Dies(Filter::Source), haunt_body());
        // An instant or sorcery card put into a graveyard as it resolves: the card in the
        // graveyard triggers.
        let mut spell =
            TriggeredAbility::new(TriggerCond::Custom(SPELL_RESOLVED.into()), haunt_body());
        spell.zone = FunctionZone::Graveyard;
        Some(vec![
            AbilityDef::new(AbilityKind::Triggered(permanent), text),
            AbilityDef::new(AbilityKind::Triggered(spell), text),
        ])
    }

    fn custom_trigger(
        &self,
        g: &Game,
        name: &str,
        src: ObjectId,
        _ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        if name != SPELL_RESOLVED {
            return None;
        }
        Some(match ev {
            Event::ZoneChange {
                old,
                new,
                from: Zone::Stack,
                to: Zone::Graveyard(_),
                cause: MoveCause::Resolve,
                ..
            } if *new == src && g.obj(*old).kind == ObjKind::Card => vec![EventInfo {
                object: Some(*new),
                lki: Some(*old),
                player: Some(g.obj(*new).owner),
                ..Default::default()
            }],
            _ => vec![],
        })
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != EXILE_HAUNTING {
            return false;
        }
        exile_haunting(g, ctx);
        true
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        (name == HAUNTED).then(|| {
            ctx.source.is_some_and(|s| {
                g.obj(s)
                    .linked
                    .get(&HAUNT_LINK)
                    .is_some_and(|v| v.contains(&id))
            })
        })
    }
}

/// "Exile it haunting target creature": the card (still in the graveyard it was put
/// into) is exiled and haunts the target.
fn exile_haunting(g: &mut Game, ctx: &mut Ctx) {
    let Some(card) = ctx.event.as_ref().and_then(|e| e.object) else {
        return;
    };
    if !g.is_live(card) || !matches!(g.obj(card).zone, Zone::Graveyard(_)) {
        return;
    }
    let Some(target) = ctx
        .targets
        .first()
        .and_then(|v| v.first())
        .and_then(|e| e.object())
    else {
        return;
    };
    let new = g.move_object_ev(MoveEv {
        obj: card,
        to: Zone::Exile,
        pos: LibraryPosition::Top,
        cause: MoveCause::Exile,
        by: Some(ctx.controller),
        etb: EtbInfo::default(),
        source: None,
    });
    if let Some(new) = new.filter(|n| g.obj(*n).zone == Zone::Exile) {
        g.objects[new.0 as usize]
            .linked
            .insert(HAUNT_LINK, vec![target]);
        g.log(|g| format!("{} haunts {}", g.describe(new), g.describe(target)));
    }
}

inventory::submit! { KeywordRegistration(&Haunt) }
