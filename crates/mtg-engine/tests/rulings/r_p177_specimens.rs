//! Rulings batch P177 — Gather Specimens (a replacement effect changing the controller a
//! creature enters under) and Aethersnatch (gaining control of a spell).

use crate::r_p177_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// It's P1's precombat main phase; P0 casts Gather Specimens (an instant) and it resolves.
fn specimens_on_p1s_turn() -> TestGame {
    supported("Gather Specimens");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P0, "Gather Specimens", &[]);
    t.set_step(P1, Step::PrecombatMain);
    t
}

#[test]
fn gather_specimens_takes_an_unearthed_creature_but_unearth_still_applies() {
    cr!("614.12", "702.84a");
    ruling!(
        "Gather Specimens",
        "Although Gather Specimens changes whose control the creature enters under, the rest of the effect works as normal."
    );
    let mut t = specimens_on_p1s_turn();
    let zombie = t.graveyard(P1, "Dregscape Zombie");
    t.lands(P1, "Swamp", 1);
    activate_containing(&mut t, P1, zombie, "Unearth").expect("unearth");
    t.resolve_all();
    let now = t.g.current(zombie);
    assert!(t.on_battlefield(now));
    assert_eq!(controller(&mut t, now), P0);
    assert!(t.obj_now(now).has_keyword(KeywordKind::Haste));
    // If it would leave the battlefield, it's exiled instead.
    destroy(&mut t, now);
    assert_eq!(t.zone(now), Zone::Exile);
    assert!(!t.in_graveyard(P1, "Dregscape Zombie"));
}

#[test]
fn gather_specimens_applies_first_so_you_choose_what_clone_copies() {
    cr!("614.12", "616.1");
    ruling!(
        "Gather Specimens",
        "The Gather Specimens replacement effect is applied before any other replacement effects that would also modify how the creature enters."
    );
    let mut t = specimens_on_p1s_turn();
    let serra = t.battlefield(P1, "Serra Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // P1 casts Clone; it enters under P0's control and P0 chooses the Bears to copy.
    give_mana_for(&mut t, P1, "Clone");
    let clone = t.hand(P1, "Clone");
    t.cast(P1, clone).go();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(serra)]);
    t.resolve_all();
    let now = t.g.current(clone);
    assert!(t.on_battlefield(now));
    assert_eq!(controller(&mut t, now), P0);
    assert_eq!(t.obj_now(now).chars.name.as_str(), "Grizzly Bears");
}

#[test]
fn two_gather_specimens_cancel_out() {
    cr!("614.5", "616.1");
    ruling!(
        "Gather Specimens",
        "if two opposing players have each cast Gather Specimens during the same turn and a creature would enter under the control of one of them, it really will enter under that player's control."
    );
    let mut t = specimens_on_p1s_turn();
    cast_resolve(&mut t, P1, "Gather Specimens", &[]);
    t.set_step(P1, Step::PrecombatMain);
    give_mana_for(&mut t, P1, "Grizzly Bears");
    let bears = t.hand(P1, "Grizzly Bears");
    t.cast(P1, bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(controller(&mut t, bears), P1);
}

#[test]
fn aethersnatch_can_target_a_spell_without_targets() {
    cr!("115.1", "110.2a");
    ruling!(
        "Aethersnatch",
        "Aethersnatch can target any spell, even one without targets."
    );
    supported("Aethersnatch");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    give_mana_for(&mut t, P1, "Grizzly Bears");
    let bears = t.hand(P1, "Grizzly Bears");
    let spell = t.cast(P1, bears).go();
    cast_resolve(&mut t, P0, "Aethersnatch", &[obj(spell)]);
    assert!(t.on_battlefield(bears));
    assert_eq!(controller(&mut t, bears), P0);
}

#[test]
fn aethersnatch_target_opponent_becomes_illegal_unless_changed() {
    cr!("115.7", "608.2b");
    ruling!(
        "Aethersnatch",
        "Notably, if you were originally chosen as a “target opponent” for a spell you gain control with Aethersnatch, you are now an illegal target as you aren’t your own opponent."
    );
    // P1 casts Duress ("Target opponent reveals their hand. You choose a noncreature,
    // nonland card from it. That player discards that card.") targeting P0. P0 takes it
    // and keeps the target: the spell doesn't resolve.
    for change in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        t.hand(P0, "Lightning Bolt");
        t.hand(P1, "Shock");
        give_mana_for(&mut t, P1, "Duress");
        let duress = t.hand(P1, "Duress");
        let spell = t.cast_with(P1, duress, &[Entity::Player(P0)]).unwrap();
        give_mana_for(&mut t, P0, "Aethersnatch");
        let snatch = t.hand(P0, "Aethersnatch");
        t.cast_with(P0, snatch, &[obj(spell)]).unwrap();
        t.answer_yes(P0, change);
        if change {
            t.answer_targets(P0, &[Entity::Player(P1)]);
        }
        t.resolve_all();
        if change {
            assert!(t.in_hand(P0, "Lightning Bolt"));
            assert!(t.in_graveyard(P1, "Shock"), "P1 discarded");
        } else {
            assert!(t.in_hand(P0, "Lightning Bolt"));
            assert!(t.in_hand(P1, "Shock"));
        }
        assert!(t.in_graveyard(P1, "Duress"), "change {change}");
    }
}

