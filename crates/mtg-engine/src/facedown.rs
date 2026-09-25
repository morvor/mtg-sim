//! Face-down spells and permanents (CR 708) and turning them face up.

use crate::ability::*;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::mana::ManaCost;
use crate::object::*;
use crate::types::*;
use smol_str::SmolStr;

/// Characteristics of a face-down object (CR 708.2a): a 2/2 creature with no text, no
/// name, no subtypes, and no mana cost. Disguise and cloak also give it ward {2}
/// (CR 702.168a, 701.58a).
pub fn face_down_characteristics(g: &Game, id: ObjectId) -> Characteristics {
    let o = g.obj(id);
    let mut c = Characteristics {
        name: SmolStr::default(),
        power: Some(2),
        toughness: Some(2),
        card_types: CardTypeSet::single(CardType::Creature),
        rules_text: std::sync::Arc::from(""),
        ..Default::default()
    };
    let kind = o.choices.text.as_deref().unwrap_or("");
    if kind == KeywordKind::Disguise.name() || kind == "Cloak" {
        c.abilities.push(AbilityDef::new(
            AbilityKind::Keyword(Keyword::with_cost(
                KeywordKind::Ward,
                Cost::mana(ManaCost::parse("{2}").unwrap()),
            )),
            "Ward {2}",
        ));
    }
    c
}

/// Turns a face-up permanent face down (e.g. "turn target creature face down"). It gets
/// a new timestamp (CR 613.7f). Returns true if it did.
pub fn turn_face_down(g: &mut Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    if o.face_down || o.zone != Zone::Battlefield || !g.is_live(id) {
        return false;
    }
    let ts = g.new_timestamp();
    let ob = &mut g.objects[id.0 as usize];
    ob.face_down = true;
    ob.timestamp = ts;
    g.dirty = true;
    true
}

/// Turns a face-down permanent face up (CR 708.8). Returns true if it did.
pub fn turn_face_up(g: &mut Game, id: ObjectId, _special_action: bool) -> bool {
    let o = g.obj(id);
    if !o.face_down || o.zone != Zone::Battlefield {
        return false;
    }
    // CR 708.8: a face-down permanent that's not a card (e.g. a manifested token) can't be
    // turned face up unless it represents a card.
    if o.card.is_none() {
        return false;
    }
    let ts = g.new_timestamp();
    let ob = &mut g.objects[id.0 as usize];
    ob.face_down = false;
    ob.timestamp = ts; // CR 613.7f
    g.dirty = true;
    g.recompute();
    // CR 614.1e: "As [this] is turned face up, ..." replacement effects apply as it turns
    // face up, before anything sees it face up.
    let effects: Vec<(crate::eval::Ctx, Effect)> = g
        .obj(id)
        .chars
        .abilities
        .iter()
        .filter_map(|a| match &a.kind {
            AbilityKind::Static(s) => match &s.effect {
                StaticEffect::Replacement(ReplacementDef {
                    event: ReplacementEvent::TurnedFaceUp,
                    action: ReplacementAction::AsEnters(e),
                    ..
                }) => {
                    let mut ctx = crate::eval::Ctx::for_object(g, id);
                    ctx.link = a.link;
                    Some((ctx, (**e).clone()))
                }
                _ => None,
            },
            _ => None,
        })
        .collect();
    for (mut ctx, e) in effects {
        let before = g.effects.len();
        g.exec(&e, &mut ctx);
        crate::layers::as_enters_copiable(g, id, before);
    }
    g.emit(Event::TurnedFaceUp { obj: id });
    true
}
