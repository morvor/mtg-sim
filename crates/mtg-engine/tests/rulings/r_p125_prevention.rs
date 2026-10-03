//! Rulings batch P125 — fogs and group damage prevention (CR 615.1a): "Prevent all combat
//! damage that would be dealt this turn [by a group]", where the group is checked as the
//! damage would be dealt; life totals that can't change (CR 119.7, 119.8).

use crate::r_p125_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn rescue_retriever_protects_attacking_soldiers_while_not_attacking() {
    cr!("615.1a", "506.4");
    ruling!(
        "Rescue Retriever",
        "Rescue Retriever prevents damage to other attacking Soldiers even if it isn't also attacking."
    );
    supported("Rescue Retriever");
    let mut t = TestGame::new(2);
    let retriever = t.battlefield(P0, "Rescue Retriever");
    let vanguard = t.battlefield(P0, "Elite Vanguard");
    let ogre = t.battlefield(P1, "Gray Ogre");
    fight_it_out(&mut t, &[vanguard], &[(ogre, vanguard)]);
    assert!(t.on_battlefield(vanguard), "no damage was dealt to the Vanguard");
    assert_eq!(damage_marked(&t, vanguard), 0);
    assert!(!t.on_battlefield(ogre));
    assert!(t.on_battlefield(retriever));
}

#[test]
fn flare_of_fortitude_doesnt_prevent_damage() {
    cr!("119.7", "119.8", "702.15b");
    ruling!(
        "Flare of Fortitude",
        "The effect of Flare of Fortitude doesn't prevent damage. Rather, it changes the results of that damage."
    );
    supported("Flare of Fortitude");
    // P0's lifelink creature deals damage to P1: P1 loses life, P0 gains none.
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    cast_new(&mut t, P0, "Flare of Fortitude", &[]);
    t.resolve_all();
    fight_it_out(&mut t, &[hawk], &[]);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);

    // P1's lifelink creature deals damage to P0: P0 loses no life, P1 still gains 2.
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P1, "Vampire Nighthawk");
    t.set_step(P1, Step::BeginningOfCombat);
    cast_new(&mut t, P0, "Flare of Fortitude", &[]);
    t.resolve_all();
    attack_with(&mut t, &[(hawk, Entity::Player(P0))]);
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 22);
}

#[test]
fn teferis_protection_stops_paying_life() {
    cr!("119.4", "119.8");
    ruling!(
        "Teferi's Protection",
        "You can't pay a cost that includes the payment of any amount of life other than 0 life."
    );
    supported("Teferi's Protection");
    supported("Gitaxian Probe");
    // Gitaxian Probe ({U/P}) with no mana: only payable with 2 life.
    for protected in [false, true] {
        let mut t = TestGame::new(2);
        if protected {
            cast_new(&mut t, P0, "Teferi's Protection", &[]);
            t.resolve_all();
        }
        let probe = t.hand(P0, "Gitaxian Probe");
        let r = t.cast(P0, probe).target(Entity::Player(P1)).try_go();
        assert_eq!(r.is_ok(), !protected, "{r:?}");
        assert_eq!(t.life(P0), if protected { 20 } else { 18 });
    }
}

// --- Tanglesap -----------------------------------------------------------------------------

#[test]
fn tanglesap_checks_trample_as_damage_is_dealt() {
    cr!("615.1a", "702.19b");
    ruling!(
        "Tanglesap",
        "Creatures are checked to see whether they have trample at the time they’d deal combat damage, not at the time Tanglesap resolves."
    );
    supported("Tanglesap");
    supported("Run Amok");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Tanglesap", &[]);
    t.resolve_all();
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[giant, bears]));
    // The Giant gains trample after Tanglesap resolved.
    cast_new(&mut t, P0, "Run Amok", &[obj(giant)]);
    t.resolve_all();
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 14, "only the trampling Giant's 6 damage is dealt");
}

