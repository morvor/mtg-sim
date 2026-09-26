//! Planechase (CR 901): planar decks, the planar controller, planeswalking (CR 701.31),
//! and the planar die with its Planeswalker and chaos symbols (CR 107.11, 107.12).
//!
//! Plane and phenomenon cards stay in the command zone all game (CR 311.2, 312.2, 901.4).
//! A player's planar deck is the face-down plane and phenomenon cards they own in the
//! command zone, in command-zone order (the first is the top); with the single planar
//! deck option (CR 901.15) all of them form one communal deck.
//!
//! Planechase is played on its own (`Variant::Planechase`, CR 901.2) or along with
//! Two-Headed Giant (CR 901.12) or Grand Melee (CR 901.14) (`GameConfig::planechase`).
//! In Grand Melee each turn marker has its own planar controller (CR 901.14a); each
//! face-up planar card belongs to the planar controller of one marker, recorded in
//! [`PlanarState::slots`].

use crate::ability::*;
use crate::casting::Illegal;
use crate::decision::{Action, SpecialAction};
use crate::events::Event;
use crate::game::{Game, PendingTrigger, Variant};
use crate::object::{EventInfo, GameObject, StackKind, Zone};
use crate::types::*;
use smol_str::SmolStr;
use std::collections::{BTreeMap, BTreeSet};

/// `Event::Custom` name: chaos ensues (CR 311.7). "Whenever chaos ensues" triggers.
pub const CHAOS_ENSUES: &str = "chaos ensues";
/// `Event::Custom` name: a player rolled the planar die (CR 901.9).
pub const ROLLED_PLANAR_DIE: &str = "roll planar die";
/// `Event::Custom` name: a player planeswalked; the object is the plane planeswalked to
/// (CR 901.11).
pub const PLANESWALKED: &str = "planeswalk";
/// `SpecialAction::Other` name of rolling the planar die (CR 901.9, 116.2i).
pub const PLANAR_DIE_ACTION: &str = "roll the planar die";
/// `Effect::Custom` name: the planeswalking ability's effect (CR 901.8): its controller
/// planeswalks.
pub const PLANESWALK_EFFECT: &str = "planeswalk";
/// `Effect::Custom` name: the controller rolls the planar die because of an effect (not
/// the special action, CR 116.2i).
pub const ROLL_PLANAR_DIE_EFFECT: &str = "roll the planar die (effect)";
/// `Effect::Custom` name: "chaos ensues" as a resolving spell or ability says so
/// (CR 311.7): chaos abilities trigger.
pub const CHAOS_ENSUES_EFFECT: &str = "chaos ensues (effect)";
/// `TriggerCond::Custom` name of the inherent planeswalking ability (CR 901.8).
pub const PLANESWALKING_ABILITY: &str = "planeswalking ability";
/// `Restriction::Custom` name: "each blank roll of the planar die is a {CHAOS} roll"
/// (Chaotic Aether).
pub const BLANK_ROLLS_ARE_CHAOS: &str = "blank planar die rolls are chaos rolls";
/// `Restriction::Custom` name: "if a player would planeswalk as a result of rolling the
/// planar die, chaos ensues instead" (Fixed Point in Time).
pub const PLANESWALK_ROLLS_ARE_CHAOS: &str = "planeswalking from the planar die is chaos";
/// `Effect::Custom` name prefix: "reveal cards from the top of your planar deck until you
/// reveal N plane cards. Simultaneously planeswalk to [all] of them. Put all other cards
/// revealed this way on the bottom of your planar deck in any order." N follows.
pub const PLANESWALK_TO_PLANES: &str = "planeswalk to planes:";

/// Planechase bookkeeping, in `Game::planechase`.
#[derive(Clone, Debug, Default)]
pub struct PlanarState {
    /// Grand Melee Planechase: the turn marker (by number) whose planar controller
    /// controls each face-up plane and phenomenon card (CR 901.14).
    pub slots: BTreeMap<ObjectId, u32>,
    /// Grand Melee Planechase: markers whose planar controller left the game in a way that
    /// reduced the number of turn markers; no other player became their planar
    /// controller (CR 901.14b).
    pub dissolved: BTreeSet<u32>,
}

