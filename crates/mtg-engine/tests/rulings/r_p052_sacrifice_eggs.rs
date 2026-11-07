//! Rulings batch P052 — artifacts sacrificed for value: last known information of the
//! sacrificed artifact (CR 608.2h, 113.7a), "when this is put into a graveyard from the
//! battlefield" (CR 603.6c, 603.10a), destroying sets of permanents (CR 701.8), effects
//! that aren't targeted (CR 115.1, 702.16), mana abilities (CR 605), life lost this turn
//! (CR 119.3), and searching for a number of cards (CR 701.23).

use crate::r_p006_common::choices_since;
use crate::r_p046_common::*;
use crate::r_p052_common::*;
use crate::r_s01_common::*;
use crate::r_s20_common::tap_for_mana;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn set_counters(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    t.g.objects[id.0 as usize].counters.insert(kind.into(), n);
}

#[test]
fn culling_dais_and_golden_urn_use_their_last_known_counters() {
    cr!("608.2h", "113.7a", "602.2b");
    ruling!(
        "Culling Dais",
        "As Culling Dais's last ability resolves, its last existence on the battlefield is checked to determine how many charge counters were on it."
    );
    supported("Culling Dais");
    let mut t = TestGame::new(2);
    let dais = t.battlefield(P0, "Culling Dais");
    set_counters(&mut t, dais, "charge", 3);
    let hand = t.hand_size(P0);
    activate_resolve(&mut t, P0, dais, 1, &[]);
    assert!(t.in_graveyard(P0, "Culling Dais"));
    assert_eq!(t.hand_size(P0), hand + 3);
    ruling!(
        "Golden Urn",
        "As Golden Urn’s last ability resolves, its last existence on the battlefield is checked to determine how many charge counters were on it."
    );
    supported("Golden Urn");
    let mut t = TestGame::new(2);
    let urn = t.battlefield(P0, "Golden Urn");
    set_counters(&mut t, urn, "charge", 4);
    activate_resolve(&mut t, P0, urn, 0, &[]);
    assert!(t.in_graveyard(P0, "Golden Urn"));
    assert_eq!(t.life(P0), 24);
}

#[test]
fn an_implement_draws_whatever_put_it_into_a_graveyard() {
    cr!("603.6c", "603.10a");
    ruling!(
        "Implement of Combustion",
        "An Implement’s last ability triggers no matter why it’s put into a graveyard from the battlefield. You don’t have to activate its first ability."
    );
    for name in [
        "Implement of Combustion",
        "Implement of Examination",
        "Implement of Ferocity",
        "Implement of Improvement",
        "Implement of Malice",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let implement = t.battlefield(P0, name);
        let hand = t.hand_size(P0);
        t.g.destroy(implement, None);
        t.resolve_all();
        assert!(t.in_graveyard(P0, name));
        assert_eq!(t.hand_size(P0), hand + 1, "{name}");
    }
}

#[test]
fn an_implements_draw_resolves_first_and_its_target_is_chosen_before() {
    cr!("602.2b", "601.2c", "603.3", "405.5");
    ruling!(
        "Implement of Ferocity",
        "If you activate an Implement’s first ability, you’ll draw a card from its second ability before the first ability resolves. If the first ability has a target, you need a legal target to activate that ability, and you choose that target before seeing what card you’ll draw."
    );
    let mut t = TestGame::new(2);
    let implement = t.battlefield(P0, "Implement of Ferocity");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseTargets { .. }),
        |g| g.player(P0).hand.len(),
    );
    fill_pool(&mut t, P0, 1);
    t.activate(P0, implement, 0, &[obj(bears)]).expect("activate");
    assert_eq!(*seen.lock().unwrap(), vec![hand]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.g.resolve_top();
    t.settle();
    // The draw resolved; the counter isn't there yet.
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    // No creature to target: the ability can't be activated.
    let mut t = TestGame::new(2);
    let implement = t.battlefield(P0, "Implement of Ferocity");
    fill_pool(&mut t, P0, 1);
    assert!(t.activate(P0, implement, 0, &[]).is_err());
    assert!(t.on_battlefield(implement));
}

#[test]
fn gate_to_the_afterlife_sees_a_creature_dying_with_it() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Gate to the Afterlife",
        "If Gate to the Afterlife is put into a graveyard at the same time a nontoken creature you control dies, its first ability triggers for that creature."
    );
    supported("Gate to the Afterlife");
    supported("Akroma's Vengeance");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gate to the Afterlife");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    let vengeance = t.hand(P0, "Akroma's Vengeance");
    fill_pool(&mut t, P0, 6);
    t.answer_yes(P0, false).answer_yes(P0, false);
    t.cast(P0, vengeance).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Gate to the Afterlife"));
    assert_eq!(t.life(P0), 22);
}

