//! Rulings batch P044 — delayed triggered abilities created by a resolving spell or
//! ability (CR 603.7): when they exist, what they refer to, and who controls what they
//! do.

use crate::r_p044_common::*;
use crate::r_s01_common::*;
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn massacre_girl_chain_reaction() {
    cr!("603.7a", "704.3", "704.5f", "611.2c");
    ruling!(
        "Massacre Girl",
        "A creature with 0 toughness doesn't die immediately. Rather, the creature dies the next time a player would receive priority."
    );
    ruling!(
        "Massacre Girl",
        "If more than one creature dies at once, Massacre Girl's delayed triggered ability triggers that many times."
    );
    supported("Massacre Girl");
    // Two 1/1s die from the enters ability (after the delayed ability exists): two
    // triggers take the Giant from 2/2 to 0/0; its death shrinks the Wurm once more.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Llanowar Elves");
    t.battlefield(P1, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let girl = t.enter(P0, "Massacre Girl");
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert!(!t.on_battlefield(wurm), "6/4: -1/-1 four times");
    assert_eq!(t.pt(girl), (4, 4));
}

#[test]
fn massacre_girl_delayed_ability_starts_on_resolution() {
    cr!("603.7a", "611.2c");
    ruling!(
        "Massacre Girl",
        "Creatures that die while Massacre Girl's triggered ability is still on the stack won't cause its delayed triggered ability to trigger since that delayed triggered ability hasn't been created yet."
    );
    ruling!(
        "Massacre Girl",
        "Massacre Girl's enters-the-battlefield triggered ability and its delayed triggered ability each affect only creatures on the battlefield at the time that those abilities resolve."
    );
    ruling!(
        "Massacre Girl",
        "Once Massacre Girl's enters-the-battlefield ability has triggered, it doesn't matter whether Massacre Girl remains on the battlefield."
    );
    // A creature dies with the enters ability on the stack: no delayed trigger.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.enter(P0, "Massacre Girl");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.pt(giant), (2, 2));
    // A creature entering later isn't affected until another creature dies.
    let late = t.battlefield(P1, "Hill Giant");
    t.settle();
    assert_eq!(t.pt(late), (3, 3));
    let elves = t.battlefield(P1, "Llanowar Elves");
    kill(&mut t, elves);
    t.resolve_all();
    assert_eq!(t.pt(late), (2, 2));
    assert_eq!(t.pt(giant), (1, 1));
    // Massacre Girl leaves with its ability on the stack: the delayed ability is still
    // created, and the Elves' death shrinks the Bears to 0/0.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let girl = t.enter(P0, "Massacre Girl");
    t.settle();
    t.g.move_object(girl, Zone::Hand(P0), MoveCause::Effect, None);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn death_frenzy_counts_every_death_this_turn() {
    cr!("603.7a", "611.2c", "704.5f");
    ruling!(
        "Death Frenzy",
        "Only creatures on the battlefield when Death Frenzy resolves will get -2/-2. However, if a creature enters the battlefield and then dies later that turn, you’ll gain 1 life for it."
    );
    ruling!(
        "Death Frenzy",
        "You gain 1 life for each creature that Death Frenzy causes to have a toughness of 0 or less. The delayed triggered ability is created before state-based actions cause any creatures with 0 or less toughness to die."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Llanowar Elves");
    t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_spell(&mut t, P0, "Death Frenzy", &[]);
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.pt(giant), (1, 1));
    let late = t.battlefield(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(t.pt(late), (2, 2));
    kill(&mut t, late);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
}

#[test]
fn desperate_measures_draws_when_it_dies_under_your_control() {
    cr!("603.7a", "603.7c", "603.10a");
    ruling!(
        "Desperate Measures",
        "As long as you control it, you will get to draw two cards when the target creature goes to the graveyard, regardless of whether it died due to having 0 toughness or something else."
    );
    ruling!(
        "Desperate Measures",
        "If the creature is no longer a creature at the time it goes to the graveyard, you will still draw two cards if you controlled it at that time."
    );
    supported("Desperate Measures");
    // 0 toughness.
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let hand = t.hand_size(P0);
    cast_spell(&mut t, P0, "Desperate Measures", &[Entity::Object(elves)]);
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    assert_eq!(t.hand_size(P0), hand + 2);
    // Something else.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    cast_spell(&mut t, P0, "Desperate Measures", &[Entity::Object(bears)]);
    assert_eq!(t.hand_size(P0), hand);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // An opponent's creature: nothing.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let hand = t.hand_size(P0);
    cast_spell(&mut t, P0, "Desperate Measures", &[Entity::Object(bears)]);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // No longer a creature: P0's Mind Stone (a 2/2 under March of the Machines) stops
    // being a creature when March leaves, then goes to the graveyard.
    let mut t = TestGame::new(2);
    let march = t.battlefield(P0, "March of the Machines");
    let ring = t.battlefield(P0, "Mind Stone");
    let hand = t.hand_size(P0);
    cast_spell(&mut t, P0, "Desperate Measures", &[Entity::Object(ring)]);
    assert_eq!(t.pt(ring), (3, 1));
    kill(&mut t, march);
    t.resolve_all();
    kill(&mut t, ring);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mind Stone"));
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn showstopper_affects_only_creatures_you_control_on_resolution() {
    cr!("611.2c", "603.3a", "603.10a");
    ruling!(
        "Showstopper",
        "Creatures that come under your control after Showstopper resolves won’t have the ability."
    );
    ruling!(
        "Showstopper",
        "If another player gains control of a creature you control after Showstopper resolves, and then later that turn that creature dies, the creature’s controller when it died controls the triggered ability."
    );
    supported("Showstopper");
    // A creature P0 gains control of afterwards doesn't have it.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let target = t.battlefield(P1, "Craw Wurm");
    cast_spell(&mut t, P0, "Showstopper", &[]);
    cast_spell(&mut t, P0, "Act of Treason", &[Entity::Object(theirs)]);
    assert_eq!(t.obj_now(theirs).controller, P0);
    kill(&mut t, theirs);
    assert_eq!(t.stack_len(), 0);
    // P0's own creature has it.
    t.answer_targets(P0, &[Entity::Object(target)]);
    kill(&mut t, mine);
    t.resolve_all();
    assert_eq!(t.obj_now(target).damage, 2);
    // P1 gains control of P0's Bears: when they die, P1 controls the trigger and targets
    // a creature P0 controls.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    cast_spell(&mut t, P0, "Showstopper", &[]);
    cast_spell(&mut t, P1, "Ray of Command", &[Entity::Object(bears)]);
    assert_eq!(t.obj_now(bears).controller, P1);
    t.answer_targets(P1, &[Entity::Object(giant)]);
    kill(&mut t, bears);
    // The granted ability, and Ray of Command's "When you lose control of the creature,
    // tap it" (leaving the battlefield is losing control of it): both P1's.
    assert_eq!(t.stack_len(), 2);
    for ab in t.g.stack.clone() {
        assert_eq!(t.g.obj(ab).controller, P1);
    }
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 2);
}

#[test]
fn searing_blood_hits_the_controller_when_it_dies() {
    cr!("603.7c", "603.10a");
    ruling!(
        "Searing Blood",
        "If the target creature dies that turn, Searing Blood will deal 3 damage to whoever controls the creature when it dies, who isn't necessarily the player who controlled it when Searing Blood resolved."
    );
    supported("Searing Blood");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_spell(&mut t, P0, "Searing Blood", &[Entity::Object(giant)]);
    assert!(t.on_battlefield(giant));
    cast_spell(&mut t, P0, "Act of Treason", &[Entity::Object(giant)]);
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn otherworldly_outburst_needs_the_creature_to_survive_and_then_die() {
    cr!("608.2b", "603.7a", "603.7c");
    ruling!(
        "Otherworldly Outburst",
        "If the targeted creature dies before Otherworldly Outburst resolves, the spell doesn’t resolve. You won’t get an Eldrazi Horror token."
    );
    ruling!(
        "Otherworldly Outburst",
        "If the targeted creature doesn’t die this turn, the delayed triggered ability simply won’t trigger."
    );
    // Dies in response.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_only(&mut t, P0, "Otherworldly Outburst", &[Entity::Object(bears)]);
    kill(&mut t, bears);
    t.resolve_all();
    assert!(with_subtype(&t, P0, "Horror").is_empty());
    // Doesn't die this turn; dying next turn does nothing.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_spell(&mut t, P0, "Otherworldly Outburst", &[Entity::Object(bears)]);
    t.advance_to(P1, Step::Upkeep);
    kill(&mut t, bears);
    t.resolve_all();
    assert!(with_subtype(&t, P0, "Horror").is_empty());
    // It dies this turn: the token.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_spell(&mut t, P0, "Otherworldly Outburst", &[Entity::Object(bears)]);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Horror").len(), 1);
}

#[test]
fn infernal_scarring_draw_goes_to_the_creatures_controller() {
    cr!("603.3a", "603.10a");
    ruling!(
        "Infernal Scarring",
        "The player who controlled the creature when it died is the one who will draw a card."
    );
    supported("Infernal Scarring");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_spell(&mut t, P0, "Infernal Scarring", &[Entity::Object(bears)]);
    assert_eq!(t.pt(bears), (4, 2));
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h0);
    assert_eq!(t.hand_size(P1), h1 + 1);
}

#[test]
fn kelsien_delayed_abilities() {
    cr!("603.7a", "608.2b", "122.1");
    ruling!(
        "Kelsien, the Plague",
        "Each time Kelsien's last ability resolves, it creates a separate delayed triggered ability."
    );
    ruling!(
        "Kelsien, the Plague",
        "If Kelsien leaves the battlefield, you keep any experience counters you've received."
    );
    ruling!(
        "Kelsien, the Plague",
        "If the target of Kelsien's last ability dies before the ability resolves, you won't get an experience counter."
    );
    supported("Kelsien, the Plague");
    // Two activations on the Bears: both delayed abilities trigger when they die.
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kelsien, the Plague");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, k, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    t.g.untap(k);
    t.activate(P0, k, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(player_counters(&t, P0, "experience"), 2);
    assert_eq!(t.pt(k), (4, 4));
    // Kelsien leaves: the counters stay.
    kill(&mut t, k);
    t.resolve_all();
    assert_eq!(player_counters(&t, P0, "experience"), 2);
    // The target dies in response: no counter.
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kelsien, the Plague");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, k, 0, &[Entity::Object(bears)]).unwrap();
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(player_counters(&t, P0, "experience"), 0);
}

/// Resolves until the next turn's upkeep triggers have resolved (P1's turn).
fn to_next_upkeep(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
}

#[test]
fn delayed_cantrips_draw_at_the_next_upkeep() {
    cr!("603.7a", "608.2b");
    ruling!(
        "Balduvian Rage",
        "If the target creature is an illegal target by the time Balduvian Rage tries to resolve, the spell won't resolve. You won't draw a card next turn."
    );
    ruling!(
        "Prophecy",
        "The shuffling and drawing of a card next upkeep happens even if the card is not a land."
    );
    ruling!(
        "Portent",
        "This does not cause a player to lose if they have less than 3 cards in their library. It allows you to look at and reorder or shuffle whatever remaining cards there might be."
    );
    // Balduvian Rage: the attacker dies in response.
    supported("Balduvian Rage");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    give_mana_for(&mut t, P0, "Balduvian Rage");
    t.lands(P0, "Wastes", 1);
    let rage = t.hand(P0, "Balduvian Rage");
    t.g.turn.priority = Some(P0);
    t.cast(P0, rage).target(bears).x(1).go();
    kill(&mut t, bears);
    t.resolve_all();
    let hand = t.hand_size(P0);
    to_next_upkeep(&mut t);
    assert_eq!(t.hand_size(P0), hand);
    // Prophecy: the top card isn't a land; still the draw.
    let mut t = TestGame::new(2);
    cast_spell(&mut t, P0, "Prophecy", &[Entity::Player(P1)]);
    assert_eq!(t.life(P0), 20);
    let hand = t.hand_size(P0);
    to_next_upkeep(&mut t);
    assert_eq!(t.hand_size(P0), hand + 1);
    // Portent on a two-card library: no loss; the draw still happens.
    let mut t = TestGame::new(2);
    let lib: Vec<ObjectId> = t.g.player(P1).library.iter().copied().collect();
    for id in lib.into_iter().skip(2) {
        t.g.move_object(id, Zone::Exile, MoveCause::Effect, None);
    }
    assert_eq!(t.library_size(P1), 2);
    cast_spell(&mut t, P0, "Portent", &[Entity::Player(P1)]);
    t.settle();
    assert!(!t.has_lost(P1));
    assert_eq!(t.library_size(P1), 2);
    let hand = t.hand_size(P0);
    to_next_upkeep(&mut t);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn solfatara_stops_playing_lands_but_not_putting_them_onto_the_battlefield() {
    cr!("305.1", "305.2");
    ruling!(
        "Solfatara",
        "Does not prevent effects from putting a land onto the battlefield."
    );
    ruling!(
        "Solfatara",
        "Prevents all land playing that turn, not just one land."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Exploration");
    cast_spell(&mut t, P0, "Solfatara", &[Entity::Player(P0)]);
    let a = t.hand(P0, "Forest");
    let b = t.hand(P0, "Forest");
    assert!(t.play_land(P0, a).is_err());
    assert!(t.play_land(P0, b).is_err());
    let forest = t.library_top(P0, "Forest");
    cast_spell(&mut t, P0, "Rampant Growth", &[]);
    assert!(t.on_battlefield(forest));
}

#[test]
fn dazzling_beauty_blocks_unblockable_and_trampling_attackers() {
    cr!("509.1h", "702.19d");
    ruling!(
        "Dazzling Beauty",
        "Works on creatures that can’t be blocked and on creatures that have special blocking requirements."
    );
    ruling!(
        "Dazzling Beauty",
        "Is not very useful against Trample creatures since Trample damage comes through as if a zero toughness creature blocked the Trampler."
    );
    supported("Dazzling Beauty");
    for (attacker, life) in [("Phantom Warrior", 20), ("Colossal Dreadmaw", 14)] {
        supported(attacker);
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, attacker);
        attack_with(&mut t, &[(a, Entity::Player(P1))]);
        t.answer(
            P1,
            DecisionKind::Blockers,
            mtg_engine::decision::Answer::Blockers(vec![]),
        );
        t.advance_to(P0, Step::DeclareBlockers);
        cast_only(&mut t, P1, "Dazzling Beauty", &[Entity::Object(a)]);
        t.resolve_all();
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), life, "{attacker}");
    }
}

#[test]
fn stangg_twin_stays_if_stangg_left_first() {
    cr!("603.7a", "610.3");
    ruling!(
        "Stangg",
        "If Stangg leaves the battlefield before its ability has resolved, that ability will still create a Stangg Twin token, but its delayed triggered ability that exiles the token will never have a chance to trigger."
    );
    supported("Stangg");
    let mut t = TestGame::new(2);
    let s = t.enter(P0, "Stangg");
    t.settle();
    t.g.move_object(s, Zone::Hand(P0), MoveCause::Effect, None);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Stangg Twin").len(), 1);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.named_on_battlefield("Stangg Twin").len(), 1);
}

#[test]
fn zirilan_exiles_the_dragon_only_if_it_is_phased_in() {
    cr!("702.26b", "702.26d", "603.7c");
    ruling!(
        "Zirilan of the Claw",
        "If the Dragon is put onto the battlefield and then phases out, the “exile it” ability will still apply."
    );
    supported("Zirilan of the Claw");
    for phase_back in [true, false] {
        let mut t = TestGame::new(2);
        let z = t.battlefield(P0, "Zirilan of the Claw");
        t.lands(P0, "Mountain", 3);
        t.library_top(P0, "Shivan Dragon");
        t.activate(P0, z, 0, &[]).unwrap();
        t.resolve_all();
        let d = t.named_on_battlefield("Shivan Dragon")[0];
        mtg_engine::kw::phasing::phase_out(&mut t.g, vec![d]);
        if phase_back {
            mtg_engine::kw::phasing::phase_in(&mut t.g, d);
        }
        t.advance_to_step(Step::End);
        t.resolve_all();
        assert_eq!(t.in_exile("Shivan Dragon"), phase_back, "{phase_back}");
    }
}
