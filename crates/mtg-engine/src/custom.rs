//! Named custom predicates, values, conditions, triggers, and effects — escape hatches
//! for behavior that doesn't fit the ability language. Referenced from the AST by name
//! (e.g. `Filter::Custom("...")`, `TriggerCond::Custom("chapter:1")`).

use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::object::*;
use crate::types::*;

/// "with an activated ability that isn't a mana ability" (e.g. cycling, which exists in
/// every zone, CR 702.29b).
pub const HAS_NONMANA_ACTIVATED_ABILITY: &str = "has_nonmana_activated_ability";

pub fn custom_filter(g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> bool {
    let _ = (g, id, ctx);
    match name {
        HAS_NONMANA_ACTIVATED_ABILITY => g.obj(id).chars.abilities.iter().any(
            |a| matches!(&a.kind, crate::ability::AbilityKind::Activated(x) if !x.is_mana_ability),
        ),
        // CR 702.171b: the saddled designation.
        "saddled" => g.obj(id).saddled,
        // "Equipment attached to it" where "it" is each object an effect applies to.
        "attached_to_affected" => ctx
            .vars
            .get(&crate::ability::vars::AFFECTED)
            .and_then(|v| v.first().copied())
            .is_some_and(|e| g.obj(id).attached_to == Some(e)),
        // "Aura attached to it" where "it" is the object the source is attached to.
        "attached_to_host" => {
            let host = ctx.source.and_then(|s| g.obj(s).attached_to);
            host.is_some() && g.obj(id).attached_to == host
        }
        // "with toughness greater than its power".
        "toughness_gt_power" => g.obj(id).toughness() > g.obj(id).power(),
        _ => false,
    }
}

fn cast_info<'a>(g: &'a Game, ctx: &'a Ctx) -> Option<&'a CastInfo> {
    g.cast_info(ctx)
}

pub fn custom_value(g: &Game, name: &str, ctx: &Ctx) -> i64 {
    // Values referring to stickers (CR 123.6d, 123.6e, 123.8a).
    if let Some(v) = crate::stickers::sticker_value(g, name, ctx) {
        return v;
    }
    // Die roll results (CR 706).
    if let Some(v) = crate::dice::custom_value(g, name, ctx) {
        return v;
    }
    let _ = (g, ctx);
    // "for each of its colors": the source, the object it's attached to, or the object
    // an effect is being applied to.
    if let Some(which) = name.strip_prefix("colors_of:") {
        let obj = match which {
            "source" => ctx.source,
            "host" => ctx
                .source
                .and_then(|s| g.obj(s).attached_to)
                .and_then(|e| e.object()),
            "affected" => ctx
                .vars
                .get(&crate::ability::vars::AFFECTED)
                .and_then(|v| v.first().copied())
                .and_then(|e| e.object()),
            _ => None,
        };
        return obj.map_or(0, |o| g.obj(o).chars.colors.count() as i64);
    }
    // The most counters of a kind any opponent has ("an opponent is poisoned").
    if let Some(k) = name.strip_prefix("max_opponent_counters:") {
        return g
            .players
            .iter()
            .filter(|p| !p.has_lost && g.are_opponents(ctx.controller, p.id))
            .map(|p| p.counter(k) as i64)
            .max()
            .unwrap_or(0);
    }
    // The total counters of a kind your opponents have ("for each poison counter your
    // opponents have").
    if let Some(k) = name.strip_prefix("opponents_counters:") {
        return g
            .players
            .iter()
            .filter(|p| !p.has_lost && g.are_opponents(ctx.controller, p.id))
            .map(|p| p.counter(k) as i64)
            .sum();
    }
    // "mana_spent_of:U": amount of mana of one type spent to cast it (adamant).
    if let Some(t) = name.strip_prefix("mana_spent_of:") {
        let Some(t) = t
            .chars()
            .next()
            .and_then(crate::mana::ManaType::from_letter)
        else {
            return 0;
        };
        return cast_info(g, ctx).map_or(0, |c| {
            c.mana_spent.iter().filter(|m| **m == t).count() as i64
        });
    }
    match name {
        // "at least three mana of the same color was spent to cast it" (adamant).
        "max_mana_spent_of_one_color" => cast_info(g, ctx).map_or(0, |c| {
            [
                crate::mana::ManaType::W,
                crate::mana::ManaType::U,
                crate::mana::ManaType::B,
                crate::mana::ManaType::R,
                crate::mana::ManaType::G,
            ]
            .iter()
            .map(|t| c.mana_spent.iter().filter(|m| *m == t).count() as i64)
            .max()
            .unwrap_or(0)
        }),
        // Creatures that died under the controller's control this turn.
        "creatures_you_controlled_died_this_turn" => g
            .history
            .creatures_died
            .iter()
            .filter(|o| g.obj(**o).controller == ctx.controller)
            .count() as i64,
        // "the total number of cards in all players' hands"
        "cards_in_all_hands" => g
            .players_in_game()
            .into_iter()
            .map(|p| g.player(p).hand.len() as i64)
            .sum(),
        // "the total life lost by your opponents this turn"
        "life_lost_by_opponents_this_turn" => g
            .history
            .life_lost
            .iter()
            .filter(|(p, _)| g.are_opponents(ctx.controller, **p))
            .map(|(_, n)| *n as i64)
            .sum(),
        // Number of spells the controller has cast this turn.
        "spells_you_cast_this_turn" => g
            .history
            .spells_cast
            .iter()
            .filter(|(p, _)| *p == ctx.controller)
            .count() as i64,
        _ => 0,
    }
}

pub fn custom_condition(g: &Game, name: &str, ctx: &Ctx) -> bool {
    let _ = (g, ctx);
    // Conditions defined by keyword implementations (`kw/`).
    if let Some(b) = crate::kw::custom_condition(g, name, ctx) {
        return b;
    }
    // "you both own and control [this] and its meld partner" (CR 701.42a).
    if let Some(b) = crate::merge::custom_condition(g, name, ctx) {
        return b;
    }
    // "If you rolled doubles" (CR 706.5).
    if let Some(b) = crate::dice::custom_condition(name, ctx) {
        return b;
    }
    // "If it's a creature card" about a revealed face-down permanent (CR 708.12).
    if let Some(b) = crate::facedown::custom_condition(g, name, ctx) {
        return b;
    }
    // Main phase counting and "after upkeep" timing (CR 505.1b, 503.2).
    if let Some(b) = crate::turn_structure::custom_condition(g, name, ctx) {
        return b;
    }
    let you = ctx.controller;
    let h = &g.history;
    // "you've cast another red spell this turn": a spell of that color (as it was on the
    // stack) other than the source.
    if let Some(c) = name.strip_prefix("you_cast_another_spell_this_turn:") {
        let Some(color) = c.chars().next().and_then(Color::from_letter) else {
            return false;
        };
        return g.history.spells_cast.iter().any(|(p, o)| {
            *p == ctx.controller && Some(*o) != ctx.source && g.obj(*o).chars.colors.contains(color)
        });
    }
    // "[color] is the most common color among all permanents or is tied for most
    // common".
    if let Some(c) = name.strip_prefix("most_common_color:") {
        let Some(color) = c.chars().next().and_then(Color::from_letter) else {
            return false;
        };
        let count = |c: Color| {
            g.permanents()
                .filter(|o| o.chars.colors.contains(c))
                .count()
        };
        let n = count(color);
        return Color::ALL.iter().all(|c| count(*c) <= n);
    }
    match name {
        // "if you attacked this turn" (raid): you declared one or more attackers this turn;
        // only the active player declares attackers (CR 508.1).
        "you_attacked_this_turn" => g.turn.active == you && !h.attackers.is_empty(),
        // "if a permanent you controlled left the battlefield this turn" (last known
        // information of the permanents that left).
        "permanent_you_controlled_left_this_turn" => h
            .permanents_left
            .iter()
            .any(|o| g.obj(*o).controller == you),
        "creature_died_under_your_control_this_turn" => {
            h.creatures_died.iter().any(|o| g.obj(*o).controller == you)
        }
        "you_descended_this_turn" => h.descended.get(&you).is_some_and(|n| *n > 0),
        "card_left_your_graveyard_this_turn" => {
            h.cards_left_graveyard.get(&you).is_some_and(|n| *n > 0)
        }
        "you_cast_noncreature_spell_this_turn" => h
            .spells_cast
            .iter()
            .any(|(p, s)| *p == you && !g.obj(*s).chars.card_types.contains(CardType::Creature)),
        // Werewolves (the previous turn's history).
        "no_spells_cast_last_turn" => g.last_turn_history.spells_cast.is_empty(),
        "a_player_cast_two_spells_last_turn" => {
            let spells = &g.last_turn_history.spells_cast;
            spells
                .iter()
                .any(|(p, _)| spells.iter().filter(|(q, _)| q == p).count() >= 2)
        }
        "you_lost_life_last_turn" => g
            .last_turn_history
            .life_lost
            .get(&you)
            .is_some_and(|n| *n > 0),
        // "a permanent left the battlefield under your control this turn" (revolt).
        "permanent_left_under_your_control_this_turn" => g
            .history
            .permanents_left
            .iter()
            .any(|o| g.obj(*o).controller == ctx.controller),
        // "you were the starting player" (CR 103.1).
        "you_were_the_starting_player" => g.turn.starting_player == ctx.controller,
        // "an opponent lost life this turn".
        "opponent_lost_life_this_turn" => g
            .history
            .life_lost
            .iter()
            .any(|(p, n)| *n > 0 && g.are_opponents(ctx.controller, *p)),
        _ => false,
    }
}

/// Custom triggers. `chapter:N[,M]` implements Saga chapter abilities (CR 714.2c):
/// triggers when lore counters are put on the source, bringing the count from below N
/// to N or more.
pub fn custom_trigger(
    g: &Game,
    name: &str,
    src: ObjectId,
    ctl: PlayerId,
    ev: &Event,
) -> Vec<EventInfo> {
    let _ = ctl;
    // Triggers defined by keyword implementations (`kw/`).
    if let Some(v) = crate::kw::custom_trigger(g, name, src, ctl, ev) {
        return v;
    }
    // "When you unlock this door" (CR 709.5h), "Whenever this creature mutates".
    if let Some(v) = crate::rooms::custom_trigger(name, src, ev)
        .or_else(|| crate::merge::custom_trigger(name, src, ev))
        .or_else(|| crate::dice::custom_trigger(g, name, ctl, ev))
        .or_else(|| crate::variants::custom_trigger(g, name, src, ctl, ev))
        .or_else(|| crate::dungeons::custom_trigger(g, name, src, ev))
    {
        return v;
    }
    if let Some(ns) = name.strip_prefix("chapter:") {
        if let Event::CountersAdded {
            target: Entity::Object(o),
            kind,
            n,
        } = ev
        {
            if *o == src && kind.as_str() == counters::LORE {
                let after = g.obj(src).counter(counters::LORE);
                let before = after.saturating_sub(*n);
                let mut out = Vec::new();
                for part in ns.split(',') {
                    if let Ok(k) = part.trim().parse::<u32>() {
                        if before < k && after >= k {
                            out.push(EventInfo {
                                object: Some(src),
                                amount: k as i32,
                                ..Default::default()
                            });
                        }
                    }
                }
                return out;
            }
        }
        return vec![];
    }
    // "sticker placed:you" / "sticker placed:self": "Whenever you place a sticker",
    // "Whenever you put a sticker on ~".
    if let Some(which) = name.strip_prefix("sticker placed:") {
        if let Event::Custom {
            name: n,
            player: Some(p),
            obj: Some(o),
            amount,
        } = ev
        {
            let ok = n == crate::stickers::PLACED_EVENT
                && match which {
                    "you" => *p == ctl,
                    "self" => *o == src,
                    _ => false,
                };
            if ok {
                return vec![EventInfo {
                    object: Some(*o),
                    player: Some(*p),
                    amount: *amount,
                    ..Default::default()
                }];
            }
        }
        return vec![];
    }
    // "damage prevented:self": "Whenever damage that would be dealt to this is prevented"
    // (CR 615.13), once per prevention effect applied to simultaneous damage.
    if name == "damage prevented:self" {
        if let Event::DamagePrevented {
            source,
            target: Entity::Object(o),
            amount,
            ..
        } = ev
        {
            if *o == src {
                return vec![EventInfo {
                    object: Some(src),
                    other: Some(*source),
                    amount: *amount as i32,
                    ..Default::default()
                }];
            }
        }
        return vec![];
    }
    vec![]
}

pub fn custom_effect(g: &mut Game, name: &str, ctx: &mut Ctx) {
    // Effects defined by keyword implementations (`kw/`).
    if crate::kw::custom_effect(g, name, ctx) {
        return;
    }
    // "named-token:N:Name": create N tokens by name (CR 111.11).
    if let Some(spec) = name.strip_prefix("named-token:") {
        crate::tokens::create_named_tokens(g, spec, ctx);
        return;
    }
    // "The game is a draw" (CR 104.4c, 104.4e).
    if crate::game_end::custom_effect(g, name, ctx) {
        return;
    }
    // "Target unblocked attacking creature becomes blocked" (CR 509.1h, 702.22i).
    if name == crate::oracle::patterns::k702_banding::TARGET_BECOMES_BLOCKED {
        let objs: Vec<ObjectId> = ctx
            .vars
            .get(&crate::ability::vars::AFFECTED)
            .map(|v| v.iter().filter_map(|e| e.object()).collect())
            .unwrap_or_default();
        for t in objs {
            crate::combat::become_blocked(g, t);
        }
        return;
    }
    // The planeswalking ability (CR 901.8, 701.31).
    if name == crate::planechase::PLANESWALK_EFFECT {
        crate::planechase::planeswalk(g, ctx.controller);
        return;
    }
    if name == crate::planechase::ROLL_PLANAR_DIE_EFFECT {
        // Outside a Planechase game there's no planar die: nothing happens.
        if crate::planechase::is_planechase(g) {
            crate::planechase::roll_planar_die(g, ctx.controller);
        }
        return;
    }
    // "exile them, then meld them into [result]" (CR 701.42a).
    if crate::merge::custom_effect(g, name, ctx) {
        return;
    }
    // "Roll again", rerolling stored results (CR 706.3c, 706.8b).
    if crate::dice::custom_effect(g, name, ctx) {
        return;
    }
    // Revealing a face-down permanent (CR 708.9, 708.12).
    if crate::facedown::custom_effect(g, name, ctx) {
        return;
    }
    match name {
        crate::kw::suspend::CAST_SUSPENDED => crate::kw::suspend::cast_suspended(g, ctx),
        crate::kw::miracle::CAST_MIRACLE => crate::kw::miracle::cast_miracle(g, ctx),
        _ => {}
    }
}

/// "can't have more than N [kind] counters on it" (CR 704.5r).
pub fn counter_limits(g: &Game, id: ObjectId) -> Vec<(CounterKind, u32)> {
    crate::counter_rules::counter_limits(g, id)
}
