//! Rulings batch P208 — enchant (CR 303.4, 608.2h, 113.7a): an Aura's ability that refers
//! to "enchanted creature" uses the creature the Aura last enchanted when the Aura left
//! the battlefield before the ability resolved, or the creature it enchants now if it
//! moved.

use crate::r_p208_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s04_common::*;
use crate::r_s06_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts the Aura into its owner's graveyard and settles (the Aura's trigger stays on the
/// stack).
fn aura_leaves(t: &mut TestGame, aura: ObjectId) {
    put_in_graveyard(t, aura);
    t.settle();
    assert!(!t.on_battlefield(aura));
}

#[test]
fn enters_triggers_grant_keywords_to_the_creature_last_enchanted() {
    cr!("608.2h", "113.7a", "603.6a");
    ruling!(
        "Aspect of Manticore",
        "If Aspect of Manticore leaves the battlefield before its enters-the-battlefield triggered ability resolves, the creature it last enchanted before it left gains first strike until end of turn."
    );
    ruling!(
        "Cradle of Safety",
        "If Cradle of Safety leaves the battlefield before its enters-the-battlefield triggered ability resolves, the creature it enchanted as it left gains hexproof until end of turn."
    );
    ruling!(
        "Starlit Mantle",
        "If Starlit Mantle leaves the battlefield before its enters-the-battlefield triggered ability resolves, the creature it last enchanted before it left gains hexproof until end of turn."
    );
    for (aura, kw) in [
        ("Aspect of Manticore", KeywordKind::FirstStrike),
        ("Cradle of Safety", KeywordKind::Hexproof),
        ("Starlit Mantle", KeywordKind::Hexproof),
    ] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let a = cast_aura_leave_trigger(&mut t, P0, aura, bears);
        assert_eq!(t.stack_len(), 1, "{aura}'s trigger");
        assert!(!has_kw(&t, bears, kw));
        aura_leaves(&mut t, a);
        t.resolve_all();
        assert!(has_kw(&t, bears, kw), "{aura}");
        assert_eq!(t.pt(bears), (2, 2), "{aura}'s bonus is gone");
        // Until end of turn.
        t.advance_to(P1, Step::Upkeep);
        assert!(!has_kw(&t, bears, kw), "{aura}");
    }
}

#[test]
fn enters_and_upkeep_triggers_put_counters_on_the_creature_last_enchanted() {
    cr!("608.2h", "113.7a", "122.1");
    ruling!(
        "Ephara's Enlightenment",
        "If Ephara's Enlightenment isn't on the battlefield when its enters-the-battlefield ability resolves, put a +1/+1 counter on the creature it was enchanting when it left the battlefield."
    );
    ruling!(
        "Hydra's Growth",
        "If Hydra's Growth leaves the battlefield before its enters-the-battlefield triggered ability resolves, the creature it last enchanted before it left gets the +1/+1 counter."
    );
    ruling!(
        "Forced Adaptation",
        "If Forced Adaptation’s ability triggers but Forced Adaptation isn’t on the battlefield when the ability resolves, put a +1/+1 counter on the creature it was enchanting when it left the battlefield."
    );
    for aura in ["Ephara's Enlightenment", "Hydra's Growth"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let a = cast_aura_leave_trigger(&mut t, P0, aura, bears);
        assert_eq!(t.stack_len(), 1, "{aura}'s trigger");
        aura_leaves(&mut t, a);
        t.resolve_all();
        assert_eq!(t.counters(bears, counters::PLUS1), 1, "{aura}");
    }
    supported("Forced Adaptation");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = attach_new(&mut t, P0, "Forced Adaptation", bears);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "counter"), 1);
    aura_leaves(&mut t, a);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
}