/// Chaos ensues because a resolving spell or ability says so (CR 311.7).
pub fn chaos_ensues(g: &mut Game, p: PlayerId) {
    g.emit(Event::Custom {
        name: SmolStr::new(CHAOS_ENSUES),
        player: Some(p),
        obj: None,
        amount: 0,
    });
}

/// A face of the planar die (CR 901.3a): one Planeswalker symbol {PW} (CR 107.11), one
/// chaos symbol {CHAOS} (CR 107.12), four blank faces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanarFace {
    Blank,
    Chaos,
    Planeswalker,
}

/// The face shown by a roll of 1–6.
pub fn face_for(roll: u32) -> PlanarFace {
    match roll {
        1 => PlanarFace::Planeswalker,
        2 => PlanarFace::Chaos,
        _ => PlanarFace::Blank,
    }
}

/// Whether this is a Planechase game: on its own, or along with another variant
/// (CR 901.12, 901.14).
pub fn is_planechase(g: &Game) -> bool {
    g.config.variant == Variant::Planechase || g.config.planechase
}

fn is_grand_melee(g: &Game) -> bool {
    g.config.variant == Variant::GrandMelee
}

fn is_two_headed(g: &Game) -> bool {
    g.config.variant == Variant::TwoHeadedGiant
}

/// Whether an object is a plane or phenomenon card (by its card, even while face down).
fn is_planar_card(o: &GameObject) -> bool {
    let chars = o.card.as_ref().map(|c| &c.front().chars).unwrap_or(&o.base);
    chars.card_types.contains(CardType::Plane) || chars.card_types.contains(CardType::Phenomenon)
}

/// Whether an object is a card of type `t` (by its card, whatever its state).
fn card_is(o: &GameObject, t: CardType) -> bool {
    let chars = o.card.as_ref().map(|c| &c.front().chars).unwrap_or(&o.base);
    chars.card_types.contains(t)
}

/// The next player in turn order after `p` who is still in the game (`p` if they are).
fn in_game_or_next(g: &Game, p: PlayerId) -> PlayerId {
    if g.player(p).in_game() {
        p
    } else {
        g.next_player(p)
    }
}

/// Grand Melee: the holder of turn marker `slot` (before the markers are handed out, the
/// player who will start the game with it, CR 807.4b).
fn marker_holder(g: &Game, slot: u32) -> Option<PlayerId> {
    let ms = crate::multiplayer::grand_melee::markers(g);
    if ms.is_empty() {
        return crate::multiplayer::grand_melee::starting_holders(g)
            .get(slot.checked_sub(1)? as usize)
            .copied();
    }
    ms.iter().find(|m| m.number == slot).map(|m| m.holder)
}

/// Grand Melee: the turn markers that have a planar controller (CR 901.14a, 901.14b).
fn live_slots(g: &Game) -> Vec<u32> {
    let ms = crate::multiplayer::grand_melee::markers(g);
    let numbers: Vec<u32> = if ms.is_empty() {
        (1..=crate::multiplayer::grand_melee::starting_holders(g).len() as u32).collect()
    } else {
        ms.iter().map(|m| m.number).collect()
    };
    numbers
        .into_iter()
        .filter(|n| !g.planechase.dissolved.contains(n))
        .collect()
}

/// Grand Melee: the planar controller of turn marker `slot` — its holder, or if they've
/// left the game, the next player in turn order (CR 901.6, 901.14a).
fn slot_controller(g: &Game, slot: u32) -> Option<PlayerId> {
    if g.planechase.dissolved.contains(&slot) {
        return None;
    }
    marker_holder(g, slot).map(|h| in_game_or_next(g, h))
}

/// Grand Melee: the marker whose planar controller `p` is.
fn slot_of_player(g: &Game, p: PlayerId) -> Option<u32> {
    live_slots(g)
        .into_iter()
        .find(|s| slot_controller(g, *s) == Some(p))
}

