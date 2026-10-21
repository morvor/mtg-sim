//! Structure-coverage tests, stream 2 (item `structure-tests-2`): spell-ability structures
//! that alone kept their cards from being verified (see `docs/CARD_COVERAGE.md`). Each test
//! checks that a real card's spell does what its text says.

use super::structure_activated_2::{next_turn, shuffled, target_choices, with_p1p1};
use super::structure_activated_6::entity_choices;
use super::structure_seed_common::*;
use super::structure_triggered_5::obj;
use super::structure_triggered_7::{kill, tokens};
use mtg_engine::combat::{block_declaration_legal, block_options};
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{Color, ColorSet};
use mtg_engine::*;

/// Puts `name` in P0's hand with `n` `land`s to pay for it.
fn ready(t: &mut TestGame, name: &str, land: &str, n: usize) -> ObjectId {
    t.lands(P0, land, n);
    t.hand(P0, name)
}

#[test]
fn vivisurgeons_insight_draws_three_and_proliferates() {
    cr!("121.1", "701.34a");
    // "Draw three cards. Proliferate."
    supported("Vivisurgeon's Insight");
    let mut t = TestGame::new(2);
    let mine = with_p1p1(&mut t, P0, "Grizzly Bears", 1);
    let skipped = with_p1p1(&mut t, P0, "Hill Giant", 1);
    t.g.add_counters(Entity::Player(P1), "poison", 1, None);
    let c = ready(&mut t, "Vivisurgeon's Insight", "Island", 5);
    t.answer_choose(P0, &[obj(mine), Entity::Player(P1)]);
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.counters(mine, "+1/+1"), 2);
    assert_eq!(t.counters(skipped, "+1/+1"), 1, "not chosen");
    assert_eq!(t.g.player(P1).counters.get("poison").copied(), Some(2));
}

#[test]
fn molten_blast_deals_two_damage_or_destroys_an_artifact() {
    cr!("700.2a", "601.2b", "115.1");
    // "Choose one — • ~ deals 2 damage to target creature or planeswalker. • Destroy
    // target artifact."
    supported("Molten Blast");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ring = t.battlefield(P1, "Sol Ring");
    let walker = t.battlefield(P1, "Ajani Goldmane");
    let c = ready(&mut t, "Molten Blast", "Mountain", 3);
    t.cast(P0, c).modes(&[0]).target(bears).go();
    let ch = target_choices(&t, P0).pop().unwrap();
    assert!(ch.contains(&obj(bears)) && ch.contains(&obj(walker)));
    assert!(!ch.contains(&obj(ring)) && !ch.contains(&Entity::Player(P1)));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(ring));
    let c = ready(&mut t, "Molten Blast", "Mountain", 3);
    t.cast(P0, c).modes(&[1]).target(ring).go();
    let ch = target_choices(&t, P0).pop().unwrap();
    assert!(
        ch.contains(&obj(ring)) && !ch.contains(&obj(walker)),
        "artifacts"
    );
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Sol Ring"));
    assert_eq!(t.counters(walker, "loyalty"), 4, "untouched");
}

#[test]
fn death_denied_returns_x_creature_cards() {
    cr!("107.3a", "115.1");
    // "Return X target creature cards from your graveyard to your hand."
    supported("Death Denied");
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    let c3 = t.graveyard(P0, "Craw Wurm");
    let land = t.graveyard(P0, "Forest");
    let theirs = t.graveyard(P1, "Grizzly Bears");
    let c = ready(&mut t, "Death Denied", "Swamp", 4);
    t.cast(P0, c).x(2).targets(&[obj(a), obj(b)]).go();
    let ch = target_choices(&t, P0);
    for slot in &ch {
        assert!(!slot.contains(&obj(land)) && !slot.contains(&obj(theirs)));
    }
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Hand(P0));
    assert_eq!(t.zone(b), Zone::Hand(P0));
    assert_eq!(t.zone(c3), Zone::Graveyard(P0), "only X of them");
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn dance_with_devils_makes_two_devils_that_ping_when_they_die() {
    cr!("111.1", "603.6c", "115.1");
    // "Create two 1/1 red Devil creature tokens. They have "When this token dies, it
    // deals 1 damage to any target.""
    supported("Dance with Devils");
    let mut t = TestGame::new(2);
    let c = ready(&mut t, "Dance with Devils", "Mountain", 4);
    t.cast(P0, c).go();
    t.resolve_all();
    let d = tokens(&t, P0, "Devil");
    assert_eq!(d.len(), 2);
    assert!(tokens(&t, P1, "Devil").is_empty());
    for x in &d {
        assert_eq!(t.pt(*x), (1, 1));
        assert_eq!(t.obj_now(*x).chars.colors, ColorSet::single(Color::Red));
    }
    t.answer_targets(P0, &[Entity::Player(P1)]);
    kill(&mut t, d[0]);
    assert_eq!(t.life(P1), 19);
    assert_eq!(tokens(&t, P0, "Devil").len(), 1);
}

