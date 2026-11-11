//! Rulings batch P144 — repeatable impulse draw: Hellcarver Demon's casting during
//! resolution, values computed on resolution, costs, modes, and the other one-offs of
//! the batch.

use crate::r_p107_common::{act, mana};
use crate::r_p144_common::*;
use crate::r_s01_common::with_subtype;
use crate::r_s02_common::{can_play_land, destroy};
use crate::r_s09_common::declare;
use crate::r_s29_common::choose_modes;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn exiled(t: &TestGame, id: ObjectId) -> bool {
    t.zone(id) == Zone::Exile
}

/// P0 attacks P1 with `attackers`, and the game runs to the combat damage step: damage is
/// dealt and its triggers are put on the stack (not resolved).
fn to_combat_damage(t: &mut TestGame, attackers: &[ObjectId]) {
    let decl: Vec<(ObjectId, Entity)> =
        attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    declare(t, P0, &decl);
    t.resolve_all();
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
}

#[test]
fn hellcarver_demon_is_mandatory_and_exiles_face_up() {
    cr!("603.5", "406.3", "701.21a");
    supported("Hellcarver Demon");
    ruling!(
        "Hellcarver Demon",
        "The triggered ability is mandatory. All the permanents you control, including all lands, will be sacrificed."
    );
    ruling!(
        "Hellcarver Demon",
        "The cards from your library are exiled face up."
    );
    let mut t = TestGame::new(2);
    let lib = on_top(
        &mut t,
        P0,
        &[
            "Island", "Island", "Island", "Island", "Island", "Island", "Wastes",
        ],
    );
    let h = t.battlefield(P0, "Hellcarver Demon");
    let others = [
        t.battlefield(P0, "Mountain"),
        t.battlefield(P0, "Swamp"),
        t.battlefield(P0, "Memnite"),
        t.battlefield(P0, "Oracle's Vault"),
    ];
    t.hand(P0, "Lightning Bolt");
    to_combat_damage(&mut t, &[h]);
    let from = t.asked().len();
    t.resolve_all();
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::YesNo { .. })));
    assert!(others.iter().all(|o| !t.on_battlefield(*o)));
    assert!(t.on_battlefield(h));
    assert_eq!(t.hand_size(P0), 0);
    assert!(lib[..6]
        .iter()
        .all(|c| exiled(&t, *c) && !t.obj_now(*c).face_down));
}

#[test]
fn hellcarver_demon_casts_during_resolution_ignoring_only_type_timing() {
    cr!("608.2g", "601.2", "405.2", "118.9");
    supported("Hellcarver Demon");
    supported("Gorilla War Cry");
    ruling!(
        "Hellcarver Demon",
        "You cast nonland cards from exile as part of the resolution of Hellcarver Demon’s ability. You may cast those cards in any order. Timing restrictions based on the card’s type (such as creature or sorcery) are ignored. Other restrictions are not (such as “Cast [this card] only before the combat damage step”). You cast all of the cards you like, putting them onto the stack, then Hellcarver Demon’s ability finishes resolving. The spells you cast this way will then resolve as normal, one at a time, in the opposite order that they were put on the stack."
    );
    // A creature and a sorcery are cast in the combat damage step; Gorilla War Cry ("Cast
    // this spell only during combat before blockers are declared") can't be.
    let mut t = TestGame::new(2);
    let lib = on_top(
        &mut t,
        P0,
        &[
            "Grizzly Bears",
            "Lava Spike",
            "Gorilla War Cry",
            "Island",
            "Island",
            "Island",
            "Wastes",
        ],
    );
    let h = t.battlefield(P0, "Hellcarver Demon");
    to_combat_damage(&mut t, &[h]);
    assert_eq!(t.life(P1), 14);
    // Cast the Lava Spike first, then the Bears. Gorilla War Cry isn't offered.
    t.answer_choose_moved(P0, &[obj(lib[0]), obj(lib[1])]);
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let from = t.asked().len();
    t.resolve();
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 1);
    assert!(!offered[0].contains(&obj(t.g.current(lib[2]))));
    // The ability has finished; both spells are on the stack, the Bears on top.
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.zone(lib[0]), Zone::Stack);
    assert_eq!(t.zone(lib[1]), Zone::Stack);
    assert_eq!(*t.g.stack.last().unwrap(), t.g.current(lib[0]));
    assert!(exiled(&t, lib[2]));
    t.resolve();
    assert!(t.on_battlefield(lib[0]));
    assert_eq!(t.life(P1), 14);
    t.resolve();
    assert_eq!(t.life(P1), 11);
}