/// The planar controller (CR 901.6): normally the active player; if they've left the
/// game, the next player in turn order still in it. In Two-Headed Giant, the primary
/// player of the active team (or of the next team still in the game, CR 901.12b). In
/// Grand Melee, the planar controller of the turn being played (CR 901.14a).
pub fn planar_controller(g: &Game) -> Option<PlayerId> {
    if !is_planechase(g) {
        return None;
    }
    let a = g.turn.active;
    if is_grand_melee(g) {
        return slot_of_player(g, in_game_or_next(g, a))
            .and_then(|s| slot_controller(g, s))
            .or_else(|| planar_controllers(g).first().copied());
    }
    if is_two_headed(g) {
        let team_in = |q: PlayerId| g.team_members(q).iter().any(|m| g.player(*m).in_game());
        let rep = if team_in(a) { a } else { g.next_player(a) };
        return Some(g.primary_player(rep));
    }
    Some(in_game_or_next(g, a))
}

/// Every planar controller: one, or in Grand Melee one for each turn marker
/// (CR 901.14a).
pub fn planar_controllers(g: &Game) -> Vec<PlayerId> {
    if !is_planechase(g) {
        return vec![];
    }
    if is_grand_melee(g) {
        let mut out: Vec<PlayerId> = Vec::new();
        for s in live_slots(g) {
            if let Some(p) = slot_controller(g, s) {
                if !out.contains(&p) {
                    out.push(p);
                }
            }
        }
        return out;
    }
    planar_controller(g).into_iter().collect()
}

/// The planar controller who controls the face-up planar card `id` (CR 901.6, 901.14).
fn controller_of(g: &Game, id: ObjectId) -> Option<PlayerId> {
    if is_grand_melee(g) {
        return g
            .planechase
            .slots
            .get(&id)
            .and_then(|s| slot_controller(g, *s));
    }
    planar_controller(g)
}

/// Whether `p` may planeswalk: they're a planar controller (CR 701.31a) — in
/// Two-Headed Giant, or a member of the planar controller's team, to whom the
/// planar controller's "you" applies (CR 901.12c).
fn may_planeswalk(g: &Game, p: PlayerId) -> bool {
    let pcs = planar_controllers(g);
    pcs.contains(&p)
        || (is_two_headed(g) && pcs.iter().any(|pc| g.player(*pc).team == g.player(p).team))
}

/// The planar controller whose planes `p` planeswalks away from and to.
fn acting_controller(g: &Game, p: PlayerId) -> PlayerId {
    if is_two_headed(g) {
        planar_controller(g).unwrap_or(p)
    } else {
        p
    }
}

/// A player's planar deck, top card first (the communal deck with the single planar deck
/// option, CR 901.15c).
pub fn planar_deck(g: &Game, p: PlayerId) -> Vec<ObjectId> {
    g.command
        .iter()
        .copied()
        .filter(|id| {
            let o = g.obj(*id);
            o.face_down && is_planar_card(o) && (g.config.single_planar_deck || o.owner == p)
        })
        .collect()
}

/// The face-up plane and phenomenon cards.
pub fn face_up_planar_cards(g: &Game) -> Vec<ObjectId> {
    g.command
        .iter()
        .copied()
        .filter(|id| {
            let o = g.obj(*id);
            !o.face_down && is_planar_card(o)
        })
        .collect()
}

/// The face-up plane and phenomenon cards `p` controls as a planar controller.
fn face_up_controlled_by(g: &Game, p: PlayerId) -> Vec<ObjectId> {
    face_up_planar_cards(g)
        .into_iter()
        .filter(|id| controller_of(g, *id) == Some(p))
        .collect()
}

/// Called as characteristics are computed: the planar controller controls each face-up
/// plane and phenomenon card (CR 901.6, 311.5, 312.4); with the single planar deck option
/// they're also considered the owner of every card in the planar deck (CR 901.15b).
pub fn apply_planar_control(g: &mut Game) {
    let Some(pc) = planar_controller(g) else {
        return;
    };
    let single = g.config.single_planar_deck;
    for id in g.command.clone() {
        let o = &g.objects[id.0 as usize];
        if !is_planar_card(o) {
            continue;
        }
        let face_up = !o.face_down;
        let ctl = if face_up {
            controller_of(g, id).unwrap_or(pc)
        } else {
            pc
        };
        let o = &mut g.objects[id.0 as usize];
        if face_up {
            o.controller = ctl;
        }
        if single {
            o.owner = pc;
            o.base_controller = pc;
            if face_up {
                o.controller = ctl;
            }
        }
    }
}