#[test]
fn tanglesap_applies_to_attackers_and_blockers() {
    cr!("615.1a");
    ruling!(
        "Tanglesap",
        "Tanglesap prevents combat damage that would be dealt by all creatures without trample, regardless of whether they’re attacking or blocking."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let armodon = t.battlefield(P1, "Trained Armodon");
    cast_new(&mut t, P0, "Tanglesap", &[]);
    t.resolve_all();
    fight_it_out(&mut t, &[giant], &[(armodon, giant)]);
    assert_eq!(damage_marked(&t, giant), 0);
    assert_eq!(damage_marked(&t, armodon), 0);
}

// --- Terrifying Presence, Encircling Fissure -----------------------------------------------

#[test]
fn terrifying_presence_with_an_illegal_target_prevents_nothing() {
    cr!("608.2b", "615.1a");
    ruling!(
        "Terrifying Presence",
        "if that creature is an illegal target when Terrifying Presence tries to resolve, it won’t resolve and none of its effects will happen. No damage will be prevented."
    );
    supported("Terrifying Presence");
    // It works: only the target deals combat damage.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Terrifying Presence", &[obj(giant)]);
    t.resolve_all();
    fight_it_out(&mut t, &[giant, bears], &[]);
    assert_eq!(t.life(P1), 17);

    // The target is exiled in response: the spell doesn't resolve.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let spell = cast_new(&mut t, P0, "Terrifying Presence", &[obj(wurm)]);
    exile_now(&mut t, wurm);
    t.resolve_all();
    assert!(!resolved(&t, spell));
    fight_it_out(&mut t, &[giant, bears], &[]);
    assert_eq!(t.life(P1), 15);
}

#[test]
fn encircling_fissure_covers_creatures_arriving_later() {
    cr!("615.1a", "611.2c");
    ruling!(
        "Encircling Fissure",
        "Encircling Fissure prevents all combat damage that would be dealt by any creature controlled by the target opponent, even if that creature wasn’t on the battlefield or wasn’t a creature as Encircling Fissure resolved."
    );
    supported("Encircling Fissure");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    cast_new(&mut t, P0, "Encircling Fissure", &[Entity::Player(P1)]);
    t.resolve_all();
    // P1's Ogre enters afterwards and blocks the Giant.
    let ogre = t.battlefield(P1, "Gray Ogre");
    fight_it_out(&mut t, &[giant], &[(ogre, giant)]);
    assert_eq!(damage_marked(&t, giant), 0, "the Ogre's damage was prevented");
    assert!(!t.on_battlefield(ogre), "P0's Giant still deals its damage");
}

// --- Blessed Respite, Spike Weaver, Knight-Captain of Eos ----------------------------------

#[test]
fn blessed_respite_targeting_yourself_goes_to_your_graveyard() {
    cr!("608.2n", "701.24a");
    ruling!(
        "Blessed Respite",
        "If you target yourself with Blessed Respite, it will not be shuffled into your library. It will go to your graveyard after it resolves."
    );
    supported("Blessed Respite");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    let library = t.library_size(P0);
    cast_new(&mut t, P0, "Blessed Respite", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.library_size(P0), library + 2);
    assert_eq!(t.graveyard_size(P0), 1);
    assert!(t.in_graveyard(P0, "Blessed Respite"));
}

#[test]
fn blessed_respite_with_an_empty_graveyard() {
    cr!("615.1a", "701.24a");
    ruling!(
        "Blessed Respite",
        "You may cast Blessed Respite even if there are no cards in the target player's graveyard. If you do, they will still shuffle their library and combat damage will still be prevented."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    cast_new(&mut t, P0, "Blessed Respite", &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(shuffled(&t, P1));
    fight_it_out(&mut t, &[giant], &[]);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn spike_weaver_fog_includes_your_own_creatures() {
    cr!("615.1a", "602.2");
    ruling!("Spike Weaver", "The second ability includes your own creatures as well.");
    supported("Spike Weaver");
    let mut t = TestGame::new(2);
    let weaver = t.battlefield(P0, "Spike Weaver");
    put_counters(&mut t, weaver, "+1/+1", 3);
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Wastes", 1);
    t.activate(P0, weaver, 1, &[]).expect("activate");
    t.resolve_all();
    assert_eq!(t.counters(weaver, "+1/+1"), 2);
    fight_it_out(&mut t, &[giant], &[]);
    assert_eq!(t.life(P1), 20, "P0's own Giant deals no combat damage");
}

#[test]
fn knight_captain_of_eos_can_sacrifice_any_soldier() {
    cr!("118.3", "602.2");
    ruling!(
        "Knight-Captain of Eos",
        "You can sacrifice any Soldier to activate the second ability."
    );
    supported("Knight-Captain of Eos");
    let mut t = TestGame::new(2);
    let captain = t.battlefield(P0, "Knight-Captain of Eos");
    let vanguard = t.battlefield(P0, "Elite Vanguard");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Plains", 1);
    t.answer_choose(P0, &[obj(vanguard)]);
    t.activate(P0, captain, 0, &[]).expect("activate with a nontoken Soldier");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Elite Vanguard"));
    fight_it_out(&mut t, &[giant], &[]);
    assert_eq!(t.life(P1), 20);
}
