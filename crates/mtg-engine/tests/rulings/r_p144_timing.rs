//! Rulings batch P144 — repeatable impulse draw: a card exiled by one of these abilities
//! is played with the normal timing rules and costs (CR 305.1, 305.2, 307.1, 601.2f-h):
//! a land only in a main phase with the stack empty and a land play available, a sorcery
//! or creature only in a main phase with the stack empty, paying its mana cost. One table
//! case per card (the rulings restate the same rule for each card's own wording).

use crate::r_p107_common::{act, mana};
use crate::r_p144_common::*;
use crate::r_s02_common::create_token;
use crate::r_s06_common::attach_new;
use crate::r_s09_common::declare;
use crate::r_s16_common::set_speed;
use crate::r_s29_common::choose_modes;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// P0 attacks P1 with `attackers` and the combat runs to its end (with no blocks).
fn attack_through(t: &mut TestGame, attackers: &[ObjectId]) {
    let decl: Vec<(ObjectId, Entity)> =
        attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    declare(t, P0, &decl);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
}

/// P0 attacks P1 with `attackers`; the attack triggers resolve.
fn attack_only(t: &mut TestGame, attackers: &[ObjectId]) {
    let decl: Vec<(ObjectId, Entity)> =
        attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    declare(t, P0, &decl);
    t.resolve_all();
}

/// A Prodigal Pyromancer of P0's deals 1 damage to `target`.
fn ping(t: &mut TestGame, target: Entity) {
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    act(t, P0, pyro, "damage", &[target]).expect("ping");
    t.resolve_all();
}

const BOTH: &[Exiled] = &[Exiled::Land, Exiled::Sorcery];
const LAND: &[Exiled] = &[Exiled::Land];
const SORCERY: &[Exiled] = &[Exiled::Sorcery];
const CREATURE: &[Exiled] = &[Exiled::Creature];
const LAND_CREATURE: &[Exiled] = &[Exiled::Land, Exiled::Creature];

