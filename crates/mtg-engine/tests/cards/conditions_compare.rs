//! Conditions that compare numbers, players and groups (`oracle/patterns/
//! conditions_compare.rs`): parity, two values compared, a target introduced in a leading
//! condition, counts of same-named and differently named objects, player comparisons
//! against each other player, extremes with ties, existence and game state.

use mtg_engine::decision::Action;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// Whether `p` may activate the `index`th activated ability of `src` now.
fn can(t: &mut TestGame, p: PlayerId, src: ObjectId, index: usize) -> bool {
    t.g.recompute();
    let uid = t
        .g
        .obj(src)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .nth(index)
        .map(|a| a.uid)
        .expect("no such ability");
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p).iter().any(
        |a| matches!(a, Action::Activate { source, ability, .. } if *source == src && *ability == uid),
    )
}

fn can_cast(t: &mut TestGame, p: PlayerId, c: ObjectId) -> bool {
    t.g.recompute();
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Cast { card, .. } if *card == c))
}

#[test]
fn parity_of_a_count_chaos_lord() {
    cr!("107.1", "603.3");
    compiles("Chaos Lord");
    // One permanent (Chaos Lord): odd, so nothing happens.
    let mut t = TestGame::new(2);
    let lord = t.battlefield(P0, "Chaos Lord");
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.obj_now(lord).controller, P0);
    // Two permanents: even, the opponent gains control of it.
    let mut t = TestGame::new(2);
    let lord = t.battlefield(P0, "Chaos Lord");
    t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.obj_now(lord).controller, P1);
}

#[test]
fn parity_of_a_revealed_cards_mana_value_yennett() {
    ruling!(
        "Yennett, Cryptic Sovereign",
        "the card you draw will be the card you revealed"
    );
    compiles("Yennett, Cryptic Sovereign");
    // Odd (Divination, 3): cast it for free; it draws two cards.
    let mut t = TestGame::new(2);
    let y = t.battlefield(P0, "Yennett, Cryptic Sovereign");
    t.library_top(P0, "Divination");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(y, Entity::Player(P1))], &[]);
    assert!(t.in_graveyard(P0, "Divination"));
    assert_eq!(t.hand_size(P0), hand + 2);
    // Even (Grizzly Bears, 2): it can't be cast; the revealed card is drawn.
    let mut t = TestGame::new(2);
    let y = t.battlefield(P0, "Yennett, Cryptic Sovereign");
    t.library_top(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(y, Entity::Player(P1))], &[]);
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn two_values_compared_ed_e() {
    cr!("603.4");
    compiles("ED-E, Lonesome Eyebot");
    let mut t = TestGame::new(2);
    let ede = t.battlefield(P0, "ED-E, Lonesome Eyebot");
    t.g.add_counters(Entity::Object(ede), "quest", 1, None);
    // One attacker, one quest counter: not greater, no counter.
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(ede, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(ede, "quest"), 1);
    // Two attackers: greater.
    let mut t = TestGame::new(2);
    let ede = t.battlefield(P0, "ED-E, Lonesome Eyebot");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(ede), "quest", 1, None);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[(ede, Entity::Player(P1)), (bears, Entity::Player(P1))],
        &[],
    );
    assert_eq!(t.counters(ede, "quest"), 2);
}

#[test]
fn target_in_a_leading_condition_is_checked_on_resolution() {
    cr!("601.2c", "608.2b");
    ruling!(
        "Hidetsugu's Second Rite",
        "checked only as the spell resolves"
    );
    compiles("Hidetsugu's Second Rite");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let rite = t.hand(P0, "Hidetsugu's Second Rite");
    // Any player can be targeted, whatever their life total.
    t.cast(P0, rite).target(P1).go();
    t.g.players[1].life = 10;
    t.resolve();
    assert_eq!(t.life(P1), 0);
    // Not at exactly 10 life as it resolves: nothing happens.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let rite = t.hand(P0, "Hidetsugu's Second Rite");
    t.g.players[1].life = 10;
    t.cast(P0, rite).target(P1).go();
    t.g.players[1].life = 11;
    t.resolve();
    assert_eq!(t.life(P1), 11);
}

#[test]
fn difference_between_life_totals_psychic_transfer() {
    cr!("119.1");
    compiles("Psychic Transfer");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    t.g.players[0].life = 12;
    t.g.players[1].life = 17;
    let pt = t.hand(P0, "Psychic Transfer");
    t.cast(P0, pt).target(P1).go();
    t.resolve();
    assert_eq!((t.life(P0), t.life(P1)), (17, 12));
    // A difference of 6: nothing happens.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    t.g.players[0].life = 18;
    t.g.players[1].life = 12;
    let pt = t.hand(P0, "Psychic Transfer");
    t.cast(P0, pt).target(P1).go();
    t.resolve();
    assert_eq!((t.life(P0), t.life(P1)), (18, 12));
}

#[test]
fn three_or_more_lands_with_the_same_name() {
    cr!("201.2", "602.5b");
    compiles("Endless Atlas");
    compiles("Sceptre of Eternal Glory");
    let mut t = TestGame::new(2);
    let atlas = t.battlefield(P0, "Endless Atlas");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Plains", 2);
    assert!(!can(&mut t, P0, atlas, 0));
    t.lands(P0, "Island", 1);
    assert!(can(&mut t, P0, atlas, 0));
}

