//! Goblin Welder: "{T}: Choose target artifact a player controls and target artifact card in
//! that player's graveyard. If both targets are still legal as this ability resolves, that
//! player simultaneously sacrifices the artifact and returns the artifact card to the
//! battlefield." If either target is illegal, nothing happens (ruling).

use super::ManualAbility;
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::{Event, MoveCause};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::{CardType, Entity};

const WELD: &str = "card:Goblin Welder:swap the artifact and the artifact card";
const TEXT: &str = "{T}: Choose target artifact a player controls and target artifact card in that player's graveyard. If both targets are still legal as this ability resolves, that player simultaneously sacrifices the artifact and returns the artifact card to the battlefield.";

inventory::submit! { ManualAbility {
    card: "Goblin Welder",
    face: 0,
    text: TEXT,
    build: |_| {
        let artifact = TargetSpec::object(Filter::Type(CardType::Artifact), "target artifact a player controls");
        let mut card = TargetSpec::object(
            Filter::And(vec![
                Filter::Type(CardType::Artifact),
                Filter::Card,
                Filter::InZone(ZoneKind::Graveyard),
            ]),
            "target artifact card in that player's graveyard",
        );
        card.related_to = Some((0, TargetGroup::SameController));
        let body = Body::simple(vec![artifact, card], Effect::Custom(WELD.into()));
        vec![AbilityDef::new(
            AbilityKind::Activated(ActivatedAbility::new(Cost::tap(), body)),
            TEXT,
        )]
    },
    reason: "simultaneously sacrifices an artifact and returns an artifact card from the same player's graveyard: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != WELD {
            return false;
        }
        let one = |i: usize| ctx.targets.get(i).and_then(|v| v.first()).copied();
        let (Some(Entity::Object(art)), Some(Entity::Object(card))) = (one(0), one(1)) else {
            return true;
        };
        let p = g.obj(art).controller;
        if g.obj(card).zone != Zone::Graveyard(p) {
            return true;
        }
        let mut moves = vec![MoveEv {
            obj: card,
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: MoveCause::Return,
            by: Some(p),
            etb: EtbInfo {
                controller: Some(p),
                ..Default::default()
            },
            source: ctx.source,
        }];
        let sacrificed = !g.cant_be_sacrificed(art);
        if sacrificed {
            moves.insert(
                0,
                MoveEv {
                    obj: art,
                    to: Zone::Graveyard(g.obj(art).owner),
                    pos: LibraryPosition::Top,
                    cause: MoveCause::Sacrifice,
                    by: Some(p),
                    etb: EtbInfo::default(),
                    source: None,
                },
            );
        }
        g.move_objects(moves);
        if sacrificed {
            g.history.sacrificed.push((p, art));
            g.emit(Event::Sacrificed { obj: art, player: p });
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