#[test]
fn scourglass_spares_anything_that_is_an_artifact_or_a_land() {
    cr!("701.8a", "205.2b");
    ruling!(
        "Scourglass",
        "Any permanent that’s an artifact or a land won’t be destroyed, regardless of what other card types it may have."
    );
    supported("Scourglass");
    let mut t = TestGame::new(2);
    let glass = t.battlefield(P0, "Scourglass");
    let thopter = t.battlefield(P1, "Ornithopter");
    let arbor = t.battlefield(P1, "Dryad Arbor");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let pacifism = t.battlefield(P0, "Pacifism");
    t.g.attach(pacifism, obj(bears));
    t.set_step(P0, Step::Upkeep);
    activate_resolve(&mut t, P0, glass, 0, &[]);
    assert!(t.on_battlefield(thopter) && t.on_battlefield(arbor));
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(pacifism));
}

#[test]
fn powder_keg_destroys_exactly_the_mana_value_and_its_counter_is_optional() {
    cr!("701.8a", "202.3", "603.5");
    ruling!(
        "Powder Keg",
        "Only destroys artifacts and creatures with exactly the specified cost. It does not mean “less than or equal to”."
    );
    supported("Powder Keg");
    let mut t = TestGame::new(2);
    let keg = t.battlefield(P0, "Powder Keg");
    set_counters(&mut t, keg, "fuse", 2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let myr = t.battlefield(P1, "Darksteel Myr");
    let giant = t.battlefield(P1, "Hill Giant");
    let curator = t.battlefield(P1, "Angelic Curator");
    activate_resolve(&mut t, P0, keg, 0, &[]);
    assert!(t.on_battlefield(elves) && t.on_battlefield(giant) && t.on_battlefield(myr));
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(curator));
    ruling!(
        "Powder Keg",
        "Putting on a counter is optional. If you forget, you can’t go back later even if it is something you usually do."
    );
    let mut t = TestGame::new(2);
    let keg = t.battlefield(P0, "Powder Keg");
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(keg, "fuse"), 0);
    t.answer_yes(P0, true);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(keg, "fuse"), 1);
}

#[test]
fn eye_of_doom_lets_each_player_choose_untargetably_and_counters_stay() {
    cr!("702.16b", "115.1", "702.12b", "122.1");
    supported("Eye of Doom");
    ruling!(
        "Eye of Doom",
        "None of the permanents are targeted. A player can choose a permanent with protection from artifacts, for example."
    );
    ruling!(
        "Eye of Doom",
        "If a permanent with a doom counter isn’t destroyed by Eye of Doom’s last ability (perhaps because it has indestructible or it regenerated), the doom counter will remain on that permanent."
    );
    let mut t = TestGame::new(2);
    let curator = t.battlefield(P1, "Angelic Curator");
    let myr = t.battlefield(P0, "Darksteel Myr");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(myr)]);
    t.answer_choose(P1, &[obj(curator)]);
    let m = mark(&t);
    let eye = t.enter(P0, "Eye of Doom");
    t.resolve_all();
    let choosers: Vec<PlayerId> = t.asked()[m..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(choosers, vec![P0, P1]);
    assert!(choices_since(&t, P1, m)[0].0.contains(&obj(curator)));
    assert_eq!(t.counters(curator, "doom"), 1);
    assert_eq!(t.counters(myr, "doom"), 1);
    assert_eq!(t.counters(bears, "doom"), 0);
    activate_resolve(&mut t, P0, eye, 0, &[]);
    assert!(!t.on_battlefield(curator));
    assert!(t.on_battlefield(myr) && t.on_battlefield(bears));
    assert_eq!(t.counters(myr, "doom"), 1);
}

#[test]
fn abzan_monument_with_no_creatures_makes_a_0_0_spirit_that_dies() {
    cr!("608.2h", "704.5f");
    ruling!(
        "Abzan Monument",
        "If you control no creatures as Abzan Monument’s last ability resolves, X is 0. In this case, you’ll create a 0/0 white Spirit creature token, which will likely immediately die unless something else is raising its toughness."
    );
    supported("Abzan Monument");
    let mut t = TestGame::new(2);
    let monument = t.battlefield(P0, "Abzan Monument");
    // Impact Tremors: "Whenever a creature you control enters, this enchantment deals 1
    // damage to each opponent."
    t.battlefield(P0, "Impact Tremors");
    activate_resolve(&mut t, P0, monument, 0, &[]);
    assert_eq!(t.life(P1), 19);
    assert!(with_subtype(&t, P0, "Spirit").is_empty());
    // With a creature: an X/X Spirit, X its greatest toughness.
    let mut t = TestGame::new(2);
    let monument = t.battlefield(P0, "Abzan Monument");
    t.battlefield(P0, "Wall of Wood");
    activate_resolve(&mut t, P0, monument, 0, &[]);
    let spirit = with_subtype(&t, P0, "Spirit");
    assert_eq!(spirit.len(), 1);
    assert_eq!(t.pt(spirit[0]), (3, 3));
}

