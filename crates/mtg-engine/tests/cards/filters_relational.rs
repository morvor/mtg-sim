//! Relational qualifiers in object phrases (`oracle/patterns/filters_relational.rs`,
//! `relational.rs`): comparisons with a referent or a value, extremes ("with the greatest
//! power"), sharing, names, exclusions, and requirements on objects chosen together.

use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::oracle::phrases::parse_object_phrase;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// The candidates of the last target choice `p` was asked to make.
fn last_target_candidates(t: &TestGame, p: PlayerId) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseTargets { candidates, .. } if q == p => Some(candidates),
            _ => None,
        })
        .last()
        .expect("no target choice")
}

#[test]
fn relational_cards_compile() {
    assert_compiles(&[
        // Comparisons with a referent or a value.
        "Havi, the All-Father",
        "Clement, the Worrywort",
        "Riveteers Ascendancy",
        "Vedalken Shackles",
        "Dominating Vampire",
        "Beguiler of Wills",
        "Gilt-Leaf Winnower",
        "Cut Down",
        "Meek Attack",
        "Immobilizer Eldrazi",
        "Drown in the Loch",
        "Covert Technician",
        // Extremes.
        "Porphyry Nodes",
        "Purging Scythe",
        "Desecrator Hag",
        "Topple",
        "Triumph of Gerrard",
        "Blot Out",
        "Flare of Malice",
        "Break Under Pressure",
        "Culling Scales",
        // Totals and different names.
        "Patch Up",
        "Protean Hulk",
        "Rampaging Yao Guai",
        "Shared Summons",
        "Behold the Sinister Six!",
        "Technomancer",
        // Sharing and names.
        "Spreading Plague",
        "Martyr's Bond",
        "Izzet Staticaster",
        "Mists of Lórien",
        "Crown of Vigor",
        "Haunted One",
        "Cylian Sunsinger",
        "Mistform Warchief",
        "Semblance Anvil",
        // Exclusions.
        "Kjeldoran Pride",
        "Due Diligence",
        "Saw",
        // Values of groups.
        "Ancient Ooze",
        "Dragon Man, Reformed Robot",
        "Omnipresence",
    ]);
}

#[test]
fn grammar_reads_relational_qualifiers() {
    // Each qualifier is read as a whole, leaving nothing behind.
    for phrase in [
        "creature card with lesser mana value",
        "creature an opponent controls with greater power",
        "creature with power less than or equal to the number of islands you control",
        "creature with toughness greater than its power",
        "creature with total power and toughness 5 or less",
        "creature cards with total mana value 6 or less",
        "creature cards with different names",
        "creature with the greatest power among creatures you control",
        "nonland permanent with the lowest mana value",
        "creatures that share a color with it",
        "creature that shares a creature type with enchanted creature",
        "creatures with the same name as that creature",
        "nonland permanent with the same mana value as that permanent",
        "creature other than enchanted creature",
        "card other than a basic land card",
        "permanent other than that creature or ~",
    ] {
        let (_, _, rest) =
            parse_object_phrase(phrase).unwrap_or_else(|| panic!("not parsed: {phrase}"));
        assert!(rest.trim().is_empty(), "{phrase}: left {rest:?}");
    }
    // "with toughness greater than its power": its own power, not another object's.
    let (f, _, _) = parse_object_phrase("creature with toughness greater than its power").unwrap();
    let j = serde_json::to_string(&f).unwrap();
    assert!(j.contains("ValueCmp"), "{j}");
    // "with total mana value 6 or less" is a requirement on the objects together.
    let (f, _, _) = parse_object_phrase("creature cards with total mana value 6 or less").unwrap();
    assert!(matches!(
        mtg_engine::relational::groups_of(&f).as_slice(),
        [TargetGroup::TotalAtMost(TotalStat::ManaValue, _)]
    ));
}

// ---------------------------------------------------------------------------
// Comparisons
// ---------------------------------------------------------------------------

