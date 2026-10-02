//! Control changes (CR 108.4, 613.1b, 701.12): "[player] gains control of [objects]
//! [duration]", "each player gains control of all [permanents] they own", exchanges, "you
//! and target opponent each gain control of all creatures the other controls" (patterns in
//! `src/oracle/patterns/control_change_grammar.rs` and `attach_control_grammar.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn controller(t: &TestGame, id: ObjectId) -> PlayerId {
    t.obj_now(id).controller
}

/// Gives `p` control of `id` indefinitely, as a resolved "gain control of" effect would.
fn give_control(t: &mut TestGame, id: ObjectId, p: PlayerId) {
    use mtg_engine::ability::{Duration, Effect, PlayerRef, Sel};
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    ctx.targets = vec![vec![Entity::Object(t.g.current(id))]];
    t.g.exec(
        &Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

#[test]
fn control_cards_compile() {
    assert_compiles(&[
        "Jinxed Ring",
        "Brooding Saurian",
        "Shield Broker",
        "Coveted Falcon",
        "Twist Allegiance",
        "Wellspring",
        "Aura Graft",
        "Fumble",
        "Unexpected Request",
        "Modify Memory",
        "Besmirch",
        "Kitsune, Dragon's Daughter",
        "Murderous Spoils",
        "Yes Man, Personal Securitron",
        "Yasova Dragonclaw",
        "Stiltzkin, Moogle Merchant",
        "Grab the Reins",
    ]);
}

#[test]
fn jinxed_ring_target_opponent_gains_control_of_it() {
    cr!("108.4", "611.2a");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Jinxed Ring");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, ring, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(controller(&t, ring), P1);
    // The effect has no duration: it lasts (CR 611.2a).
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(controller(&t, ring), P1);
}

#[test]
fn besmirch_control_lasts_until_end_of_turn() {
    cr!("611.2a", "613.1b", "514.2");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.tap(giant);
    let spell = t.hand(P0, "Besmirch");
    t.lands(P0, "Mountain", 3);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, spell).target(Entity::Object(giant)).go();
    t.resolve_all();
    assert_eq!(controller(&t, giant), P0);
    assert!(!t.obj_now(giant).tapped);
    assert!(t
        .obj_now(giant)
        .has_keyword(mtg_engine::keywords::KeywordKind::Haste));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(controller(&t, giant), P1);
}

#[test]
fn shield_broker_control_lasts_while_the_shield_counter_remains() {
    cr!("611.2b", "122.1c");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.enter(P0, "Shield Broker");
    t.resolve_all();
    assert_eq!(t.counters(giant, "shield"), 1);
    assert_eq!(controller(&t, giant), P0);
    // Once the shield counter is gone (removed instead of the creature being dealt
    // damage or destroyed), P1 controls it again.
    t.g.remove_counters(Entity::Object(giant), "shield", 1);
    t.g.recompute();
    t.settle();
    assert_eq!(t.counters(giant, "shield"), 0);
    assert_eq!(controller(&t, giant), P1);
}

#[test]
fn yes_man_when_they_do_draws_only_if_the_opponent_gained_control() {
    cr!("603.12", "108.4");
    ruling!(
        "Yes Man, Personal Securitron",
        "You won't draw two cards, because that effect is part of a reflexive triggered ability that triggers only if the target opponent gains control of Yes Man."
    );
    let mut t = TestGame::new(2);
    let yes_man = t.battlefield(P0, "Yes Man, Personal Securitron");
    t.set_step(P0, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    t.activate(P0, yes_man, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(controller(&t, yes_man), P1);
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.counters(yes_man, "quest"), 1);

    // Gone before the ability resolves: no control change, no cards.
    let mut t = TestGame::new(2);
    let yes_man = t.battlefield(P0, "Yes Man, Personal Securitron");
    t.set_step(P0, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    t.activate(P0, yes_man, 0, &[Entity::Player(P1)]).unwrap();
    t.g.destroy(yes_man, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn kitsune_exchanges_control_of_two_creatures_controlled_by_different_players() {
    cr!("701.12a", "701.12b");
    ruling!(
        "Kitsune, Dragon's Daughter",
        "Gaining control of a creature doesn't cause you to gain control of any Auras or Equipment attached to it."
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let blade = t.battlefield(P1, "Bonesplitter");
    assert!(t.g.attach(blade, Entity::Object(theirs)));
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(mine), Entity::Object(theirs)]);
    t.enter(P0, "Kitsune, Dragon's Daughter");
    t.resolve_all();
    assert_eq!(controller(&t, mine), P1);
    assert_eq!(controller(&t, theirs), P0);
    assert_eq!(controller(&t, blade), P1);
}

#[test]
fn murderous_spoils_gains_control_of_the_equipment_that_was_attached() {
    cr!("608.2h", "611.2a");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let blade = t.battlefield(P1, "Bonesplitter");
    let other = t.battlefield(P1, "Short Sword");
    assert!(t.g.attach(blade, Entity::Object(giant)));
    let spell = t.hand(P0, "Murderous Spoils");
    t.lands(P0, "Swamp", 6);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, spell).target(Entity::Object(giant)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(controller(&t, blade), P0);
    // Not an Equipment that wasn't attached to it.
    assert_eq!(controller(&t, other), P1);
}

#[test]
fn brooding_saurian_returns_permanents_to_their_owners() {
    cr!("108.4", "111.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Brooding Saurian");
    let stolen = t.battlefield(P1, "Hill Giant");
    let lent = t.battlefield(P0, "Grizzly Bears");
    give_control(&mut t, stolen, P0);
    give_control(&mut t, lent, P1);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(controller(&t, stolen), P1);
    assert_eq!(controller(&t, lent), P0);
}

#[test]
fn coveted_falcon_draws_a_card_for_each_permanent_the_opponent_gained() {
    cr!("108.4", "611.2c");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let hand = t.hand_size(P0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    let falcon = t.battlefield(P0, "Coveted Falcon");
    // "When this creature is turned face up": turn it face down, then face up.
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, falcon));
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    t.g.exec(
        &mtg_engine::ability::Effect::TurnFaceUp {
            what: mtg_engine::ability::Sel::All(mtg_engine::ability::Filter::FaceDown),
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(controller(&t, a), P1);
    assert_eq!(controller(&t, b), P1);
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn twist_allegiance_swaps_all_creatures_until_end_of_turn() {
    cr!("611.2c", "613.1b");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.g.tap(theirs);
    let spell = t.hand(P0, "Twist Allegiance");
    t.lands(P0, "Mountain", 7);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(controller(&t, mine), P1);
    assert_eq!(controller(&t, theirs), P0);
    // "Untap those creatures. Those creatures gain haste": both sets.
    assert!(!t.obj_now(theirs).tapped);
    for c in [mine, theirs] {
        assert!(t
            .obj_now(c)
            .has_keyword(mtg_engine::keywords::KeywordKind::Haste));
    }
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(controller(&t, mine), P0);
    assert_eq!(controller(&t, theirs), P1);
}

#[test]
fn modify_memory_draws_if_you_control_neither_creature() {
    cr!("701.12b", "608.2c");
    let mut t = TestGame::new(3);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P2, "Hill Giant");
    let spell = t.hand(P0, "Modify Memory");
    t.lands(P0, "Island", 5);
    t.set_step(P0, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    t.cast(P0, spell)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve_all();
    assert_eq!(controller(&t, a), P2);
    assert_eq!(controller(&t, b), P1);
    assert_eq!(t.hand_size(P0), hand - 1 + 3);
}

#[test]
fn modify_memory_doesnt_draw_if_you_control_one_of_them() {
    cr!("701.12b");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let spell = t.hand(P0, "Modify Memory");
    t.lands(P0, "Island", 5);
    t.set_step(P0, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    t.cast(P0, spell)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve_all();
    assert_eq!(controller(&t, a), P1);
    assert_eq!(controller(&t, b), P0);
    assert_eq!(t.hand_size(P0), hand - 1);
}

#[test]
fn wellspring_gains_control_of_the_enchanted_land_until_end_of_turn() {
    cr!("611.2a", "303.4a");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    let aura = t.hand(P0, "Wellspring");
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Forest", 1);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, aura).target(Entity::Object(land)).go();
    t.resolve_all();
    assert_eq!(controller(&t, land), P0);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(controller(&t, land), P1);
}

#[test]
fn aura_graft_moves_the_aura_to_another_permanent_it_can_enchant() {
    cr!("701.3a", "303.4");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let knight = t.battlefield(P0, "Black Knight");
    let giant = t.battlefield(P0, "Hill Giant");
    let aura = t.battlefield(P1, "Holy Strength");
    assert!(t.g.attach(aura, Entity::Object(bears)));
    let spell = t.hand(P0, "Aura Graft");
    t.lands(P0, "Island", 2);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, spell).target(Entity::Object(aura)).go();
    t.resolve_all();
    assert_eq!(controller(&t, aura), P0);
    // Not the Black Knight (protection from white), nor the creature it was on.
    assert_eq!(t.obj_now(aura).attached_to, Some(Entity::Object(giant)));
    let _ = knight;
}

#[test]
fn fumble_gains_control_of_the_attachments_and_moves_them() {
    cr!("608.2h", "701.3a");
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Hill Giant");
    let blade = t.battlefield(P1, "Bonesplitter");
    assert!(t.g.attach(blade, Entity::Object(target)));
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Fumble");
    t.lands(P0, "Island", 2);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, spell).target(Entity::Object(target)).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Hill Giant"));
    assert_eq!(controller(&t, blade), P0);
    assert_eq!(t.obj_now(blade).attached_to, Some(Entity::Object(bears)));
}

#[test]
fn unexpected_request_unattaches_the_equipment_at_the_next_end_step() {
    cr!("603.7a", "701.3d");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let blade = t.battlefield(P0, "Bonesplitter");
    let spell = t.hand(P0, "Unexpected Request");
    t.lands(P0, "Mountain", 3);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(blade)]);
    t.cast(P0, spell).target(Entity::Object(giant)).go();
    t.resolve_all();
    assert_eq!(controller(&t, giant), P0);
    assert_eq!(t.obj_now(blade).attached_to, Some(Entity::Object(giant)));
    assert_eq!(t.pt(giant), (5, 3));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.obj_now(blade).attached_to, None);
}

#[test]
fn axis_of_mortality_two_target_players_exchange_life_totals() {
    cr!("701.12a", "119.7");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Axis of Mortality");
    t.g.players[P1.idx()].life = 5;
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P0), Entity::Player(P1)]);
    t.set_step(P0, Step::Untap);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 5);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn trove_warden_returns_cards_under_their_owners_control() {
    cr!("110.2a", "607.2a");
    let mut t = TestGame::new(2);
    let warden = t.battlefield(P0, "Trove Warden");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let forest = t.hand(P0, "Forest");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.play_land(P0, forest).unwrap();
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bears)), mtg_engine::object::Zone::Exile);
    // P1 controls Trove Warden when it dies: the card returns under its owner's control.
    give_control(&mut t, warden, P1);
    t.g.destroy(t.g.current(warden), None);
    t.g.flush_events();
    t.resolve_all();
    let bears = t.g.current(bears);
    assert!(t.on_battlefield(bears));
    assert_eq!(controller(&t, bears), P0);
}

#[test]
fn akiri_unattaches_an_equipment_and_the_creature_becomes_tapped_and_indestructible() {
    cr!("701.3d", "702.12b");
    ruling!(
        "Akiri, Fearless Voyager",
        "The Equipment that's unattached remains on the battlefield."
    );
    let mut t = TestGame::new(2);
    let akiri = t.battlefield(P0, "Akiri, Fearless Voyager");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blade = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(blade, Entity::Object(bears)));
    t.lands(P0, "Plains", 1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(blade)]);
    t.activate(P0, akiri, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(blade));
    assert_eq!(t.obj_now(blade).attached_to, None);
    assert!(t.obj_now(bears).tapped);
    assert!(t
        .obj_now(bears)
        .has_keyword(mtg_engine::keywords::KeywordKind::Indestructible));
}
