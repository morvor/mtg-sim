//! Rulings batch P141 — repeatable artifact tokens: copies (Mirrorworks, Court of
//! Vantress), token factories and the costs that feed them (Golem Foundry, Myrsmith,
//! Pentavus, Thopter Foundry, Prototype Portal, Stridehangar Automaton), and what the
//! tokens are (Scrap, Metallic Sliver, Urza's subtype, Myr).

use crate::r_p141_common::*;
use mtg_engine::ability::Modification;
use mtg_engine::designations::become_monarch;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::CardType;
use mtg_engine::*;

#[test]
fn mirrorworks_copies_the_copiable_values_of_a_copy() {
    cr!("707.2", "707.3", "111.10");
    ruling!(
        "Mirrorworks",
        "The copiable values of the token's characteristics are the same as the copiable values of the characteristics of the artifact it's copying."
    );
    supported("Mirrorworks");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirrorworks");
    let o = t.battlefield(P0, "Ornithopter");
    t.lands(P0, "Wastes", 2);
    // Sculpting Steel enters as a copy of Ornithopter; Mirrorworks copies that.
    t.answer_choose(P0, &[obj(o)]);
    enter(&mut t, P0, "Sculpting Steel");
    t.answer_yes(P0, true);
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Ornithopter");
    assert!(t.obj_now(toks[0]).is(CardType::Creature));
}

#[test]
fn court_of_vantress_token_copies_only_the_printed_permanent() {
    cr!("707.2", "707.4");
    ruling!(
        "Court of Vantress",
        "The token copies exactly what was printed on the original permanent and nothing else"
    );
    supported("Court of Vantress");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Court of Vantress");
    become_monarch(&mut t.g, P0);
    let o = t.battlefield(P0, "Ornithopter");
    put_counters(&mut t, o, "+1/+1", 2);
    pump(&mut t, o, 3, 3);
    t.g.objects[o.0 as usize].tapped = true;
    t.answer_targets(P0, &[obj(o)]);
    t.answer_yes(P0, true);
    into_upkeep(&mut t, P0);
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    let tok = t.obj_now(toks[0]);
    assert_eq!(tok.chars.name, "Ornithopter");
    assert!(!tok.tapped);
    assert_eq!(t.counters(toks[0], "+1/+1"), 0);
    assert_eq!(t.pt(toks[0]), (0, 2));
}

#[test]
fn court_of_vantress_becoming_a_copy_lasts_and_doesnt_enter() {
    cr!("707.2", "611.2", "603.6a");
    ruling!(
        "Court of Vantress",
        "The copy effect lasts indefinitely. Often, it will last until it is overwritten by another copy effect"
    );
    ruling!(
        "Court of Vantress",
        "When Court of Vantress becomes a copy of a permanent, it's neither entering nor leaving the battlefield. Any enters-the-battlefield or leaves-the-battlefield abilities won't trigger."
    );
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Court of Vantress");
    let w = t.battlefield(P0, "Ichor Wellspring");
    t.answer_targets(P0, &[obj(w)]);
    t.answer_yes(P0, true);
    into_upkeep(&mut t, P0);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.obj_now(c).chars.name, "Ichor Wellspring");
    assert_eq!(t.hand_size(P0), hand, "no enters trigger");
    assert_eq!(stacked_triggers(&t), 0);
    // Still a copy on the next turn (with Court's upkeep ability).
    t.advance_to(P1, turn::Step::End);
    assert_eq!(t.obj_now(c).chars.name, "Ichor Wellspring");
    into_upkeep(&mut t, P0);
    assert_eq!(stacked_triggers(&t), 1);
}

#[test]
fn dovin_gets_one_counter_per_creature_dealing_combat_damage() {
    cr!("603.7", "510.2");
    ruling!(
        "Dovin, Grand Arbiter",
        "The delayed triggered ability created by Dovin's first ability triggers for each creature you control that deals combat damage to a player this turn."
    );
    supported("Dovin, Grand Arbiter");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Dovin, Grand Arbiter");
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, d, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(d, "loyalty"), 4);
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.counters(d, "loyalty"), 6);
}