#[test]
fn clement_returns_only_a_creature_with_lesser_mana_value_than_the_entering_one() {
    cr!("608.2b", "115.1");
    let mut t = TestGame::new(2);
    let clement = t.battlefield(P0, "Clement, the Worrywort"); // mana value 3
    let bears = t.battlefield(P0, "Grizzly Bears"); // 2
    let wurm = t.battlefield(P0, "Craw Wurm"); // 6
    t.answer_targets(P0, &[bears.into()]);
    let giant = t.enter(P0, "Hill Giant"); // 4
    t.settle();
    let cands = last_target_candidates(&t, P0);
    assert!(cands.contains(&bears.into()));
    assert!(cands.contains(&clement.into()));
    assert!(!cands.contains(&wurm.into()), "greater mana value");
    assert!(!cands.contains(&giant.into()), "equal mana value");
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn vedalken_shackles_compares_power_with_the_number_of_islands() {
    cr!("608.2h", "115.1");
    let mut t = TestGame::new(2);
    let shackles = t.battlefield(P0, "Vedalken Shackles");
    t.lands(P0, "Island", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 2);
    t.activate(P0, shackles, 0, &[bears.into()]).unwrap();
    let cands = last_target_candidates(&t, P0);
    assert!(cands.contains(&bears.into()));
    assert!(!cands.contains(&giant.into()), "power 3 is more than two Islands");
    t.resolve();
    assert_eq!(t.obj_now(bears).controller, P0);
}

#[test]
fn gilt_leaf_winnower_targets_creatures_whose_power_and_toughness_differ() {
    cr!("608.2b");
    ruling!(
        "Gilt-Leaf Winnower",
        "The power and toughness of the target non-Elf creature are checked again"
    );
    let mut t = TestGame::new(2);
    let lions = t.battlefield(P1, "Savannah Lions"); // 2/1
    let bears = t.battlefield(P1, "Grizzly Bears"); // 2/2
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[lions.into()]);
    t.enter(P0, "Gilt-Leaf Winnower");
    t.settle();
    let cands = last_target_candidates(&t, P0);
    assert!(cands.contains(&lions.into()));
    assert!(!cands.contains(&bears.into()));
    // Its toughness becomes equal to its power before the ability resolves: the target is
    // illegal and the ability doesn't resolve.
    t.g.add_counters(lions.into(), "+0/+1", 1, None);
    t.resolve_all();
    assert!(t.on_battlefield(lions));
}