/// Turns the top card of `p`'s planar deck face up (it receives a new timestamp, CR
/// 613.7h), for the planar controller `pc`. Returns it.
fn turn_top_face_up(g: &mut Game, p: PlayerId, pc: PlayerId) -> Option<ObjectId> {
    let top = *planar_deck(g, p).first()?;
    crate::variants::turn_face_up_in_command(g, top);
    if is_grand_melee(g) {
        if let Some(s) = slot_of_player(g, pc) {
            g.planechase.slots.insert(top, s);
        }
    }
    g.recompute();
    Some(top)
}

/// Puts a card on the bottom of its owner's planar deck face down. A face-up plane or
/// phenomenon turned face down becomes a new object (CR 311.6, 312.6, 901.7a).
fn to_bottom_face_down(g: &mut Game, id: ObjectId) {
    let new = if g.obj(id).face_down {
        id
    } else {
        let new = g.create_incarnation(id, Zone::Command);
        g.objects[new.0 as usize].face_down = true;
        new
    };
    g.planechase.slots.remove(&id);
    g.command.retain(|x| *x != id);
    g.command.push(new);
    g.dirty = true;
}

/// Shuffles each planar deck (CR 103.3a): the face-down planar cards of each deck are put
/// in a random order among their places in the command zone.
pub fn shuffle_planar_decks(g: &mut Game) {
    use rand::seq::SliceRandom;
    if !is_planechase(g) {
        return;
    }
    let owners: Vec<Option<PlayerId>> = if g.config.single_planar_deck {
        vec![None]
    } else {
        g.player_ids().into_iter().map(Some).collect()
    };
    for owner in owners {
        let slots: Vec<usize> = g
            .command
            .iter()
            .enumerate()
            .filter(|(_, id)| {
                let o = g.obj(**id);
                o.face_down && is_planar_card(o) && owner.is_none_or(|p| o.owner == p)
            })
            .map(|(i, _)| i)
            .collect();
        let mut ids: Vec<ObjectId> = slots.iter().map(|i| g.command[*i]).collect();
        ids.shuffle(&mut g.rng);
        for (i, id) in slots.into_iter().zip(ids) {
            g.command[i] = id;
        }
    }
}

/// Sets the starting plane (CR 103.7, 901.5): the starting player turns the top card of
/// their planar deck face up; phenomena go to the bottom until a plane is turned face up.
/// No abilities trigger during this process. In Grand Melee each player who starts the
/// game with a turn marker sets a starting plane (CR 901.14a).
pub fn set_starting_plane(g: &mut Game) {
    if !is_planechase(g) || !face_up_planar_cards(g).is_empty() {
        return;
    }
    let setters: Vec<PlayerId> = if is_grand_melee(g) {
        crate::multiplayer::grand_melee::starting_holders(g)
    } else {
        vec![g.turn.starting_player]
    };
    for p in setters {
        for _ in 0..planar_deck(g, p).len() {
            let Some(top) = turn_top_face_up(g, p, p) else {
                break;
            };
            if g.obj(top).chars.is(CardType::Plane) {
                break;
            }
            to_bottom_face_down(g, top);
        }
    }
}

/// Ends effects that last until a player planeswalks (CR 901.11) — and those that last
/// until a player planeswalks away from a plane, if `from_plane`.
fn end_until_planeswalk_effects(g: &mut Game, from_plane: bool) {
    let ends = |d: &Duration| match d {
        Duration::UntilPlaneswalk { away_from_plane } => !*away_from_plane || from_plane,
        _ => false,
    };
    g.effects.retain(|e| !ends(&e.duration));
    g.rule_effects.retain(|e| !ends(&e.duration));
    g.player_effects.retain(|e| !ends(&e.duration));
    g.replacements.retain(|e| !ends(&e.duration));
    g.play_grants.retain(|x| !ends(&x.duration));
    g.dirty = true;
}

