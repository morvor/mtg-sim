//! Rulings batch P014 — power/toughness-defining abilities in play: damage that stays
//! marked while the count drops (CR 120.6, 704.5g), toughness checked only by
//! state-based actions (CR 704.3), spells that reach the graveyard before state-based
//! actions are checked (CR 608.2n), simultaneous deaths and entries, cast triggers that
//! resolve first, and the triggered and static abilities printed on these creatures.

use crate::r_p014_common::*;
use crate::r_p038_common::{cast_only, cast_resolve, enter, kill, mana_all, tokens_of};
use crate::r_s01_common::*;
use crate::r_s06_common::{activate_containing, damage};
use mtg_engine::events::Event;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn add(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

fn pool(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).mana_pool.total() as u32
}

// ---------------------------------------------------------------------------
// Marked damage and a shrinking count
// ---------------------------------------------------------------------------

#[test]
fn marked_damage_becomes_lethal_as_the_count_drops() {
    cr!("120.6", "704.5g", "604.3");
    ruling!(
        "Tishana, Voice of Thunder",
        "Because damage remains marked on a creature until it's removed as the turn ends, the damage Tishana takes during combat may become lethal if cards leave your hand later in the turn"
    );
    ruling!(
        "Crusader of Odric",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to Crusader of Odric may become lethal if other creatures you control leave the battlefield during that turn."
    );
    ruling!(
        "Master of Etherium",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to Master of Etherium may become lethal if other artifacts you control leave the battlefield during that turn, and damage dealt to other artifact creatures you control may become lethal if Master of Etherium leaves the battlefield during that turn."
    );
    supported("Tishana, Voice of Thunder");
    supported("Crusader of Odric");
    supported("Master of Etherium");
    // Tishana with three cards in hand, dealt 2 damage; a card is cast from hand.
    let mut t = TestGame::new(2);
    let src = t.battlefield(P1, "Hill Giant");
    let cards: Vec<ObjectId> = (0..3).map(|_| t.hand(P0, "Grizzly Bears")).collect();
    let tishana = t.battlefield(P0, "Tishana, Voice of Thunder");
    t.g.recompute();
    assert_eq!(t.pt(tishana), (3, 3));
    damage(&mut t, src, 2, tishana);
    assert!(t.on_battlefield(tishana));
    add(&mut t, P0, ManaType::G, 2);
    t.cast(P0, cards[0]).go();
    t.settle();
    assert!(t.in_graveyard(P0, "Tishana, Voice of Thunder"));
    // Crusader of Odric and another creature, dealt 1 damage; the other creature dies.
    let mut t = TestGame::new(2);
    let src = t.battlefield(P1, "Hill Giant");
    let crusader = t.battlefield(P0, "Crusader of Odric");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    damage(&mut t, src, 1, crusader);
    assert!(t.on_battlefield(crusader));
    kill(&mut t, bears);
    assert!(t.in_graveyard(P0, "Crusader of Odric"));
    // Master of Etherium with Sol Ring and Ornithopter (a 3/3; Ornithopter 1/3).
    let mut t = TestGame::new(2);
    let src = t.battlefield(P1, "Hill Giant");
    let master = t.battlefield(P0, "Master of Etherium");
    let ring = t.battlefield(P0, "Sol Ring");
    let thopter = t.battlefield(P0, "Ornithopter");
    t.g.recompute();
    assert_eq!((t.pt(master), t.pt(thopter)), ((3, 3), (1, 3)));
    damage(&mut t, src, 2, master);
    damage(&mut t, src, 2, thopter);
    assert!(t.on_battlefield(master) && t.on_battlefield(thopter));
    kill(&mut t, ring);
    assert!(t.in_graveyard(P0, "Master of Etherium"));
    assert!(
        t.in_graveyard(P0, "Ornithopter"),
        "Master left: Ornithopter is 0/2"
    );
}