#[test]
fn cut_down_adds_power_and_toughness() {
    cr!("608.2b");
    ruling!(
        "Cut Down",
        "The total power and toughness of a creature is determined by adding"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears"); // 2 + 2
    let giant = t.battlefield(P1, "Hill Giant"); // 3 + 3
    let lions = t.battlefield(P1, "Gray Ogre"); // 2 + 2
    t.lands(P0, "Swamp", 1);
    let c = t.hand(P0, "Cut Down");
    t.cast(P0, c).target(bears).go();
    let cands = last_target_candidates(&t, P0);
    assert!(cands.contains(&bears.into()));
    assert!(cands.contains(&lions.into()));
    assert!(!cands.contains(&giant.into()));
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn drown_in_the_loch_counts_the_targets_controllers_graveyard() {
    cr!("700.2", "608.2b");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant"); // mana value 4
    let wurm = t.battlefield(P1, "Craw Wurm"); // mana value 6
    for _ in 0..4 {
        t.graveyard(P1, "Island");
    }
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    let d = t.hand(P0, "Drown in the Loch");
    t.cast(P0, d).modes(&[1]).target(giant).go();
    let cands = last_target_candidates(&t, P0);
    assert!(cands.contains(&giant.into()));
    assert!(!cands.contains(&wurm.into()));
    t.resolve();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

// ---------------------------------------------------------------------------
// Extremes
// ---------------------------------------------------------------------------

#[test]
fn porphyry_nodes_destroys_the_creature_with_the_least_power() {
    cr!("608.2c");
    ruling!(
        "Porphyry Nodes",
        "The first ability of Porphyry Nodes doesn't target"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Porphyry Nodes");
    let elves = t.battlefield(P1, "Llanowar Elves"); // 1/1
    let bears = t.battlefield(P0, "Grizzly Bears"); // 2/2
    // Protection from white: not targeted, still destroyed.
    let knight = t.battlefield(P1, "Black Knight"); // 2/2, protection from white
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(elves));
    assert!(t.on_battlefield(bears));
    assert!(t.on_battlefield(knight));
    // Tied for least power: the controller chooses one of them.
    t.answer_choose(P0, &[knight.into()]);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(knight), "the chosen one");
    assert!(t.on_battlefield(bears));
}

#[test]
fn topple_targets_only_a_creature_with_the_greatest_power() {
    cr!("115.1", "608.2b");
    ruling!(
        "Topple",
        "if that creature is not still the one with greatest power on resolution"
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant"); // 3
    let ogre = t.battlefield(P0, "Gray Ogre"); // 2
    let giant2 = t.battlefield(P0, "Hill Giant"); // 3, tied
    t.lands(P0, "Plains", 3);
    let c = t.hand(P0, "Topple");
    t.cast(P0, c).target(giant).go();
    let cands = last_target_candidates(&t, P0);
    assert!(cands.contains(&giant.into()));
    assert!(cands.contains(&giant2.into()), "tied for the greatest");
    assert!(!cands.contains(&ogre.into()));
    // Another creature becomes stronger before Topple resolves: its target is illegal.
    t.g.add_counters(ogre.into(), "+1/+1", 2, None);
    t.resolve();
    assert!(t.on_battlefield(giant));
}

#[test]
fn blot_out_exiles_the_opponents_creature_or_planeswalker_with_the_greatest_mana_value() {
    cr!("608.2d");
    ruling!(
        "Blot Out",
        "Consider all creatures and planeswalkers the target opponent controls as one group"
    );
    ruling!("Blot Out", "A permanent with hexproof may be exiled this way");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant"); // 4
    let elspeth = t.battlefield(P1, "Elspeth, Sun's Champion"); // 6
    let mine = t.battlefield(P0, "Craw Wurm"); // 6, but not theirs
    t.lands(P0, "Swamp", 3);
    let c = t.hand(P0, "Blot Out");
    t.cast(P0, c).target(P1).go();
    t.resolve();
    assert!(t.in_exile("Elspeth, Sun's Champion"));
    assert!(t.on_battlefield(giant));
    assert!(t.on_battlefield(mine));
    let _ = elspeth;
}

#[test]
fn flare_of_malice_each_opponent_sacrifices_their_greatest() {
    cr!("701.21a", "608.2d");
    let mut t = TestGame::new(3);
    let a = t.battlefield(P1, "Grizzly Bears"); // 2
    let b = t.battlefield(P1, "Craw Wurm"); // 6
    let c = t.battlefield(P2, "Hill Giant"); // 4
    let d = t.battlefield(P2, "Llanowar Elves"); // 1
    let mine = t.battlefield(P0, "Craw Wurm");
    t.lands(P0, "Swamp", 5);
    let f = t.hand(P0, "Flare of Malice");
    t.cast(P0, f).go();
    t.resolve();
    assert!(t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
    assert!(!t.on_battlefield(c));
    assert!(t.on_battlefield(d));
    assert!(t.on_battlefield(mine));
}

// ---------------------------------------------------------------------------
// Totals and different names
// ---------------------------------------------------------------------------

#[test]
fn patch_up_targets_must_have_total_mana_value_three_or_less() {
    cr!("601.2c", "115.1");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears"); // 2
    let elves = t.graveyard(P0, "Llanowar Elves"); // 1
    let giant = t.graveyard(P0, "Hill Giant"); // 4
    t.lands(P0, "Plains", 3);
    let c = t.hand(P0, "Patch Up");
    // Bears and Hill Giant together are too much: only a legal group is chosen.
    t.cast(P0, c)
        .targets(&[bears.into(), giant.into()])
        .go();
    t.resolve();
    assert!(t.named_on_battlefield("Grizzly Bears").len() == 1);
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
    assert!(t.in_graveyard(P0, "Hill Giant"));
    let _ = elves;
}

#[test]
fn patch_up_returns_cards_whose_total_is_within_the_limit() {
    cr!("601.2c");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears"); // 2
    let elves = t.graveyard(P0, "Llanowar Elves"); // 1
    t.lands(P0, "Plains", 3);
    let c = t.hand(P0, "Patch Up");
    t.cast(P0, c).targets(&[bears.into(), elves.into()]).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
}

#[test]
fn protean_hulk_finds_creatures_with_total_mana_value_six_or_less() {
    cr!("701.23a");
    ruling!(
        "Protean Hulk",
        "You can find any number of creature cards, so long as their total mana value is 6 or less"
    );
    let mut t = TestGame::new(2);
    let hulk = t.battlefield(P0, "Protean Hulk");
    let w = t.library_top(P0, "Hill Giant"); // 4
    let x = t.library_top(P0, "Hill Giant"); // 4
    let y = t.library_top(P0, "Grizzly Bears"); // 2
    // Choosing both Hill Giants (8) isn't allowed: the cards found are a legal group.
    t.answer_choose(P0, &[w.into(), x.into(), y.into()]);
    t.g.destroy(hulk, None);
    t.resolve_all();
    let total: u32 = t
        .g
        .battlefield
        .iter()
        .filter(|o| t.g.obj(**o).controller == P0)
        .map(|o| t.g.mana_value_of(*o))
        .sum();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(total, 6);
}

#[test]
fn shared_summons_finds_cards_with_different_names() {
    cr!("201.2", "701.23a");
    let mut t = TestGame::new(2);
    let a = t.library_top(P0, "Grizzly Bears");
    let b = t.library_top(P0, "Grizzly Bears");
    let giant = t.library_top(P0, "Hill Giant");
    t.lands(P0, "Forest", 5);
    let c = t.hand(P0, "Shared Summons");
    t.answer_choose(P0, &[a.into(), giant.into()]);
    t.cast(P0, c).go();
    t.resolve();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Hill Giant"));
    // Two Grizzly Bears don't have different names: they can't both be found.
    let mut t = TestGame::new(2);
    let a = t.library_top(P0, "Grizzly Bears");
    let b2 = t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 5);
    let c = t.hand(P0, "Shared Summons");
    t.answer_choose(P0, &[a.into(), b2.into()]);
    t.cast(P0, c).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 1);
    let _ = b;
}

// ---------------------------------------------------------------------------
// Sharing, names, exclusions
// ---------------------------------------------------------------------------

#[test]
fn spreading_plague_destroys_other_creatures_sharing_a_color_with_the_entering_one() {
    cr!("105.2", "608.2c");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spreading Plague");
    let bears = t.battlefield(P1, "Grizzly Bears"); // green
    let giant = t.battlefield(P1, "Hill Giant"); // red
    let elves = t.enter(P0, "Llanowar Elves"); // green
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(giant));
    assert!(t.on_battlefield(elves), "other creatures only");
}

#[test]
fn izzet_staticaster_damages_every_other_creature_named_like_the_target() {
    cr!("201.2", "608.2c");
    ruling!(
        "Izzet Staticaster",
        "Other creatures with that name are not targeted"
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Izzet Staticaster"); // 0/3
    let theirs = t.battlefield(P1, "Izzet Staticaster");
    let third = t.battlefield(P1, "Izzet Staticaster");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, mine, 0, &[theirs.into()]).unwrap();
    t.resolve_all();
    // The target and each other creature with its name, the source included, once each.
    assert_eq!(t.obj_now(theirs).damage, 1);
    assert_eq!(t.obj_now(third).damage, 1);
    assert_eq!(t.obj_now(mine).damage, 1);
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn crown_of_vigor_pumps_the_enchanted_creature_and_those_sharing_a_type() {
    cr!("608.2c", "205.3m");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves"); // Elf Druid
    let other_elf = t.battlefield(P1, "Elvish Mystic"); // Elf Druid
    let bears = t.battlefield(P0, "Grizzly Bears"); // Bear
    t.lands(P0, "Forest", 2);
    let crown = t.hand(P0, "Crown of Vigor");
    t.cast(P0, crown).target(elves).go();
    t.resolve();
    let crown = t.named_on_battlefield("Crown of Vigor")[0];
    t.activate(P0, crown, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(elves), (2, 2));
    assert_eq!(t.pt(other_elf), (2, 2));
    assert_eq!(t.pt(bears), (2, 2), "no creature type in common");
}

#[test]
fn kjeldoran_pride_moves_only_to_a_creature_other_than_the_enchanted_one() {
    cr!("115.1");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Island", 3);
    let pride = t.hand(P0, "Kjeldoran Pride");
    t.cast(P0, pride).target(bears).go();
    t.resolve();
    let pride = t.named_on_battlefield("Kjeldoran Pride")[0];
    t.activate(P0, pride, 0, &[giant.into()]).unwrap();
    let cands = last_target_candidates(&t, P0);
    assert!(cands.contains(&giant.into()));
    assert!(!cands.contains(&bears.into()));
}

// ---------------------------------------------------------------------------
// Values of groups
// ---------------------------------------------------------------------------

#[test]
fn ancient_ooze_counts_the_total_mana_value_of_other_creatures() {
    cr!("604.3", "202.3");
    let mut t = TestGame::new(2);
    let ooze = t.battlefield(P0, "Ancient Ooze");
    t.battlefield(P0, "Grizzly Bears"); // 2
    t.battlefield(P0, "Hill Giant"); // 4
    t.battlefield(P1, "Craw Wurm"); // not yours
    t.g.recompute();
    assert_eq!(t.pt(ooze), (6, 6));
}


#[test]
fn uncage_the_menagerie_finds_creatures_with_mana_value_x_and_different_names() {
    cr!("201.2", "107.3a");
    let mut t = TestGame::new(2);
    let a = t.library_top(P0, "Grizzly Bears"); // 2
    let b = t.library_top(P0, "Grizzly Bears"); // 2
    let ogre = t.library_top(P0, "Gray Ogre"); // 3
    let elf = t.library_top(P0, "Elvish Mystic"); // 1
    t.lands(P0, "Forest", 4);
    let c = t.hand(P0, "Uncage the Menagerie");
    t.answer_choose(P0, &[a.into(), b.into()]);
    t.cast(P0, c).x(2).go();
    t.resolve();
    // Only cards with mana value 2 qualify, and two Grizzly Bears share a name.
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    let _ = (ogre, elf);
}

#[test]
fn booby_trap_names_a_card_other_than_a_basic_land() {
    cr!("201.4", "607.5a");
    let mut t = TestGame::new(2);
    // A basic land card's name isn't a legal choice: nothing is named.
    t.answer(P0, DecisionKind::Name, Answer::Text("Forest".into()));
    let a = t.enter(P0, "Booby Trap");
    assert_eq!(t.obj_now(a).choices.card_name.as_deref(), Some(""));
    // A nonbasic land's name is.
    t.answer(P0, DecisionKind::Name, Answer::Text("Wasteland".into()));
    let b = t.enter(P0, "Booby Trap");
    assert_eq!(t.obj_now(b).choices.card_name.as_deref(), Some("Wasteland"));
    t.answer(P0, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    let c = t.enter(P0, "Booby Trap");
    assert_eq!(
        t.obj_now(c).choices.card_name.as_deref(),
        Some("Grizzly Bears")
    );
}

#[test]
fn reciprocate_exiles_only_a_creature_that_dealt_damage_to_you_this_turn() {
    cr!("120.3a", "400.7");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    // Grizzly Bears deals combat damage to P0.
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 18);
    t.lands(P0, "Plains", 1);
    let r = t.hand(P0, "Reciprocate");
    t.cast(P0, r).target(bears).go();
    let cands = last_target_candidates(&t, P0);
    assert!(cands.contains(&bears.into()));
    assert!(!cands.contains(&giant.into()), "didn't deal damage to you");
    t.resolve();
    assert!(t.in_exile("Grizzly Bears"));
}

// ---------------------------------------------------------------------------
// Groups related to a target or the source
// ---------------------------------------------------------------------------

#[test]
fn legions_to_ashes_exiles_the_target_and_that_players_tokens_with_its_name() {
    cr!("201.2", "608.2c");
    ruling!("Legions to Ashes", "The target permanent need not be a token");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears"); // not a token: stays
    let mine = t.battlefield(P0, "Grizzly Bears"); // not that player's
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 2);
    let c = t.hand(P0, "Legions to Ashes");
    t.cast(P0, c).target(bears).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(other), "only tokens with that name");
    assert!(t.on_battlefield(mine));
}

