//! Casual variants (CR 900–905) and supplemental card types: Archenemy schemes,
//! Planechase planes, Vanguard, dungeons, and attractions. Hook points called by the
//! turn structure and SBAs.

use crate::events::Event;
use crate::game::Game;
use crate::object::{EventInfo, GameObject, StackKind, Zone};
use crate::types::*;
use smol_str::SmolStr;

/// `Event::Custom` name: a player set a scheme in motion (CR 701.32); the object is the
/// scheme.
pub const SET_IN_MOTION: &str = "set in motion";
/// `Event::Custom` name: a scheme was abandoned (CR 701.33); the object is the scheme.
pub const ABANDONED: &str = "abandon";
/// `Event::Custom` name: a player rolled to visit their Attractions (CR 701.52a); the
/// amount is the result.
pub const ROLLED_TO_VISIT: &str = "roll to visit";
/// `TriggerCond::Custom` name: "When you encounter [this phenomenon]" (CR 312.5).
pub const ENCOUNTER: &str = "encounter this phenomenon";
/// `TriggerCond::Custom` name: "When you set this scheme in motion" (CR 701.32, 904.9).
pub const SET_THIS_IN_MOTION: &str = "set this scheme in motion";
/// `TriggerCond::Custom` name: a visit ability (CR 702.159, 717.5): "whenever you roll to
/// visit your Attractions and the result matches one of the lit-up numbers".
pub const VISIT: &str = "visit";

/// CR 613.7i, 613.7j: at the beginning of the game, each face-up vanguard card and each
/// conspiracy card in the command zone receives a timestamp.
pub fn begin_game(g: &mut Game) {
    for id in g.command.clone() {
        let o = g.obj(id);
        let gets =
            (o.base.is(CardType::Vanguard) && !o.face_down) || o.base.is(CardType::Conspiracy);
        if gets {
            let ts = g.new_timestamp();
            g.objects[id.0 as usize].timestamp = ts;
            g.dirty = true;
        }
    }
}

/// Turns a face-down card in the command zone (a plane, phenomenon, scheme or
/// conspiracy card) face up. It receives a new timestamp at that time (CR 613.7h,
/// 613.7j). Returns true if it did.
pub fn turn_face_up_in_command(g: &mut Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    if o.zone != Zone::Command || !o.face_down {
        return false;
    }
    let ts = g.new_timestamp();
    let o = &mut g.objects[id.0 as usize];
    o.face_down = false;
    o.timestamp = ts;
    g.dirty = true;
    true
}

// --- Archenemy: schemes (CR 314, 701.32, 701.33, 904) -------------------------------

/// Whether an object is a scheme card (by its card, even while face down).
fn is_scheme_card(o: &GameObject) -> bool {
    let chars = o.card.as_ref().map(|c| &c.front().chars).unwrap_or(&o.base);
    chars.card_types.contains(CardType::Scheme)
}

/// Whether a scheme card is an ongoing scheme (CR 205.4g).
fn is_ongoing(o: &GameObject) -> bool {
    let chars = o.card.as_ref().map(|c| &c.front().chars).unwrap_or(&o.base);
    chars.supertypes.contains(Supertype::Ongoing)
}

/// A player's scheme deck (CR 904.3), top card first: the face-down scheme cards they own
/// in the command zone, in command-zone order.
pub fn scheme_deck(g: &Game, p: PlayerId) -> Vec<ObjectId> {
    g.command
        .iter()
        .copied()
        .filter(|id| {
            let o = g.obj(*id);
            o.face_down && o.owner == p && is_scheme_card(o)
        })
        .collect()
}

/// The face-up scheme cards in the command zone.
pub fn face_up_schemes(g: &Game) -> Vec<ObjectId> {
    g.command
        .iter()
        .copied()
        .filter(|id| {
            let o = g.obj(*id);
            !o.face_down && is_scheme_card(o)
        })
        .collect()
}

/// Sets the top scheme of `p`'s scheme deck in motion (CR 701.32b): it's moved off the
/// top of the deck and turned face up, and "when you set this scheme in motion" abilities
/// trigger. Only an archenemy may set a scheme in motion (CR 701.32a). Returns the scheme.
pub fn set_in_motion(g: &mut Game, p: PlayerId) -> Option<ObjectId> {
    if !crate::life_totals::is_archenemy(g, p) {
        return None;
    }
    let top = *scheme_deck(g, p).first()?;
    // Off the deck: to the end of the command-zone order, face up.
    g.command.retain(|x| *x != top);
    g.command.push(top);
    turn_face_up_in_command(g, top);
    g.recompute();
    g.log(|g| format!("{p} sets {} in motion", g.obj(top).chars.name));
    g.emit(Event::Custom {
        name: SmolStr::new(SET_IN_MOTION),
        player: Some(p),
        obj: Some(top),
        amount: 0,
    });
    Some(top)
}

