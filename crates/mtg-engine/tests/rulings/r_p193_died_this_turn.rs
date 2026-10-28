//! Rulings batch P193 — abilities that look back at what died or was put into a graveyard
//! from the battlefield this turn (CR 700.4, 603.10): what counts (tokens, noncreature
//! cards that were creatures, cards that have since left the graveyard), what doesn't
//! (creatures exiled instead, countered spells), and when it's counted.

use crate::r_s01_common::*;
use crate::r_s02_common::{create_token, destroy, target_candidates};
use crate::r_s05_common::move_to;
use crate::r_s13_common::{commander, commander_game};
use crate::r_s18_common::exiled_named;
use crate::r_s29_common::{cast_and_resolve, put_counters};
use crate::r_s31_common::put_together;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A 1/1 creature token for `p` that then dies.
fn token_dies(t: &mut TestGame, p: PlayerId) {
    let tok = create_token(t, p, "Soldier");
    destroy(t, tok);
}

#[test]
fn bloodcrazed_paladin_counts_tokens_animated_lands_and_cards_gone_from_graveyards() {
    cr!("700.4", "603.10a", "614.1c");
    ruling!(
        "Bloodcrazed Paladin",
        "including token creatures and noncreature cards that were creatures as they left the battlefield. It doesn't check whether any of them are still in graveyards."
    );
    supported("Bloodcrazed Paladin");
    supported("Living Lands");
    let mut t = TestGame::new(2);
    token_dies(&mut t, P1);
    let living = t.battlefield(P0, "Living Lands");
    let forest = t.battlefield(P0, "Forest");
    destroy(&mut t, forest);
    destroy(&mut t, living);
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    move_to(&mut t, bears, Zone::Exile);
    let paladin = t.enter(P0, "Bloodcrazed Paladin");
    t.settle();
    assert_eq!(t.counters(paladin, counters::PLUS1), 3);
}

#[test]
fn bloodcrazed_paladin_doesnt_count_creatures_exiled_instead() {
    cr!("700.4", "614.6");
    ruling!(
        "Bloodcrazed Paladin",
        "won't count creatures that were put directly into a zone other than the graveyard, perhaps because of a replacement effect"
    );
    supported("Rest in Peace");
    let mut t = TestGame::new(2);
    let rip = t.battlefield(P1, "Rest in Peace");
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    assert_eq!(exiled_named(&t, "Grizzly Bears").len(), 1);
    move_to(&mut t, rip, Zone::Hand(P1));
    let paladin = t.enter(P0, "Bloodcrazed Paladin");
    t.settle();
    assert_eq!(t.counters(paladin, counters::PLUS1), 0);
}

#[test]
fn canopy_stalker_counts_itself_and_creatures_that_die_in_response() {
    cr!("700.4", "603.10a", "608.2h");
    ruling!(
        "Canopy Stalker",
        "Canopy Stalker's triggered ability counts itself, as well as any other creatures that die before the triggered ability resolves."
    );
    supported("Canopy Stalker");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    let stalker = t.battlefield(P0, "Canopy Stalker");
    destroy(&mut t, bears);
    destroy(&mut t, stalker);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, other);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}

#[test]
fn canopy_stalkers_controller_loses_before_its_ability_goes_on_the_stack() {
    cr!("704.3", "704.5a", "104.3b");
    ruling!(
        "Canopy Stalker",
        "If your life total is brought to 0 or less at the same time that Canopy Stalker is dealt lethal damage, you lose the game before its last ability goes on the stack."
    );
    let mut t = TestGame::new(2);
    let stalker = t.battlefield(P0, "Canopy Stalker");
    let src = t.battlefield(P1, "Grizzly Bears");
    t.g.players[0].life = 2;
    t.g.deal_damage_batch(
        vec![(src, Entity::Player(P0), 2), (src, Entity::Object(stalker), 2)],
        false,
    );
    t.g.flush_events();
    t.settle();
    assert!(t.has_lost(P0));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P0), 0);
}