#[test]
fn deputy_of_detention_exiles_that_players_other_permanents_with_the_name() {
    cr!("201.2", "610.3");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[a.into()]);
    t.enter(P0, "Deputy of Detention");
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
    assert!(t.on_battlefield(giant));
    assert!(t.on_battlefield(mine), "an opponent's only");
}

#[test]
fn surgical_extraction_may_leave_cards_even_in_the_graveyard() {
    cr!("701.23b", "201.2");
    ruling!(
        "Surgical Extraction",
        "you can choose to leave some or all of the cards with the same name"
    );
    let mut t = TestGame::new(2);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    let lib = t.library_top(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    let s = t.hand(P0, "Surgical Extraction");
    // The graveyard: leave the other copy. The hand: take it. The library: leave it.
    t.answer_choose(P0, &[]);
    let in_hand = t.g.player(P1).hand[0];
    t.answer_choose(P0, &[in_hand.into()]);
    t.answer_choose(P0, &[]);
    t.cast(P0, s).target(a).go();
    t.resolve();
    assert_eq!(t.zone(t.g.current(b)), mtg_engine::object::Zone::Graveyard(P1));
    assert!(!t.in_hand(P1, "Grizzly Bears"));
    assert!(t.g.player(P1).library.contains(&t.g.current(lib)));
}

#[test]
fn extirpate_exiles_every_copy_in_the_graveyard() {
    cr!("701.23b", "201.2");
    ruling!(
        "Extirpate",
        "but you do have to exile the cards from the player's graveyard"
    );
    let mut t = TestGame::new(2);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Grizzly Bears");
    let giant = t.graveyard(P1, "Hill Giant");
    t.lands(P0, "Swamp", 1);
    let s = t.hand(P0, "Extirpate");
    t.cast(P0, s).target(a).go();
    t.resolve();
    assert!(t.in_exile("Grizzly Bears"));
    assert_ne!(t.zone(t.g.current(b)), mtg_engine::object::Zone::Graveyard(P1));
    assert_eq!(t.zone(giant), mtg_engine::object::Zone::Graveyard(P1));
}

#[test]
fn steel_hellkite_destroys_only_permanents_of_players_it_dealt_combat_damage() {
    cr!("510.2", "107.3a");
    ruling!(
        "Steel Hellkite",
        "destroys only nonland permanents whose mana value is exactly equal to X"
    );
    let mut t = TestGame::new(3);
    let hellkite = t.battlefield(P0, "Steel Hellkite");
    let bears = t.battlefield(P1, "Grizzly Bears"); // mana value 2
    let giant = t.battlefield(P1, "Hill Giant"); // 4
    let other = t.battlefield(P2, "Grizzly Bears"); // P2 wasn't dealt damage
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(hellkite, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 15);
    t.lands(P0, "Plains", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, hellkite, 1, &[]).unwrap();
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(giant), "mana value isn't X");
    assert!(t.on_battlefield(other));
}