/// Turns a face-up scheme face down and puts it on the bottom of its owner's scheme deck
/// (CR 701.33b, 704.6e).
fn scheme_to_bottom(g: &mut Game, id: ObjectId) {
    g.command.retain(|x| *x != id);
    g.command.push(id);
    g.objects[id.0 as usize].face_down = true;
    g.dirty = true;
}

/// Abandons a scheme (CR 701.33): only a face-up ongoing scheme may be abandoned, and only
/// in an Archenemy game. Returns true if it was.
pub fn abandon(g: &mut Game, id: ObjectId) -> bool {
    let id = g.current(id);
    let o = g.obj(id);
    if g.config.variant != crate::game::Variant::Archenemy
        || o.zone != Zone::Command
        || o.face_down
        || !is_scheme_card(o)
        || !is_ongoing(o)
    {
        return false;
    }
    let owner = o.owner;
    scheme_to_bottom(g, id);
    g.emit(Event::Custom {
        name: SmolStr::new(ABANDONED),
        player: Some(owner),
        obj: Some(id),
        amount: 0,
    });
    true
}

/// CR 505.3, 703.4e, 904.9: immediately after the archenemy's precombat main phase
/// begins, they set the top card of their scheme deck in motion (a turn-based action).
pub fn archenemy_main_phase(g: &mut Game, p: PlayerId) {
    set_in_motion(g, p);
    g.flush_events();
}

/// Whether a triggered ability of any scheme has triggered but not yet left the stack.
fn scheme_trigger_pending(g: &Game) -> bool {
    let is_scheme = |id: ObjectId| is_scheme_card(g.obj(id));
    g.pending_triggers.iter().any(|t| is_scheme(t.source))
        || g.stack.iter().any(|s| {
            matches!(
                g.obj(*s).stack.as_deref().map(|x| &x.kind),
                Some(StackKind::Triggered { source, .. }) if is_scheme(*source)
            )
        })
}

/// CR 704.6e, 314.6, 904.10: a face-up non-ongoing scheme returns face down to the bottom
/// of its owner's scheme deck once no scheme's triggered abilities are on the stack or
/// waiting to be put there.
fn scheme_sba(g: &mut Game) -> bool {
    if scheme_trigger_pending(g) {
        return false;
    }
    let done: Vec<ObjectId> = face_up_schemes(g)
        .into_iter()
        .filter(|id| !is_ongoing(g.obj(*id)))
        .collect();
    for id in &done {
        scheme_to_bottom(g, *id);
    }
    !done.is_empty()
}

// --- Planechase: phenomena (CR 312, 704.6f) ------------------------------------------

/// CR 704.6f, 312.7: if a phenomenon card is face up in the command zone and isn't the
/// source of a triggered ability that has triggered but not yet left the stack, the
/// planar controller planeswalks.
fn phenomenon_sba(g: &mut Game) -> bool {
    let Some(pc) = crate::planechase::planar_controller(g) else {
        return false;
    };
    let waiting = crate::planechase::face_up_planar_cards(g)
        .into_iter()
        .any(|id| g.obj(id).chars.is(CardType::Phenomenon) && !g.is_source_of_stack_trigger(id));
    if waiting {
        crate::planechase::planeswalk(g, pc);
    }
    waiting
}

// --- Attractions (CR 701.52, 702.159, 717) -------------------------------------------

/// Rolls to visit `p`'s Attractions (CR 701.52a): roll a six-sided die; each Attraction
/// they control with that number lit up has been visited, and its visit ability triggers.
pub fn roll_to_visit(g: &mut Game, p: PlayerId) -> i64 {
    let dice = crate::dice::roll_dice(g, p, 1, 6, crate::dice::DiceIgnore::None, 0, None);
    let result = dice.first().map_or(0, |d| d.result());
    g.emit(Event::Custom {
        name: SmolStr::new(ROLLED_TO_VISIT),
        player: Some(p),
        obj: None,
        amount: result as i32,
    });
    result
}

/// Whether an Attraction has the number `n` lit up (CR 717.1).
pub fn lit_up(o: &GameObject, n: i64) -> bool {
    o.card
        .as_ref()
        .is_some_and(|c| c.attraction_lights.iter().any(|l| *l as i64 == n))
}

/// CR 505.5, 703.4g, 717.4: immediately after lore counters are placed, if the active
/// player controls any Attractions, they roll to visit their Attractions.
pub fn roll_to_visit_attractions(g: &mut Game, p: PlayerId) {
    g.recompute();
    let any = g
        .permanents()
        .any(|o| o.controller == p && o.chars.has_subtype("Attraction"));
    if any {
        roll_to_visit(g, p);
        g.flush_events();
    }
}