#[test]
fn prison_break_returns_a_creature_card_with_a_counter() {
    cr!("122.6", "115.1");
    // "Return target creature card from your graveyard to the battlefield with an
    // additional +1/+1 counter on it."
    supported("Prison Break");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let land = t.graveyard(P0, "Forest");
    let theirs = t.graveyard(P1, "Hill Giant");
    let c = ready(&mut t, "Prison Break", "Swamp", 5);
    t.cast(P0, c).target(bears).go();
    let ch = target_choices(&t, P0).pop().unwrap();
    assert!(!ch.contains(&obj(land)) && !ch.contains(&obj(theirs)));
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(b.len(), 1);
    assert_eq!(t.obj_now(b[0]).controller, P0);
    assert_eq!(t.counters(b[0], "+1/+1"), 1);
    assert_eq!(t.pt(b[0]), (3, 3));
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn melt_terrain_destroys_a_land_and_damages_its_controller() {
    cr!("701.8a", "120.3a");
    // "Destroy target land. ~ deals 2 damage to that land's controller."
    supported("Melt Terrain");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let c = ready(&mut t, "Melt Terrain", "Mountain", 4);
    t.cast(P0, c).target(land).go();
    let ch = target_choices(&t, P0).pop().unwrap();
    assert!(!ch.contains(&obj(bears)), "lands only");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Forest"));
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
    // Your own land: you're dealt the damage.
    let mut t = TestGame::new(2);
    let c = ready(&mut t, "Melt Terrain", "Mountain", 4);
    let mine = t.battlefield(P0, "Forest");
    t.cast(P0, c).target(mine).go();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (18, 20));
}

#[test]
fn lunar_frenzy_gives_plus_x_first_strike_and_trample() {
    cr!("107.3a", "611.2a");
    // "Target creature you control gets +X/+0 and gains first strike and trample until
    // end of turn."
    supported("Lunar Frenzy");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let c = ready(&mut t, "Lunar Frenzy", "Mountain", 4);
    t.cast(P0, c).x(3).target(bears).go();
    let ch = target_choices(&t, P0).pop().unwrap();
    assert!(!ch.contains(&obj(theirs)), "a creature you control");
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 2));
    for k in [KeywordKind::FirstStrike, KeywordKind::Trample] {
        assert!(t.obj_now(bears).has_keyword(k));
        assert!(!t.obj_now(theirs).has_keyword(k));
    }
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Trample));
}

#[test]
fn eladamris_call_tutors_a_creature_card() {
    cr!("701.23a", "701.20a");
    // "Search your library for a creature card, reveal that card, put it into your hand,
    // then shuffle."
    supported("Eladamri's Call");
    let mut t = TestGame::new(2);
    let giant = t.library_top(P0, "Hill Giant");
    let land = t.library_top(P0, "Forest");
    t.lands(P0, "Forest", 1);
    let c = ready(&mut t, "Eladamri's Call", "Plains", 1);
    t.answer_choose(P0, &[obj(giant)]);
    t.cast(P0, c).go();
    t.resolve_all();
    let ch = entity_choices(&t, P0).pop().unwrap();
    assert!(
        ch.contains(&obj(giant)) && !ch.contains(&obj(land)),
        "creature cards"
    );
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.hand_size(P0), 1);
    assert!(shuffled(&t, P0));
    let revealed = t.g.turn_events.iter().any(|e| {
        matches!(e, Event::Custom { name, .. } if name.as_str() == mtg_engine::reveal::REVEALED)
    });
    assert!(revealed, "the card is revealed");
}

#[test]
fn maw_of_the_mire_destroys_a_land_and_gains_four() {
    cr!("701.8a", "119.3");
    // "Destroy target land. You gain 4 life."
    supported("Maw of the Mire");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let c = ready(&mut t, "Maw of the Mire", "Swamp", 5);
    t.cast(P0, c).target(land).go();
    let ch = target_choices(&t, P0).pop().unwrap();
    assert!(!ch.contains(&obj(bears)));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Forest"));
    assert_eq!((t.life(P0), t.life(P1)), (24, 20));
}

