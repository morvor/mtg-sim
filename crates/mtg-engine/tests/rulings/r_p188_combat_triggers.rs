//! Rulings batch P188 — combat and triggered abilities: blocked creatures stay blocked
//! (Frostpeak Yeti, Haunting Figment) and attackers stay attacking (Futurist Operative),
//! "enchanted" (Metathran Elite), Aetherling, Fungusaur, required targets (Bogardan
//! Firefiend, Man-o'-War, Seasoned Marshal), Alabaster Dragon, cast triggers (Chakram
//! Retriever, Pyre Hound), intervening "if" clauses (Hollowborn Barghest, Salt Road
//! Ambushers), Hellfire Mongrel in Two-Headed Giant, Jeskai Infiltrator, and the monarch
//! (Canal Courier).

use crate::r_p108_common::two_headed_giant;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s03_common::to_blockers;
use crate::r_s06_common::{attach_new, give_control};
use crate::r_s11_common::{manifest_card, turn_face_up};
use crate::r_s25_common::targets_of;
use crate::r_s28_common::cast_card;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn blocked(t: &TestGame, id: ObjectId) -> bool {
    t.g.combat.as_ref().unwrap().is_blocked(t.g.current(id))
}

fn attacking(t: &TestGame, id: ObjectId) -> bool {
    let id = t.g.current(id);
    t.g.combat
        .as_ref()
        .is_some_and(|c| c.attackers.iter().any(|a| a.id == id))
}

/// Destroys the permanent as an effect would, and puts triggered abilities on the stack.
fn destroy(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.destroy(id, None);
    t.g.flush_events();
    t.settle();
}

// ---------------------------------------------------------------------------------------
// Blocked creatures stay blocked; attackers stay attacking
// ---------------------------------------------------------------------------------------

#[test]
fn frostpeak_yeti_cant_be_blocked_after_blockers_is_still_blocked() {
    cr!("509.1h", "506.4");
    ruling!(
        "Frostpeak Yeti",
        "Activating Frostpeak Yeti's ability after it's already been blocked won't cause it to become unblocked."
    );
    supported("Frostpeak Yeti");
    let mut t = TestGame::new(2);
    let yeti = t.battlefield(P0, "Frostpeak Yeti");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.lands(P0, "Snow-Covered Island", 2);
    to_blockers(&mut t, &[(yeti, Entity::Player(P1))], &[(wall, yeti)]);
    assert!(blocked(&t, yeti));
    t.activate(P0, yeti, 0, &[]).expect("yeti");
    t.resolve_all();
    assert!(blocked(&t, yeti));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(wall).damage, 3);
}

#[test]
fn haunting_figment_stays_blocked_after_an_instant_is_cast() {
    cr!("509.1h", "506.4");
    ruling!(
        "Haunting Figment",
        "Casting an instant or sorcery spell after Haunting Figment has become blocked will not cause it to become unblocked."
    );
    supported("Haunting Figment");
    let mut t = TestGame::new(2);
    let fig = t.battlefield(P0, "Haunting Figment");
    let wall = t.battlefield(P1, "Wall of Stone");
    to_blockers(&mut t, &[(fig, Entity::Player(P1))], &[(wall, fig)]);
    assert!(blocked(&t, fig));
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(blocked(&t, fig));
    t.advance_to(P0, Step::EndOfCombat);
    // Only the Bolt's damage.
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.obj_now(wall).damage, 2);
    // Before blockers, a spell this turn makes it unblockable: the Wall can't block.
    let mut t = TestGame::new(2);
    let fig = t.battlefield(P0, "Haunting Figment");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    to_blockers(&mut t, &[(fig, Entity::Player(P1))], &[(wall, fig)]);
    assert!(!blocked(&t, fig));
}