// --- Triggers ------------------------------------------------------------------------

/// Trigger conditions of nontraditional cards: encountering a phenomenon, setting a
/// scheme in motion, and visiting an Attraction.
pub fn custom_trigger(
    g: &Game,
    name: &str,
    src: ObjectId,
    ctl: PlayerId,
    ev: &Event,
) -> Option<Vec<EventInfo>> {
    if !matches!(name, ENCOUNTER | SET_THIS_IN_MOTION | VISIT) {
        return None;
    }
    let Event::Custom {
        name: ev_name,
        player,
        obj,
        amount,
    } = ev
    else {
        return Some(vec![]);
    };
    let hit = match name {
        // CR 312.5: "when you move this card off a planar deck and turn it face up".
        ENCOUNTER => ev_name.as_str() == crate::planechase::PLANESWALKED && *obj == Some(src),
        // CR 701.32b: the scheme is considered set in motion.
        SET_THIS_IN_MOTION => ev_name.as_str() == SET_IN_MOTION && *obj == Some(src),
        // CR 702.159a, 717.5: you rolled to visit and the result is lit up.
        _ => {
            ev_name.as_str() == ROLLED_TO_VISIT
                && *player == Some(ctl)
                && lit_up(g.obj(src), *amount as i64)
        }
    };
    Some(if hit {
        vec![EventInfo {
            object: Some(src),
            player: *player,
            amount: *amount,
            ..Default::default()
        }]
    } else {
        vec![]
    })
}

/// CR 704.5t: dungeon completion. Returns true if an action was performed.
pub fn dungeon_sba(g: &mut Game) -> bool {
    crate::dungeons::completion_sba(g)
}

/// CR 704.6e/f: archenemy scheme and planechase phenomenon SBAs.
pub fn variant_sbas(g: &mut Game) -> bool {
    let mut performed = false;
    if g.config.variant == crate::game::Variant::Archenemy {
        performed |= scheme_sba(g);
    }
    performed |= phenomenon_sba(g);
    performed
}

/// Nontraditional Magic cards (CR 108.2a): planes, phenomena, vanguards, schemes,
/// dungeons, and Attractions. They're never part of a player's deck.
pub fn is_nontraditional(card: &crate::card::CardDef) -> bool {
    card.faces.iter().any(|f| {
        let c = &f.chars;
        [
            CardType::Plane,
            CardType::Phenomenon,
            CardType::Vanguard,
            CardType::Scheme,
            CardType::Dungeon,
        ]
        .iter()
        .any(|t| c.card_types.contains(*t))
            || c.has_subtype("Attraction")
    })
}

/// Moves of cards that stay where they are: plane, phenomenon, vanguard, scheme, and
/// conspiracy cards remain in the command zone if they would leave it (CR 311.2, 312.2,
/// 313.2, 314.2, 315.3); a dungeon card leaves it only as it leaves the game (CR 309.2c);
/// and a conspiracy card that isn't in the game can't be brought into it (CR 315.3).
pub fn stays_in_command_zone(g: &Game, mv: &crate::replacement::MoveEv) -> bool {
    let o = g.obj(mv.obj);
    let Some(card) = o
        .card
        .as_ref()
        .filter(|_| o.kind == crate::object::ObjKind::Card)
    else {
        return false;
    };
    let types = card.front().chars.card_types;
    match o.zone {
        Zone::Command if mv.to != Zone::Command => {
            [
                CardType::Plane,
                CardType::Phenomenon,
                CardType::Vanguard,
                CardType::Scheme,
                CardType::Conspiracy,
            ]
            .iter()
            .any(|t| types.contains(*t))
                || (types.contains(CardType::Dungeon) && !matches!(mv.to, Zone::Outside(_)))
        }
        Zone::Outside(_) => {
            types.contains(CardType::Conspiracy) && !matches!(mv.to, Zone::Outside(_))
        }
        _ => false,
    }
}

/// Where a nontraditional card listed with a player's deck starts the game (CR 108.2a,
/// 108.5): never in the library. Vanguards start face up in the command zone (CR 902.3);
/// planes, phenomena, schemes and Attractions start face down there as supplementary
/// decks (CR 901.4, 904.4, 717.2); dungeons begin outside the game (CR 309.2).
/// Returns (zone, face down).
pub fn nontraditional_start(
    card: &crate::card::CardDef,
    owner: PlayerId,
) -> (crate::object::Zone, bool) {
    use crate::object::Zone;
    let c = &card.front().chars;
    if c.card_types.contains(CardType::Dungeon) {
        (Zone::Outside(owner), false)
    } else if c.card_types.contains(CardType::Vanguard) {
        (Zone::Command, false)
    } else {
        (Zone::Command, true)
    }
}