#[test]
fn aethersnatch_a_stolen_commanders_owner_chooses_the_command_zone() {
    cr!("903.9a", "903.9b");
    ruling!(
        "Aethersnatch",
        "If you gain control of a commander spell, the commander’s owner chooses whether to put it in the command zone if it later leaves the battlefield."
    );
    let mut t = crate::r_s13_common::commander_game();
    let giant = crate::r_s13_common::commander(&mut t, P1, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    give_mana_for(&mut t, P1, "Hill Giant");
    let spell = t.cast(P1, giant).go();
    cast_resolve(&mut t, P0, "Aethersnatch", &[obj(spell)]);
    assert!(t.on_battlefield(giant));
    assert_eq!(controller(&mut t, giant), P0);
    let from = t.asked().len();
    destroy(&mut t, giant);
    t.resolve_all();
    let asked = t.asked()[from..].to_vec();
    assert!(
        asked.iter().any(|(p, _)| *p == P1),
        "the owner was asked: {asked:?}"
    );
    assert!(asked.iter().all(|(p, _)| *p != P0), "{asked:?}");
    assert_eq!(t.zone(giant), Zone::Command);
}

#[test]
fn aethersnatch_commander_damage_counts_regardless_of_controller() {
    cr!("903.10a", "704.6c");
    ruling!(
        "Aethersnatch",
        "Combat damage dealt by a commander is tracked without regard to who controlled the commander at the time it dealt damage."
    );
    let mut t = TestGame::with_config(
        3,
        mtg_engine::game::GameConfig {
            variant: mtg_engine::game::Variant::Commander,
            ..mtg_engine::game::GameConfig::free_for_all()
        },
    );
    let giant = crate::r_s13_common::commander(&mut t, P1, "Hill Giant");
    // P1's commander already dealt P2 19 combat damage.
    t.g.players[P2.idx()]
        .commander_damage
        .insert("Hill Giant".into(), 19);
    t.set_step(P1, Step::PrecombatMain);
    give_mana_for(&mut t, P1, "Hill Giant");
    let spell = t.cast(P1, giant).go();
    cast_resolve(&mut t, P0, "Aethersnatch", &[obj(spell)]);
    assert_eq!(controller(&mut t, giant), P0);
    // On P0's turn, P0 attacks P2 with it: 22 commander damage.
    t.set_step(P0, Step::PrecombatMain);
    let now = t.g.current(giant);
    t.g.objects[now.0 as usize].summoning_sick = false;
    t.attack(&[(now, Entity::Player(P2))], &[]);
    assert!(t.has_lost(P2));
}