#[test]
fn fight_triggers_use_the_creature_last_enchanted_without_the_bonus() {
    cr!("608.2h", "113.7a", "701.14a");
    ruling!(
        "Cartouche of Strength",
        "If Cartouche of Strength leaves the battlefield before its triggered ability resolves, the creature it last enchanted will fight the target creature if it's still on the battlefield at that time. However, it won't have +1/+1 from Cartouche of Strength anymore."
    );
    ruling!(
        "Warbriar Blessing",
        "If Warbriar Blessing leaves the battlefield before its enters-the-battlefield triggered ability resolves, the creature it last enchanted before it left will be the one to fight."
    );
    for aura in ["Cartouche of Strength", "Warbriar Blessing"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let foe = t.battlefield(P1, "Grizzly Bears");
        let a =
            cast_aura_leave_trigger_with(&mut t, P0, aura, bears, &[Entity::Object(foe)]);
        assert_eq!(t.stack_len(), 1, "{aura}'s trigger");
        aura_leaves(&mut t, a);
        t.answer_yes(P0, true);
        t.resolve_all();
        // Both 2/2s fight: without the Aura's toughness bonus, both die.
        assert!(!t.on_battlefield(foe), "{aura}");
        assert!(!t.on_battlefield(bears), "{aura}");
        assert!(t.in_graveyard(P0, "Grizzly Bears"));
        assert!(t.in_graveyard(P1, "Grizzly Bears"));
    }
    // Control: if Warbriar Blessing stays, its +0/+2 saves the Bears.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let foe = t.battlefield(P1, "Grizzly Bears");
    cast_aura_leave_trigger_with(
        &mut t,
        P0,
        "Warbriar Blessing",
        bears,
        &[Entity::Object(foe)],
    );
    t.resolve_all();
    assert!(!t.on_battlefield(foe));
    assert!(t.on_battlefield(bears));
}

#[test]
fn winters_rest_taps_the_creature_it_enchanted_even_if_destroyed() {
    cr!("608.2h", "701.26a");
    ruling!(
        "Winter's Rest",
        "If Winter's Rest is destroyed while its enters-the-battlefield triggered ability is on the stack, the creature that it enchanted is tapped as that ability resolves."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let a = cast_aura_leave_trigger(&mut t, P0, "Winter's Rest", giant);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, a);
    assert!(!t.on_battlefield(a));
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
}

#[test]
fn ghostly_wings_returns_the_creature_enchanted_when_it_left() {
    cr!("608.2h", "113.7a", "602.2");
    ruling!(
        "Ghostly Wings",
        "If Ghostly Wings leaves the battlefield before its activated ability resolves, the creature that was enchanted immediately before Ghostly Wings left is the one that will be returned to its owner’s hand, if it’s still on the battlefield."
    );
    supported("Ghostly Wings");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wings = attach_new(&mut t, P0, "Ghostly Wings", bears);
    t.hand(P0, "Hill Giant");
    t.activate(P0, wings, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.hand_size(P0), 0);
    aura_leaves(&mut t, wings);
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn dreadful_apathy_exiles_the_permanent_it_last_enchanted_even_if_not_a_creature() {
    cr!("608.2h", "113.7a", "303.4c", "704.5m");
    ruling!(
        "Dreadful Apathy",
        "If Dreadful Apathy leaves the battlefield while its last ability is on the stack, the permanent that's exiled is the one Dreadful Apathy enchanted before leaving the battlefield, even if that permanent is no longer a creature."
    );
    supported("Dreadful Apathy");
    supported("One with the Stars");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let apathy = attach_new(&mut t, P0, "Dreadful Apathy", bears);
    t.lands(P0, "Plains", 3);
    t.activate(P0, apathy, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 1);
    // In response, One with the Stars makes the Bears a noncreature enchantment: Dreadful
    // Apathy (enchant creature) is put into the graveyard as a state-based action.
    attach_new(&mut t, P1, "One with the Stars", bears);
    t.settle();
    assert!(!t.obj_now(bears).is(CardType::Creature));
    assert!(t.in_graveyard(P0, "Dreadful Apathy"));
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn mire_blight_destroys_the_creature_it_enchanted_when_it_triggered() {
    cr!("603.2", "603.4", "608.2h");
    ruling!(
        "Mire Blight",
        "If Mire Blight’s ability triggers, the creature it was enchanting at that time is the one that will be destroyed when the ability resolves. It doesn’t matter if Mire Blight leaves the battlefield or somehow becomes attached to another creature by that time."
    );
    supported("Mire Blight");
    // Mire Blight moves to another creature.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let shock = t.battlefield(P0, "Llanowar Elves");
    let blight = attach_new(&mut t, P0, "Mire Blight", bears);
    damage(&mut t, shock, 1, bears);
    assert_eq!(triggers_on_stack(&t, "destroy"), 1);
    assert!(t.g.attach(blight, Entity::Object(giant)));
    t.g.recompute();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(giant));
    // Mire Blight leaves the battlefield.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let shock = t.battlefield(P0, "Llanowar Elves");
    let blight = attach_new(&mut t, P0, "Mire Blight", bears);
    damage(&mut t, shock, 1, bears);
    assert_eq!(triggers_on_stack(&t, "destroy"), 1);
    aura_leaves(&mut t, blight);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn mirror_mockery_copies_the_creature_it_last_enchanted() {
    cr!("608.2h", "113.7a", "707.2", "111.1");
    ruling!(
        "Mirror Mockery",
        "If Mirror Mockery is no longer on the battlefield as its triggered ability resolves, its ability makes a copy of the creature it was enchanting when it left the battlefield. If it left the battlefield because it was no longer enchanting a legal creature, it makes a copy of the creature it was most recently enchanting before it left the battlefield."
    );
    supported("Mirror Mockery");
    // Mirror Mockery leaves the battlefield.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let mockery = attach_new(&mut t, P0, "Mirror Mockery", bears);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "copy"), 1);
    aura_leaves(&mut t, mockery);
    t.answer_yes(P0, true);
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Grizzly Bears");
    // One with the Stars makes the creature a noncreature enchantment, so Mirror Mockery
    // isn't enchanting a legal creature and is put into the graveyard: the copy is of
    // the permanent it was most recently enchanting.
    supported("One with the Stars");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Mirror Mockery", bears);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "copy"), 1);
    attach_new(&mut t, P1, "One with the Stars", bears);
    t.settle();
    assert!(t.in_graveyard(P0, "Mirror Mockery"));
    t.answer_yes(P0, true);
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Grizzly Bears");
}

#[test]
fn mirror_mockerys_controller_can_block_with_the_token() {
    cr!("111.2", "509.1a", "603.3a");
    ruling!(
        "Mirror Mockery",
        "If Mirror Mockery enchants a creature you don’t control, and you are the defending player, you can block with the token."
    );
    supported("Mirror Mockery");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Hill Giant");
    attach_new(&mut t, P0, "Mirror Mockery", giant);
    attack_with(&mut t, &[(giant, Entity::Player(P0))]);
    t.answer_yes(P0, true);
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).controller, P0);
    assert!(crate::r_s21_common::legal_blocks(
        &mut t,
        P0,
        &[(toks[0], giant)]
    ));
    block_and_finish(&mut t, P0, &[(toks[0], giant)]);
    // The two 3/3s traded; the attacker didn't deal damage to P0.
    assert_eq!(t.life(P0), 20);
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn followed_footsteps_copies_the_newly_enchanted_creature_if_it_moved() {
    cr!("608.2h", "707.2", "111.1");
    ruling!(
        "Followed Footsteps",
        "If Followed Footsteps moves after it has triggered, you get a copy of the newly-enchanted creature."
    );
    supported("Followed Footsteps");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let ff = attach_new(&mut t, P0, "Followed Footsteps", bears);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "copy"), 1);
    assert!(t.g.attach(ff, Entity::Object(giant)));
    t.g.recompute();
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Hill Giant");
}