#[test]
fn death_rattle_oni_counts_tokens_and_cards_that_left_the_graveyard() {
    cr!("700.4", "601.2f");
    ruling!(
        "Death-Rattle Oni",
        "Count any creature that died during the turn, including token creatures, when calculating the cost reduction. For nontoken creatures that died, it doesn't matter what happened to the creature cards after they died."
    );
    supported("Death-Rattle Oni");
    let mut t = TestGame::new(2);
    token_dies(&mut t, P1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    move_to(&mut t, bears, Zone::Hand(P0));
    // {6}{B} minus {4}.
    t.lands(P0, "Swamp", 3);
    let oni = t.hand(P0, "Death-Rattle Oni");
    t.cast(P0, oni).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Death-Rattle Oni").len(), 1);
}

#[test]
fn cry_of_the_carnarium_shrinks_only_current_creatures_and_exiles_every_creature_that_dies() {
    cr!("611.2c", "614.1a", "608.2c");
    ruling!(
        "Cry of the Carnarium",
        "Creatures that would die after getting -2/-2 will still be on the battlefield when cards are exiled from graveyards. They won’t die, and will instead be exiled after Cry of the Carnarium is entirely finished resolving."
    );
    ruling!(
        "Cry of the Carnarium",
        "Creatures that enter the battlefield later in the turn won’t get -2/-2."
    );
    ruling!(
        "Cry of the Carnarium",
        "Cry of the Carnarium’s replacement effect affects any creature that would die in the turn, even creatures not on the battlefield as it resolves."
    );
    supported("Cry of the Carnarium");
    let mut t = TestGame::new(2);
    let courser = t.battlefield(P1, "Centaur Courser");
    destroy(&mut t, courser);
    // A creature card put into the graveyard some other way stays.
    t.graveyard(P1, "Hill Giant");
    t.battlefield(P1, "Grizzly Bears");
    let big = t.battlefield(P1, "Craw Wurm");
    cast_and_resolve(&mut t, P0, "Cry of the Carnarium", &[]);
    assert_eq!(exiled_named(&t, "Centaur Courser").len(), 1);
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(exiled_named(&t, "Grizzly Bears").len(), 1);
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.pt(big), (4, 2));
    let later = t.enter(P1, "Centaur Courser");
    t.g.recompute();
    assert_eq!(t.pt(later), (3, 3));
    destroy(&mut t, later);
    assert_eq!(exiled_named(&t, "Centaur Courser").len(), 2);
}

#[test]
fn flesh_allergy_counts_every_creature_that_died_and_needs_its_target() {
    cr!("700.4", "608.2b", "702.12b");
    ruling!(
        "Flesh Allergy",
        "Flesh Allergy counts each creature put into a graveyard from the battlefield that turn, including token creatures, the creature sacrificed to cast the spell, and the targeted creature (if it is destroyed)."
    );
    ruling!(
        "Flesh Allergy",
        "If Flesh Allergy resolves but the targeted creature is not destroyed (because it has indestructible or enchanted by an Aura with umbra armor, for example), its controller will still lose life."
    );
    ruling!(
        "Flesh Allergy",
        "If the targeted creature is an illegal target by the time Flesh Allergy resolves, the spell doesn't resolve. No player will lose life."
    );
    supported("Flesh Allergy");
    supported("Darksteel Myr");
    for (target, want) in [("Grizzly Bears", 17), ("Darksteel Myr", 18), ("gone", 20)] {
        let mut t = TestGame::new(2);
        token_dies(&mut t, P0);
        let fodder = t.battlefield(P0, "Llanowar Elves");
        let victim = t.battlefield(P1, if target == "gone" { "Grizzly Bears" } else { target });
        t.lands(P0, "Swamp", 4);
        let fa = t.hand(P0, "Flesh Allergy");
        t.answer_choose(P0, &[Entity::Object(fodder)]);
        t.cast(P0, fa).target(victim).go();
        assert!(t.in_graveyard(P0, "Llanowar Elves"), "{target}");
        if target == "gone" {
            move_to(&mut t, victim, Zone::Hand(P1));
        }
        t.resolve_all();
        assert_eq!(t.life(P1), want, "{target}");
        assert_eq!(t.life(P0), 20);
    }
}