#[test]
fn goblin_surprise_pumps_your_team_or_makes_two_goblins() {
    cr!("700.2a", "611.2c", "111.1");
    // "Choose one — • Creatures you control get +2/+0 until end of turn. • Create two 1/1
    // red Goblin creature tokens."
    supported("Goblin Surprise");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let c = ready(&mut t, "Goblin Surprise", "Mountain", 3);
    t.cast(P0, c).modes(&[0]).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    assert_eq!(t.pt(theirs), (2, 2));
    assert!(tokens(&t, P0, "Goblin").is_empty(), "only the chosen mode");
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = ready(&mut t, "Goblin Surprise", "Mountain", 3);
    t.cast(P0, c).modes(&[1]).go();
    t.resolve_all();
    let g = tokens(&t, P0, "Goblin");
    assert_eq!(g.len(), 2);
    assert_eq!(t.pt(g[0]), (1, 1));
    assert_eq!(t.obj_now(g[0]).chars.colors, ColorSet::single(Color::Red));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn mirrodin_avenged_destroys_a_creature_dealt_damage_this_turn() {
    cr!("701.8a", "115.1", "121.1");
    // "Destroy target creature that was dealt damage this turn. / Draw a card."
    supported("Mirrodin Avenged");
    let mut t = TestGame::new(2);
    let hurt = t.battlefield(P1, "Hill Giant");
    let fresh = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 1);
    let s = t.hand(P0, "Shock");
    t.cast(P0, s).target(hurt).go();
    t.resolve_all();
    let c = ready(&mut t, "Mirrodin Avenged", "Swamp", 1);
    t.cast(P0, c).target(hurt).go();
    let ch = target_choices(&t, P0).pop().unwrap();
    assert!(ch.contains(&obj(hurt)));
    assert!(!ch.contains(&obj(fresh)), "not dealt damage this turn");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.on_battlefield(fresh));
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn seismic_strike_deals_damage_equal_to_your_mountains() {
    cr!("120.3e", "115.1");
    // "~ deals damage to target creature equal to the number of Mountains you control."
    supported("Seismic Strike");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P1, "Mountain", 3);
    let c = ready(&mut t, "Seismic Strike", "Mountain", 3);
    t.cast(P0, c).target(wurm).go();
    t.resolve_all();
    assert_eq!(
        t.obj_now(wurm).damage,
        3,
        "your three Mountains, not the opponent's"
    );
    assert!(t.on_battlefield(wurm));
}

#[test]
fn heat_ray_deals_x_damage_to_target_creature() {
    cr!("107.3a", "120.3e", "115.1");
    // "~ deals X damage to target creature."
    supported("Heat Ray");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = ready(&mut t, "Heat Ray", "Mountain", 4);
    t.cast(P0, c).x(3).target(wurm).go();
    let ch = target_choices(&t, P0).pop().unwrap();
    assert!(!ch.contains(&Entity::Player(P1)), "creatures only");
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).damage, 3);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn vandalize_destroys_an_artifact_a_land_or_both() {
    cr!("700.2a", "701.8a");
    // "Choose one or both — • Destroy target artifact. • Destroy target land."
    supported("Vandalize");
    for modes in [vec![0usize], vec![1], vec![0, 1]] {
        let mut t = TestGame::new(2);
        let ring = t.battlefield(P1, "Sol Ring");
        let land = t.battlefield(P1, "Forest");
        let c = ready(&mut t, "Vandalize", "Mountain", 5);
        let mut b = t.cast(P0, c).modes(&modes);
        if modes.contains(&0) {
            b = b.target(ring);
        }
        if modes.contains(&1) {
            b = b.target(land);
        }
        b.go();
        t.resolve_all();
        assert_eq!(
            t.in_graveyard(P1, "Sol Ring"),
            modes.contains(&0),
            "{modes:?}"
        );
        assert_eq!(
            t.in_graveyard(P1, "Forest"),
            modes.contains(&1),
            "{modes:?}"
        );
    }
}

#[test]
fn planar_incision_blinks_an_artifact_or_creature_with_a_counter() {
    cr!("400.7", "122.6", "115.1");
    // "Exile target artifact or creature, then return it to the battlefield under its
    // owner's control with a +1/+1 counter on it."
    supported("Planar Incision");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P1, "Forest");
    t.g.tap(bears);
    let c = ready(&mut t, "Planar Incision", "Island", 2);
    t.cast(P0, c).target(bears).go();
    let ch = target_choices(&t, P0).pop().unwrap();
    assert!(!ch.contains(&obj(land)));
    t.resolve_all();
    assert!(!t.is_live(bears), "a new object");
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(b.len(), 1);
    assert_eq!(t.obj_now(b[0]).controller, P1, "its owner's control");
    assert_eq!(t.counters(b[0], "+1/+1"), 1);
    assert!(!t.obj_now(b[0]).tapped);
    // An artifact.
    let ring = t.battlefield(P0, "Sol Ring");
    let c = ready(&mut t, "Planar Incision", "Island", 2);
    t.cast(P0, c).target(ring).go();
    t.resolve_all();
    let r = t.named_on_battlefield("Sol Ring");
    assert_eq!(r.len(), 1);
    assert_eq!(t.counters(r[0], "+1/+1"), 1);
}