#[test]
fn tainted_sigil_counts_every_players_life_lost() {
    cr!("119.3", "608.2h", "704.5a");
    supported("Tainted Sigil");
    ruling!(
        "Tainted Sigil",
        "Tainted Sigil’s ability counts the life lost by all players, including you."
    );
    let mut t = TestGame::new(3);
    let sigil = t.battlefield(P0, "Tainted Sigil");
    t.g.lose_life(P0, 3);
    t.g.lose_life(P1, 2);
    t.g.lose_life(P2, 4);
    activate_resolve(&mut t, P0, sigil, 0, &[]);
    assert_eq!(t.life(P0), 17 + 9);
    ruling!(
        "Tainted Sigil",
        "Tainted Sigil’s ability checks only whether life was lost. It doesn’t care whether life was also gained."
    );
    let mut t = TestGame::new(2);
    let sigil = t.battlefield(P0, "Tainted Sigil");
    t.g.lose_life(P1, 4);
    t.g.gain_life(P1, 6);
    assert_eq!(t.life(P1), 22);
    activate_resolve(&mut t, P0, sigil, 0, &[]);
    assert_eq!(t.life(P0), 24);
    ruling!(
        "Tainted Sigil",
        "If you lose enough life to lower your life total to 0 or less, you’ll lose the game as a state-based actions before you activate Tainted Sigil’s ability."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Tainted Sigil");
    t.g.lose_life(P0, 20);
    t.settle();
    assert!(t.has_lost(P0));
    assert!(t.named_on_battlefield("Tainted Sigil").is_empty());
}

#[test]
fn five_hundred_year_diary_counts_itself_as_a_clue() {
    cr!("605.1a", "608.2h");
    ruling!(
        "Five Hundred Year Diary",
        "Five Hundred Year Diary counts itself for its second ability."
    );
    supported("Five Hundred Year Diary");
    let mut t = TestGame::new(2);
    let diary = t.battlefield(P0, "Five Hundred Year Diary");
    assert!(tap_for_mana(&mut t, P0, diary, "Add {U}"));
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::U), 1);
    let mut t = TestGame::new(2);
    let diary = t.battlefield(P0, "Five Hundred Year Diary");
    t.battlefield(P0, "Found Footage");
    t.battlefield(P0, "Esoteric Duplicator");
    assert!(tap_for_mana(&mut t, P0, diary, "Add {U}"));
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::U), 3);
}

#[test]
fn cryptexs_first_ability_is_a_mana_ability() {
    cr!("605.1a", "605.3a", "701.59a");
    ruling!(
        "Cryptex",
        "Cryptex's first ability is a mana ability. It doesn't use the stack and can't be responded to."
    );
    supported("Cryptex");
    let mut t = TestGame::new(2);
    let cryptex = t.battlefield(P0, "Cryptex");
    let giant = t.graveyard(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    let m = mark(&t);
    assert!(tap_for_mana(&mut t, P0, cryptex, "Add one mana"));
    assert_eq!(pool(&t, P0), 1);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(cryptex, "unlock"), 1);
    assert_eq!(t.zone(giant), Zone::Exile);
    assert!(who_was_asked(&t, m, |d| matches!(d, Decision::Priority { .. })).is_empty());
}

#[test]
fn jesters_cap_exiles_three_cards_or_all_of_a_smaller_library() {
    cr!("701.23a", "701.23d");
    ruling!(
        "Jester's Cap",
        "As long as there are three or more cards in that library, you must find three cards and exile them. If there are fewer than three cards in the player's library, you exile all of them."
    );
    supported("Jester's Cap");
    let mut t = TestGame::new(2);
    let cap = t.battlefield(P0, "Jester's Cap");
    let m = mark(&t);
    let lib = t.library_size(P1);
    // P0 tries to find only one card: three are exiled anyway.
    let one = t.g.player(P1).library[0];
    t.answer_choose(P0, &[obj(one)]);
    activate_resolve(&mut t, P0, cap, 0, &[pl(P1)]);
    let (_, min, max) = choices_since(&t, P0, m)[0].clone();
    assert_eq!((min, max), (3, 3));
    assert_eq!(t.library_size(P1), lib - 3);
    assert_eq!(t.g.exile.len(), 3);
    let mut t = TestGame::new(2);
    let cap = t.battlefield(P0, "Jester's Cap");
    t.g.mill(P1, 28);
    activate_resolve(&mut t, P0, cap, 0, &[pl(P1)]);
    assert_eq!(t.library_size(P1), 0);
    assert_eq!(t.g.exile.len(), 2);
}