#[test]
fn yorvo_gets_a_second_counter_only_if_the_creature_is_still_stronger() {
    cr!("608.2c");
    ruling!(
        "Yorvo, Lord of Garenbrig",
        "compares the power of the entering green creature only after putting a +1/+1 counter"
    );
    let mut t = TestGame::new(2);
    let yorvo = t.enter(P0, "Yorvo, Lord of Garenbrig"); // 0/0 with four +1/+1 counters
    let base = t.counters(yorvo, "+1/+1");
    // A 6/4 green creature: greater than Yorvo's power even after the first counter.
    t.enter(P0, "Craw Wurm");
    t.resolve_all();
    assert_eq!(t.counters(yorvo, "+1/+1"), base + 2);
    // A 1/1: not greater.
    t.enter(P0, "Llanowar Elves");
    t.resolve_all();
    assert_eq!(t.counters(yorvo, "+1/+1"), base + 3);
}

#[test]
fn cloudstone_curio_returns_a_permanent_sharing_a_permanent_type() {
    cr!("110.4", "608.2c");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cloudstone Curio");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let forest = t.battlefield(P0, "Forest");
    t.answer_yes(P0, true);
    let giant = t.enter(P0, "Hill Giant");
    t.resolve_all();
    // Only a creature (another permanent sharing a permanent type with it) could return.
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.on_battlefield(forest));
    assert!(t.on_battlefield(giant));
    let _ = bears;
}