#[test]
fn followed_footsteps_leaving_with_its_creature_copies_that_creature() {
    cr!("608.2h", "113.7a", "707.2", "704.5m");
    ruling!(
        "Followed Footsteps",
        "If Followed Footsteps and the enchanted creature leave the battlefield simultaneously"
    );
    supported("Followed Footsteps");
    // Both leave simultaneously: a copy of the creature it enchanted.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ff = attach_new(&mut t, P0, "Followed Footsteps", bears);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "copy"), 1);
    t.g.destroy_all(vec![bears, ff], None, false);
    t.g.flush_events();
    t.settle();
    assert!(t.in_graveyard(P0, "Followed Footsteps"));
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Grizzly Bears");
    // The creature left first; Followed Footsteps then went to the graveyard as a
    // state-based action enchanting nothing: no copy.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Followed Footsteps", bears);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "copy"), 1);
    put_in_graveyard(&mut t, bears);
    t.settle();
    assert!(t.in_graveyard(P0, "Followed Footsteps"));
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
}

#[test]
fn caught_in_the_brights_exiles_the_creature_it_enchanted_as_it_left() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Caught in the Brights",
        "If Caught in the Brights leaves the battlefield in response to its triggered ability, the resolving ability will exile the creature Caught in the Brights was enchanting as it left the battlefield."
    );
    supported("Caught in the Brights");
    supported("Smuggler's Copter");
    let mut t = TestGame::new(2);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let brights = attach_new(&mut t, P0, "Caught in the Brights", giant);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(crew(&mut t, P0, copter, &[bears]));
    t.resolve_all();
    attack_with(&mut t, &[(copter, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "exile"), 1);
    aura_leaves(&mut t, brights);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
}
