//! Rulings batch P190 — cards with unique counters: triggers and intervening "if" clauses
//! that check them (CR 603.4), last known information (CR 608.2h), state triggers
//! (CR 603.8), and counters that are only memory aids.

use crate::r_p076_common::mana;
use crate::r_p190_mana_costs::only_unsupported;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s05_common::{enter, move_to};
use crate::r_s06_common::attach_new;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn put(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    t.g.add_counters(obj(t.g.current(id)), kind, n, None);
    t.g.flush_events();
    t.g.recompute();
}

/// From P1's end step to P0's upkeep, with P0's upkeep triggers on the stack.
fn to_upkeep(t: &mut TestGame) {
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
}

#[test]
fn living_artifact_counts_damage_not_life_loss() {
    cr!("120.3a", "119.3");
    ruling!(
        "Living Artifact",
        "Does not trigger on loss of life, just on damage."
    );
    supported("Living Artifact");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let la = attach_new(&mut t, P0, "Living Artifact", thopter);
    t.g.lose_life(P0, 3);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(la, "vitality"), 0);
    let bolt_src = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(bolt_src, Entity::Player(P0), 2, false);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(la, "vitality"), 2);
}

#[test]
fn living_artifact_on_an_opponents_artifact_is_controlled_by_the_auras_controller() {
    cr!("303.4", "603.3a");
    ruling!(
        "Living Artifact",
        "You can cast it targeting your opponent's artifacts. The controller of the Aura (not the controller of the artifact) controls the Living Artifact ability."
    );
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Ornithopter");
    let la = t.hand(P0, "Living Artifact");
    mana(&mut t, P0, ManaType::G, 1);
    t.cast(P0, la).target(theirs).go();
    t.resolve_all();
    assert_eq!(t.obj_now(la).attached_to, Some(obj(theirs)));
    // Damage dealt to P0 (the Aura's controller), not P1, puts counters on it.
    let src = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(src, Entity::Player(P1), 2, false);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(la, "vitality"), 0);
    t.g.deal_damage(src, Entity::Player(P0), 2, false);
    t.g.flush_events();
    t.settle();
    let trig = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(trig).controller, P0);
    t.resolve_all();
    assert_eq!(t.counters(la, "vitality"), 2);
    // At P0's upkeep, P0 removes one and gains 1 life.
    to_upkeep(&mut t);
    let life = t.life(P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.counters(la, "vitality"), 1);
    assert_eq!(t.life(P0), life + 1);
}

#[test]
fn bogardan_phoenix_returns_then_is_exiled() {
    cr!("603.6c", "700.4", "400.7");
    ruling!(
        "Bogardan Phoenix",
        "Errata streamlined the trigger and got rid of the replacement effect. In both cases, it will go to the graveyard. In one it will then come back onto the battlefield; in the other it will then be exiled."
    );
    supported("Bogardan Phoenix");
    let mut t = TestGame::new(2);
    let phoenix = t.battlefield(P0, "Bogardan Phoenix");
    t.g.destroy(phoenix, None);
    t.g.flush_events();
    t.settle();
    // It went to the graveyard (the trigger is waiting).
    assert_eq!(t.zone(phoenix), Zone::Graveyard(P0));
    t.resolve_all();
    assert!(t.on_battlefield(phoenix));
    assert_eq!(t.counters(phoenix, "death"), 1);
    t.g.destroy(t.g.current(phoenix), None);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.zone(phoenix), Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.zone(phoenix), Zone::Exile);
}