#[test]
fn body_of_knowledge_draws_for_all_damage_but_dies_first() {
    cr!("120.3e", "704.5g", "603.2");
    ruling!(
        "Body of Knowledge",
        "Creatures may be dealt damage greater than their toughness. For example, if Body of Knowledge is a 3/3 creature and a source deals 5 damage to it, you'll draw five cards."
    );
    supported("Body of Knowledge");
    let mut t = TestGame::new(2);
    let src = t.battlefield(P1, "Hill Giant");
    for _ in 0..3 {
        t.hand(P0, "Grizzly Bears");
    }
    let body = t.battlefield(P0, "Body of Knowledge");
    t.g.recompute();
    assert_eq!(t.pt(body), (3, 3));
    damage(&mut t, src, 5, body);
    // It died before the trigger resolved.
    assert!(t.in_graveyard(P0, "Body of Knowledge"));
    assert_eq!(triggers_on_stack(&t, "draw that many"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 8);
}

#[test]
fn maro_survives_a_toughness_of_zero_during_a_resolution() {
    cr!("704.3", "704.5f", "608.2");
    ruling!(
        "Maro",
        "Creatures are only put into a graveyard for having 0 toughness when state-based actions are checked."
    );
    supported("Maro");
    // Wheel of Fortune: each player discards their hand, then draws seven cards.
    let mut t = TestGame::new(2);
    let maro = t.battlefield(P0, "Maro");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Wheel of Fortune", &[]);
    assert!(t.on_battlefield(maro));
    assert_eq!(t.pt(maro), (7, 7));
}

#[test]
fn tishana_entering_with_an_empty_hand_dies_before_its_trigger_resolves() {
    cr!("704.5f", "603.3", "117.5");
    ruling!(
        "Tishana, Voice of Thunder",
        "If Tishana enters the battlefield while you have no cards in hand, it will be put into your graveyard for having 0 toughness before its triggered ability resolves."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    enter(&mut t, P0, "Tishana, Voice of Thunder");
    assert!(t.in_graveyard(P0, "Tishana, Voice of Thunder"));
    assert_eq!(triggers_on_stack(&t, "draw a card for each creature"), 1);
    t.resolve_all();
    // It counts the creatures as it resolves: just the Bears.
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn nightmare_with_no_swamps_dies_to_state_based_actions() {
    cr!("704.5f", "604.3");
    ruling!(
        "Nightmare",
        "If you control 0 swamps, then the Nightmare has 0 toughness and will be put into its owner’s graveyard as a state-based action right before the next player gains priority."
    );
    supported("Nightmare");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Swamp");
    let n = enter(&mut t, P0, "Nightmare");
    assert_eq!(t.zone(n), Zone::Graveyard(P0));
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Swamp");
    let n = enter(&mut t, P0, "Nightmare");
    assert!(t.on_battlefield(n));
}

#[test]
fn scourge_of_the_skyclaves_can_have_negative_power_and_toughness() {
    cr!("604.3", "107.1b");
    ruling!(
        "Scourge of the Skyclaves",
        "If the highest life total among players is greater than 20, Scourge of the Skyclaves has negative power and toughness equal to the difference."
    );
    supported("Scourge of the Skyclaves");
    let mut t = TestGame::new(2);
    set_life(&mut t, P1, 25);
    let s = t.hand(P0, "Scourge of the Skyclaves");
    t.g.recompute();
    assert_eq!(t.obj_now(s).chars.power, Some(-5));
    assert_eq!(t.obj_now(s).chars.toughness, Some(-5));
}

#[test]
fn sewer_nemesis_becomes_zero_zero_when_the_chosen_player_leaves() {
    cr!("800.4a", "704.5f");
    ruling!(
        "Sewer Nemesis",
        "If the chosen player leaves the game, Sewer Nemesis's power and toughness each become 0."
    );
    supported("Sewer Nemesis");
    let mut t = TestGame::new(3);
    for _ in 0..3 {
        t.graveyard(P2, "Grizzly Bears");
    }
    t.answer_choose(P0, &[pl(P2)]);
    let n = enter(&mut t, P0, "Sewer Nemesis");
    assert_eq!(t.pt(n), (3, 3));
    t.g.lose_game(P2);
    t.settle();
    assert!(t.in_graveyard(P0, "Sewer Nemesis"));
}

// ---------------------------------------------------------------------------
// The spell is in the graveyard before state-based actions
// ---------------------------------------------------------------------------

#[test]
fn an_instant_reaches_the_graveyard_before_the_toughness_check() {
    cr!("608.2n", "704.3", "704.5g");
    ruling!(
        "Tarmogoyf",
        "If an instant or sorcery spell deals damage to Tarmogoyf or lowers its toughness, that spell is put into its owner's graveyard before state-based actions are performed."
    );
    ruling!(
        "Consuming Blob",
        "If an instant or sorcery spell deals damage to Consuming Blob or reduces its toughness to 0, that spell is put into its owner's graveyard before state-based actions are performed."
    );
    ruling!(
        "Lord of Extinction",
        "If an instant or sorcery spell deals damage to Lord of Extinction or lowers its toughness, that spell is put into its owner's graveyard before state-based actions are performed."
    );
    // Tarmogoyf (1/2 with a creature card in a graveyard), Shocked by P1.
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Grizzly Bears");
    let goyf = t.battlefield(P0, "Tarmogoyf");
    t.g.turn.priority = Some(P1);
    cast_resolve(&mut t, P1, "Shock", &[obj(goyf)]);
    assert!(t.on_battlefield(goyf));
    assert_eq!(t.pt(goyf), (2, 3));
    // Consuming Blob counts only its controller's graveyard: P0's own Shock saves it, an
    // opponent's doesn't.
    for (caster, survives) in [(P0, true), (P1, false)] {
        let mut t = TestGame::new(2);
        t.graveyard(P0, "Grizzly Bears");
        let blob = t.battlefield(P0, "Consuming Blob");
        t.g.turn.priority = Some(caster);
        cast_resolve(&mut t, caster, "Shock", &[obj(blob)]);
        assert_eq!(t.on_battlefield(blob), survives, "{caster:?}");
    }
    // Lord of Extinction (2/2 with two cards in graveyards).
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Forest");
    let lord = t.battlefield(P0, "Lord of Extinction");
    cast_resolve(&mut t, P1, "Shock", &[obj(lord)]);
    assert!(t.on_battlefield(lord));
    assert_eq!(t.pt(lord), (3, 3));
}

// ---------------------------------------------------------------------------
// Simultaneity
// ---------------------------------------------------------------------------

#[test]
fn revenant_and_another_creature_dealt_lethal_damage_die_together() {
    cr!("704.3", "704.5g");
    ruling!(
        "Revenant",
        "If Revenant is dealt lethal damage at the same time as another creature you control, both die at the same time."
    );
    supported("Revenant");
    supported("Pestilence");
    // Revenant is 1/1 (one creature card in the graveyard); Pestilence deals 1 damage to
    // each creature: Llanowar Elves dying first would have made it 2/2.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    let rev = t.battlefield(P0, "Revenant");
    t.battlefield(P0, "Llanowar Elves");
    let pest = t.battlefield(P1, "Pestilence");
    t.g.recompute();
    assert_eq!(t.pt(rev), (1, 1));
    add(&mut t, P1, ManaType::B, 1);
    activate_containing(&mut t, P1, pest, "deals 1 damage").unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Revenant"));
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
}

#[test]
fn entering_at_the_same_time_isnt_affected_by_as_enters_or_enters_with_effects() {
    cr!("614.12", "614.1c", "603.6a");
    ruling!(
        "Renata, Called to the Hunt",
        "A creature entering the battlefield at the same time as Renata won't get an additional +1/+1 counter."
    );
    ruling!(
        "Ixidron",
        "If Ixidron and another creature are entering at the same time, the other creature enters face up."
    );
    supported("Renata, Called to the Hunt");
    supported("Living Death");
    // Living Death returns both creature cards at once.
    for lord in ["Renata, Called to the Hunt", "Ixidron"] {
        let mut t = TestGame::new(2);
        t.graveyard(P0, lord);
        t.graveyard(P0, "Grizzly Bears");
        cast_resolve(&mut t, P0, "Living Death", &[]);
        let bears = t.named_on_battlefield("Grizzly Bears");
        assert_eq!(bears.len(), 1, "{lord}");
        assert_eq!(t.counters(bears[0], counters::PLUS1), 0, "{lord}");
        assert!(!t.obj_now(bears[0]).face_down, "{lord}");
    }
}

#[test]
fn renata_gives_a_counter_to_creatures_that_would_enter_without_any() {
    cr!("614.1c", "122.6");
    ruling!(
        "Renata, Called to the Hunt",
        "If a creature you control would normally enter the battlefield with no +1/+1 counters on it, Renata's second ability causes it to enter the battlefield with a +1/+1 counter on it."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Renata, Called to the Hunt");
    let b = enter(&mut t, P0, "Grizzly Bears");
    assert_eq!(t.counters(b, counters::PLUS1), 1);
    let theirs = enter(&mut t, P1, "Grizzly Bears");
    assert_eq!(t.counters(theirs, counters::PLUS1), 0);
}

#[test]
fn a_copy_of_a_face_down_creature_is_a_nameless_two_two() {
    cr!("708.2", "707.2");
    ruling!(
        "Ixidron",
        "Creatures turned face down by Ixidron are 2/2 creatures with no text, no name, no subtypes, no expansion symbol, and no mana cost. These values are copiable"
    );
    supported("Clone");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    enter(&mut t, P0, "Ixidron");
    let fd = t.g.current(giant);
    assert!(t.g.obj(fd).face_down);
    t.answer_choose(P1, &[obj(fd)]);
    let clone = t.enter(P1, "Clone");
    t.g.flush_events();
    t.settle();
    let c = t.obj_now(clone);
    assert!(!c.face_down);
    assert_eq!(c.chars.name, "");
    assert_eq!((c.power(), c.toughness()), (2, 2));
    assert!(c.chars.subtypes.is_empty());
    assert!(c.chars.abilities.is_empty());
    assert!(c.chars.mana_cost.is_none() || c.chars.mana_value() == 0);
}

// ---------------------------------------------------------------------------
// Cast triggers and costs
// ---------------------------------------------------------------------------

#[test]
fn scourge_of_the_skyclaves_cast_trigger_resolves_before_it() {
    cr!("603.3", "601.2i", "702.33d");
    ruling!(
        "Scourge of the Skyclaves",
        "An ability that triggers when a player casts a spell resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    let mut t = TestGame::new(2);
    mana_all(&mut t, P0, 2);
    let s = t.hand(P0, "Scourge of the Skyclaves");
    let spell = t.cast(P0, s).kicked(true).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.obj_now(spell).chars.toughness, Some(0));
    t.resolve();
    assert_eq!((t.life(P0), t.life(P1)), (10, 10));
    assert_eq!(t.obj_now(spell).chars.toughness, Some(10));
    t.resolve_all();
    let s = t.named_on_battlefield("Scourge of the Skyclaves");
    assert_eq!(s.len(), 1);
    assert_eq!(t.pt(s[0]), (10, 10));
    // Countered in response: the life loss still happens.
    let mut t = TestGame::new(2);
    mana_all(&mut t, P0, 2);
    let s = t.hand(P0, "Scourge of the Skyclaves");
    let spell = t.cast(P0, s).kicked(true).go();
    t.settle();
    // P1 counters the spell while the trigger is still on the stack.
    cast_only(&mut t, P1, "Cancel", &[obj(spell)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Scourge of the Skyclaves"));
    assert_eq!(t.life(P1), 20);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Scourge of the Skyclaves"));
    assert_eq!(t.life(P1), 10);
}

#[test]
fn callaphe_tax_is_per_permanent_targeted_and_one_instance_each() {
    cr!("601.2f", "115.3", "613.1f");
    ruling!(
        "Callaphe, Beloved of the Sea",
        "An opponent’s spell that targets a single creature or enchantment you control more than once costs only {1} more to cast."
    );
    ruling!(
        "Callaphe, Beloved of the Sea",
        "Callaphe’s last ability applies to itself as well as other creatures and enchantments you control."
    );
    ruling!(
        "Callaphe, Beloved of the Sea",
        "Callaphe’s last ability gives only one instance of the ability to creatures you control that are also enchantments."
    );
    supported("Callaphe, Beloved of the Sea");
    supported("Seeds of Strength");
    let setup = || {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, "Callaphe, Beloved of the Sea");
        let l = t.battlefield(P0, "Merfolk Looter");
        t.set_step(P1, Step::PrecombatMain);
        (t, c, l)
    };
    // Seeds of Strength targeting Callaphe three times: {G}{W} and {1}.
    let (mut t, c, _) = setup();
    add(&mut t, P1, ManaType::G, 1);
    add(&mut t, P1, ManaType::W, 1);
    let seeds = t.hand(P1, "Seeds of Strength");
    assert!(t
        .cast(P1, seeds)
        .target(obj(c))
        .target(obj(c))
        .target(obj(c))
        .try_go()
        .is_err());
    t.clear_answers();
    add(&mut t, P1, ManaType::C, 1);
    t.cast(P1, seeds)
        .target(obj(c))
        .target(obj(c))
        .target(obj(c))
        .go();
    assert_eq!(pool(&t, P1), 0);
    // Targeting Callaphe and Merfolk Looter: {2} more.
    let (mut t, c, l) = setup();
    add(&mut t, P1, ManaType::G, 1);
    add(&mut t, P1, ManaType::W, 1);
    add(&mut t, P1, ManaType::C, 1);
    let seeds = t.hand(P1, "Seeds of Strength");
    assert!(t
        .cast(P1, seeds)
        .target(obj(c))
        .target(obj(l))
        .target(obj(c))
        .try_go()
        .is_err());
    t.clear_answers();
    add(&mut t, P1, ManaType::C, 1);
    t.cast(P1, seeds)
        .target(obj(c))
        .target(obj(l))
        .target(obj(c))
        .go();
    assert_eq!(pool(&t, P1), 0);
    // Lightning Bolt at Callaphe (an enchantment creature): {R} and just {1}.
    let (mut t, c, _) = setup();
    add(&mut t, P1, ManaType::R, 1);
    add(&mut t, P1, ManaType::C, 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(obj(c)).go();
    assert_eq!(pool(&t, P1), 0);
}

#[test]
fn eluge_flood_counters_and_cost_reduction() {
    cr!("611.3a", "601.2f", "118.7d");
    ruling!(
        "Eluge, the Shoreless Sea",
        "A land affected by Eluge’s second ability stays an Island until the flood counter is removed, even if Eluge leaves the battlefield."
    );
    ruling!(
        "Eluge, the Shoreless Sea",
        "If the first instant or sorcery spell you cast in a turn doesn’t cost any {U}, its cost will be reduced by {1} for each land you control with a flood counter on it."
    );
    supported("Eluge, the Shoreless Sea");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Island");
    let forest = t.battlefield(P0, "Forest");
    let mountain = t.battlefield(P0, "Mountain");
    t.answer_targets(P0, &[obj(forest)]);
    let eluge = enter(&mut t, P0, "Eluge, the Shoreless Sea");
    t.resolve_all();
    assert_eq!(t.counters(forest, "flood"), 1);
    assert!(t.obj_now(forest).chars.has_subtype("Island"));
    // A second flood counter on the Mountain; the lands are tapped so they can't pay.
    t.g.add_counters(obj(mountain), "flood", 1, None);
    for l in t.g.battlefield.clone() {
        if t.g.obj(l).is(CardType::Land) {
            t.g.objects[l.0 as usize].tapped = true;
        }
    }
    t.g.recompute();
    // Lava Axe ({4}{R}) costs {2}{R}; the next one costs full price.
    add(&mut t, P0, ManaType::R, 1);
    add(&mut t, P0, ManaType::C, 2);
    let axe = t.hand(P0, "Lava Axe");
    t.cast(P0, axe).target(pl(P1)).go();
    assert_eq!(pool(&t, P0), 0);
    t.resolve_all();
    add(&mut t, P0, ManaType::R, 1);
    add(&mut t, P0, ManaType::C, 2);
    let axe = t.hand(P0, "Lava Axe");
    assert!(t.cast(P0, axe).target(pl(P1)).try_go().is_err());
    t.clear_answers();
    // Eluge leaves: the Forest is still an Island.
    kill(&mut t, eluge);
    assert!(t.obj_now(forest).chars.has_subtype("Island"));
    assert!(t.obj_now(forest).chars.has_subtype("Forest"));
}

#[test]
fn mendicant_cores_copy_becomes_a_token_without_being_created() {
    cr!("707.10f", "614.1a");
    ruling!(
        "Mendicant Core, Guidelight",
        "A resolving copy of a permanent spell becomes a token, so the token isn’t “created.”"
    );
    supported("Mendicant Core, Guidelight");
    supported("Anointed Procession");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mendicant Core, Guidelight");
    t.battlefield(P0, "Anointed Procession");
    t.g.players[0].speed = Some(4);
    add(&mut t, P0, ManaType::C, 2);
    t.answer_yes(P0, true);
    let ring = t.hand(P0, "Sol Ring");
    let from = t.g.turn_events.len();
    t.cast(P0, ring).go();
    t.resolve_all();
    let rings = t.named_on_battlefield("Sol Ring");
    assert_eq!(rings.len(), 2);
    assert_eq!(
        rings.iter().filter(|r| t.obj_now(**r).is_token()).count(),
        1
    );
    assert!(!events_since(&t, from)
        .iter()
        .any(|e| matches!(e, Event::TokenCreated { .. })));
}

// ---------------------------------------------------------------------------
// Triggered abilities
// ---------------------------------------------------------------------------

#[test]
fn tokens_put_onto_the_battlefield_attacking_werent_declared_as_attackers() {
    cr!("508.4", "508.3a");
    ruling!(
        "Seraphic Greatsword",
        "Although the Angel token created by the triggered ability is attacking, it was never declared as an attacking creature"
    );
    ruling!(
        "Suki, Kyoshi Warrior",
        "Although the Ally token created by the triggered ability is attacking, it was never declared as an attacking creature"
    );
    ruling!(
        "Adeline, Resplendent Cathar",
        "Although the Human tokens created by the triggered ability are attacking, they were never declared as attacking creatures"
    );
    ruling!(
        "Adeline, Resplendent Cathar",
        "Attacking with any creatures will cause Adeline's last ability to trigger. Adeline doesn't have to be among them."
    );
    supported("Hellrider");
    supported("Seraphic Greatsword");
    supported("Suki, Kyoshi Warrior");
    // Hellrider: "Whenever a creature you control attacks, this creature deals 1 damage to
    // the player or planeswalker it's attacking." Only the declared attacker counts.
    for (who, token) in [
        ("Seraphic Greatsword", "Angel"),
        ("Suki, Kyoshi Warrior", "Ally"),
        ("Adeline, Resplendent Cathar", "Human"),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Hellrider");
        let attacker = match who {
            "Seraphic Greatsword" => {
                let b = t.battlefield(P0, "Grizzly Bears");
                crate::r_s06_common::attach_new(&mut t, P0, who, b);
                b
            }
            "Suki, Kyoshi Warrior" => t.battlefield(P0, who),
            _ => {
                t.battlefield(P0, who);
                t.battlefield(P0, "Grizzly Bears")
            }
        };
        attack_with(&mut t, &[(attacker, pl(P1))]);
        t.resolve_all();
        let toks = tokens_of(&t, P0, token);
        assert_eq!(toks.len(), 1, "{who}");
        assert!(
            t.g.combat
                .as_ref()
                .unwrap()
                .attack_target(toks[0])
                .is_some(),
            "{who}"
        );
        assert_eq!(t.life(P1), 19, "{who}: one Hellrider trigger");
    }
}

#[test]
fn psychosis_crawler_triggers_for_each_card_drawn() {
    cr!("121.2", "603.2c");
    ruling!(
        "Psychosis Crawler",
        "If an effect causes you to draw multiple cards, Psychosis Crawler will trigger that many times."
    );
    supported("Psychosis Crawler");
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    t.battlefield(P0, "Psychosis Crawler");
    cast_only(&mut t, P0, "Ancestral Recall", &[pl(P0)]);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "loses 1 life"), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn consuming_aberration_mills_a_whole_library_without_lands() {
    cr!("701.20a", "608.2c");
    ruling!(
        "Consuming Aberration",
        "If an opponent has no land cards in their library, all the cards from that library will be revealed and put into their graveyard."
    );
    supported("Consuming Aberration");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Consuming Aberration");
    let n = t.library_size(P1);
    cast_resolve(&mut t, P0, "Opt", &[]);
    assert_eq!(t.library_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), n);
    assert_eq!(t.pt(a), (n as i32, n as i32));
}

#[test]
fn sutured_ghoul_uses_the_exiled_cards_defined_power_and_toughness() {
    cr!("604.3", "208.2a", "607.2a");
    ruling!(
        "Sutured Ghoul",
        "If any of the creature cards you exile has a characteristic-defining ability that defines its power and/or toughness, that ability will apply."
    );
    supported("Sutured Ghoul");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let elder = t.graveyard(P0, "Dungrove Elder");
    let nemesis = t.graveyard(P0, "Sewer Nemesis");
    t.answer_choose(P0, &[obj(elder), obj(nemesis)]);
    let g = enter(&mut t, P0, "Sutured Ghoul");
    assert_eq!(t.zone(elder), Zone::Exile);
    assert_eq!(t.zone(nemesis), Zone::Exile);
    // Dungrove Elder counts P0's three Forests; Sewer Nemesis has no chosen player: 0.
    assert_eq!(t.pt(g), (3, 3));
    t.battlefield(P0, "Forest");
    t.g.recompute();
    assert_eq!(t.pt(g), (4, 4));
}

#[test]
fn abominable_treefolk_can_target_a_tapped_creature() {
    cr!("115.1", "502.3");
    ruling!(
        "Abominable Treefolk",
        "Abominable Treefolk's last ability can target a creature that's already tapped. That creature won't untap during its controller's next untap step."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.tap(giant);
    t.answer_targets(P0, &[obj(giant)]);
    enter(&mut t, P0, "Abominable Treefolk");
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(giant).tapped);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(giant).tapped);
}

#[test]
fn malignus_damage_to_a_protected_blocker_isnt_prevented() {
    cr!("702.16e", "615.12");
    ruling!(
        "Malignus",
        "If Malignus becomes blocked by a creature with protection from red, the damage Malignus deals to that creature won't be prevented."
    );
    supported("Malignus");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Malignus");
    let kor = t.battlefield(P1, "Kor Firewalker");
    t.g.recompute();
    assert_eq!(t.pt(m), (10, 10));
    attack_with(&mut t, &[(m, pl(P1))]);
    block_and_finish(&mut t, P1, &[(kor, m)]);
    assert!(t.in_graveyard(P1, "Kor Firewalker"));
}

#[test]
fn exdeath_intervening_if_checks_permanent_cards_twice() {
    cr!("603.4", "701.27a");
    ruling!(
        "Exdeath, Void Warlock // Neo Exdeath, Dimension's End",
        "Exdeath, Void Warlock's last ability checks your graveyard at the moment it would trigger to see if you have six or more permanent cards in your graveyard."
    );
    let name = "Exdeath, Void Warlock // Neo Exdeath, Dimension's End";
    supported(name);
    // (cards in the graveyard, one leaves in response, transforms)
    for (n, leaves, transforms) in [(5, false, false), (6, false, true), (6, true, false)] {
        let mut t = TestGame::new(2);
        let e = t.battlefield(P0, name);
        let cards: Vec<ObjectId> = (0..n).map(|_| t.graveyard(P0, "Grizzly Bears")).collect();
        t.graveyard(P0, "Lightning Bolt");
        t.set_step(P0, Step::PostcombatMain);
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(
            triggers_on_stack(&t, "transform"),
            usize::from(n >= 6),
            "{n}"
        );
        if leaves {
            t.g.exile_object(cards[0], None);
        }
        t.resolve_all();
        assert_eq!(
            t.obj_now(e).chars.name == "Neo Exdeath, Dimension's End",
            transforms,
            "{n} {leaves}"
        );
    }
}

#[test]
fn hanweir_militia_captain_checks_as_the_upkeep_begins() {
    cr!("603.4", "503.1a");
    ruling!(
        "Hanweir Militia Captain // Westvale Cult Leader",
        "If you don't control four or more creatures (including Hanweir Militia Captain itself) at the moment your upkeep begins, its ability doesn't trigger."
    );
    let name = "Hanweir Militia Captain // Westvale Cult Leader";
    supported(name);
    for others in [2, 3] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        for _ in 0..others {
            t.battlefield(P0, "Grizzly Bears");
        }
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        assert_eq!(triggers_on_stack(&t, "transform"), usize::from(others == 3));
        // A creature that arrives during the upkeep is too late.
        t.battlefield(P0, "Grizzly Bears");
        t.resolve_all();
        assert_eq!(
            t.obj_now(c).chars.name == "Westvale Cult Leader",
            others == 3
        );
    }
}