#[test]
fn hellcarver_demon_free_casts_may_pay_kicker_and_must_pay_mandatory_costs() {
    cr!("118.9", "118.8", "702.33a");
    supported("Hellcarver Demon");
    ruling!(
        "Hellcarver Demon",
        "If you cast a card “without paying its mana cost,” you can’t pay any alternative costs. On the other hand, if the card has optional additional costs (such as kicker or multikicker), you may pay those when you cast the card. If the card has mandatory additional costs (such as Momentous Fall does), you must pay those when you cast the card."
    );
    // Burst Lightning kicked ({4}, from the pool): 4 damage. Momentous Fall: Hellcarver
    // Demon (the only creature left) is sacrificed: draw 6, gain 6.
    let mut t = TestGame::new(2);
    let lib = on_top(
        &mut t,
        P0,
        &[
            "Burst Lightning",
            "Momentous Fall",
            "Island",
            "Island",
            "Island",
            "Island",
        ],
    );
    let h = t.battlefield(P0, "Hellcarver Demon");
    to_combat_damage(&mut t, &[h]);
    mana(&mut t, P0, ManaType::C, 4);
    t.answer_choose_moved(P0, &[obj(lib[0]), obj(lib[1])]);
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![0, 1]));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[obj(h)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 6 - 4);
    assert_eq!(t.zone(h), Zone::Graveyard(P0));
    assert_eq!(t.life(P0), 26);
    assert_eq!(t.hand_size(P0), 6);
}

#[test]
fn two_hellcarver_demons_resolve_one_after_the_other() {
    cr!("603.3b", "608.2g", "113.7a");
    supported("Hellcarver Demon");
    ruling!(
        "Hellcarver Demon",
        "If you control two Hellcarver Demons that deal combat damage to a player at the same time, each Demon’s ability triggers. When the first one resolves, you’ll sacrifice all permanents except that Hellcarver Demon (meaning you’ll sacrifice the other one). Then you’ll exile six cards from your library and may cast nonland cards exiled that way. After those spells resolve, the other Demon’s ability resolves. You’ll sacrifice all permanents you now control (including the first Demon and any permanents put on the battlefield as a result of the spells you just cast), exile six more cards from your library, and cast any number of them."
    );
    let mut t = TestGame::new(2);
    let lib = on_top(
        &mut t,
        P0,
        &[
            "Memnite",
            "Island",
            "Island",
            "Island",
            "Island",
            "Island",
            "Ornithopter",
            "Island",
            "Island",
            "Island",
            "Island",
            "Island",
            "Wastes",
        ],
    );
    let d = [
        t.battlefield(P0, "Hellcarver Demon"),
        t.battlefield(P0, "Hellcarver Demon"),
    ];
    to_combat_damage(&mut t, &[d[0], d[1]]);
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.life(P1), 8);
    // The first trigger: the other Demon is sacrificed; Memnite is cast and resolves.
    t.answer_choose_moved(P0, &[obj(lib[0])]);
    t.resolve();
    t.resolve();
    assert!(t.on_battlefield(lib[0]));
    let survivors: Vec<ObjectId> = d.iter().copied().filter(|x| t.on_battlefield(*x)).collect();
    assert_eq!(survivors.len(), 1);
    // The second: the first Demon and the Memnite are sacrificed; six more exiled, and
    // the Ornithopter among them is cast.
    t.answer_choose_moved(P0, &[obj(lib[6])]);
    t.resolve_all();
    assert!(d.iter().all(|x| t.zone(*x) == Zone::Graveyard(P0)));
    assert_eq!(t.zone(lib[0]), Zone::Graveyard(P0));
    assert!(t.on_battlefield(lib[6]));
    assert!(lib[7..12].iter().all(|c| exiled(&t, *c)));
    assert!(!exiled(&t, lib[12]));
}