#[test]
fn helix_pinnacle_intervening_if() {
    cr!("603.4", "608.2h");
    ruling!(
        "Helix Pinnacle",
        "Helix Pinnacle’s triggered ability has an “intervening ‘if’ clause.”"
    );
    ruling!(
        "Helix Pinnacle",
        "If Helix Pinnacle leaves the battlefield before its triggered ability resolves, check the number of counters that were on it as it last existed on the battlefield."
    );
    supported("Helix Pinnacle");
    // 99 counters as the upkeep begins: no trigger, even if a 100th is added then.
    let mut t = TestGame::new(2);
    let hp = t.battlefield(P0, "Helix Pinnacle");
    put(&mut t, hp, "tower", 99);
    to_upkeep(&mut t);
    assert_eq!(t.stack_len(), 0);
    put(&mut t, hp, "tower", 1);
    t.resolve_all();
    assert!(!t.has_lost(P1));
    // 100 counters: it triggers, but loses one before it resolves: nothing happens.
    let mut t = TestGame::new(2);
    let hp = t.battlefield(P0, "Helix Pinnacle");
    put(&mut t, hp, "tower", 100);
    to_upkeep(&mut t);
    assert_eq!(t.stack_len(), 1);
    t.g.remove_counters(obj(hp), "tower", 1);
    t.resolve_all();
    assert!(!t.has_lost(P1));
    // 100 counters and it leaves the battlefield: its last known counters count.
    let mut t = TestGame::new(2);
    let hp = t.battlefield(P0, "Helix Pinnacle");
    put(&mut t, hp, "tower", 100);
    to_upkeep(&mut t);
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, hp, Zone::Hand(P0));
    t.resolve_all();
    assert!(t.has_lost(P1));
}

#[test]
fn gemstone_mine_isnt_sacrificed_if_its_last_counter_is_removed_otherwise() {
    cr!("602.5", "122.1");
    ruling!(
        "Gemstone Mine",
        "If the last mining counter is removed from Gemstone Mine in some way other than activating its ability, Gemstone Mine won’t be sacrificed (and its ability can’t be activated)."
    );
    supported("Gemstone Mine");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Gemstone Mine");
    put(&mut t, mine, "mining", 1);
    t.g.remove_counters(obj(mine), "mining", 1);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.on_battlefield(mine));
    assert!(t.activate(P0, mine, 0, &[]).is_err());
    // With its last counter removed by its own ability, it's sacrificed.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Gemstone Mine");
    put(&mut t, mine, "mining", 1);
    t.activate(P0, mine, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(mine));
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
}

#[test]
fn a_non_dfc_copy_of_edgar_doesnt_return() {
    cr!("712.14a", "707.2");
    ruling!(
        "Edgar, Charmed Groom // Edgar Markov's Coffin",
        "If a card that isn't a double-faced card is a copy of Edgar, Charmed Groom, it won't return to the battlefield when it dies."
    );
    supported("Edgar, Charmed Groom // Edgar Markov's Coffin");
    let mut t = TestGame::new(2);
    let edgar = t.battlefield(P1, "Edgar, Charmed Groom // Edgar Markov's Coffin");
    let clone = t.hand(P0, "Clone");
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 3);
    t.cast(P0, clone).go();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(edgar)]);
    t.resolve_all();
    assert!(t.on_battlefield(clone));
    assert_eq!(t.obj_now(clone).chars.name, "Edgar, Charmed Groom");
    t.g.destroy(t.g.current(clone), None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.zone(clone), Zone::Graveyard(P0));
    // The real Edgar returns transformed.
    t.g.destroy(edgar, None);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.on_battlefield(edgar));
    assert_eq!(t.obj_now(edgar).chars.name, "Edgar Markov's Coffin");
}

#[test]
fn hoofprints_of_the_stag_triggers_per_card_drawn() {
    cr!("121.2", "603.2c");
    ruling!(
        "Hoofprints of the Stag",
        "If a spell or ability has you draw multiple cards, Hoofprints of the Stag's ability triggers that many times."
    );
    supported("Hoofprints of the Stag");
    let mut t = TestGame::new(2);
    let hp = t.battlefield(P0, "Hoofprints of the Stag");
    let div = t.hand(P0, "Divination");
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 2);
    t.cast(P0, div).go();
    t.resolve();
    assert_eq!(t.stack_len(), 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.counters(hp, "hoofprint"), 2);
}

#[test]
fn illicit_masquerade_any_impostor_counter() {
    cr!("603.10a", "122.1");
    ruling!(
        "Illicit Masquerade",
        "Illicit Masquerade's last ability affects all creatures you control with impostor counters on them, not just ones that had impostor counters put on them with Illicit Masquerade's second ability."
    );
    supported("Illicit Masquerade");
    let mut t = TestGame::new(2);
    // On the battlefield without its enters ability; a counter put by something else.
    t.battlefield(P0, "Illicit Masquerade");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, bears, "impostor", 1);
    let other = t.graveyard(P0, "Raging Goblin");
    t.answer_targets(P0, &[obj(other)]);
    t.g.destroy(bears, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(t.on_battlefield(other));
}