/// Planeswalks (CR 701.31b): each face-up plane and phenomenon card goes to the bottom of
/// its owner's planar deck face down, then `p` turns the top card of their planar deck
/// face up. Only a planar controller may planeswalk (CR 701.31a); in Grand Melee a
/// planar controller planeswalks away from the planes they control (CR 901.14). Returns
/// the plane planeswalked to.
pub fn planeswalk(g: &mut Game, p: PlayerId) -> Option<ObjectId> {
    planeswalk_to(g, p, 1).into_iter().next()
}

/// Planeswalks to the top `n` plane cards of `p`'s planar deck at once (CR 901.11c: more
/// than one plane can be face up; "simultaneously planeswalk to both of them"). With
/// `n` > 1, cards are revealed from the top until `n` plane cards are revealed; the other
/// revealed cards go to the bottom of the planar deck.
pub fn planeswalk_to(g: &mut Game, p: PlayerId, n: usize) -> Vec<ObjectId> {
    if !is_planechase(g) || !may_planeswalk(g, p) {
        return vec![];
    }
    let pc = acting_controller(g, p);
    let away = face_up_controlled_by(g, pc);
    planeswalk_away_and_to(g, p, pc, &away, &[], n)
}

/// The planeswalk itself: `away` are the face-up cards turned face down; `departed` are
/// cards planeswalked away from that left the game (CR 901.10, 901.11b), whose "planeswalk
/// away" abilities have already triggered.
fn planeswalk_away_and_to(
    g: &mut Game,
    p: PlayerId,
    pc: PlayerId,
    away: &[ObjectId],
    departed: &[ObjectId],
    n: usize,
) -> Vec<ObjectId> {
    // CR 603.10g: "when you planeswalk away from" abilities look back in time.
    crate::kwa::planeswalk::planeswalking_away(g, pc, away);
    let from_plane = away
        .iter()
        .chain(departed.iter())
        .any(|id| card_is(g.obj(*id), CardType::Plane));
    for id in away {
        to_bottom_face_down(g, *id);
    }
    g.recompute();
    // CR 901.11: continuous effects that last until a player planeswalks end.
    end_until_planeswalk_effects(g, from_plane);
    let mut to: Vec<ObjectId> = Vec::new();
    if n <= 1 {
        to.extend(turn_top_face_up(g, p, pc));
    } else {
        // Reveal until `n` plane cards are revealed; the rest go to the bottom.
        let mut others: Vec<ObjectId> = Vec::new();
        for id in planar_deck(g, p) {
            if to.len() >= n {
                break;
            }
            if card_is(g.obj(id), CardType::Plane) {
                to.push(id);
            } else {
                others.push(id);
            }
        }
        for id in &to {
            crate::variants::turn_face_up_in_command(g, *id);
            if is_grand_melee(g) {
                if let Some(s) = slot_of_player(g, pc) {
                    g.planechase.slots.insert(*id, s);
                }
            }
        }
        for id in others {
            to_bottom_face_down(g, id);
        }
        g.recompute();
    }
    for id in &to {
        g.log(|g| format!("{p} planeswalks to {}", g.obj(*id).chars.name));
        g.emit(Event::Custom {
            name: SmolStr::new(PLANESWALKED),
            player: Some(pc),
            obj: Some(*id),
            amount: 0,
        });
    }
    to
}

/// How many times `p` has rolled the planar die as a special action this turn.
fn rolls_this_turn(g: &Game, p: PlayerId) -> usize {
    g.turn_events
        .iter()
        .filter(|e| {
            matches!(e, Event::Custom { name, player: Some(q), .. }
                if name.as_str() == PLANAR_DIE_ACTION && *q == p)
        })
        .count()
}

/// Whether `p` may roll the planar die now (CR 901.9): an active player (in Two-Headed
/// Giant each member of the active team, CR 901.12d), with priority, with an empty
/// stack, in a main phase of their turn.
fn may_roll(g: &Game, p: PlayerId) -> bool {
    is_planechase(g)
        && g.active_players().contains(&p)
        && g.turn.priority == Some(p)
        && g.stack.is_empty()
        && g.turn.step.is_main()
}