#[test]
fn force_of_despair_destroys_what_entered_this_turn_and_is_a_creature_now() {
    cr!("700.4", "603.10");
    ruling!(
        "Force of Despair",
        "Force of Despair destroys all permanents that entered the battlefield this turn and are currently creatures. It doesn’t matter whether they were creatures as they entered the battlefield."
    );
    supported("Force of Despair");
    let mut t = TestGame::new(2);
    let forest = t.enter(P1, "Forest");
    let bears = t.enter(P1, "Grizzly Bears");
    let living = t.enter(P1, "Living Lands");
    let old = t.battlefield(P1, "Centaur Courser");
    // `battlefield` puts it there this turn; make it a permanent from an earlier turn.
    t.g.objects[old.0 as usize].entered_turn = 0;
    let island = t.enter(P1, "Island");
    t.g.recompute();
    cast_and_resolve(&mut t, P0, "Force of Despair", &[]);
    assert!(!t.on_battlefield(forest));
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(living));
    assert!(t.on_battlefield(old));
    assert!(t.on_battlefield(island));
}

#[test]
fn fresh_meat_counts_creatures_put_into_your_graveyard() {
    cr!("700.4", "111.8", "400.3");
    ruling!(
        "Fresh Meat",
        "Fresh Meat counts all creatures you owned that went to the graveyard this turn, including creature tokens that entered under your control and nontoken creatures you owned but didn't control."
    );
    supported("Fresh Meat");
    let mut t = TestGame::new(2);
    token_dies(&mut t, P0);
    let mine = t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::give_control(&mut t, mine, P1);
    destroy(&mut t, mine);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    let theirs = t.battlefield(P1, "Centaur Courser");
    destroy(&mut t, theirs);
    token_dies(&mut t, P1);
    cast_and_resolve(&mut t, P0, "Fresh Meat", &[]);
    assert_eq!(
        crate::r_s05_common::tokens_with_subtype(&t, P0, "Beast").len(),
        2
    );
}

#[test]
fn gadrak_counts_creatures_that_died_before_it_entered() {
    cr!("700.4", "603.2");
    ruling!(
        "Gadrak, the Crown-Scourge",
        "Gadrak's last ability counts creatures that died during the entire turn, even ones that died before Gadrak entered the battlefield."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    token_dies(&mut t, P1);
    t.enter(P0, "Gadrak, the Crown-Scourge");
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Treasure").len(), 1);
}

#[test]
fn rise_of_the_dread_marn_counts_nontoken_creatures_even_if_their_cards_left() {
    cr!("700.4");
    ruling!(
        "Rise of the Dread Marn",
        "It doesn’t matter if the cards that represented those creatures are still in the graveyard as Rise of the Dread Marn resolves."
    );
    supported("Rise of the Dread Marn");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    move_to(&mut t, bears, Zone::Exile);
    let other = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, other);
    token_dies(&mut t, P1);
    cast_and_resolve(&mut t, P0, "Rise of the Dread Marn", &[]);
    assert_eq!(
        crate::r_s05_common::tokens_with_subtype(&t, P0, "Zombie").len(),
        2
    );
}

#[test]
fn feast_of_the_victorious_dead_needs_a_death_to_trigger_and_counts_on_resolution() {
    cr!("603.4", "608.2h");
    ruling!(
        "Feast of the Victorious Dead",
        "If no creatures have died by the time your end step begins, the ability of Feast of the Victorious Dead won't trigger at all. Provided the ability does trigger, it will count the number of creatures that have died as the ability resolves."
    );
    supported("Feast of the Victorious Dead");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Feast of the Victorious Dead");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let victim = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    destroy(&mut t, victim);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // A death before the end step: it triggers, and a death in response counts too.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Feast of the Victorious Dead");
    let bears2 = t.battlefield(P0, "Grizzly Bears");
    let v1 = t.battlefield(P1, "Grizzly Bears");
    let v2 = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, v1);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, v2);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.counters(bears2, counters::PLUS1), 2);
    let _ = bears;
}

#[test]
fn gorma_doesnt_add_counters_to_creatures_entering_with_it() {
    cr!("614.12", "614.1c");
    ruling!(
        "Gorma, the Gullet",
        "In the case where Gorma and one or more other creatures enter at the same time, those creatures won't enter with additional +1/+1 counters"
    );
    supported("Gorma, the Gullet");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    let gorma = t.graveyard(P0, "Gorma, the Gullet");
    let other = t.graveyard(P0, "Centaur Courser");
    let ids = put_together(&mut t, &[gorma, other]);
    let other = ids[1].unwrap();
    assert_eq!(t.counters(other, counters::PLUS1), 0);
    let later = t.enter(P0, "Centaur Courser");
    assert_eq!(t.counters(later, counters::PLUS1), 1);
}