#[test]
fn values_counted_as_the_ability_resolves() {
    cr!("608.2h", "608.2b");
    supported("Sanar, Innovative First-Year");
    ruling!(
        "Sanar, Innovative First-Year",
        "The value of X is calculated only once, as Sanar's ability resolves."
    );
    ruling!(
        "Sanar, Innovative First-Year",
        "You pay all costs and follow all timing rules for cards cast this way. For example, if an exiled card is a sorcery card, you may cast it only during your main phase while the stack is empty."
    );
    // Sanar (blue and red) and a green permanent: X = 3 as it triggers. Before it
    // resolves, the green permanent leaves: X = 2. It reveals until two nonland cards.
    let mut t = TestGame::new(2);
    t.advance_to(P1, Step::Upkeep);
    t.battlefield(P0, "Sanar, Innovative First-Year");
    let g = t.battlefield(P0, "Grizzly Bears");
    // (The Wastes on top is drawn in P0's draw step.)
    let lib = on_top(
        &mut t,
        P0,
        &[
            "Wastes",
            "Lava Spike",
            "Island",
            "Opt",
            "Giant Growth",
            "Wastes",
        ],
    );
    let lib = &lib[1..];
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, g);
    // A red card (Lava Spike) and a blue one (Opt).
    t.answer_choose_moved(P0, &[obj(lib[0]), obj(lib[2])]);
    t.resolve_all();
    assert!(exiled(&t, lib[0]) && exiled(&t, lib[2]));
    // Giant Growth (a third nonland card) wasn't revealed, so it stays in the library.
    assert_eq!(t.zone(lib[3]), Zone::Library(P0));
    // The sorcery: not during combat, yes in a main phase with {R}.
    crate::r_p144_common::check_cast_timing(&mut t, lib[0], ManaType::R, 1);
}

#[test]
fn witch_king_uses_power_on_resolution_or_last_known() {
    cr!("608.2h", "608.2b", "113.7a");
    supported("Witch-king, Sky Scourge");
    ruling!(
        "Witch-king, Sky Scourge",
        "Use the power of the Wraiths as the triggered ability resolves to determine the value of X. If one or more of those Wraiths are no longer on the battlefield at that time, use their power from when they were last on the battlefield."
    );
    // Witch-king (5/5) and Witch-king of Angmar (5/3), both Wraiths, attack: X = 10. Before
    // the trigger resolves, the Angmar gets +3/+0 (Giant Growth gives +3/+3) and then
    // dies: its last known power, 8, counts. X = 13.
    let mut t = TestGame::new(2);
    let names: Vec<&str> = std::iter::repeat_n("Island", 14).collect();
    let lib = on_top(&mut t, P0, &names);
    let w = t.battlefield(P0, "Witch-king, Sky Scourge");
    let a = t.battlefield(P0, "Witch-king of Angmar");
    declare(
        &mut t,
        P0,
        &[(w, Entity::Player(P1)), (a, Entity::Player(P1))],
    );
    t.settle();
    let gg = t.hand(P0, "Giant Growth");
    mana(&mut t, P0, ManaType::G, 1);
    t.cast(P0, gg).target(a).go();
    t.resolve();
    assert_eq!(t.pt(a), (8, 6));
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(lib.iter().filter(|c| exiled(&t, **c)).count(), 13);
}