#[test]
fn impulse_draw_cards_follow_normal_timing_and_costs() {
    cr!("305.1", "305.2", "307.1", "302.1", "601.2f", "601.2h");
    let cases = [
        Case {
            card: "Furious Rise",
            cite: || {
                ruling!(
                    "Furious Rise",
                    "Furious Rise's effect doesn't change when you can play the exiled card. For example, if you exile a sorcery card, you can cast it only during your main phase when the stack is empty. If you exile a land card, you can't play it unless you have land plays available."
                );
            },
            setup: |t, _| {
                t.battlefield(P0, "Furious Rise");
                t.battlefield(P0, "Craw Wurm");
                t.advance_to(P0, Step::End);
            },
            kinds: BOTH,
        },
        Case {
            card: "Breya's Apprentice",
            cite: || {
                ruling!(
                    "Breya's Apprentice",
                    "If you choose the first mode, you must still pay any costs and follow any timing restrictions to play the card."
                );
            },
            setup: |t, _| {
                let b = t.battlefield(P0, "Breya's Apprentice");
                let orni = t.battlefield(P0, "Ornithopter");
                choose_modes(t, P0, &[0]);
                t.answer_choose(P0, &[obj(orni)]);
                act(t, P0, b, "Sacrifice an artifact", &[]).expect("activate");
            },
            kinds: BOTH,
        },
        Case {
            card: "Outpost Siege",
            cite: || {
                ruling!(
                    "Outpost Siege",
                    "If you exile a land card using the \"Khans\" ability, you may play that land only if you have any available land plays. Normally, this means you can play the land only if you haven't played a land yet that turn."
                );
                ruling!(
                    "Outpost Siege",
                    "The card exiled by the \"Khans\" ability is exiled face up. Playing a card exiled with the \"Khans\" ability follows the normal rules for playing the card. You must pay its costs, and you must follow all applicable timing rules. For example, if it's a creature card, you can cast it only during your main phase while the stack is empty."
                );
            },
            setup: |t, _| {
                t.answer(
                    P0,
                    DecisionKind::Option,
                    mtg_engine::decision::Answer::Index(0),
                );
                t.enter(P0, "Outpost Siege");
                t.settle();
                next_upkeep(t);
            },
            kinds: LAND_CREATURE,
        },
        Case {
            card: "Prophetic Flamespeaker",
            cite: || {
                ruling!(
                    "Prophetic Flamespeaker",
                    "If you exile a land card, you may play that land only if you have any available land plays. Normally, this means you can play the land only if you haven't played a land yet that turn."
                );
                ruling!(
                    "Prophetic Flamespeaker",
                    "The card is exiled face up. Playing it follows the normal rules for playing that card. You must pay its costs, and you must follow all applicable timing rules. For example, if it's a creature card, you can cast it only during your main phase while the stack is empty."
                );
            },
            setup: |t, _| {
                let f = t.battlefield(P0, "Prophetic Flamespeaker");
                attack_through(t, &[f]);
            },
            kinds: LAND_CREATURE,
        },
        Case {
            card: "Neyali, Suns' Vanguard",
            cite: || {
                ruling!(
                    "Neyali, Suns' Vanguard",
                    "Neyali's effect doesn't change when you can cast the exiled card. For example, if you exile a sorcery card, you can cast it only during your main phase when the stack is empty."
                );
                ruling!(
                    "Neyali, Suns' Vanguard",
                    "The card Neyali exiles is exiled face up."
                );
            },
            setup: |t, _| {
                t.battlefield(P0, "Neyali, Suns' Vanguard");
                let tok = create_token(t, P0, "Soldier");
                attack_only(t, &[tok]);
            },
            kinds: SORCERY,
        },
        Case {
            card: "Chaos Channeler",
            cite: || {
                ruling!(
                    "Chaos Channeler",
                    "Normal costs and timing permissions apply to cards played this way. Unless an effect says you can play additional lands during your turn, you cannot play a land exiled this way if you have already played one this turn."
                );
            },
            setup: |t, _| {
                let c = t.battlefield(P0, "Chaos Channeler");
                attack_only(t, &[c]);
            },
            kinds: BOTH,
        },
        Case {
            card: "Oracle's Vault",
            cite: || {
                ruling!(
                    "Oracle's Vault",
                    "Playing a card exiled with Oracle's Vault follows the normal timing rules for playing that card. For example, if the card is a creature card, you can cast that card only during your main phase while the stack is empty."
                );
                ruling!(
                    "Oracle's Vault",
                    "If the exiled card is a land card, you may play it only if you have an available land play this turn."
                );
                ruling!(
                    "Oracle's Vault",
                    "The cards exiled by Oracle's Vault are exiled face up."
                );
            },
            setup: |t, _| {
                let v = t.battlefield(P0, "Oracle's Vault");
                mana(t, P0, ManaType::C, 2);
                act(t, P0, v, "brick counter", &[]).expect("activate");
            },
            kinds: LAND_CREATURE,
        },
        Case {
            card: "Tuskeri Firewalker",
            cite: || {
                ruling!(
                    "Tuskeri Firewalker",
                    "Playing a card this way follows the normal rules for playing the card. You must pay its costs, and you must follow all applicable timing rules. For example, if the card is a sorcery card, you can cast it only during your main phase while the stack is empty."
                );
                ruling!(
                    "Tuskeri Firewalker",
                    "Under normal circumstances, you can play a land this way only if you haven't played a land yet that turn."
                );
            },
            setup: |t, _| {
                let f = t.battlefield(P0, "Tuskeri Firewalker");
                attack_only(t, &[f]);
                mana(t, P0, ManaType::C, 1);
                act(t, P0, f, "Exile the top", &[]).expect("boast");
            },
            kinds: BOTH,
        },
        Case {
            card: "Hedron Detonator",
            cite: || {
                ruling!(
                    "Hedron Detonator",
                    "Playing the card exiled with Hedron Detonator follows the normal rules for playing that card. You must pay its costs, and you must follow all applicable timing rules. For example, if the card is a creature card, you can cast that card by paying its mana cost only during your main phase while the stack is empty."
                );
                ruling!(
                    "Hedron Detonator",
                    "Unless an effect allows you to play additional lands that turn, you can play a land card exiled with Hedron Detonator only if you haven’t played a land yet that turn."
                );
            },
            setup: |t, _| {
                let h = t.battlefield(P0, "Hedron Detonator");
                let a = [
                    t.battlefield(P0, "Ornithopter"),
                    t.battlefield(P0, "Ornithopter"),
                ];
                t.answer_choose(P0, &[obj(a[0]), obj(a[1])]);
                act(t, P0, h, "Exile the top", &[]).expect("activate");
            },
            kinds: LAND_CREATURE,
        },
        Case {
            card: "Ob Nixilis, Captive Kingpin",
            cite: || {
                ruling!(
                    "Ob Nixilis, Captive Kingpin",
                    "Playing the card exiled with Ob Nixilis follows the normal rules for playing that card. You must pay its costs, and you must follow all applicable timing rules. For example, if the card is a creature card, you can cast that card by paying its mana cost only during your main phase while the stack is empty."
                );
                ruling!(
                    "Ob Nixilis, Captive Kingpin",
                    "Unless an effect allows you to play additional lands that turn, you can play a land card exiled with Ob Nixilis only if you haven't played a land yet that turn."
                );
            },
            setup: |t, _| {
                t.battlefield(P0, "Ob Nixilis, Captive Kingpin");
                ping(t, Entity::Player(P1));
            },
            kinds: LAND_CREATURE,
        },
        Case {
            card: "Ardent Dustspeaker",
            cite: || {
                ruling!(
                    "Ardent Dustspeaker",
                    "Playing the cards exiled with Ardent Dustspeaker follows the normal rules for playing those cards. You must pay their costs, if any, and you must follow all applicable timing rules. For example, if one of the cards is a creature card, you can cast that card by paying its mana cost only during your main phase while the stack is empty."
                );
                ruling!(
                    "Ardent Dustspeaker",
                    "The cards exiled by Ardent Dustspeaker’s ability are exiled face up."
                );
            },
            setup: |t, _| {
                let d = t.battlefield(P0, "Ardent Dustspeaker");
                t.graveyard(P0, "Lightning Bolt");
                t.answer_yes(P0, true);
                attack_only(t, &[d]);
            },
            kinds: CREATURE,
        },
        Case {
            card: "Invasion of Kaldheim // Pyre of the World Tree",
            cite: || {
                ruling!(
                    "Invasion of Kaldheim // Pyre of the World Tree",
                    "Playing the exiled cards follows the normal rules for playing those cards. You must pay their costs, and you must follow all applicable timing rules. For example, if one of the cards is a creature card, you can cast that card by paying its mana cost only during your main phase while the stack is empty."
                );
                ruling!(
                    "Invasion of Kaldheim // Pyre of the World Tree",
                    "Unless an effect allows you to play additional lands that turn, you can play an exiled land card only if you haven’t played a land yet that turn."
                );
            },
            setup: |t, _| {
                // The card is drawn first: the Siege exiles P0's hand.
                let drawn = t.g.draw_cards(P0, 1);
                t.g.flush_events();
                assert_eq!(drawn.len(), 1);
                t.enter(P0, "Invasion of Kaldheim");
            },
            kinds: LAND_CREATURE,
        },
        Case {
            card: "Syr Carah, the Bold",
            cite: || {
                ruling!(
                    "Syr Carah, the Bold",
                    "Syr Carah doesn't change when you can cast the exiled card. For example, if you exile a sorcery card, you can cast it only during your main phase when the stack is empty. If you exile a land card, you can play it only during your main phase and only if you have an available land play remaining."
                );
            },
            setup: |t, _| {
                let s = t.battlefield(P0, "Syr Carah, the Bold");
                act(t, P0, s, "1 damage", &[Entity::Player(P1)]).expect("activate");
            },
            kinds: BOTH,
        },
        Case {
            card: "Tectonic Giant",
            cite: || {
                ruling!(
                    "Tectonic Giant",
                    "Tectonic Giant doesn't change when you can play the exiled card. For example, if you exile a sorcery card, you can cast it only during your main phase when the stack is empty. If you exile a land card, you can play it only during your main phase and only if you have an available land play remaining."
                );
            },
            setup: |t, top| {
                let g = t.battlefield(P0, "Tectonic Giant");
                choose_modes(t, P0, &[1]);
                t.answer_choose_moved(P0, &[obj(top)]);
                attack_only(t, &[g]);
            },
            kinds: BOTH,
        },
        Case {
            card: "Raphael, Most Attitude",
            cite: || {
                ruling!(
                    "Raphael, Most Attitude",
                    "To play a card using the permission granted by Raphael's last ability, you must still follow any timing restrictions and permissions, including those based on the card's type. For instance, you can play a land from exile this way only during your main phase while the stack is empty and only if you have an available land play remaining for the turn."
                );
            },
            setup: |t, _| {
                let r = t.battlefield(P0, "Raphael, Most Attitude");
                t.answer_yes(P0, true);
                t.enter(P0, "Memnite");
                t.resolve_all();
                attack_only(t, &[r]);
            },
            kinds: LAND,
        },
        Case {
            card: "Laelia, the Blade Reforged",
            cite: || {
                ruling!(
                    "Laelia, the Blade Reforged",
                    "You must follow the normal timing permissions and restrictions while playing the exiled card. If it's a land, you can't play it unless you have land plays available."
                );
            },
            setup: |t, _| {
                let l = t.battlefield(P0, "Laelia, the Blade Reforged");
                attack_only(t, &[l]);
            },
            kinds: LAND,
        },
        Case {
            card: "Capricious Sliver",
            cite: || {
                ruling!(
                    "Capricious Sliver",
                    "You must pay all costs and follow all normal timing rules for cards played this way. For example, you may only play a land from exile this way during your main phase while the stack is empty, and only if you haven't played a land yet this turn."
                );
            },
            setup: |t, _| {
                let s = t.battlefield(P0, "Capricious Sliver");
                attack_through(t, &[s]);
            },
            kinds: BOTH,
        },
        Case {
            card: "Endrider Spikespitter",
            cite: || {
                ruling!(
                    "Endrider Spikespitter",
                    "You must pay all costs and follow all normal timing rules for cards you play using this creature’s last ability."
                );
            },
            setup: |t, _| {
                t.battlefield(P0, "Endrider Spikespitter");
                set_speed(t, P0, 4);
                next_upkeep(t);
            },
            kinds: BOTH,
        },
        Case {
            card: "Tempered in Solitude",
            cite: || {
                ruling!(
                    "Tempered in Solitude",
                    "You must pay all costs and follow all normal timing rules to play a card this way. For example, you may play a land this way only during your main phase while the stack is empty, and only if you haven't yet played a land this turn."
                );
            },
            setup: |t, _| {
                t.battlefield(P0, "Tempered in Solitude");
                let b = t.battlefield(P0, "Grizzly Bears");
                attack_only(t, &[b]);
            },
            kinds: BOTH,
        },
        Case {
            card: "Feldon, Ronom Excavator",
            cite: || {
                ruling!(
                    "Feldon, Ronom Excavator",
                    "You must pay all costs and follow all normal timing rules when casting a spell this way."
                );
            },
            setup: |t, top| {
                let f = t.battlefield(P0, "Feldon, Ronom Excavator");
                t.answer_choose_moved(P0, &[obj(top)]);
                ping(t, obj(f));
            },
            kinds: SORCERY,
        },
        Case {
            card: "Warehouse Thief",
            cite: || {
                ruling!(
                    "Warehouse Thief",
                    "You must pay all costs and follow all timing rules when playing a land or casting a spell this way. For example, you may only play a land this way during your main phase while the stack is empty, and only if you haven't previously played a land this turn."
                );
            },
            setup: |t, _| {
                let w = t.battlefield(P0, "Warehouse Thief");
                let o = t.battlefield(P0, "Ornithopter");
                mana(t, P0, ManaType::C, 2);
                t.answer_choose(P0, &[obj(o)]);
                act(t, P0, w, "Exile the top", &[]).expect("activate");
            },
            kinds: BOTH,
        },
        Case {
            card: "Kami of Celebration",
            cite: || {
                ruling!(
                    "Kami of Celebration",
                    "You must still pay all costs and follow all normal timing rules to play a card this way. For example, you may play a land this way only during your own main phase while the stack is empty, and only if you have not yet played a land this turn."
                );
            },
            setup: |t, _| {
                t.battlefield(P0, "Kami of Celebration");
                let b = t.battlefield(P0, "Grizzly Bears");
                t.g.add_counters(obj(b), counters::PLUS1, 1, None);
                attack_only(t, &[b]);
            },
            kinds: BOTH,
        },
        Case {
            card: "Geistflame Reservoir",
            cite: || {
                ruling!(
                    "Geistflame Reservoir",
                    "You must still pay all costs and follow all timing rules to play a card exiled with Geistflame Reservoir's last ability."
                );
            },
            setup: |t, _| {
                let g = t.battlefield(P0, "Geistflame Reservoir");
                mana(t, P0, ManaType::R, 2);
                act(t, P0, g, "Exile the top", &[]).expect("activate");
            },
            kinds: BOTH,
        },
        Case {
            card: "Embrace the Unknown",
            cite: || {
                ruling!(
                    "Embrace the Unknown",
                    "You pay all costs and follow all normal timing rules for cards played from exile this way. For example, if an exiled card is a land card, you may play it only during your main phase while the stack is empty."
                );
            },
            setup: |t, _| {
                let e = t.hand(P0, "Embrace the Unknown");
                mana(t, P0, ManaType::R, 3);
                t.cast(P0, e).go();
            },
            kinds: BOTH,
        },
        Case {
            card: "Gila Courser",
            cite: || {
                ruling!(
                    "Gila Courser",
                    "You pay all costs and follow all normal timing rules for cards played from exile with Gila Courser’s first ability. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
                );
            },
            setup: |t, _| {
                let g = t.battlefield(P0, "Gila Courser");
                let m = t.battlefield(P0, "Grizzly Bears");
                t.answer_choose(P0, &[obj(m)]);
                act(t, P0, g, "Saddle", &[]).expect("saddle");
                t.resolve_all();
                attack_only(t, &[g]);
            },
            kinds: BOTH,
        },
        Case {
            card: "Riku of Many Paths",
            cite: || {
                ruling!(
                    "Riku of Many Paths",
                    "You pay all costs and follow all normal timing rules for cards played from exile with Riku's triggered ability. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
                );
            },
            setup: |t, _| {
                t.battlefield(P0, "Riku of Many Paths");
                // Izzet Charm, third mode (draw two, then discard two): one mode chosen.
                let c = t.hand(P0, "Izzet Charm");
                mana(t, P0, ManaType::U, 1);
                mana(t, P0, ManaType::R, 1);
                t.cast(P0, c).modes(&[2]).go();
                choose_modes(t, P0, &[0]);
                t.resolve();
            },
            kinds: BOTH,
        },
        Case {
            card: "Stella Lee, Wild Card",
            cite: || {
                ruling!(
                    "Stella Lee, Wild Card",
                    "You pay all costs and follow all normal timing rules for cards played from exile with Stella Lee’s first ability. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
                );
            },
            setup: |t, _| {
                t.battlefield(P0, "Stella Lee, Wild Card");
                for _ in 0..2 {
                    let o = t.hand(P0, "Ornithopter");
                    t.cast(P0, o).go();
                    t.resolve_all();
                }
            },
            kinds: BOTH,
        },
        Case {
            card: "Wild Wasteland",
            cite: || {
                ruling!(
                    "Wild Wasteland",
                    "You pay all costs and follow all normal timing rules for cards played from exile with Wild Wasteland’s ability. For example, if one of the exiled cards is a land card, you may play it only during your main phase while the stack is empty."
                );
            },
            setup: |t, _| {
                t.battlefield(P0, "Wild Wasteland");
                next_upkeep(t);
            },
            kinds: BOTH,
        },
        Case {
            card: "Unstable Amulet",
            cite: || {
                ruling!(
                    "Unstable Amulet",
                    "You pay all costs and follow all normal timing rules for cards played with Unstable Amulet's last ability. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
                );
            },
            setup: |t, _| {
                let a = t.battlefield(P0, "Unstable Amulet");
                t.g.add_counters(Entity::Player(P0), counters::ENERGY, 2, None);
                act(t, P0, a, "Exile the top", &[]).expect("activate");
            },
            kinds: BOTH,
        },
        Case {
            card: "Redcap Gutter-Dweller",
            cite: || {
                ruling!(
                    "Redcap Gutter-Dweller",
                    "You pay all costs and follow all normal timing rules for cards played with the permission granted by Redcap Gutter-Dweller's last ability. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
                );
            },
            setup: |t, _| {
                t.battlefield(P0, "Redcap Gutter-Dweller");
                let m = t.battlefield(P0, "Memnite");
                t.answer_yes(P0, true);
                t.answer_choose(P0, &[obj(m)]);
                next_upkeep(t);
            },
            kinds: BOTH,
        },
        Case {
            card: "Bruse Tarl, Roving Rancher",
            cite: || {
                ruling!(
                    "Bruse Tarl, Roving Rancher",
                    "You pay all costs and follow all normal timing rules for spells cast from exile with Bruse Tarl's last ability. For example, if you exile a creature card this way, you must wait until your main phase to cast it."
                );
            },
            setup: |t, _| {
                t.enter(P0, "Bruse Tarl, Roving Rancher");
            },
            kinds: CREATURE,
        },
        Case {
            card: "Synth Eradicator",
            cite: || {
                ruling!(
                    "Synth Eradicator",
                    "You pay all costs and follow all normal timing rules for the card played from exile with Synth Eradicator's second ability. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
                );
            },
            setup: |t, _| {
                let s = t.battlefield(P0, "Synth Eradicator");
                t.answer_yes(P0, false);
                attack_only(t, &[s]);
            },
            kinds: BOTH,
        },
        Case {
            card: "Count on Luck",
            cite: || {
                ruling!(
                    "Count on Luck",
                    "You pay all costs and follow all normal timing rules for the card played from exile with this permanent’s ability. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
                );
            },
            setup: |t, _| {
                t.battlefield(P0, "Count on Luck");
                next_upkeep(t);
            },
            kinds: BOTH,
        },
        Case {
            card: "Charred Foyer // Warped Space",
            cite: || {
                ruling!(
                    "Charred Foyer // Warped Space",
                    "You pay all costs and follow all timing rules for cards played with the permission granted by Charred Foyer's ability. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
                );
            },
            setup: |t, _| {
                let r = t.battlefield(P0, "Charred Foyer // Warped Space");
                mtg_engine::rooms::unlock(&mut t.g, r, 0, P0);
                next_upkeep(t);
            },
            kinds: BOTH,
        },
        Case {
            card: "Cleon, Merry Champion",
            cite: || {
                ruling!(
                    "Cleon, Merry Champion",
                    "You pay all costs and follow all timing rules for cards played with the permission granted by Cleon's last ability. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
                );
            },
            setup: |t, _| {
                let c = t.battlefield(P0, "Cleon, Merry Champion");
                let gg = t.hand(P0, "Giant Growth");
                mana(t, P0, ManaType::G, 1);
                t.cast(P0, gg).target(c).go();
            },
            kinds: BOTH,
        },
        Case {
            card: "Possibility Technician",
            cite: || {
                ruling!(
                    "Possibility Technician",
                    "You pay all costs and follow all timing rules for cards played with the permission granted by Possibility Technician’s first ability. For example, if an exiled card is a land card, you may play it only during your main phase while the stack is empty."
                );
            },
            setup: |t, _| {
                t.enter(P0, "Possibility Technician");
            },
            kinds: BOTH,
        },
        Case {
            card: "Charforger",
            cite: || {
                ruling!(
                    "Charforger",
                    "You must follow all normal timing rules for a card you play using Charforger's last ability and, if it's a spell, you must pay its costs to cast it."
                );
            },
            setup: |t, _| {
                let c = t.battlefield(P0, "Charforger");
                t.g.add_counters(obj(c), "oil", 3, None);
                act(t, P0, c, "Exile the top", &[]).expect("activate");
            },
            kinds: BOTH,
        },
        Case {
            card: "Sword of Forge and Frontier",
            cite: || {
                ruling!(
                    "Sword of Forge and Frontier",
                    "You must follow all normal timing rules for a card you play using Sword of Forge and Frontier's second ability and, if it's a spell, you must pay its costs to cast it."
                );
            },
            setup: |t, _| {
                let b = t.battlefield(P0, "Grizzly Bears");
                attach_new(t, P0, "Sword of Forge and Frontier", obj(b));
                attack_through(t, &[b]);
            },
            kinds: SORCERY,
        },
    ];
    run_cases(&cases);
}