#[test]
fn gerrard_returns_cards_even_if_it_left_the_graveyard_or_they_died_first() {
    cr!("603.10a", "608.2b", "700.4");
    ruling!(
        "Gerrard, Weatherlight Hero",
        "Gerrard's triggered ability returns artifact and creature cards even if Gerrard leaves your graveyard before you can exile it."
    );
    ruling!(
        "Gerrard, Weatherlight Hero",
        "Gerrard's triggered ability returns artifact and creature cards even if they were put into your graveyard while Gerrard wasn't on the battlefield, so long as they were put there this turn."
    );
    supported("Gerrard, Weatherlight Hero");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    let ring = t.battlefield(P0, "Sol Ring");
    destroy(&mut t, ring);
    let anthem = t.battlefield(P0, "Glorious Anthem");
    destroy(&mut t, anthem);
    // A creature card discarded this turn wasn't put there from the battlefield.
    let discarded = t.hand(P0, "Centaur Courser");
    move_to(&mut t, discarded, Zone::Graveyard(P0));
    let gerrard = t.battlefield(P0, "Gerrard, Weatherlight Hero");
    destroy(&mut t, gerrard);
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, gerrard, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Sol Ring").len(), 1);
    assert!(t.in_graveyard(P0, "Glorious Anthem"));
    assert!(t.in_graveyard(P0, "Centaur Courser"));
    assert!(t.in_hand(P0, "Gerrard, Weatherlight Hero"));
}

#[test]
fn gerrard_as_commander_can_still_go_to_the_command_zone_after_exiling_itself() {
    cr!("903.9a", "608.2c");
    ruling!(
        "Gerrard, Weatherlight Hero",
        "If Gerrard is your commander and you can exile it from your graveyard as its triggered ability resolves, you'll still be able to put it into the command zone after you return your artifact and creature cards to the battlefield."
    );
    let mut t = commander_game();
    let cmd = commander(&mut t, P0, "Gerrard, Weatherlight Hero");
    let gerrard = move_to(&mut t, cmd, Zone::Battlefield).unwrap();
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    // Leave it in the graveyard when it dies.
    t.answer_yes(P0, false);
    destroy(&mut t, gerrard);
    assert!(t.in_graveyard(P0, "Gerrard, Weatherlight Hero"));
    // Then move it to the command zone once it's exiled.
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(
        t.g.find_in_zone(Zone::Command, "Gerrard, Weatherlight Hero")
            .len(),
        1
    );
    assert!(exiled_named(&t, "Gerrard, Weatherlight Hero").is_empty());
}

#[test]
fn faiths_reward_returns_each_kind_of_permanent_card_from_the_battlefield() {
    cr!("110.4", "700.4");
    ruling!(
        "Faith's Reward",
        "A permanent card is a card that could be put onto the battlefield. Specifically, it means an artifact, creature, enchantment, land, or planeswalker card"
    );
    supported("Faith's Reward");
    let mut t = TestGame::new(2);
    for name in [
        "Sol Ring",
        "Grizzly Bears",
        "Glorious Anthem",
        "Forest",
        "Jace Beleren",
    ] {
        let id = t.battlefield(P0, name);
        destroy(&mut t, id);
    }
    // Not a card put there from the battlefield.
    let milled = t.hand(P0, "Hill Giant");
    move_to(&mut t, milled, Zone::Graveyard(P0));
    cast_and_resolve(&mut t, P0, "Faith's Reward", &[]);
    for name in [
        "Sol Ring",
        "Grizzly Bears",
        "Glorious Anthem",
        "Forest",
        "Jace Beleren",
    ] {
        assert_eq!(t.named_on_battlefield(name).len(), 1, "{name}");
    }
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn anzrags_rampage_counts_every_artifact_put_into_a_graveyard_this_turn() {
    cr!("700.4", "111.8");
    ruling!(
        "Anzrag's Rampage",
        "Anzrag's Rampage counts the number of artifacts that were put into graveyards from the battlefield this turn for any reason, not just ones that were destroyed by Anzrag's Rampage. This includes any artifact tokens"
    );
    supported("Anzrag's Rampage");
    let mut t = TestGame::new(2);
    let treasure = create_token(&mut t, P0, "Treasure");
    destroy(&mut t, treasure);
    t.battlefield(P1, "Sol Ring");
    t.battlefield(P0, "Sol Ring");
    let before = t.library_size(P0);
    cast_and_resolve(&mut t, P0, "Anzrag's Rampage", &[]);
    assert_eq!(t.library_size(P0), before - 2);
    assert!(t.in_graveyard(P1, "Sol Ring"));
    assert_eq!(t.named_on_battlefield("Sol Ring").len(), 1);
}

#[test]
fn thrilling_encore_returns_only_creature_cards_that_were_on_the_battlefield() {
    cr!("700.4", "603.10");
    ruling!(
        "Thrilling Encore",
        "Cards that aren't creature cards won't be returned to the battlefield, even if an effect made those cards creatures when they were last on the battlefield."
    );
    ruling!(
        "Thrilling Encore",
        "Creature spells that were countered never entered the battlefield, so Thrilling Encore won't return them to the battlefield."
    );
    supported("Thrilling Encore");
    let mut t = TestGame::new(2);
    let living = t.battlefield(P1, "Living Lands");
    let forest = t.battlefield(P1, "Forest");
    destroy(&mut t, forest);
    move_to(&mut t, living, Zone::Hand(P1));
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    // A countered creature spell (cast in P1's turn).
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 4);
    let wurm = t.hand(P1, "Centaur Courser");
    t.cast(P1, wurm).go();
    t.lands(P0, "Island", 2);
    let cs = t.hand(P0, "Counterspell");
    let spell = t.g.stack[0];
    t.cast(P0, cs).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Centaur Courser"));
    cast_and_resolve(&mut t, P0, "Thrilling Encore", &[]);
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(t.obj(bears[0]).controller, P0);
    assert!(t.in_graveyard(P1, "Forest"));
    assert!(t.in_graveyard(P1, "Centaur Courser"));
}

