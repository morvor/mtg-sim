//! Rulings batch P224 (review) — each wording newly compiled by this batch: objects
//! "named ~" in counts, triggers, subjects and player conditions (CR 201.2a), and
//! "return a [kind] card from your graveyard to the battlefield tapped / with a finality
//! counter on it" (CR 608.2c, 122.1h).

use crate::r_s01_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn pack_mastiff_pumps_each_creature_you_control_named_pack_mastiff() {
    cr!("201.2a", "602.2");
    supported("Pack Mastiff");
    // "{1}{R}: Each creature you control named Pack Mastiff gets +1/+0 until end of turn."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Pack Mastiff");
    let b = t.battlefield(P0, "Pack Mastiff");
    let theirs = t.battlefield(P1, "Pack Mastiff");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let (pa, pb, pt, pbears) = (t.pt(a).0, t.pt(b).0, t.pt(theirs).0, t.pt(bears).0);
    t.lands(P0, "Mountain", 2);
    t.activate(P0, a, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(a).0, pa + 1);
    assert_eq!(t.pt(b).0, pb + 1);
    assert_eq!(t.pt(theirs).0, pt);
    assert_eq!(t.pt(bears).0, pbears);
}

#[test]
fn genasi_enforcers_pump_creatures_you_control_named_genasi_enforcers() {
    cr!("201.2a", "602.2");
    supported("Genasi Enforcers");
    // "{1}{R}: Creatures you control named Genasi Enforcers get +1/+0 until end of turn."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Genasi Enforcers");
    let b = t.battlefield(P0, "Genasi Enforcers");
    let theirs = t.battlefield(P1, "Genasi Enforcers");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let (pa, pb, pt, pbears) = (t.pt(a).0, t.pt(b).0, t.pt(theirs).0, t.pt(bears).0);
    t.lands(P0, "Mountain", 2);
    t.activate(P0, a, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(a).0, pa + 1);
    assert_eq!(t.pt(b).0, pb + 1);
    assert_eq!(t.pt(theirs).0, pt);
    assert_eq!(t.pt(bears).0, pbears);
}

#[test]
fn leitmotif_composer_makes_creatures_named_leitmotif_composer_unblockable() {
    cr!("201.2a", "509.1b");
    supported("Leitmotif Composer");
    // "{2}{U}: Creatures named Leitmotif Composer can't be blocked this turn."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Leitmotif Composer");
    let b = t.battlefield(P0, "Leitmotif Composer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Wood");
    t.lands(P0, "Island", 3);
    t.activate(P0, a, 0, &[]).unwrap();
    t.resolve_all();
    attack_with(
        &mut t,
        &[
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
        ],
    );
    assert!(!t.g.can_block(wall, a));
    assert!(!t.g.can_block(wall, b));
    assert!(t.g.can_block(wall, bears));
}

#[test]
fn charmed_stray_puts_a_counter_on_each_other_creature_you_control_named_charmed_stray() {
    cr!("201.2a", "603.2");
    supported("Charmed Stray");
    // "When this creature enters, put a +1/+1 counter on each other creature you control
    // named Charmed Stray."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Charmed Stray");
    let b = t.battlefield(P0, "Charmed Stray");
    let theirs = t.battlefield(P1, "Charmed Stray");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let new = t.enter(P0, "Charmed Stray");
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 1);
    assert_eq!(t.counters(b, counters::PLUS1), 1);
    assert_eq!(t.counters(new, counters::PLUS1), 0);
    assert_eq!(t.counters(theirs, counters::PLUS1), 0);
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
}