#[test]
fn geistflame_reservoir_counters_are_removed_as_a_cost() {
    cr!("601.2b", "602.2b", "118.3");
    supported("Geistflame Reservoir");
    ruling!(
        "Geistflame Reservoir",
        "You choose how many counters to remove from Geistflame Reservoir as you activate its second ability, not as it resolves."
    );
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Geistflame Reservoir");
    t.g.add_counters(obj(g), "charge", 3, None);
    mana(&mut t, P0, ManaType::R, 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    act(&mut t, P0, g, "Remove any number", &[Entity::Player(P1)]).unwrap();
    // On the stack: the counters are already gone.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.counters(g, "charge"), 1);
    // More counters arriving before it resolves don't change it.
    t.g.add_counters(obj(g), "charge", 5, None);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn eruth_replacement_with_a_short_library() {
    cr!("614.1a", "121.3", "704.5b");
    supported("Eruth, Tormented Prophet");
    ruling!(
        "Eruth, Tormented Prophet",
        "You may choose to apply Eruth's replacement effect even if there are fewer than two cards in your library. If there are no cards in your library, you won't exile any cards this way, but you also won't lose the game for having attempted to draw from an empty library."
    );
    // One card: it's exiled. Then none: the draw is replaced anyway, and P0 doesn't lose.
    let mut t = TestGame::new(2);
    t.g.players[P0.idx()].library.clear();
    let lib = on_top(&mut t, P0, &["Island"]);
    t.battlefield(P0, "Eruth, Tormented Prophet");
    let _ = t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    assert!(exiled(&t, lib[0]));
    assert_eq!(t.hand_size(P0), 0);
    let _ = t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    assert!(!t.has_lost(P0));
}

#[test]
fn bruse_tarl_leaving_after_first_strike_damage_stops_regular_damage() {
    cr!("510.4", "702.4c", "611.3a");
    supported("Bruse Tarl, Roving Rancher");
    ruling!(
        "Bruse Tarl, Roving Rancher",
        "If Bruse Tarl leaves the battlefield after other Oxen you control have dealt first-strike damage but before regular combat damage, those Oxen won't deal regular combat damage (unless they still have double strike for some other reason) ."
    );
    // An Ox token attacks with double strike from Bruse Tarl; after the first-strike
    // damage step, Bruse Tarl dies: the Ox deals no regular damage.
    let mut t = TestGame::new(2);
    on_top(&mut t, P0, &["Island", "Wastes", "Wastes"]);
    let bt = t.enter(P0, "Bruse Tarl, Roving Rancher");
    t.resolve_all();
    let ox = with_subtype(&t, P0, "Ox")[0];
    t.g.objects[ox.0 as usize].summoning_sick = false;
    declare(&mut t, P0, &[(ox, Entity::Player(P1))]);
    t.resolve_all();
    t.advance_to(P0, Step::FirstStrikeDamage);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    destroy(&mut t, bt);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn sacrifice_costs_can_use_the_source_or_any_artifact() {
    cr!("602.2b", "118.3");
    supported("Dark-Dweller Oracle");
    ruling!(
        "Dark-Dweller Oracle",
        "You can sacrifice Dark-Dweller Oracle to pay the cost for its own ability."
    );
    let mut t = TestGame::new(2);
    let lib = on_top(&mut t, P0, &["Island", "Wastes"]);
    let d = t.battlefield(P0, "Dark-Dweller Oracle");
    mana(&mut t, P0, ManaType::C, 1);
    t.answer_choose(P0, &[obj(d)]);
    act(&mut t, P0, d, "Exile the top", &[]).unwrap();
    assert_eq!(t.zone(d), Zone::Graveyard(P0));
    t.resolve_all();
    assert!(exiled(&t, lib[0]));
    assert!(can_play_land(&mut t, P0, lib[0]));

    supported("Breya's Apprentice");
    ruling!(
        "Breya's Apprentice",
        "You can sacrifice any artifact, not just a Thopter."
    );
    ruling!("Breya's Apprentice", "The exiled card is exiled face up.");
    let mut t = TestGame::new(2);
    let lib = on_top(&mut t, P0, &["Island", "Wastes"]);
    let b = t.battlefield(P0, "Breya's Apprentice");
    let v = t.battlefield(P0, "Oracle's Vault");
    choose_modes(&mut t, P0, &[0]);
    t.answer_choose(P0, &[obj(v)]);
    act(&mut t, P0, b, "Sacrifice an artifact", &[]).unwrap();
    assert_eq!(t.zone(v), Zone::Graveyard(P0));
    t.resolve_all();
    assert!(exiled(&t, lib[0]) && !t.obj_now(lib[0]).face_down);
}

#[test]
fn riku_modes_are_each_chosen_at_most_once() {
    cr!("700.2", "700.2d");
    supported("Riku of Many Paths");
    ruling!(
        "Riku of Many Paths",
        "You can't choose the same mode of Riku's triggered ability more than once each time it triggers."
    );
    // Izzet Charm has one mode chosen: X = 1. A modal spell with two modes chosen (an
    // escalated Collective Defiance is too big; Izzet Charm is cast twice instead): the
    // trigger offers each of its three modes once, up to X of them.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Riku of Many Paths");
    let c = t.hand(P0, "Izzet Charm");
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, c).modes(&[2]).go();
    let from = t.asked().len();
    // Asking for the counter mode twice isn't allowed: one counter at most.
    choose_modes(&mut t, P0, &[1, 1]);
    t.settle();
    let offered: Vec<(u32, u32, bool)> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseModes {
                min,
                max,
                allow_repeat,
                ..
            } => Some((*min, *max, *allow_repeat)),
            _ => None,
        })
        .collect();
    assert_eq!(offered, vec![(0, 1, false)]);
    t.resolve_all();
    assert!(t.counters(r, counters::PLUS1) <= 1);
}