#[test]
fn arixmethes_as_a_land_and_attacking_after_waking() {
    cr!("205.1b", "302.6", "202.3");
    ruling!(
        "Arixmethes, Slumbering Isle",
        "While Arixmethes is a land, it's still green and blue, it's still legendary, and its mana value is still 4."
    );
    ruling!(
        "Arixmethes, Slumbering Isle",
        "Once Arixmethes is a creature again, it can attack on the same turn as long as you've controlled it since your most recent turn began."
    );
    supported("Arixmethes, Slumbering Isle");
    let mut t = TestGame::new(2);
    let ax = t.battlefield(P0, "Arixmethes, Slumbering Isle");
    put(&mut t, ax, "slumber", 1);
    let o = t.obj(ax);
    assert!(o.is(CardType::Land) && !o.is(CardType::Creature));
    assert!(o.chars.colors.contains(Color::Green) && o.chars.colors.contains(Color::Blue));
    assert!(o.chars.supertypes.contains(Supertype::Legendary));
    assert_eq!(t.g.mana_value_of(ax), 4);
    // Casting a spell wakes it; it has been under P0's control since the turn began.
    let opt = t.hand(P0, "Opt");
    mana(&mut t, P0, ManaType::U, 1);
    t.answer_yes(P0, true);
    t.cast(P0, opt).go();
    t.resolve_all();
    assert!(t.obj(ax).is(CardType::Creature));
    attack_with(&mut t, &[(ax, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 8);
}

#[test]
fn phylactery_lich_with_no_artifacts_is_sacrificed() {
    cr!("614.12", "603.8");
    ruling!(
        "Phylactery Lich",
        "If you control no artifacts as Phylactery Lich enters the battlefield, its first ability won’t do anything. As soon as it enters the battlefield, its last ability will trigger"
    );
    supported("Phylactery Lich");
    let mut t = TestGame::new(2);
    let lich = enter(&mut t, P0, "Phylactery Lich");
    t.resolve_all();
    assert_eq!(t.zone(lich), Zone::Graveyard(P0));
    // With an artifact, it gets the counter as the Lich enters: nothing triggers.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ornithopter");
    let lich = enter(&mut t, P0, "Phylactery Lich");
    assert_eq!(t.counters(a, "phylactery"), 1);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert!(t.on_battlefield(lich));
}

#[test]
fn phylactery_lich_chooses_an_artifact_already_on_the_battlefield_without_targeting() {
    cr!("614.12a", "115.1", "702.18a");
    ruling!(
        "Phylactery Lich",
        "If Phylactery Lich and an artifact are entering the battlefield under your control at the same time, you can’t put a phylactery counter on that artifact."
    );
    ruling!(
        "Phylactery Lich",
        "Phylactery Lich’s first ability doesn’t target the artifact."
    );
    supported("Eerie Ultimatum");
    // Returned together with Ornithopter by Eerie Ultimatum: the Ornithopter can't get
    // the counter, so with no other artifact the Lich is sacrificed.
    let return_both = |t: &mut TestGame| {
        let lich = t.graveyard(P0, "Phylactery Lich");
        let thopter = t.graveyard(P0, "Ornithopter");
        mana(t, P0, ManaType::W, 2);
        mana(t, P0, ManaType::B, 3);
        mana(t, P0, ManaType::G, 2);
        let c = t.hand(P0, "Eerie Ultimatum");
        t.answer_choose(P0, &[obj(lich), obj(thopter)]);
        t.cast(P0, c).go();
        t.resolve_all();
        (lich, thopter)
    };
    let mut t = TestGame::new(2);
    let (lich, thopter) = return_both(&mut t);
    assert!(t.on_battlefield(thopter));
    assert_eq!(t.counters(thopter, "phylactery"), 0);
    assert_eq!(t.zone(lich), Zone::Graveyard(P0));
    // An artifact already on the battlefield gets it.
    let mut t = TestGame::new(2);
    let memnite = t.battlefield(P0, "Memnite");
    let (lich, thopter) = return_both(&mut t);
    assert_eq!(t.counters(thopter, "phylactery"), 0);
    assert_eq!(t.counters(memnite, "phylactery"), 1);
    assert!(t.on_battlefield(lich));
    // Not targeted: an artifact with shroud can be chosen.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    attach_new(&mut t, P0, "Lightning Greaves", thopter);
    let greaves_too = t.g.find_in_zone(Zone::Battlefield, "Lightning Greaves")[0];
    assert!(t.obj(thopter).has_keyword(KeywordKind::Shroud));
    t.answer_choose(P0, &[obj(thopter)]);
    let lich = enter(&mut t, P0, "Phylactery Lich");
    assert_eq!(t.counters(thopter, "phylactery"), 1);
    assert_eq!(t.counters(greaves_too, "phylactery"), 0);
    t.resolve_all();
    assert!(t.on_battlefield(lich));
}

#[test]
fn phylactery_lich_checks_any_phylactery_counter() {
    cr!("603.8");
    ruling!(
        "Phylactery Lich",
        "Phylactery Lich’s last ability checks your permanents for any phylactery counters, not just the specific one that it caused you to put on an artifact."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ornithopter");
    let b = t.battlefield(P0, "Memnite");
    put(&mut t, a, "phylactery", 1);
    put(&mut t, b, "phylactery", 1);
    let l1 = t.battlefield(P0, "Phylactery Lich");
    let l2 = t.battlefield(P0, "Phylactery Lich");
    move_to(&mut t, a, Zone::Hand(P0));
    t.resolve_all();
    assert!(t.on_battlefield(l1) && t.on_battlefield(l2));
    // The last one leaves: both are sacrificed.
    move_to(&mut t, b, Zone::Hand(P0));
    t.resolve_all();
    assert!(!t.on_battlefield(l1) && !t.on_battlefield(l2));
}

#[test]
fn phylactery_lich_state_trigger_retriggers_if_countered() {
    cr!("603.8");
    ruling!(
        "Phylactery Lich",
        "Phylactery Lich’s last ability is a “state trigger.” Once a state trigger triggers, it won’t trigger again as long as the ability is on the stack. If the ability is countered and the trigger condition is still true, it will immediately trigger again."
    );
    let mut t = TestGame::new(2);
    let lich = t.battlefield(P0, "Phylactery Lich");
    t.g.recompute();
    t.settle();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let trig = t.g.stack[0];
    let stifle = t.hand(P1, "Stifle");
    mana(&mut t, P1, ManaType::U, 1);
    t.cast(P1, stifle).target(trig).go();
    t.resolve();
    // Stifle resolved and countered the trigger; it triggered again.
    assert!(t.in_graveyard(P1, "Stifle"));
    assert!(t.on_battlefield(lich));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(lich));
}

#[test]
fn tetzimoc_destroys_creatures_with_any_prey_counter() {
    cr!("122.1");
    ruling!(
        "Tetzimoc, Primal Death",
        "Tetzimoc's triggered ability doesn't care how a prey counter got onto a creature an opponent controls or whose Tetzimoc put that counter on the creature."
    );
    only_unsupported("Tetzimoc, Primal Death", "Reveal ~ from your hand");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let clean = t.battlefield(P1, "Raging Goblin");
    let mine = t.battlefield(P0, "Memnite");
    // Prey counters P1 put on their own creature, and on P0's.
    t.g.add_counters(obj(theirs), "prey", 1, None);
    t.g.add_counters(obj(mine), "prey", 1, None);
    t.g.flush_events();
    enter(&mut t, P0, "Tetzimoc, Primal Death");
    t.resolve_all();
    assert!(!t.on_battlefield(theirs));
    assert!(t.on_battlefield(clean));
    assert!(t.on_battlefield(mine));
}

#[test]
fn book_of_exalted_deeds_angel_keeps_the_ability() {
    cr!("611.2c", "122.1", "104.3b");
    ruling!(
        "The Book of Exalted Deeds",
        "The Angel won't lose the ability it gains if it stops being an Angel after the ability resolves."
    );
    ruling!(
        "The Book of Exalted Deeds",
        "The enlightenment counter placed by the last ability serves as a memory aid. It isn't connected to the granted ability. The Angel keeps that ability even if it loses the counter."
    );
    supported("The Book of Exalted Deeds");
    supported("Imagecrafter");
    let mut t = TestGame::new(2);
    let book = t.battlefield(P0, "The Book of Exalted Deeds");
    let angel = t.battlefield(P0, "Serra Angel");
    let crafter = t.battlefield(P0, "Imagecrafter");
    mana(&mut t, P0, ManaType::W, 3);
    t.activate(P0, book, 0, &[obj(angel)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(angel, "enlightened"), 1);
    // It stops being an Angel, and loses its counter.
    t.activate(P0, crafter, 0, &[obj(angel)]).unwrap();
    t.resolve_all();
    assert!(!t.obj(angel).chars.has_subtype("Angel"));
    t.g.remove_counters(obj(angel), "enlightened", 1);
    t.g.flush_events();
    // P0 still can't lose the game.
    t.g.players[0].life = 0;
    t.settle();
    assert!(!t.has_lost(P0));
}

#[test]
fn fasting_puts_a_counter_then_checks() {
    cr!("608.2c");
    ruling!(
        "Fasting",
        "When you resolve the beginning-of-upkeep triggered ability, you first put a hunger counter on Fasting, then you check how many hunger counters it has. If there are five or more, Fasting is destroyed."
    );
    supported("Fasting");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Fasting");
    put(&mut t, f, "hunger", 3);
    to_upkeep(&mut t);
    t.resolve_all();
    assert!(t.on_battlefield(f));
    assert_eq!(t.counters(f, "hunger"), 4);
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Fasting");
    put(&mut t, f, "hunger", 4);
    to_upkeep(&mut t);
    t.resolve_all();
    assert_eq!(t.zone(f), Zone::Graveyard(P0));
}

#[test]
fn azors_elocutors_one_counter_per_source() {
    cr!("120.2", "603.2c");
    ruling!(
        "Azor's Elocutors",
        "The second ability of Azor’s Elocutors removes one filibuster counter per source, no matter how much damage that source dealt."
    );
    supported("Azor's Elocutors");
    let mut t = TestGame::new(2);
    let az = t.battlefield(P1, "Azor's Elocutors");
    put(&mut t, az, "filibuster", 4);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.counters(az, "filibuster"), 2);
}

#[test]
fn azors_elocutors_wins_only_as_its_upkeep_ability_resolves() {
    cr!("603.2", "608.2c");
    ruling!(
        "Azor's Elocutors",
        "You’ll only win the game if Azor’s Elocutors has five or more filibuster counters on it when the first ability resolves."
    );
    let mut t = TestGame::new(2);
    let az = t.battlefield(P0, "Azor's Elocutors");
    put(&mut t, az, "filibuster", 5);
    t.settle();
    assert!(!t.has_lost(P1));
    assert_eq!(t.stack_len(), 0);
    let mut t = TestGame::new(2);
    let az = t.battlefield(P0, "Azor's Elocutors");
    put(&mut t, az, "filibuster", 4);
    to_upkeep(&mut t);
    t.resolve_all();
    assert!(t.has_lost(P1));
}

#[test]
fn removing_judgment_counters_doesnt_remove_faithbound_judge_from_combat() {
    cr!("506.4");
    ruling!(
        "Faithbound Judge // Sinner's Judgment",
        "Removing judgment counters from Faithbound Judge after it has attacked won't remove it from combat."
    );
    supported("Faithbound Judge // Sinner's Judgment");
    let mut t = TestGame::new(2);
    let judge = t.battlefield(P0, "Faithbound Judge // Sinner's Judgment");
    put(&mut t, judge, "judgment", 3);
    attack_with(&mut t, &[(judge, Entity::Player(P1))]);
    t.g.remove_counters(obj(judge), "judgment", 3);
    t.g.flush_events();
    t.g.recompute();
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn arbiter_of_the_ideal_permanent_enters_as_an_enchantment() {
    cr!("603.6a", "614.12", "122.1");
    ruling!(
        "Arbiter of the Ideal",
        "The permanent will be an enchantment as it enters the battlefield and will cause constellation abilities of permanents you control to trigger."
    );
    supported("Arbiter of the Ideal");
    supported("Eidolon of Blossoms");
    let mut t = TestGame::new(2);
    let arb = t.battlefield(P0, "Arbiter of the Ideal");
    t.battlefield(P0, "Eidolon of Blossoms");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.g.tap(arb);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.g.untap(arb);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.obj_now(bears).is(CardType::Enchantment));
    assert_eq!(t.counters(bears, "manifestation"), 1);
    // Eidolon of Blossoms' constellation ability drew a card.
    assert_eq!(t.hand_size(P0), hand + 1);
}