#[test]
fn artifact_tokens_are_what_their_cards_say() {
    cr!("111.4", "202.1b", "205.3i", "205.3g");
    ruling!(
        "Farid, Enterprising Salvager",
        "The token's name is Scrap, and it has no abilities."
    );
    ruling!(
        "Sliversmith",
        "Unlike the card Metallic Sliver, the token has no mana cost. Its mana value is 0."
    );
    ruling!(
        "Urza's Factory",
        "Urza's is a land subtype that has no inherent rules meaning. Other effects may refer to it."
    );
    supported("Farid, Enterprising Salvager");
    supported("Sliversmith");
    supported("Urza's Factory");
    // Farid: a nontoken artifact goes to the graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Farid, Enterprising Salvager");
    let o = t.battlefield(P0, "Ornithopter");
    destroy(&mut t, o);
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    let s = t.obj_now(toks[0]);
    assert_eq!(s.chars.name, "Scrap");
    assert!(s.chars.abilities.is_empty());
    assert!(s.is(CardType::Artifact) && s.chars.colors.is_colorless());
    // Sliversmith's token has mana value 0.
    let mut t = TestGame::new(2);
    let sm = t.battlefield(P0, "Sliversmith");
    t.lands(P0, "Wastes", 1);
    t.hand(P0, "Grizzly Bears");
    t.activate(P0, sm, 0, &[]).unwrap();
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Metallic Sliver");
    assert_eq!(t.g.mana_value_of(toks[0]), 0);
    // Urza's Factory has the subtype Urza's.
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Urza's Factory");
    assert!(t.obj_now(f).chars.has_subtype("Urza's"));
    t.lands(P0, "Wastes", 7);
    t.activate(P0, f, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Assembly-Worker").len(), 1);
}

#[test]
fn thopter_assembly_counts_any_thopter() {
    cr!("603.4");
    ruling!(
        "Thopter Assembly",
        "The triggered ability looks for any permanent you control with the creature type Thopter"
    );
    supported("Thopter Assembly");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thopter Assembly");
    t.battlefield(P0, "Ornithopter");
    into_upkeep(&mut t, P0);
    assert_eq!(stacked_triggers(&t), 0);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thopter Assembly");
    into_upkeep(&mut t, P0);
    assert_eq!(stacked_triggers(&t), 1);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 5);
}

#[test]
fn golem_foundry_trigger_resolves_before_the_spell_and_myrsmith_makes_one() {
    cr!("603.3", "405.5", "118.12");
    ruling!(
        "Golem Foundry",
        "Whenever you cast an artifact spell, Golem Foundry's first ability triggers and goes on the stack on top of it. It will resolve before the artifact spell does."
    );
    ruling!(
        "Myrsmith",
        "While resolving Myrsmith's ability, you can't pay more than {1} to get more than one Myr."
    );
    supported("Golem Foundry");
    supported("Myrsmith");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Golem Foundry");
    let o = t.hand(P0, "Ornithopter");
    let spell = t.cast(P0, o).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    yes(&mut t, P0);
    t.resolve();
    assert_eq!(t.counters(f, "charge"), 1);
    assert!(t.g.stack.contains(&spell));
    // Myrsmith: plenty of mana, one Myr.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Myrsmith");
    t.lands(P0, "Wastes", 5);
    let o = t.hand(P0, "Ornithopter");
    t.cast(P0, o).go();
    yes(&mut t, P0);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Myr").len(), 1);
    assert_eq!(tapped_lands(&t), 1);
}

fn tapped_lands(t: &TestGame) -> usize {
    crate::r_s01_common::tapped_lands(t, P0)
}

#[test]
fn pentavus_uses_any_counter_and_any_pentavite() {
    cr!("602.2b", "118.3");
    ruling!(
        "Pentavus",
        "You can remove any +1/+1 counter on Pentavus to activate its first activated ability"
    );
    ruling!(
        "Pentavus",
        "You can sacrifice any Pentavite creature you control (such as an Adaptive Automaton with Pentavite as its chosen type) to activate Pentavus’s second activated ability"
    );
    supported("Pentavus");
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Pentavus");
    put_counters(&mut t, p, "+1/+1", 1);
    t.lands(P0, "Wastes", 2);
    t.activate(P0, p, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(p, "+1/+1"), 0);
    assert_eq!(with_subtype(&t, P0, "Pentavite").len(), 1);
    // A Pentavite that Pentavus didn't make.
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Pentavus");
    put_counters(&mut t, p, "+1/+1", 1);
    let other = create_token(&mut t, P0, "Pentavite");
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[obj(other)]);
    t.activate(P0, p, 1, &[]).unwrap();
    t.resolve_all();
    assert!(!t.is_live(other) || !t.on_battlefield(other));
    assert_eq!(t.counters(p, "+1/+1"), 2);
}

#[test]
fn stridehangar_adds_a_thopter_to_tokens_created_under_your_control() {
    cr!("614.1a", "111.2");
    ruling!(
        "Stridehangar Automaton",
        "You don’t need to control the spell or ability that creates the tokens, but the tokens do need to be created under your control."
    );
    supported("Stridehangar Automaton");
    supported("Fateful Absence");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stridehangar Automaton");
    let b = t.battlefield(P0, "Grizzly Bears");
    // The opponent destroys P0's Bears: P0 investigates.
    t.set_step(P1, turn::Step::PrecombatMain);
    cast_new(&mut t, P1, "Fateful Absence", &[obj(b)]);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Clue").len(), 1);
    assert_eq!(with_subtype(&t, P0, "Thopter").len(), 1);
    // P0's effect creating a token under the opponent's control: no Thopter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stridehangar Automaton");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Fateful Absence", &[obj(theirs)]);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P1, "Clue").len(), 1);
    assert_eq!(with_subtype(&t, P1, "Thopter").len(), 0);
    assert_eq!(with_subtype(&t, P0, "Thopter").len(), 0);
}

