//! Rulings batch P155 — a mana value or power "as it last existed" (CR 608.2h): a
//! permanent that left the battlefield (a sacrificed, destroyed or exiled copy whose mana
//! value on the battlefield was the copied card's), a card that left the graveyard or the
//! hand, and X in a mana cost off the stack (CR 202.3e).

use crate::r_p155_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p`'s Air Elemental ({3}{U}{U}, mana value 5) and a Mirror Image (mana value 3 as a
/// card) copying it: the copy.
fn copy_of_air_elemental(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let ae = t.battlefield(p, "Air Elemental");
    let c = mirror_image_of(t, p, ae);
    assert_eq!(mana_value(t, c), 5);
    c
}

/// `p`'s Gilded Lotus ({5}) and a Sculpting Steel (mana value 3 as a card) copying it:
/// the copy.
fn copy_of_gilded_lotus(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let lotus = t.battlefield(p, "Gilded Lotus");
    let c = sculpting_steel_of(t, p, lotus);
    assert_eq!(mana_value(t, c), 5);
    c
}

#[test]
fn feed_the_swarm_uses_the_mana_value_as_it_last_existed_on_the_battlefield() {
    cr!("608.2h", "202.3");
    ruling!(
        "Feed the Swarm",
        "The amount of life you lose is determined by the permanent's mana value as it last existed on the battlefield."
    );
    supported("Feed the Swarm");
    let mut t = TestGame::new(2);
    let copy = copy_of_air_elemental(&mut t, P1);
    cast_resolve(&mut t, P0, "Feed the Swarm", "1B", &[obj(copy)]);
    assert!(t.in_graveyard(P1, "Mirror Image"));
    assert_eq!(mana_value(&t, copy), 3, "a Mirror Image in the graveyard");
    assert_eq!(t.life(P0), 15, "lost 5, the copy's mana value on the battlefield");
}

#[test]
fn scrap_trawler_compares_with_the_mana_value_as_it_last_existed_on_the_battlefield() {
    cr!("608.2h", "603.10a", "115.1");
    ruling!(
        "Scrap Trawler",
        "The target artifact card must have a lesser mana value than the artifact that caused Scrap Trawler's ability to trigger by being put into a graveyard. Use the artifact's mana value as it last existed on the battlefield to determine what may be returned."
    );
    supported("Scrap Trawler");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Scrap Trawler");
    let copy = copy_of_gilded_lotus(&mut t, P0);
    let jugg = t.graveyard(P0, "Juggernaut");
    let lotus_card = t.graveyard(P0, "Gilded Lotus");
    let from = t.asked().len();
    // Juggernaut (4) is a legal target (4 < 5), a Gilded Lotus card (5) isn't.
    t.answer_targets(P0, &[obj(jugg)]);
    kill(&mut t, copy);
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    let cands = target_candidates(&t, P0, from);
    assert!(cands.iter().flatten().any(|e| *e == obj(jugg)));
    assert!(!cands.iter().flatten().any(|e| *e == obj(lotus_card)));
    t.resolve_all();
    assert!(t.in_hand(P0, "Juggernaut"));
}

#[test]
fn scrap_trawler_x_in_a_mana_cost_is_0_on_the_battlefield_and_in_a_graveyard() {
    cr!("202.3e", "603.3d");
    ruling!(
        "Scrap Trawler",
        "While on the battlefield or in a graveyard, {X} in an object's mana cost is 0."
    );
    supported("Walking Ballista");
    // A Walking Ballista ({X}{X}) on the battlefield has mana value 0: when it dies, no
    // artifact card has a lesser mana value, not even a Memnite ({0}).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Scrap Trawler");
    let ballista = t.battlefield(P0, "Walking Ballista");
    put(&mut t, ballista, counters::PLUS1, 1);
    t.graveyard(P0, "Memnite");
    assert_eq!(mana_value(&t, ballista), 0);
    kill(&mut t, ballista);
    assert_eq!(t.stack_len(), 0, "no legal target: {}", t.dump_log());
    assert!(!t.in_hand(P0, "Memnite"));
    // In a graveyard, the Ballista has mana value 0 and is a legal target when a
    // Juggernaut (4) dies.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Scrap Trawler");
    let jugg = t.battlefield(P0, "Juggernaut");
    let ballista = t.graveyard(P0, "Walking Ballista");
    assert_eq!(mana_value(&t, ballista), 0);
    t.answer_targets(P0, &[obj(ballista)]);
    kill(&mut t, jugg);
    t.resolve_all();
    assert!(t.in_hand(P0, "Walking Ballista"));
}

#[test]
fn jackdaw_savior_compares_with_the_mana_value_as_it_last_existed_on_the_battlefield() {
    cr!("608.2h", "603.10a", "115.1");
    ruling!(
        "Jackdaw Savior",
        "The target creature card must have a lesser mana value than the creature that caused Jackdaw Savior’s last ability to trigger. Use the mana value of that creature as it last existed on the battlefield to determine which creature cards in your graveyard are legal targets for that ability."
    );
    supported("Jackdaw Savior");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jackdaw Savior");
    let copy = copy_of_air_elemental(&mut t, P0);
    let giant = t.graveyard(P0, "Hill Giant");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(giant)]);
    kill(&mut t, copy);
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    // Hill Giant (4 < 5) is offered; the Mirror Image card (2) that died isn't "another".
    let cands = target_candidates(&t, P0, from);
    assert!(cands.iter().flatten().any(|e| *e == obj(giant)));
    t.resolve_all();
    assert_eq!(count_named(&t, "Hill Giant"), 1);
}

#[test]
fn havi_compares_with_the_mana_value_as_it_last_existed_on_the_battlefield() {
    cr!("608.2h", "603.10a", "115.1");
    ruling!(
        "Havi, the All-Father",
        "Use the mana value of the legendary creature that died as it last existed on the battlefield to determine what may be targeted with Havi's last ability."
    );
    supported("Havi, the All-Father");
    supported("Jedit Ojanen");
    supported("Mirri, Cat Warrior");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Havi, the All-Father");
    // A Mirror Image entering as a copy of Jedit Ojanen ({4}{W}{W}{U}, 7): a legendary
    // creature of mana value 7 on the battlefield, 3 as a card; the legend rule puts it
    // into the graveyard (P0 keeps Jedit), and Havi triggers.
    let jedit = t.battlefield(P0, "Jedit Ojanen");
    let mirri = t.graveyard(P0, "Mirri, Cat Warrior");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(jedit)]);
    let copy = t.enter(P0, "Mirror Image");
    assert_eq!(mana_value(&t, copy), 7);
    t.answer_choose(P0, &[obj(jedit)]);
    t.answer_targets(P0, &[obj(mirri)]);
    t.settle();
    assert!(t.in_graveyard(P0, "Mirror Image"), "{}", t.dump_log());
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(count_named(&t, "Mirri, Cat Warrior"), 1, "{}", t.dump_log());
    assert!(t.obj_now(mirri).tapped);
}

#[test]
fn morbid_curiosity_uses_the_mana_value_immediately_before_the_sacrifice() {
    cr!("608.2h", "601.2b");
    ruling!(
        "Morbid Curiosity",
        "To determine how many cards to draw, use the mana value of the permanent immediately before you sacrificed it."
    );
    supported("Morbid Curiosity");
    let mut t = TestGame::new(2);
    let copy = copy_of_air_elemental(&mut t, P0);
    t.answer_choose(P0, &[obj(copy)]);
    cast_resolve(&mut t, P0, "Morbid Curiosity", "1BB", &[]);
    assert!(t.in_graveyard(P0, "Mirror Image"));
    assert_eq!(t.hand_size(P0), 5, "drew five: {}", t.dump_log());
}

#[test]
fn expel_the_unworthy_uses_the_mana_value_as_it_last_existed_on_the_battlefield() {
    cr!("608.2h");
    interp!("moved-object-values-as-it-last-existed");
    ruling!(
        "Expel the Unworthy",
        "Use the creature's mana value as it last existed on the battlefield to determine how much life its controller gains."
    );
    supported("Expel the Unworthy");
    let mut t = TestGame::new(2);
    // P1's Mirror Image copying Grizzly Bears ({1}{G}, 2): mana value 3 or less.
    let bears = t.battlefield(P1, "Grizzly Bears");
    let copy = mirror_image_of(&mut t, P1, bears);
    cast_resolve(&mut t, P0, "Expel the Unworthy", "1W", &[obj(copy)]);
    assert!(t.in_exile("Mirror Image"));
    assert_eq!(mana_value(&t, copy), 3, "a Mirror Image in exile");
    assert_eq!(t.life(P1), 22, "gained 2, the copy's mana value on the battlefield");
}

#[test]
fn sister_hospitaller_uses_the_mana_value_as_it_last_existed_in_the_graveyard() {
    cr!("608.2h", "614.1c", "400.7");
    interp!("moved-object-values-as-it-last-existed");
    ruling!(
        "Sister Hospitaller",
        "Use the mana value of the card as it last existed in the graveyard to determine the amount of life you gain."
    );
    supported("Sister Hospitaller");
    supported("Clone");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // A Clone ({3}{U}, 4) returned from the graveyard enters as a copy of Grizzly Bears
    // (2): the life gained is 4, the card's mana value in the graveyard.
    let clone = t.graveyard(P0, "Clone");
    t.answer_targets(P0, &[obj(clone)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.enter(P0, "Sister Hospitaller");
    t.resolve_all();
    let returned = t.g.current(clone);
    assert_eq!(t.zone(returned), Zone::Battlefield);
    assert_eq!(t.obj_now(returned).chars.name, "Grizzly Bears");
    assert_eq!(mana_value(&t, returned), 2);
    assert_eq!(t.life(P0), 24, "{}", t.dump_log());
}

#[test]
fn convert_to_slime_uses_the_mana_values_as_they_last_existed_on_the_battlefield() {
    cr!("608.2h");
    ruling!(
        "Convert to Slime",
        "Use the mana value of the destroyed permanents as they last existed on the battlefield to determine the value of X."
    );
    supported("Convert to Slime");
    let mut t = TestGame::new(2);
    // Delirium: four card types among cards in P0's graveyard.
    for n in ["Forest", "Shock", "Grizzly Bears", "Memnite"] {
        t.graveyard(P0, n);
    }
    let lotus_copy = copy_of_gilded_lotus(&mut t, P1);
    let elemental_copy = copy_of_air_elemental(&mut t, P1);
    let spell = t.hand(P0, "Convert to Slime");
    mana(&mut t, P0, "3BG");
    t.answer_targets(P0, &[obj(lotus_copy)]);
    t.answer_targets(P0, &[obj(elemental_copy)]);
    t.answer_targets(P0, &[]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Sculpting Steel") && t.in_graveyard(P1, "Mirror Image"));
    let oozes = tokens(&t, P0);
    assert_eq!(oozes.len(), 1, "{}", t.dump_log());
    assert_eq!(t.pt(oozes[0]), (10, 10), "5 + 5, not 3 + 2");
}

#[test]
fn imskir_uses_the_sacrificed_artifacts_mana_value_as_it_last_existed_on_the_battlefield() {
    cr!("608.2h", "602.2b");
    ruling!(
        "Imskir Iron-Eater",
        "Use the mana value of the sacrificed artifact as it last existed on the battlefield to determine how much damage is dealt."
    );
    supported("Imskir Iron-Eater");
    let mut t = TestGame::new(2);
    let imskir = t.battlefield(P0, "Imskir Iron-Eater");
    let copy = copy_of_gilded_lotus(&mut t, P0);
    t.answer_choose(P0, &[obj(copy)]);
    activate_resolve(&mut t, P0, imskir, "Sacrifice an artifact", "3R", &[pl(P1)]);
    assert!(t.in_graveyard(P0, "Sculpting Steel"));
    assert_eq!(t.life(P1), 15, "{}", t.dump_log());
}

#[test]
fn rakdos_uses_the_sacrificed_creatures_mana_value_as_it_last_existed_on_the_battlefield() {
    cr!("608.2h", "603.10a");
    ruling!(
        "Rakdos, the Muscle",
        "Use the mana value of the sacrificed creature as it last existed on the battlefield to determine how many cards to exile."
    );
    supported("Rakdos, the Muscle");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rakdos, the Muscle");
    let copy = copy_of_air_elemental(&mut t, P0);
    t.answer_targets(P0, &[pl(P1)]);
    sacrifice(&mut t, P0, copy);
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.library_size(P1), 25, "five exiled");
}

#[test]
fn illuminor_szeras_adds_the_sacrificed_creatures_mana_value_as_it_last_existed() {
    cr!("608.2h", "602.2b", "605.1a");
    ruling!(
        "Illuminor Szeras",
        "Use the mana value of the sacrificed creature as it last existed on the battlefield to determine the amount of {B} to add."
    );
    supported("Illuminor Szeras");
    let mut t = TestGame::new(2);
    let szeras = t.battlefield(P0, "Illuminor Szeras");
    let copy = copy_of_air_elemental(&mut t, P0);
    t.answer_choose(P0, &[obj(copy)]);
    t.activate(P0, szeras, 0, &[]).unwrap();
    t.settle();
    assert!(t.in_graveyard(P0, "Mirror Image"));
    assert_eq!(pool(&t, P0), 5, "{}", t.dump_log());
    assert_eq!(t.g.players[P0.idx()].mana_pool.count(ManaType::B), 5);
}

#[test]
fn birthing_ritual_uses_the_sacrificed_creatures_mana_value_as_it_last_existed() {
    cr!("608.2h");
    ruling!(
        "Birthing Ritual",
        "Use the mana value of the sacrificed creature as it last existed on the battlefield to determine the value of X."
    );
    supported("Birthing Ritual");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Birthing Ritual");
    let copy = copy_of_air_elemental(&mut t, P0);
    // A Craw Wurm ({4}{G}{G}, 6) among the top seven: 6 <= 1 + 5.
    let wurm = t.library_top(P0, "Craw Wurm");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(copy)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(wurm)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mirror Image"));
    assert_eq!(count_named(&t, "Craw Wurm"), 1, "{}", t.dump_log());
}

#[test]
fn shadow_uses_the_sacrificed_permanents_mana_value_as_it_last_existed() {
    cr!("608.2h");
    ruling!(
        "Shadow, Mysterious Assassin",
        "Use the mana value of the sacrificed permanent as it last existed on the battlefield to determine how much life each opponent loses."
    );
    supported("Shadow, Mysterious Assassin");
    let mut t = TestGame::new(2);
    let shadow = t.battlefield(P0, "Shadow, Mysterious Assassin");
    let copy = copy_of_air_elemental(&mut t, P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(copy)]);
    attack_with(&mut t, &[(shadow, pl(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mirror Image"));
    assert_eq!(t.life(P1), 20 - 3 - 5, "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn terminal_velocity_uses_the_mana_value_as_it_last_existed_on_the_battlefield() {
    cr!("608.2h", "603.10a");
    ruling!(
        "Terminal Velocity",
        "Use the permanent’s mana value as it last existed on the battlefield to determine how much damage the triggered ability deals."
    );
    supported("Terminal Velocity");
    supported("Clone");
    let mut t = TestGame::new(2);
    let ae = t.battlefield(P1, "Air Elemental");
    let giant = t.battlefield(P1, "Hill Giant");
    let clone = t.hand(P0, "Clone");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(clone)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(ae)]);
    cast_resolve(&mut t, P0, "Terminal Velocity", "4RR", &[]);
    let c = t.g.current(clone);
    assert_eq!(t.zone(c), Zone::Battlefield);
    assert_eq!(mana_value(&t, c), 5, "a copy of Air Elemental");
    // It leaves: 5 damage (not Clone's 4) to each creature: Air Elemental (4/4) dies.
    kill(&mut t, c);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Air Elemental"), "{}", t.dump_log());
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Clone"));
    let _ = giant;
}

#[test]
fn tormented_thoughts_uses_the_power_as_it_last_existed_on_the_battlefield() {
    cr!("608.2h", "601.2b");
    ruling!(
        "Tormented Thoughts",
        "The targeted player will discard a number of cards equal to the power of the sacrificed creature as it last existed on the battlefield."
    );
    let mut t = TestGame::new(2);
    for _ in 0..6 {
        t.hand(P1, "Island");
    }
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Giant Growth", "G", &[obj(bears)]);
    assert_eq!(t.pt(bears), (5, 5));
    t.answer_choose(P0, &[obj(bears)]);
    cast_resolve(&mut t, P0, "Tormented Thoughts", "2B", &[pl(P1)]);
    assert_eq!(t.hand_size(P1), 1, "discarded five: {}", t.dump_log());
}

#[test]
fn galufs_final_act_uses_the_power_as_it_last_existed_on_the_battlefield() {
    cr!("608.2h", "603.10a");
    ruling!(
        "Galuf's Final Act",
        "Use the dying creature's power as it last existed on the battlefield to determine how many counters to put on the target creature."
    );
    supported("Galuf's Final Act");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Memnite");
    cast_resolve(&mut t, P0, "Galuf's Final Act", "1G", &[obj(bears)]);
    cast_resolve(&mut t, P0, "Giant Growth", "G", &[obj(bears)]);
    assert_eq!(t.pt(bears), (6, 5));
    t.answer_targets(P0, &[obj(other)]);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.counters(other, counters::PLUS1), 6, "{}", t.dump_log());
}

#[test]
fn calamity_of_the_titans_revealed_card_stays_revealed_and_keeps_its_hand_mana_value() {
    cr!("701.20a", "608.2h", "601.2b");
    ruling!(
        "Calamity of the Titans",
        "The colorless creature card stays revealed until Calamity of the Titans leaves the stack."
    );
    ruling!(
        "Calamity of the Titans",
        "The colorless creature card you reveal doesn't have to still be in your hand as Calamity of the Titans resolves. If it's not, use its mana value as it last existed in your hand."
    );
    supported("Calamity of the Titans");
    supported("Kozilek's Channeler");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    // Kozilek's Channeler ({4}{C}, 5) is revealed as the additional cost.
    let channeler = t.hand(P0, "Kozilek's Channeler");
    t.answer_choose(P0, &[obj(channeler)]);
    let spell = cast(&mut t, P0, "Calamity of the Titans", "4CC", &[]);
    assert!(mtg_engine::reveal::is_revealed(&t.g, channeler));
    // It leaves the hand in response: a new object, no longer revealed; the spell still
    // uses mana value 5: Grizzly Bears (2) is exiled, Craw Wurm (6) isn't.
    let gone = to_graveyard(&mut t, channeler);
    assert!(!mtg_engine::reveal::is_revealed(&t.g, gone));
    assert!(on_stack(&t, spell));
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile, "{}", t.dump_log());
    assert_eq!(t.zone(wurm), Zone::Battlefield);
    // Revealed until the spell leaves the stack, when it stays in hand.
    let mut t = TestGame::new(2);
    let channeler = t.hand(P0, "Kozilek's Channeler");
    t.answer_choose(P0, &[obj(channeler)]);
    cast(&mut t, P0, "Calamity of the Titans", "4CC", &[]);
    assert!(mtg_engine::reveal::is_revealed(&t.g, channeler));
    t.resolve_all();
    assert!(!mtg_engine::reveal::is_revealed(&t.g, channeler));
    assert!(t.in_hand(P0, "Kozilek's Channeler"));
}

#[test]
fn the_x_of_a_spell_on_the_stack_counts_in_its_mana_value() {
    cr!("202.3e");
    ruling!(
        "Manaplasm",
        "While a spell with {X} in its cost is on the stack, its mana value takes the chosen value of X into account. For example, if you cast Blaze (a spell with the mana cost {X}{R}) and choose a value of 5 for that X, then your Manaplasm will get +6/+6 until end of turn."
    );
    supported("Manaplasm");
    let mut t = TestGame::new(2);
    let plasm = t.battlefield(P0, "Manaplasm");
    let blaze = cast_blaze(&mut t, P0, 5, pl(P1));
    assert_eq!(mana_value(&t, blaze), 6);
    t.resolve_all();
    assert_eq!(t.pt(plasm), (7, 7));
    assert_eq!(t.life(P1), 15);
    let _ = Answer::Default;
}