#[test]
fn grim_return_cant_target_a_countered_creature_card() {
    cr!("700.4", "115.1");
    ruling!(
        "Grim Return",
        "Creature spells that are countered never entered the battlefield, so those cards can’t be chosen as the target of Grim Return."
    );
    supported("Grim Return");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 3);
    let courser = t.hand(P1, "Centaur Courser");
    t.cast(P1, courser).go();
    t.lands(P0, "Island", 2);
    let cs = t.hand(P0, "Counterspell");
    let spell = t.g.stack[0];
    t.cast(P0, cs).target(spell).go();
    t.resolve_all();
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    let from = t.asked().len();
    let bears_card = t.g.current(bears);
    cast_and_resolve(&mut t, P0, "Grim Return", &[Entity::Object(bears_card)]);
    assert_eq!(
        target_candidates(&t, P0, from)[0],
        vec![Entity::Object(bears_card)]
    );
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn salvager_of_ruin_cant_target_itself() {
    cr!("602.2b", "601.2c");
    ruling!(
        "Salvager of Ruin",
        "Because targets are chosen for activated abilities before costs (such as “Sacrifice Salvager of Ruin”) are paid, Salvager of Ruin’s ability can’t target itself."
    );
    supported("Salvager of Ruin");
    let mut t = TestGame::new(2);
    let salvager = t.battlefield(P0, "Salvager of Ruin");
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    let bears_card = t.g.current(bears);
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(bears_card)]);
    t.activate(P0, salvager, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(
        target_candidates(&t, P0, from)[0],
        vec![Entity::Object(bears_card)]
    );
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Salvager of Ruin"));
}

#[test]
fn structural_assault_removes_a_shield_counter_before_dealing_damage() {
    cr!("122.1c", "608.2c");
    ruling!(
        "Structural Assault",
        "a shield counter will be removed before Structural Assault deals damage to each creature. Unless that artifact creature has another shield counter on it, the damage won't be prevented."
    );
    supported("Structural Assault");
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Memnite");
    put_counters(&mut t, myr, "shield", 1);
    t.battlefield(P1, "Sol Ring");
    let shielded_twice = t.battlefield(P1, "Ornithopter");
    put_counters(&mut t, shielded_twice, "shield", 2);
    cast_and_resolve(&mut t, P0, "Structural Assault", &[]);
    assert!(t.in_graveyard(P1, "Sol Ring"));
    assert!(t.in_graveyard(P1, "Memnite"));
    assert!(t.on_battlefield(shielded_twice));
    assert_eq!(t.counters(shielded_twice, "shield"), 0);
}