#[test]
fn futurist_operative_untapped_while_attacking_stays_attacking() {
    cr!("506.4", "508.1f");
    ruling!(
        "Futurist Operative",
        "Untapping Futurist Operative once it's been declared as an attacker won't remove it from combat."
    );
    supported("Futurist Operative");
    let mut t = TestGame::new(2);
    let op = t.battlefield(P0, "Futurist Operative");
    attack_with(&mut t, &[(op, Entity::Player(P1))]);
    assert!(t.obj_now(op).tapped);
    // Tapped: a 1/1 Human Citizen.
    assert_eq!(t.pt(op), (1, 1));
    t.lands(P0, "Island", 3);
    t.activate(P0, op, 0, &[]).expect("untap");
    t.resolve_all();
    assert!(!t.obj_now(op).tapped);
    assert!(attacking(&t, op));
    assert_eq!(t.pt(op), (3, 4));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn metathran_elite_is_enchanted_by_any_aura() {
    cr!("303.4b", "509.1b");
    ruling!(
        "Metathran Elite",
        "Being “enchanted” means there is an Aura on it."
    );
    supported("Metathran Elite");
    supported("Weakness");
    for aura in [false, true] {
        let mut t = TestGame::new(2);
        let elite = t.battlefield(P0, "Metathran Elite");
        let bears = t.battlefield(P1, "Grizzly Bears");
        if aura {
            // The opponent's Weakness (-2/-1).
            attach_new(&mut t, P1, "Weakness", elite);
            assert_eq!(t.pt(elite), (0, 2));
        }
        to_blockers(&mut t, &[(elite, Entity::Player(P1))], &[(bears, elite)]);
        assert_eq!(blocked(&t, elite), !aura);
    }
}

// ---------------------------------------------------------------------------------------
// Aetherling
// ---------------------------------------------------------------------------------------

#[test]
fn aetherling_returns_only_if_its_own_ability_exiled_it() {
    cr!("610.3", "603.7");
    ruling!(
        "Aetherling",
        "Aetherling's first ability will return it to the battlefield only if that ability also exiled it. If Aetherling left the battlefield in response to that ability, it won't return, even if it was exiled by another spell or ability."
    );
    supported("Aetherling");
    for respond in [false, true] {
        let mut t = TestGame::new(2);
        let ae = t.battlefield(P0, "Aetherling");
        t.lands(P0, "Island", 1);
        t.activate(P0, ae, 0, &[]).expect("blink");
        if respond {
            // Swords to Plowshares in response.
            t.answer_targets(P1, &[o(ae)]);
            t.lands(P1, "Plains", 1);
            let stp = t.hand(P1, "Swords to Plowshares");
            t.cast(P1, stp).go();
        }
        t.resolve_all();
        assert!(t.in_exile("Aetherling"));
        t.advance_to(P1, Step::Upkeep);
        let back = t.named_on_battlefield("Aetherling");
        assert_eq!(back.len(), usize::from(!respond), "respond: {respond}");
    }
}

#[test]
fn aetherling_with_zero_or_less_power_can_pump_and_deals_no_damage() {
    cr!("510.1a", "602.2");
    ruling!(
        "Aetherling",
        "Aetherling's last ability can be activated even if its power is 0 or less. If a creature would assign 0 or less damage in combat, it doesn't assign combat damage at all."
    );
    let mut t = TestGame::new(2);
    let ae = t.battlefield(P0, "Aetherling");
    t.lands(P0, "Wastes", 5);
    // "{1}: This creature gets -1/+1 until end of turn." five times: 4/5 -> -1/10.
    for _ in 0..5 {
        t.activate(P0, ae, 3, &[]).expect("-1/+1");
        t.resolve_all();
    }
    assert_eq!(t.pt(ae), (-1, 10));
    t.attack(&[(ae, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
    assert!(!t.g.turn_events.iter().any(|e| matches!(
        e,
        mtg_engine::events::Event::Damage { source, .. } if *source == ae
    )));
}

// ---------------------------------------------------------------------------------------
// Fungusaur
// ---------------------------------------------------------------------------------------

#[test]
fn fungusaur_damaged_by_two_creatures_at_once_gets_one_counter() {
    cr!("510.2", "603.2c");
    ruling!(
        "Fungusaur",
        "If more than one creature damages it at one time, it only gets one counter."
    );
    supported("Fungusaur");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Fungusaur");
    let e1 = t.battlefield(P1, "Llanowar Elves");
    let e2 = t.battlefield(P1, "Llanowar Elves");
    t.answer_targets(P0, &[o(f)]);
    cast_card(&mut t, P0, "Giant Growth");
    t.resolve_all();
    to_blockers(&mut t, &[(f, Entity::Player(P1))], &[(e1, f), (e2, f)]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.obj_now(f).damage, 2);
    assert_eq!(t.counters(f, "+1/+1"), 1);
}

// ---------------------------------------------------------------------------------------
// Required targets
// ---------------------------------------------------------------------------------------

#[test]
fn bogardan_firefiend_must_target_its_controllers_own_creature() {
    cr!("603.3d", "115.1");
    ruling!(
        "Bogardan Firefiend",
        "You must pick a target creature, even if you are the only player with creatures on the battlefield."
    );
    supported("Bogardan Firefiend");
    let mut t = TestGame::new(2);
    let ff = t.battlefield(P0, "Bogardan Firefiend");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // P0 tries to choose no target.
    t.answer_targets(P0, &[]);
    destroy(&mut t, ff);
    assert_eq!(t.stack_len(), 1);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(targets_of(&t, top), vec![o(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn man_o_war_alone_must_target_itself() {
    cr!("603.3d", "115.1");
    ruling!(
        "Man-o'-War",
        "If there are no other creatures on the battlefield when Man-o'-War enters the battlefield, its ability must target itself."
    );
    supported("Man-o'-War");
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[]);
    let mow = t.enter(P0, "Man-o'-War");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(targets_of(&t, top), vec![o(mow)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Man-o'-War"));
}

#[test]
fn seasoned_marshal_can_target_a_tapped_creature() {
    cr!("115.1", "701.26a");
    ruling!(
        "Seasoned Marshal",
        "Can target an already tapped creature if you want."
    );
    supported("Seasoned Marshal");
    let mut t = TestGame::new(2);
    let marshal = t.battlefield(P0, "Seasoned Marshal");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[giant.0 as usize].tapped = true;
    t.answer_targets(P0, &[o(giant)]);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(marshal, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(targets_of(&t, top), vec![o(giant)]);
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn alabaster_dragon_is_shuffled_into_its_owners_library() {
    cr!("108.3", "400.3");
    ruling!(
        "Alabaster Dragon",
        "This will always be shuffled into its owner’s library."
    );
    supported("Alabaster Dragon");
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P1, "Alabaster Dragon");
    give_control(&mut t, dragon, P0);
    assert_eq!(t.obj_now(dragon).controller, P0);
    let (l0, l1) = (t.library_size(P0), t.library_size(P1));
    destroy(&mut t, dragon);
    t.resolve_all();
    assert_eq!(t.library_size(P1), l1 + 1);
    assert_eq!(t.library_size(P0), l0);
    assert_eq!(t.zone(dragon), Zone::Library(P1));
}

// ---------------------------------------------------------------------------------------
// Cast triggers resolve first
// ---------------------------------------------------------------------------------------

#[test]
fn chakram_retrievers_trigger_resolves_first_and_players_can_act_in_between() {
    cr!("601.2i", "603.3", "117.3b");
    ruling!(
        "Chakram Retriever",
        "Chakram Retriever’s last ability resolves before the spell that caused it to trigger."
    );
    ruling!(
        "Chakram Retriever",
        "Players can cast spells and activate abilities after Chakram Retriever’s last ability resolves but before the spell that caused it to trigger does."
    );
    supported("Chakram Retriever");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chakram Retriever");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.g.objects[elves.0 as usize].tapped = true;
    t.answer_targets(P0, &[o(elves)]);
    let spell = cast_card(&mut t, P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    // The trigger resolved; the Bears spell is still on the stack.
    assert!(!t.obj_now(elves).tapped);
    assert_eq!(t.g.stack, vec![spell]);
    // P1 casts a spell now, before the Bears resolves.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(o(elves)).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    assert_eq!(t.g.stack, vec![spell]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn pyre_hounds_trigger_resolves_even_if_the_spell_is_countered() {
    cr!("603.3", "113.7a", "701.6a");
    ruling!(
        "Pyre Hound",
        "Pyre Hound's triggered ability resolves before the spell that caused it to trigger. The ability will resolve even if that spell is countered."
    );
    supported("Pyre Hound");
    let mut t = TestGame::new(2);
    let hound = t.battlefield(P0, "Pyre Hound");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(o(spell)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.counters(hound, "+1/+1"), 1);
}

// ---------------------------------------------------------------------------------------
// Intervening "if" clauses
// ---------------------------------------------------------------------------------------

#[test]
fn hollowborn_barghest_checks_your_hand_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Hollowborn Barghest",
        "The first ability checks whether you have no cards in hand both when it triggers and when it resolves. If you have any cards in your hand at the beginning of your upkeep, it won’t trigger at all. If you have any cards in your hand when the ability resolves, it does nothing."
    );
    supported("Hollowborn Barghest");
    // 0: empty hand; 1: a card in hand at the upkeep; 2: a card added in response.
    for case in 0..3 {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Hollowborn Barghest");
        // P1 holds a card so its own trigger never matters.
        t.hand(P1, "Island");
        if case == 1 {
            t.hand(P0, "Island");
        }
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        assert_eq!(t.stack_len(), usize::from(case != 1), "case {case}");
        if case == 2 {
            t.hand(P0, "Island");
        }
        t.resolve_all();
        assert_eq!(t.life(P1), if case == 0 { 18 } else { 20 }, "case {case}");
    }
}

#[test]
fn hollowborn_barghest_checks_the_opponents_hand_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Hollowborn Barghest",
        "The second ability behaves the same way, except that it’s checking your opponent instead of you."
    );
    for case in 0..3 {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Hollowborn Barghest");
        if case == 1 {
            t.hand(P1, "Island");
        }
        t.advance_to(P1, Step::Upkeep);
        t.settle();
        assert_eq!(t.stack_len(), usize::from(case != 1), "case {case}");
        if case == 2 {
            t.hand(P1, "Island");
        }
        t.resolve_all();
        assert_eq!(t.life(P1), if case == 0 { 18 } else { 20 }, "case {case}");
    }
}

#[test]
fn salt_road_ambushers_needs_a_creature_before_it_is_turned_face_up() {
    cr!("603.4", "701.40b");
    ruling!(
        "Salt Road Ambushers",
        "The face-down permanent must be a creature both before it's turned face up and when the triggered ability resolves to have +1/+1 counters placed on it."
    );
    supported("Salt Road Ambushers");
    supported("Imprisoned in the Moon");
    for moon in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Salt Road Ambushers");
        let m = manifest_card(&mut t, P0, "Grizzly Bears");
        if moon {
            // Imprisoned in the Moon: "Enchanted permanent is a colorless land with
            // '{T}: Add {C}' and loses all other card types and abilities."
            cast_card_at(&mut t, "Imprisoned in the Moon", m);
            assert!(!t.obj_now(m).is(CardType::Creature));
        }
        t.lands(P0, "Forest", 2);
        assert!(turn_face_up(&mut t, P0, m));
        t.settle();
        assert_eq!(t.stack_len(), usize::from(!moon), "moon: {moon}");
        t.resolve_all();
        assert_eq!(t.counters(m, "+1/+1"), if moon { 0 } else { 2 });
    }
}

fn cast_card_at(t: &mut TestGame, name: &str, target: ObjectId) {
    t.answer_targets(P0, &[o(target)]);
    cast_card(t, P0, name);
    t.resolve_all();
}

// ---------------------------------------------------------------------------------------
// Hellfire Mongrel in Two-Headed Giant
// ---------------------------------------------------------------------------------------

#[test]
fn hellfire_mongrel_triggers_for_each_player_of_the_opposing_team() {
    cr!("805.4d", "810.4");
    ruling!(
        "Hellfire Mongrel",
        "In a Two-Headed Giant game, this ability will potentially trigger twice at the beginning of the opposing team’s upkeep — once for each player on that team."
    );
    supported("Hellfire Mongrel");
    for p3_cards in [0usize, 3] {
        let mut t = two_headed_giant();
        t.battlefield(P0, "Hellfire Mongrel");
        for _ in 0..p3_cards {
            t.hand(P3, "Island");
        }
        let life = t.life(P2);
        t.advance_to(P2, Step::Upkeep);
        t.settle();
        let n = if p3_cards == 0 { 2 } else { 1 };
        assert_eq!(t.stack_len(), n, "P3 has {p3_cards} cards");
        t.resolve_all();
        assert_eq!(t.life(P2), life - 2 * n as i32);
        assert_eq!(t.life(P3), t.life(P2));
    }
}

// ---------------------------------------------------------------------------------------
// Jeskai Infiltrator
// ---------------------------------------------------------------------------------------

#[test]
fn jeskai_infiltrator_manifested_cards_keep_their_owners() {
    cr!("108.3", "701.40a", "708.1");
    ruling!(
        "Jeskai Infiltrator",
        "A card's owner is public information at all times. If the two cards you exile are owned by different players"
    );
    supported("Jeskai Infiltrator");
    let mut t = TestGame::new(2);
    let inf = t.battlefield(P1, "Jeskai Infiltrator");
    give_control(&mut t, inf, P0);
    // P0 has controlled it continuously since the turn began.
    let inf = t.g.current(inf);
    t.g.objects[inf.0 as usize].summoning_sick = false;
    t.library_top(P0, "Hill Giant");
    t.attack(&[(inf, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    let fd: Vec<&mtg_engine::object::GameObject> = t
        .g
        .permanents()
        .filter(|x| x.face_down && x.controller == P0)
        .collect();
    assert_eq!(fd.len(), 2);
    let mut owners: Vec<PlayerId> = fd.iter().map(|x| x.owner).collect();
    owners.sort();
    assert_eq!(owners, vec![P0, P1]);
    let theirs = fd.iter().find(|x| x.owner == P1).unwrap();
    assert_eq!(
        theirs.card.as_ref().unwrap().front().chars.name,
        "Jeskai Infiltrator"
    );
}

// ---------------------------------------------------------------------------------------
// The monarch (Canal Courier)
// ---------------------------------------------------------------------------------------

#[test]
fn becoming_the_monarch_again_doesnt_trigger_become_the_monarch() {
    cr!("725.1", "603.2");
    ruling!(
        "Canal Courier",
        "Abilities that trigger whenever you “become the monarch” trigger only if you aren’t already the monarch."
    );
    supported("Custodi Lich");
    // Canal Courier: "When this creature enters, you become the monarch." Custodi Lich:
    // "Whenever you become the monarch, target player sacrifices a creature."
    for already in [false, true] {
        let mut t = TestGame::new(2);
        if already {
            t.g.monarch = Some(P0);
        }
        t.battlefield(P0, "Custodi Lich");
        t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.enter(P0, "Canal Courier");
        t.g.flush_events();
        t.settle();
        t.resolve_all();
        assert_eq!(t.g.monarch, Some(P0));
        assert_eq!(
            t.in_graveyard(P1, "Grizzly Bears"),
            !already,
            "already: {already}"
        );
    }
}

#[test]
fn the_monarch_draws_at_end_step_and_loses_it_to_combat_damage() {
    cr!("725.2");
    ruling!(
        "Canal Courier",
        "Being the monarch carries two inherent triggered abilities. “At the beginning of the monarch’s end step, that player draws a card” and “Whenever a creature deals combat damage to the monarch, its controller becomes the monarch.”"
    );
    let mut t = TestGame::new(2);
    t.enter(P0, "Canal Courier");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P0));
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // P1's Bears deals combat damage to P0: P1 becomes the monarch.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::PrecombatMain);
    t.attack(&[(bears, Entity::Player(P0))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.g.monarch, Some(P1));
}