/// The special action of rolling the planar die, when available (CR 901.9).
pub fn special_actions(g: &Game, p: PlayerId) -> Vec<Action> {
    if !may_roll(g, p) {
        return vec![];
    }
    // Only if its cost can be paid.
    let n = rolls_this_turn(g, p) as u32;
    let cost = Cost::mana(crate::mana::ManaCost::generic(n));
    if n > 0 && !g.can_pay_cost(p, &cost, None, &crate::eval::Ctx::new(None, p)) {
        return vec![];
    }
    vec![Action::Special(SpecialAction::Other {
        name: PLANAR_DIE_ACTION.into(),
        obj: None,
    })]
}

/// Rolls the planar die as a special action (CR 901.9): it costs {N}, where N is the
/// number of times the player has already done so this turn (each player's own count in
/// Two-Headed Giant, CR 901.12d).
pub fn perform_special_action(
    g: &mut Game,
    p: PlayerId,
    sa: &SpecialAction,
) -> Option<Result<(), Illegal>> {
    let SpecialAction::Other { name, .. } = sa else {
        return None;
    };
    if name.as_str() != PLANAR_DIE_ACTION {
        return None;
    }
    if !may_roll(g, p) {
        return Some(Err(Illegal("can't roll the planar die now".into())));
    }
    let n = rolls_this_turn(g, p) as u32;
    if n > 0 {
        let cost = Cost::mana(crate::mana::ManaCost::generic(n));
        let ctx = crate::eval::Ctx::new(None, p);
        if !g.pay_cost(p, &cost, None, &ctx) {
            return Some(Err(Illegal("can't pay to roll the planar die".into())));
        }
    }
    g.emit(Event::Custom {
        name: SmolStr::new(PLANAR_DIE_ACTION),
        player: Some(p),
        obj: None,
        amount: n as i32,
    });
    roll_planar_die(g, p);
    Some(Ok(()))
}

/// Whether a rule effect with the custom restriction `name` applies.
fn rule_applies(g: &Game, name: &str) -> bool {
    g.rule_effects
        .iter()
        .any(|e| matches!(&e.restriction, Restriction::Custom(n) if n.as_str() == name))
}

/// Rolls the planar die (CR 901.9a–c): on the chaos symbol, chaos ensues; on the
/// Planeswalker symbol, the planeswalking ability triggers; a blank does nothing (unless
/// an effect makes blank rolls chaos rolls).
pub fn roll_planar_die(g: &mut Game, p: PlayerId) -> PlanarFace {
    let rolled = face_for(g.random_range(1, 6));
    // "Each blank roll of the planar die is a {CHAOS} roll" (Chaotic Aether).
    let face = if rolled == PlanarFace::Blank && rule_applies(g, BLANK_ROLLS_ARE_CHAOS) {
        PlanarFace::Chaos
    } else {
        rolled
    };
    g.log(|_| format!("{p} rolls the planar die: {face:?}"));
    g.emit(Event::Custom {
        name: SmolStr::new(ROLLED_PLANAR_DIE),
        player: Some(p),
        obj: None,
        amount: 0,
    });
    // CR 706.7, 901.9d: it's a die roll, with no numerical result.
    crate::dice::planar_die_rolled(g, p);
    match face {
        PlanarFace::Blank => {}
        PlanarFace::Chaos => chaos_ensues(g, p),
        PlanarFace::Planeswalker => planeswalking_ability_triggers(g, p),
    }
    g.flush_events();
    face
}

/// The inherent "planeswalking ability" (CR 901.8): "Whenever you roll the Planeswalker
/// symbol on the planar die, planeswalk." It has no source (an object outside every zone
/// with no characteristics stands in for one) and is controlled by the player whose roll
/// caused it to trigger.
fn planeswalking_ability_triggers(g: &mut Game, p: PlayerId) {
    let src = g.create_token_object(Default::default(), p);
    let mut t = TriggeredAbility::new(
        TriggerCond::Custom(SmolStr::new(PLANESWALKING_ABILITY)),
        Body::effect(Effect::Custom(SmolStr::new(PLANESWALK_EFFECT))),
    );
    t.zone = FunctionZone::Command;
    let ability = AbilityDef::new(
        AbilityKind::Triggered(t),
        "Whenever you roll the Planeswalker symbol on the planar die, planeswalk.",
    );
    g.trigger_order += 1;
    g.pending_triggers.push(PendingTrigger {
        source: src,
        controller: p,
        ability,
        event: EventInfo {
            player: Some(p),
            ..Default::default()
        },
        source_lki: None,
        saved: None,
        body: None,
        order: g.trigger_order,
    });
}