#[test]
fn spawnbroker_compares_with_the_first_targets_power() {
    cr!("115.1", "608.2b");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant"); // power 3
    let small = t.battlefield(P1, "Grizzly Bears"); // 2
    let big = t.battlefield(P1, "Craw Wurm"); // 6
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[mine.into()]);
    t.answer_targets(P0, &[small.into()]);
    t.enter(P0, "Spawnbroker");
    t.settle();
    let cands = last_target_candidates(&t, P0);
    assert!(cands.contains(&small.into()));
    assert!(!cands.contains(&big.into()));
    t.resolve_all();
    assert_eq!(t.obj_now(small).controller, P0);
    assert_eq!(t.obj_now(mine).controller, P1);
}

#[test]
fn highcliff_felidar_destroys_each_opponents_greatest_creature() {
    cr!("608.2d", "101.4");
    let mut t = TestGame::new(3);
    let a = t.battlefield(P1, "Grizzly Bears"); // 2
    let b = t.battlefield(P1, "Hill Giant"); // 3
    let c = t.battlefield(P2, "Llanowar Elves"); // 1
    let d = t.battlefield(P2, "Craw Wurm"); // 6
    let mine = t.battlefield(P0, "Craw Wurm");
    t.enter(P0, "Highcliff Felidar");
    t.resolve_all();
    assert!(t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
    assert!(t.on_battlefield(c));
    assert!(!t.on_battlefield(d));
    assert!(t.on_battlefield(mine));
}

#[test]
fn an_ability_after_a_comma_list_describes_its_last_item() {
    cr!("115.1", "702.9b");
    // Return to the Earth: "Destroy target artifact, enchantment, or creature with flying."
    let mut t = TestGame::new(2);
    let artifact = t.battlefield(P1, "Ornithopter"); // an artifact creature with flying
    let relic = t.battlefield(P1, "Howling Mine"); // an artifact without flying
    let bears = t.battlefield(P1, "Grizzly Bears"); // a creature without flying
    let drake = t.battlefield(P1, "Wind Drake"); // a creature with flying
    t.lands(P0, "Forest", 4);
    let s = t.hand(P0, "Return to the Earth");
    t.cast(P0, s).target(relic).go();
    let cands = last_target_candidates(&t, P0);
    assert!(cands.contains(&relic.into()), "any artifact");
    assert!(cands.contains(&artifact.into()));
    assert!(cands.contains(&drake.into()));
    assert!(!cands.contains(&bears.into()), "only a creature with flying");
    t.resolve();
    assert!(!t.on_battlefield(relic));
    // Exorcise: "... or creature with power 4 or greater": a power only for creatures.
    let (f, _, _) = parse_object_phrase("artifact, enchantment, or creature with power 4 or greater")
        .unwrap();
    assert!(matches!(&f, Filter::Or(v) if matches!(v[0], Filter::Type(_))), "{f:?}");
    // Mana value describes them all.
    let (f, _, _) =
        parse_object_phrase("artifact, creature, or enchantment with mana value 3 or less").unwrap();
    assert!(matches!(&f, Filter::And(_)), "{f:?}");
}