#[test]
fn prototype_portal_needs_an_exiled_card() {
    cr!("107.3", "602.2");
    ruling!(
        "Prototype Portal",
        "You may not activate the second ability if no card has been exiled with Prototype Portal."
    );
    supported("Prototype Portal");
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Prototype Portal");
    t.lands(P0, "Wastes", 3);
    assert!(!can_activate(&mut t, P0, p));
    // Positive control: with Ornithopter exiled (X = 0).
    let mut t = TestGame::new(2);
    let o = t.hand(P0, "Ornithopter");
    t.answer_choose_moved(P0, &[obj(o)]);
    yes(&mut t, P0);
    let p = enter(&mut t, P0, "Prototype Portal");
    t.resolve_all();
    assert!(t.in_exile("Ornithopter"));
    assert!(can_activate(&mut t, P0, p));
    t.activate(P0, p, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
}

#[test]
fn thopter_foundry_can_sacrifice_itself() {
    cr!("602.2b", "118.3");
    ruling!(
        "Thopter Foundry",
        "You may sacrifice Thopter Foundry to pay the cost of its own ability."
    );
    supported("Thopter Foundry");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Thopter Foundry");
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[obj(f)]);
    t.activate(P0, f, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Thopter Foundry"));
    assert_eq!(with_subtype(&t, P0, "Thopter").len(), 1);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn roxanne_triggers_only_for_artifact_tokens_tapped_for_mana() {
    cr!("106.12", "605.1a");
    ruling!(
        "Roxanne, Starfall Savant",
        "You're tapping an artifact token for mana only if you're activating a mana ability of that token that includes the tap symbol ({T}) in its cost."
    );
    supported("Roxanne, Starfall Savant");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Roxanne, Starfall Savant");
    let tr = create_token(&mut t, P0, "Treasure");
    let stone = t.battlefield(P0, "Mind Stone");
    // A nontoken artifact tapped for mana: one mana.
    t.activate(P0, stone, 0, &[]).unwrap();
    t.settle();
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    // The Treasure token: two.
    t.activate(P0, tr, 0, &[]).unwrap();
    t.settle();
    assert_eq!(t.g.player(P0).mana_pool.total(), 3);
}

#[test]
fn myr_reservoir_mana_needs_the_subtype_myr() {
    cr!("106.6", "205.3");
    ruling!(
        "Myr Reservoir",
        "A card, spell, or permanent is a Myr only if it has the subtype Myr, regardless of its name."
    );
    supported("Myr Reservoir");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Myr Reservoir");
    let overseer = t.hand(P0, "Steel Overseer");
    assert!(t.cast(P0, overseer).try_go().is_err());
    let sire = t.hand(P0, "Myr Sire");
    assert!(t.cast(P0, sire).try_go().is_ok());
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Myr Sire").len(), 1);
}

fn can_cast(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    crate::r_s02_common::can_cast(t, p, card, CastMethod::Normal)
}

#[test]
fn saheeli_emblem_reduces_only_generic_mana() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Saheeli, Filigree Master",
        "The cost reduction provided by Saheeli's emblem can never reduce the colored mana costs of an artifact spell."
    );
    supported("Saheeli, Filigree Master");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Saheeli, Filigree Master");
    put_counters(&mut t, s, "loyalty", 1);
    t.activate(P0, s, 2, &[]).unwrap();
    t.resolve_all();
    t.lands(P0, "Island", 1);
    let strix = t.hand(P0, "Baleful Strix");
    let overseer = t.hand(P0, "Steel Overseer");
    assert!(!can_cast(&mut t, P0, strix));
    assert!(can_cast(&mut t, P0, overseer));
}

#[test]
fn sivriss_turned_artifact_can_sacrifice_itself() {
    cr!("602.2b", "118.3");
    ruling!(
        "Sivriss, Nightmare Speaker",
        "If Sivriss somehow becomes an artifact, you may sacrifice it to its own ability."
    );
    supported("Sivriss, Nightmare Speaker");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sivriss, Nightmare Speaker");
    crate::r_s26_common::modify_until_eot(&mut t, s, vec![Modification::AddTypes(vec![CardType::Artifact])]);
    t.library_top(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(s)]);
    t.activate(P0, s, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Sivriss, Nightmare Speaker"));
    assert!(t.in_hand(P0, "Grizzly Bears"), "the opponent didn't pay");
    let _ = Zone::Exile;
}