/// Whether a stack object or pending trigger is a planeswalking ability.
fn is_planeswalking_ability(a: &Ability) -> bool {
    matches!(&a.kind, AbilityKind::Triggered(t)
        if matches!(&t.trigger, TriggerCond::Custom(n) if n.as_str() == PLANESWALKING_ABILITY))
}

/// The planeswalking ability resolves (CR 901.8): its controller planeswalks — unless an
/// effect says that chaos ensues instead of planeswalking as a result of rolling the
/// planar die (Fixed Point in Time).
pub fn planeswalking_ability_resolves(g: &mut Game, p: PlayerId) {
    if rule_applies(g, PLANESWALK_ROLLS_ARE_CHAOS) {
        chaos_ensues(g, p);
        return;
    }
    planeswalk(g, p);
}

// --- Players leaving the game (CR 901.10, 901.14b) -----------------------------------

/// What happened to the planar cards of a player leaving the game, between
/// [`player_leaving`] and [`player_left`].
#[derive(Clone, Debug, Default)]
pub struct Departure {
    /// Face-up planar cards the player owned that are leaving the game with them, with
    /// the planar controller who controlled each.
    pub face_up: Vec<(ObjectId, PlayerId)>,
}

/// Called as `p` leaves the game, before the objects they own leave it (CR 800.4a,
/// 901.10). The new planar controller takes over (CR 901.6); abilities from phenomena
/// `p` owns stay on the stack under the new planar controller's control (CR 901.10b).
/// In Grand Melee, if `p` leaving reduces the number of turn markers, `p` stops being a
/// planar controller and the planar cards they controlled go to the bottom of their
/// owners' planar decks; no one planeswalks (CR 901.14b).
pub fn player_leaving(g: &mut Game, p: PlayerId) -> Departure {
    let mut dep = Departure::default();
    if !is_planechase(g) {
        return dep;
    }
    // CR 901.15b: with the single planar deck option, the new planar controller owns the
    // communal planar deck.
    apply_planar_control(g);
    if is_grand_melee(g) && crate::multiplayer::grand_melee::departure_reduces_markers(g) {
        let theirs: Vec<u32> = live_slots(g)
            .into_iter()
            .filter(|s| marker_holder(g, *s) == Some(p))
            .collect();
        let controlled: Vec<ObjectId> = face_up_planar_cards(g)
            .into_iter()
            .filter(|id| {
                g.planechase
                    .slots
                    .get(id)
                    .is_some_and(|s| theirs.contains(s))
            })
            .collect();
        for s in theirs {
            g.planechase.dissolved.insert(s);
        }
        for id in controlled {
            if g.obj(id).owner != p {
                to_bottom_face_down(g, id);
            } else {
                g.planechase.slots.remove(&id);
            }
        }
        g.dirty = true;
        return dep;
    }
    let Some(new_pc) = planar_controller(g) else {
        return dep;
    };
    // CR 901.10b: abilities from phenomena the player owned stay on the stack, controlled
    // by the new planar controller.
    for s in g.stack.clone() {
        let src = g.ability_source_of(s);
        if src == s {
            continue;
        }
        let from_phenomenon = g
            .try_obj(src)
            .is_some_and(|o| card_is(o, CardType::Phenomenon))
            || g.obj(s)
                .stack
                .as_deref()
                .and_then(|x| x.source_lki.as_deref())
                .is_some_and(|c| c.is(CardType::Phenomenon));
        if from_phenomenon && g.obj(src).owner == p {
            let o = &mut g.objects[s.0 as usize];
            o.owner = new_pc;
            o.controller = new_pc;
            o.base_controller = new_pc;
        }
    }
    for t in g.pending_triggers.iter_mut() {
        let o = &g.objects[t.source.0 as usize];
        if o.owner == p && card_is(o, CardType::Phenomenon) {
            t.controller = new_pc;
        }
    }
    // Face-up planar cards the player owns leave the game with them; whoever controlled
    // them planeswalks away from them (CR 901.11a, 901.11b). Their "planeswalk away"
    // abilities trigger now, while they still exist (CR 603.10g).
    for id in face_up_planar_cards(g) {
        if g.obj(id).owner == p {
            let ctl = controller_of(g, id).unwrap_or(new_pc);
            dep.face_up.push((id, ctl));
            crate::kwa::planeswalk::planeswalking_away(g, ctl, &[id]);
        }
    }
    dep
}