#[test]
fn ardent_dustspeaker_two_lands_need_two_land_plays() {
    cr!("305.2", "305.2a");
    supported("Ardent Dustspeaker");
    ruling!(
        "Ardent Dustspeaker",
        "Unless an effect allows you to play additional lands that turn, you can play land cards exiled with Ardent Dustspeaker only if you haven’t played a land yet that turn. In most cases, if you exile two land cards, you won’t be able to play both."
    );
    let mut t = TestGame::new(2);
    let lib = on_top(&mut t, P0, &["Island", "Swamp", "Wastes"]);
    let d = t.battlefield(P0, "Ardent Dustspeaker");
    t.graveyard(P0, "Shock");
    t.answer_yes(P0, true);
    declare(&mut t, P0, &[(d, Entity::Player(P1))]);
    t.resolve_all();
    assert!(exiled(&t, lib[0]) && exiled(&t, lib[1]));
    to_main(&mut t);
    let island = t.g.current(lib[0]);
    t.play_land(P0, island).unwrap();
    assert!(!can_play_land(&mut t, P0, lib[1]));
}

#[test]
fn undead_butler_return_trigger_targets_after_the_exile() {
    // This ruling is attached to Traveling Minister in the data, but it describes Undead
    // Butler ("When this creature dies, you may exile it. When you do, return target
    // creature card from your graveyard to your hand."). Tested on Undead Butler, cited
    // under the card it's attached to (docs/RULING_DISCREPANCIES.md).
    cr!("603.12", "115.1");
    supported("Undead Butler");
    supported("Traveling Minister");
    ruling!(
        "Traveling Minister",
        "After you exile Undead Butler from your graveyard, a second ability triggers. You choose the target creature card to return at that time."
    );
    // A creature card put into the graveyard after Undead Butler's death trigger was put
    // on the stack (but before it resolves) can be the target: it's chosen only once the
    // reflexive trigger triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Traveling Minister");
    let b = t.battlefield(P0, "Undead Butler");
    destroy(&mut t, b);
    assert_eq!(t.stack_len(), 1);
    let late = t.graveyard(P0, "Craw Wurm");
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(late)]);
    t.resolve_all();
    assert!(exiled(&t, b));
    assert_eq!(t.zone(late), Zone::Hand(P0));
    assert!(t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
}