#[test]
fn eight_artifacts_with_the_same_name_as_one_another() {
    cr!("201.2", "104.2b");
    ruling!(
        "Mechanized Production",
        "All eight of the permanents sharing a name must be artifacts"
    );
    compiles("Mechanized Production");
    let setup = |n: usize| {
        let mut t = TestGame::new(2);
        let thopters: Vec<ObjectId> = (0..n).map(|_| t.battlefield(P0, "Ornithopter")).collect();
        // A nonartifact permanent with another name doesn't count.
        t.battlefield(P0, "Grizzly Bears");
        let mp = t.battlefield(P0, "Mechanized Production");
        t.g.attach(mp, Entity::Object(thopters[0]));
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        t
    };
    // Six plus the copy: seven, not enough.
    let t = setup(6);
    assert!(!t.has_lost(P1));
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 7);
    // Seven plus the copy: eight.
    let t = setup(7);
    assert!(t.has_lost(P1));
}

#[test]
fn nine_lands_with_different_names_among_basic_sphere_and_locus_lands() {
    cr!("201.2", "205.4c");
    compiles("Monument to Perfection");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Monument to Perfection");
    for name in [
        "Plains",
        "Island",
        "Swamp",
        "Mountain",
        "Forest",
        "Wastes",
        "Snow-Covered Plains",
        "Snow-Covered Island",
    ] {
        t.battlefield(P0, name);
    }
    // A land that's neither basic, a Sphere, nor a Locus doesn't count; nor does a
    // second Island.
    t.battlefield(P0, "Command Tower");
    t.battlefield(P0, "Island");
    assert!(!can(&mut t, P0, m, 1));
    t.battlefield(P0, "Snow-Covered Swamp");
    assert!(can(&mut t, P0, m, 1));
}

#[test]
fn fewer_creatures_than_each_opponent() {
    cr!("601.3");
    ruling!(
        "Even the Odds",
        "doesn\u{2019}t matter how many creatures you control as Even the Odds resolves"
    );
    compiles("Even the Odds");
    let mut t = TestGame::new(3);
    t.lands(P0, "Plains", 4);
    let spell = t.hand(P0, "Even the Odds");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P2, "Grizzly Bears");
    assert!(can_cast(&mut t, P0, spell));
    // One opponent controls as many as you: not fewer than each opponent.
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    assert!(!can_cast(&mut t, P0, spell));
}

#[test]
fn more_creatures_than_each_other_player_ties_count_against_you() {
    ruling!(
        "Advice from the Fae",
        "If any single player controls at least as many creatures as you"
    );
    compiles("Advice from the Fae");
    compiles("Peer Pressure");
    for (theirs, cards) in [(1, 2), (2, 1)] {
        let mut t = TestGame::new(3);
        t.lands(P0, "Island", 6);
        for _ in 0..2 {
            t.battlefield(P0, "Grizzly Bears");
        }
        t.battlefield(P1, "Grizzly Bears");
        for _ in 0..theirs {
            t.battlefield(P2, "Grizzly Bears");
        }
        let advice = t.hand(P0, "Advice from the Fae");
        let hand = t.hand_size(P0);
        t.cast(P0, advice).go();
        t.resolve();
        assert_eq!(t.hand_size(P0), hand - 1 + cards, "{theirs} creatures");
    }
}

#[test]
fn greatest_toughness_or_tied_abzan_beastmaster() {
    ruling!(
        "Abzan Beastmaster",
        "You'll draw a maximum of one card, even if you control multiple creatures tied"
    );
    compiles("Abzan Beastmaster");
    for (their, draws) in [("Grizzly Bears", 1), ("Hill Giant", 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Abzan Beastmaster");
        t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P1, their);
        t.advance_to(P0, Step::Upkeep);
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + draws, "{their}");
    }
}

#[test]
fn a_creature_has_a_counter_tenacious_hunter() {
    compiles("Tenacious Hunter");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Tenacious Hunter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.recompute();
    assert!(!t.obj_now(h).has_keyword(KeywordKind::Vigilance));
    t.g.add_counters(Entity::Object(bears), "-1/-1", 1, None);
    t.g.recompute();
    assert!(t.obj_now(h).has_keyword(KeywordKind::Vigilance));
    assert!(t.obj_now(h).has_keyword(KeywordKind::Deathtouch));
}

#[test]
fn players_who_lost_the_game_hot_pursuit() {
    cr!("800.4");
    // (Its first ability, "that creature is also goaded" for as long as Hot Pursuit
    // remains on the battlefield, belongs to the goad durations.)
    assert!(card("Hot Pursuit")
        .unsupported_text()
        .iter()
        .all(|u| !u.contains("lost the game")));
    let run = |lost: usize| {
        let mut t = TestGame::new(4);
        t.battlefield(P0, "Hot Pursuit");
        let bears = t.battlefield(P1, "Grizzly Bears");
        mtg_engine::kwa::suspect_detain::suspect(&mut t.g, bears);
        for i in 0..lost {
            t.g.players[2 + i].has_lost = true;
            t.g.players[2 + i].left_game = true;
        }
        t.set_step(P0, Step::PrecombatMain);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.resolve_all();
        t.obj_now(bears).controller
    };
    assert_eq!(run(1), P1);
    assert_eq!(run(2), P0);
}