/// Called once the objects `p` owned have left the game (CR 901.10): if that included a
/// face-up plane or phenomenon card, the planar controller turns the top card of their
/// planar deck face up — they planeswalk (CR 901.11a). This isn't a state-based action.
/// If a plane left the game while a planeswalking ability was on the stack, that ability
/// ceases to exist (CR 901.10a).
pub fn player_left(g: &mut Game, p: PlayerId, dep: Departure) {
    if dep.face_up.is_empty() {
        return;
    }
    let plane_left = dep
        .face_up
        .iter()
        .any(|(id, _)| card_is(g.obj(*id), CardType::Plane));
    if plane_left {
        for s in g.stack.clone() {
            let is_pw = matches!(
                g.obj(s).stack.as_deref().map(|x| &x.kind),
                Some(StackKind::Triggered { ability, .. }) if is_planeswalking_ability(ability)
            );
            if is_pw {
                g.stack.retain(|x| *x != s);
                g.objects[s.0 as usize].zone = Zone::Nowhere;
            }
        }
        g.pending_triggers
            .retain(|t| !is_planeswalking_ability(&t.ability));
    }
    let mut pcs: Vec<PlayerId> = Vec::new();
    for (_, ctl) in &dep.face_up {
        let ctl = if g.player(*ctl).in_game() {
            *ctl
        } else {
            planar_controller(g).unwrap_or(*ctl)
        };
        if !pcs.contains(&ctl) {
            pcs.push(ctl);
        }
    }
    let departed: Vec<ObjectId> = dep.face_up.iter().map(|(id, _)| *id).collect();
    for pc in pcs {
        if !g.player(pc).in_game() || pc == p {
            continue;
        }
        let away = face_up_controlled_by(g, pc);
        planeswalk_away_and_to(g, pc, pc, &away, &departed, 1);
    }
}

// --- "You" in Two-Headed Giant Planechase (CR 901.12c) ------------------------------

/// Whether `ctx` is that of an ability of a face-up plane or phenomenon (or a spell or
/// ability it created).
fn from_planar_card(g: &Game, ctx: &crate::eval::Ctx) -> bool {
    ctx.source.is_some_and(|s| {
        let s = g.ability_source_of(s);
        g.try_obj(s)
            .is_some_and(|o| is_planar_card(o) && o.zone == Zone::Command && !o.face_down)
    })
}

/// CR 901.12c: in Two-Headed Giant Planechase, an ability of the face-up plane or
/// phenomenon that refers to "you" applies to both members of the planar controller's
/// team. The players "you" refers to, if that applies to `ctx`.
pub fn you_players(g: &Game, ctx: &crate::eval::Ctx) -> Option<Vec<PlayerId>> {
    if !g.config.planechase || !is_two_headed(g) || !from_planar_card(g, ctx) {
        return None;
    }
    let mut v: Vec<PlayerId> = g
        .team_members(ctx.controller)
        .into_iter()
        .filter(|q| g.player(*q).in_game())
        .collect();
    v.retain(|q| *q != ctx.controller);
    v.insert(0, ctx.controller);
    Some(v)
}

/// Whether `p` is a "you" of the ability with context `ctx` (CR 901.12c).
pub fn is_you(g: &Game, p: PlayerId, ctx: &crate::eval::Ctx) -> bool {
    you_players(g, ctx).is_some_and(|v| v.contains(&p))
}
