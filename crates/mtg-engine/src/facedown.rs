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
    // Only face-down spells and permanents are 2/2 creatures. A face-down card elsewhere
    // (exiled face down, a hidden agenda conspiracy or a card of a planar or scheme deck
    // in the command zone) has no characteristics (CR 406.3a, 315.5b).
    if !matches!(o.zone, Zone::Battlefield | Zone::Stack) {
        return Characteristics {
            rules_text: std::sync::Arc::from(""),
            ..Default::default()
        };
    }
    // A face-down spell has the characteristics the ability it was cast with lists.
    let cast_as = match o.stack.as_deref().map(|s| &s.cast.method) {
        Some(CastMethod::FaceDown(k)) => Some(k.name()),
        _ => None,
    };
    let kind = cast_as
        .or(o.choices.text.as_deref())
        .unwrap_or("")
        .to_string();
    kind_characteristics(&kind)
}

/// The characteristics of a spell cast face down with `kind` (CR 708.2, 708.4).
pub fn face_down_spell_characteristics(kind: KeywordKind) -> Characteristics {
    kind_characteristics(kind.name())
}

fn kind_characteristics(kind: &str) -> Characteristics {
    let mut c = Characteristics {
        name: SmolStr::default(),
        power: Some(2),
        toughness: Some(2),
        card_types: CardTypeSet::single(CardType::Creature),
        rules_text: std::sync::Arc::from(""),
        ..Default::default()
    };
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

/// The kinds of face-down status recorded on a face-down object (in `choices.text`).
const FACE_DOWN_KINDS: [&str; 5] = ["Morph", "Megamorph", "Disguise", "Cloak", "Manifest"];

/// Turns a face-up permanent face down (e.g. "turn target creature face down"). It
/// becomes a 2/2 face-down creature with no text, no name, no subtypes, and no mana cost
/// (CR 708.2a), and gets a new timestamp (CR 613.7f). A face-down permanent can't be
/// turned face down: nothing happens (CR 708.2b). Returns true if it did.
pub fn turn_face_down(g: &mut Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    if o.face_down || o.zone != Zone::Battlefield || !g.is_live(id) {
        return false;
    }
    // CR 730.2j: a face-up merged permanent with a double-faced component can't be turned
    // face down.
    if crate::merge::cant_turn_face_down(g, id) {
        return false;
    }
    let ts = g.new_timestamp();
    let ob = &mut g.objects[id.0 as usize];
    ob.face_down = true;
    ob.timestamp = ts;
    // No characteristics are listed by the effect: none of those of an earlier face-down
    // status (such as disguise's ward {2}) apply.
    if ob
        .choices
        .text
        .as_deref()
        .is_some_and(|t| FACE_DOWN_KINDS.contains(&t))
    {
        ob.choices.text = None;
    }
    g.dirty = true;
    // CR 730.2f: each face-up component of a merged permanent is turned face down.
    crate::merge::turned_face(g, id, true);
    g.emit(Event::TurnedFaceDown { obj: id });
    true
}

/// CR 708.5: a player may look at face-down spells and permanents they control (even
/// phased-out ones), but not at face-down objects in other zones or controlled by
/// another player. Face-up objects in public zones can be seen by everyone.
pub fn can_look_at(g: &Game, p: PlayerId, id: ObjectId) -> bool {
    let o = g.obj(id);
    if !o.face_down {
        // CR 400.2, 402.3: public zones and your own hand; in a library, only a revealed
        // top card or one you may look at (CR 401.2, 401.5).
        return o.zone.is_public()
            || o.zone == Zone::Hand(p)
            || crate::zones::can_see_in_library(g, p, id);
    }
    // CR 406.3: face-down cards in exile, once a player is allowed to look at them.
    if o.zone == Zone::Exile {
        return crate::zones::may_look(g, p, id);
    }
    // CR 315.7: a player may look at a face-down conspiracy card they control.
    if o.zone == Zone::Command
        && o.card
            .as_ref()
            .is_some_and(|c| c.front().chars.is(CardType::Conspiracy))
    {
        return o.controller == p;
    }
    matches!(o.zone, Zone::Stack | Zone::Battlefield) && o.controller == p
}

/// `Event::Custom` name of a face-down object being revealed to all players (CR 708.9).
pub const REVEALED: &str = "face-down object revealed";

/// Reveals a face-down object (and the face-down components of a merged permanent) to all
/// players (CR 708.9).
pub fn reveal(g: &mut Game, id: ObjectId) {
    let mut objs = vec![id];
    objs.extend(g.obj(id).merged_with.iter().copied());
    for o in objs {
        if !g.obj(o).face_down {
            continue;
        }
        let owner = g.obj(o).owner;
        let name = g
            .obj(o)
            .card
            .as_ref()
            .map_or_else(|| "a token".to_string(), |c| c.name.to_string());
        g.log(|_| format!("{owner} reveals face-down {name}"));
        g.emit(Event::Custom {
            name: REVEALED.into(),
            player: Some(owner),
            obj: Some(o),
            amount: 0,
        });
    }
}

/// CR 708.9: a face-down permanent leaving the battlefield, or a face-down spell leaving
/// the stack for a zone other than the battlefield, is revealed as it moves.
pub fn moving(g: &mut Game, id: ObjectId, to: Zone) {
    let o = g.obj(id);
    let from = o.zone;
    let merged_face_down = o.merged_with.iter().any(|c| g.obj(*c).face_down);
    if !(o.face_down || merged_face_down) || to == from {
        return;
    }
    let reveals = match from {
        Zone::Battlefield => true,
        Zone::Stack => to != Zone::Battlefield,
        _ => false,
    };
    if reveals {
        reveal(g, id);
    }
}

/// `Effect::Custom`: "reveal [the face-down permanent in `vars::IT`]".
pub const REVEAL_IT: &str = "facedown:reveal it";
/// `Condition::Custom`: "if it's a creature card" about a revealed face-down permanent.
pub const REVEALED_CREATURE_CARD: &str = "facedown:revealed is a creature card";

/// CR 708.12: an effect that needs information about a revealed face-down permanent uses
/// the characteristics of the object itself, ignoring any continuous effects applying to
/// it: those of the card (its face that would be up), or a token's own.
pub fn revealed_characteristics(g: &Game, id: ObjectId) -> Characteristics {
    let o = g.obj(id);
    match &o.card {
        Some(card) => card.characteristics(FaceState::Front),
        None => o.base.clone(),
    }
}

fn it(ctx: &crate::eval::Ctx) -> Vec<ObjectId> {
    ctx.vars
        .get(&vars::IT)
        .map(|v| v.iter().filter_map(|e| e.object()).collect())
        .unwrap_or_default()
}

/// `Effect::Custom` effects of this module. Returns true if handled.
pub fn custom_effect(g: &mut Game, name: &str, ctx: &mut crate::eval::Ctx) -> bool {
    if name != REVEAL_IT {
        return false;
    }
    for o in it(ctx) {
        reveal(g, o);
    }
    true
}

/// `Condition::Custom` conditions of this module.
pub fn custom_condition(g: &Game, name: &str, ctx: &crate::eval::Ctx) -> Option<bool> {
    if name != REVEALED_CREATURE_CARD {
        return None;
    }
    let objs = it(ctx);
    Some(
        !objs.is_empty()
            && objs.iter().all(|o| {
                g.obj(*o).card.is_some() && revealed_characteristics(g, *o).is(CardType::Creature)
            }),
    )
}

/// CR 708.9: when a player leaves the game (`Some(p)`), their face-down permanents and
/// spells are revealed; at the end of the game (`None`), everyone's are. So are face-down
/// conspiracy cards (hidden agenda, CR 702.106e).
pub fn reveal_all(g: &mut Game, owner: Option<PlayerId>) {
    let conspiracies = g
        .command
        .iter()
        .filter(|id| g.obj(**id).face_down && g.obj(**id).base.is(CardType::Conspiracy));
    let ids: Vec<ObjectId> = g
        .battlefield
        .iter()
        .chain(g.stack.iter())
        .chain(conspiracies)
        .copied()
        .filter(|id| owner.is_none_or(|p| g.obj(*id).owner == p))
        .collect();
    for id in ids {
        reveal(g, id);
    }
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
    // CR 701.40g, 701.58g: one represented by an instant or sorcery card is revealed and
    // stays face down; "turned face up" abilities don't trigger.
    let front = revealed_characteristics(g, id);
    // CR 730.2g: so is a merged permanent that contains an instant or sorcery card.
    if front.is(CardType::Instant)
        || front.is(CardType::Sorcery)
        || crate::merge::cant_turn_face_up(g, id)
    {
        reveal(g, id);
        return false;
    }
    let ts = g.new_timestamp();
    let ob = &mut g.objects[id.0 as usize];
    ob.face_down = false;
    ob.timestamp = ts; // CR 613.7f
    g.dirty = true;
    // CR 730.2f: each face-down component of a merged permanent is turned face up.
    crate::merge::turned_face(g, id, false);
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