#[test]
fn gladewalker_ritualist_draws_when_another_ritualist_you_control_enters() {
    cr!("201.2a", "603.2", "603.6a");
    supported("Gladewalker Ritualist");
    // "Changeling. Whenever another creature you control named Gladewalker Ritualist
    // enters, draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gladewalker Ritualist");
    let hand = t.hand_size(P0);
    // Not another Ritualist (a changeling has every creature type, not that name), not
    // one P1 controls.
    t.enter(P0, "Grizzly Bears");
    t.enter(P1, "Gladewalker Ritualist");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // Another Ritualist: only the first one's ability triggers ("another").
    t.enter(P0, "Gladewalker Ritualist");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn faerie_miscreant_draws_only_if_you_control_another_miscreant() {
    cr!("201.2a", "603.4");
    supported("Faerie Miscreant");
    // "When this creature enters, if you control another creature named Faerie
    // Miscreant, draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Faerie Miscreant");
    let hand = t.hand_size(P0);
    t.enter(P0, "Faerie Miscreant");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    t.enter(P0, "Faerie Miscreant");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn biovisionary_wins_with_four_creatures_named_biovisionary() {
    cr!("201.2a", "104.2b", "603.4");
    supported("Biovisionary");
    // "At the beginning of the end step, if you control four or more creatures named
    // Biovisionary, you win the game."
    let run = |mine: usize, theirs: usize| {
        let mut t = TestGame::new(2);
        for _ in 0..mine {
            t.battlefield(P0, "Biovisionary");
        }
        for _ in 0..theirs {
            t.battlefield(P1, "Biovisionary");
        }
        t.advance_to(P0, Step::End);
        t.resolve_all();
        t.has_lost(P1)
    };
    assert!(!run(3, 1));
    assert!(run(4, 0));
}

#[test]
fn bonders_ornament_draws_for_each_player_who_controls_one() {
    cr!("201.2a", "602.2");
    supported("Bonder's Ornament");
    // "{4}, {T}: Each player who controls a permanent named Bonder's Ornament draws a
    // card."
    let mut t = TestGame::new(3);
    let mine = t.battlefield(P0, "Bonder's Ornament");
    t.battlefield(P1, "Bonder's Ornament");
    let before = [t.hand_size(P0), t.hand_size(P1), t.hand_size(P2)];
    t.lands(P0, "Wastes", 4);
    t.activate(P0, mine, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(
        [t.hand_size(P0), t.hand_size(P1), t.hand_size(P2)],
        [before[0] + 1, before[1] + 1, before[2]]
    );
}

#[test]
fn blossoming_tortoise_returns_a_land_card_tapped() {
    cr!("701.17a", "608.2c");
    supported("Blossoming Tortoise");
    // "Whenever this creature enters or attacks, mill three cards, then return a land
    // card from your graveyard to the battlefield tapped."
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Grizzly Bears", "Forest", "Island"]);
    t.answer_choose(P0, &[Entity::Object(cards[1])]);
    t.enter(P0, "Blossoming Tortoise");
    t.resolve_all();
    let forest = t.g.current(cards[1]);
    assert!(t.on_battlefield(forest));
    assert!(t.obj(forest).tapped);
    assert_eq!(t.zone(t.g.current(cards[2])), Zone::Graveyard(P0));
}

#[test]
fn port_of_karfell_returns_a_creature_card_tapped() {
    cr!("701.17a", "608.2c");
    supported("Port of Karfell");
    // "{3}{U}{B}{B}, {T}, Sacrifice this land: Mill four cards, then return a creature
    // card from your graveyard to the battlefield tapped."
    let mut t = TestGame::new(2);
    let port = t.battlefield(P0, "Port of Karfell");
    let cards = stack_library(&mut t, P0, &["Hill Giant", "Forest", "Island", "Swamp"]);
    t.answer_choose(P0, &[Entity::Object(cards[0])]);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 4);
    t.activate(P0, port, 1, &[]).unwrap();
    t.resolve_all();
    let giant = t.g.current(cards[0]);
    assert!(t.on_battlefield(giant));
    assert!(t.obj(giant).tapped);
    assert!(t.in_graveyard(P0, "Port of Karfell"));
}

#[test]
fn teval_may_return_a_land_card_tapped() {
    cr!("701.17a", "608.2c");
    supported("Teval, the Balanced Scale");
    // "Whenever Teval attacks, mill three cards. Then you may return a land card from
    // your graveyard to the battlefield tapped."
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let teval = t.battlefield(P0, "Teval, the Balanced Scale");
        let cards = stack_library(&mut t, P0, &["Grizzly Bears", "Forest", "Island"]);
        t.answer_yes(P0, yes);
        t.answer_choose(P0, &[Entity::Object(cards[1])]);
        attack_with(&mut t, &[(teval, Entity::Player(P1))]);
        t.resolve_all();
        let forest = t.g.current(cards[1]);
        assert_eq!(t.on_battlefield(forest), yes);
        if yes {
            assert!(t.obj(forest).tapped);
        }
        assert_eq!(t.graveyard_size(P0), if yes { 2 } else { 3 });
    }
}

#[test]
fn scavengers_talent_level_3_returns_a_creature_with_a_finality_counter() {
    cr!("716.2a", "122.1h", "608.2c");
    supported("Scavenger's Talent");
    // Level 3: "At the beginning of your end step, you may sacrifice three other nonland
    // permanents. If you do, return a creature card from your graveyard to the
    // battlefield with a finality counter on it."
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Scavenger's Talent");
    mtg_engine::classes::set_level(&mut t.g, class, 3);
    t.g.recompute();
    let giant = t.graveyard(P0, "Hill Giant");
    let fodder: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Ornithopter")).collect();
    t.answer_yes(P0, true);
    t.answer_choose(
        P0,
        &fodder.iter().map(|f| Entity::Object(*f)).collect::<Vec<_>>(),
    );
    t.answer_choose(P0, &[Entity::Object(giant)]);
    // (The level 2 ability, "Whenever you sacrifice a permanent, target player mills two
    // cards", triggers three times; its targets are the default ones.)
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(fodder.iter().all(|f| !t.on_battlefield(*f)));
    let giant = t.g.current(giant);
    assert!(t.on_battlefield(giant));
    assert!(!t.obj(giant).tapped);
    assert_eq!(t.counters(giant, counters::FINALITY), 1);
}