#[test]
fn marsh_gas_gives_all_creatures_minus_two_power() {
    cr!("611.2c", "613.4c");
    // "All creatures get -2/-0 until end of turn."
    supported("Marsh Gas");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let c = ready(&mut t, "Marsh Gas", "Swamp", 1);
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.pt(mine), (1, 3));
    assert_eq!(t.pt(theirs), (0, 2));
    let late = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(
        t.pt(late),
        (2, 2),
        "only the creatures there as it resolved"
    );
    next_turn(&mut t);
    assert_eq!(t.pt(mine), (3, 3));
    assert_eq!(t.pt(theirs), (2, 2));
}

#[test]
fn artificers_epiphany_discards_only_without_artifacts() {
    cr!("121.1", "701.9a", "608.2c");
    // "Draw two cards. If you control no artifacts, discard a card."
    supported("Artificer's Epiphany");
    for artifact in [true, false] {
        let mut t = TestGame::new(2);
        if artifact {
            t.battlefield(P0, "Sol Ring");
        } else {
            t.battlefield(P1, "Sol Ring");
        }
        let c = ready(&mut t, "Artificer's Epiphany", "Island", 3);
        t.cast(P0, c).go();
        t.resolve_all();
        assert_eq!(t.library_size(P0), 28);
        assert_eq!(
            t.hand_size(P0),
            if artifact { 2 } else { 1 },
            "artifact: {artifact}"
        );
        assert_eq!(t.graveyard_size(P0), if artifact { 1 } else { 2 });
    }
}

#[test]
fn dark_offering_destroys_a_nonblack_creature_and_gains_three() {
    cr!("701.8a", "115.1", "119.3");
    // "Destroy target nonblack creature. You gain 3 life."
    supported("Dark Offering");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let black = t.battlefield(P1, "Vampire Nighthawk");
    let c = ready(&mut t, "Dark Offering", "Swamp", 6);
    t.cast(P0, c).target(bears).go();
    let ch = target_choices(&t, P0).pop().unwrap();
    assert!(
        ch.contains(&obj(bears)) && !ch.contains(&obj(black)),
        "nonblack"
    );
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!((t.life(P0), t.life(P1)), (23, 20));
}

#[test]
fn hazardous_blast_pings_opposing_creatures_and_stops_them_blocking() {
    cr!("120.3e", "509.1b", "611.2c");
    // "~ deals 1 damage to each creature your opponents control. Creatures your
    // opponents control can't block this turn."
    supported("Hazardous Blast");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Wood");
    t.battlefield(P1, "Llanowar Elves");
    let c = ready(&mut t, "Hazardous Blast", "Mountain", 4);
    t.cast(P0, c).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    assert_eq!(t.obj_now(wall).damage, 1);
    assert_eq!(t.obj_now(mine).damage, 0);
    let late = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(mine, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    let opts = block_options(&t.g, &[P1]);
    assert!(!block_declaration_legal(&t.g, &opts, &[(wall, mine)]));
    assert!(
        !block_declaration_legal(&t.g, &opts, &[(late, mine)]),
        "a rule for creatures your opponents control, even ones that arrive later"
    );
}

#[test]
fn thassas_bounty_draws_three_and_mills_target_player() {
    cr!("121.1", "701.17a", "115.1");
    // "Draw three cards. Target player mills three cards."
    supported("Thassa's Bounty");
    for target in [P0, P1] {
        let mut t = TestGame::new(2);
        let c = ready(&mut t, "Thassa's Bounty", "Island", 6);
        t.cast(P0, c).target(Entity::Player(target)).go();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), 3);
        assert_eq!(
            t.graveyard_size(target),
            3 + (target == P0) as usize,
            "plus the spell"
        );
        let other = if target == P0 { P1 } else { P0 };
        assert_eq!(t.graveyard_size(other), (other == P0) as usize);
        assert_eq!(t.library_size(P0), if target == P0 { 24 } else { 27 });
    }
}
