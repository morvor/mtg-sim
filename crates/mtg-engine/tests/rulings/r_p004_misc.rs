//! Rulings batch P004 — assorted: Unexpected Results, Equipment that attaches itself as
//! it enters, counters on noncreature permanents, phasing out "until" (The Moment),
//! descend, suspended cards, Amalia, creature Equipment and creature mana abilities.

use crate::r_p004_common::*;
use crate::r_p023_common::wolf_token;
use crate::r_s01_common::attack_with;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts Unexpected Results in their main phase with `top` as their whole library.
fn unexpected_results(t: &mut TestGame, top: &str) -> ObjectId {
    t.set_step(P0, Step::PrecombatMain);
    t.g.players[0].library.clear();
    let c = t.library_top(P0, top);
    cast_now(t, P0, "Unexpected Results", &[]);
    c
}

#[test]
fn unexpected_results_land_and_nonland_cards() {
    cr!("608.2c", "601.2", "608.3", "117.1");
    ruling!(
        "Unexpected Results",
        "If you can’t cast the card (perhaps because there are no legal targets), or if you choose not to, the card will remain on top of the library."
    );
    ruling!(
        "Unexpected Results",
        "If you reveal a land card and put that card onto the battlefield, Unexpected Results will be put into its owner’s hand directly from the stack. It won’t be put into any graveyard."
    );
    ruling!(
        "Unexpected Results",
        "If you reveal a land card, Unexpected Results will be returned to your hand only if you put that land card onto the battlefield. If you don’t, Unexpected Results will be put into its owner’s graveyard."
    );
    ruling!(
        "Unexpected Results",
        "If you reveal a nonland card, you may cast it during the resolution of Unexpected Results. Ignore timing restrictions based on the card’s type. Other timing restrictions, such as “Cast [this card] only during combat,” must be followed."
    );
    ruling!(
        "Unexpected Results",
        "If you cast a spell “without paying its mana cost,” you can’t pay alternative costs such as overload costs. You can pay additional costs such as kicker costs. If the card has mandatory additional costs, you must pay those."
    );
    // A land put onto the battlefield: Unexpected Results goes to its owner's hand.
    let mut t = TestGame::new(2);
    unexpected_results(&mut t, "Forest");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    assert!(t.in_hand(P0, "Unexpected Results"));
    assert!(!t.in_graveyard(P0, "Unexpected Results"));
    // Declined: it goes to the graveyard and the land stays on top.
    let mut t = TestGame::new(2);
    let f = unexpected_results(&mut t, "Forest");
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Unexpected Results"));
    assert_eq!(t.zone(f), Zone::Library(P0));
    // A creature card: cast during the resolution, without paying its mana cost.
    let mut t = TestGame::new(2);
    unexpected_results(&mut t, "Hill Giant");
    t.g.players[0].mana_pool.empty();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert!(t.in_graveyard(P0, "Unexpected Results"));
    // No legal target (Doom Blade, no creatures), or declined: it stays on top.
    for (card, yes) in [("Doom Blade", true), ("Grizzly Bears", false)] {
        let mut t = TestGame::new(2);
        let c = unexpected_results(&mut t, card);
        t.answer_yes(P0, yes);
        t.resolve_all();
        assert_eq!(t.zone(c), Zone::Library(P0), "{card}");
        assert_eq!(t.g.player(P0).library.last(), Some(&c), "{card}: not on top");
    }
    // Other timing restrictions apply: Gorilla War Cry (only during combat) can't be cast
    // in the main phase.
    let mut t = TestGame::new(2);
    let c = unexpected_results(&mut t, "Gorilla War Cry");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.zone(c), Zone::Library(P0));
    // A mandatory additional cost must be paid: Bone Splinters sacrifices a creature.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    unexpected_results(&mut t, "Bone Splinters");
    t.g.players[0].mana_pool.empty();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(theirs)]);
    t.answer_choose(P0, &[obj(mine)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "the additional cost wasn't paid");
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn equipment_that_attaches_itself_as_it_enters() {
    cr!("603.3d", "301.5c", "701.3b");
    ruling!(
        "Rosethorn Halberd",
        "If there are no creatures to attach Rosethorn Halberd to when it enters the battlefield, it simply remains unattached."
    );
    ruling!(
        "Pirate's Cutlass",
        "You can cast Pirate's Cutlass even if you control no Pirates. If you control no Pirates as Pirate's Cutlass enters, its triggered ability will have no effect."
    );
    ruling!(
        "Thunder Lasso",
        "You may target a creature that is already tapped with Thunder Lasso's third ability."
    );
    // Rosethorn Halberd with no creature: it stays unattached.
    let mut t = TestGame::new(2);
    let h = t.enter(P0, "Rosethorn Halberd");
    t.resolve_all();
    assert!(t.on_battlefield(h));
    assert_eq!(t.obj_now(h).attached_to, None);
    // Pirate's Cutlass with no Pirate: it can be cast, and its trigger does nothing.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Grizzly Bears");
    let c = cast_now(&mut t, P0, "Pirate's Cutlass", &[]);
    t.resolve_all();
    let c = t.g.current(c);
    assert!(t.on_battlefield(c));
    assert_eq!(t.obj_now(c).attached_to, None);
    // Thunder Lasso can target an already tapped creature.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lasso = t.battlefield(P0, "Thunder Lasso");
    assert!(t.g.attach(lasso, obj(bears)));
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.tap(giant);
    t.answer_targets(P0, &[obj(giant)]);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    let top = *t.g.stack.last().expect("no trigger");
    assert_eq!(
        t.g.obj(top).stack.as_ref().unwrap().chosen[0].targets[0],
        vec![obj(giant)]
    );
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
}

#[test]
fn counters_on_a_noncreature_permanent_matter_once_it_becomes_a_creature() {
    cr!("122.1a", "613.4c");
    ruling!(
        "Earthen Arms",
        "Any permanent can have a +1/+1 counter put on it. Those counters won’t have any effect unless that permanent is or becomes a creature."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let forest = t.battlefield(P0, "Forest");
    cast_now(&mut t, P0, "Earthen Arms", &[obj(forest)]);
    t.resolve_all();
    assert_eq!(t.counters(forest, counters::PLUS1), 2);
    cast_now(&mut t, P0, "Animate Land", &[obj(forest)]);
    t.resolve_all();
    assert_eq!(t.pt(forest), (5, 5));
}

#[test]
fn lightning_serpent_put_onto_the_battlefield_has_x_0() {
    cr!("107.3m", "122.6");
    ruling!(
        "Lightning Serpent",
        "If Lightning Serpent enters without X being paid because an effect puts it onto the battlefield or an effect lets you cast it without paying its mana cost, then X is 0."
    );
    let mut t = TestGame::new(2);
    let s = t.enter(P0, "Lightning Serpent");
    t.settle();
    assert_eq!(t.counters(s, "+1/+0"), 0);
    assert_eq!(t.pt(s), (2, 1));
}

#[test]
fn the_moment_phasing_out_ends_for_as_long_as_effects_but_isnt_leaving() {
    cr!("702.26b", "702.26d", "702.26f", "611.2b", "610.3");
    ruling!(
        "The Moment",
        "Any continuous effects with a \"for as long as\" duration ignore phased-out objects. Any such effects will expire if their conditions are no longer met after ignoring the phased-out objects."
    );
    ruling!(
        "The Moment",
        "Any one-shot effects that are waiting \"until [this] leaves the battlefield,\" won't happen when a permanent phases out."
    );
    // Sower of Temptation (control for as long as it remains on the battlefield) phases
    // out: the Bears goes back to P1.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let sower = t.enter(P0, "Sower of Temptation");
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    let moment = t.battlefield(P0, "The Moment");
    mana(&mut t, P0);
    t.activate(P0, moment, 0, &[obj(sower)]).expect("The Moment");
    t.resolve_all();
    assert!(t.obj_now(sower).phased_out);
    assert_eq!(t.obj_now(bears).controller, P1, "the control effect didn't end");
    // Fairgrounds Warden phases out: the exiled card doesn't return; it returns once the
    // Warden actually leaves.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let w = banish_trigger(&mut t, "Fairgrounds Warden", bears);
    t.resolve_all();
    let moment = t.battlefield(P0, "The Moment");
    mana(&mut t, P0);
    t.activate(P0, moment, 0, &[obj(w)]).expect("The Moment");
    t.resolve_all();
    assert!(t.obj_now(w).phased_out);
    assert!(t.in_exile("Grizzly Bears"), "returned when the Warden phased out");
    // The Moment leaves: the Warden phases in, the Bears is still exiled.
    destroy_and_flush(&mut t, moment);
    t.settle();
    assert!(!t.obj_now(w).phased_out);
    assert!(t.in_exile("Grizzly Bears"));
    destroy_and_flush(&mut t, w);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn a_token_put_into_a_graveyard_isnt_descending() {
    cr!("701.58a", "111.7");
    ruling!(
        "Child of the Volcano",
        "A permanent card is an artifact, battle, creature, enchantment, land, or planeswalker card. Tokens are not cards, and while tokens are put into the graveyard before ceasing to exist, that action doesn't count as a player having descended."
    );
    for card in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let c = t.battlefield(P0, "Child of the Volcano");
        if card {
            let b = t.battlefield(P0, "Grizzly Bears");
            destroy_now(&mut t, b);
        } else {
            let w = wolf_token(&mut t, P0);
            destroy_now(&mut t, w);
        }
        t.settle();
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(t.counters(c, counters::PLUS1), card as u32, "card={card}");
    }
}

#[test]
fn rose_tyler_counts_suspended_cards() {
    cr!("702.62b", "122.1");
    ruling!(
        "Rose Tyler",
        "A suspended card is a card in exile with suspend that has at least one time counter on it. That includes any cards which were put there by an effect that says they \"gain suspend\" or similar."
    );
    for suspended in [false, true] {
        let mut t = TestGame::new(2);
        let rose = t.battlefield(P0, "Rose Tyler");
        // An exiled card with a time counter: suspended only if it has suspend.
        let c = t.exile(P0, if suspended { "Rift Bolt" } else { "Grizzly Bears" });
        t.g.add_counters(obj(c), counters::TIME, 1, None);
        attack_with(&mut t, &[(rose, Entity::Player(P1))]);
        t.resolve_all();
        assert_eq!(t.counters(rose, counters::TIME), suspended as u32, "{suspended}");
    }
}

#[test]
fn amalia_checks_its_power_after_exploring() {
    cr!("701.44a", "608.2h", "113.7a");
    ruling!(
        "Amalia Benavides Aguirre",
        "Amalia Benavides Aguirre's power is checked only once, as the triggered ability resolves. If it is no longer on the battlefield at that point, use its power as it last existed on the battlefield."
    );
    ruling!(
        "Amalia Benavides Aguirre",
        "Amalia Benavides Aguirre's power must be exactly 20 during the resolution of its last ability after it finishes exploring in order to destroy all other creatures. If it's more or less than 20, nothing is destroyed."
    );
    // Power 19 + explore (a nonland card: +1/+1 counter) = 20: destroy all others.
    for (counters_on, expect_destroy) in [(17u32, true), (16, false), (18, false)] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Amalia Benavides Aguirre");
        t.g.add_counters(obj(a), counters::PLUS1, counters_on, None);
        t.library_top(P0, "Grizzly Bears");
        t.battlefield(P1, "Hill Giant");
        t.answer_yes(P0, false);
        t.g.gain_life(P0, 1);
        t.g.flush_events();
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield("Hill Giant").is_empty(),
            expect_destroy,
            "{counters_on}"
        );
        assert!(t.on_battlefield(a));
    }
}

#[test]
fn an_equipment_that_is_a_creature_cant_equip() {
    cr!("301.5c", "205.1b");
    ruling!(
        "Dan Lewis",
        "An Equipment that's also a creature can't be equipped to anything unless it has the reconfigure ability."
    );
    ruling!(
        "Tough Cookie",
        "As Tough Cookie's second ability resolves, the target permanent keeps any other types and subtypes it had before it became an artifact creature."
    );
    // Tough Cookie makes Bonesplitter a 4/4 artifact creature that's still Equipment.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let cookie = t.battlefield(P0, "Tough Cookie");
    let bs = t.battlefield(P0, "Bonesplitter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    mana(&mut t, P0);
    t.activate(P0, cookie, 0, &[obj(bs)]).expect("Tough Cookie");
    t.resolve_all();
    let o = t.obj_now(bs);
    assert!(o.chars.is(CardType::Creature) && o.chars.is(CardType::Artifact));
    assert!(o.chars.has_subtype("Equipment"), "lost its subtype");
    assert_eq!(t.pt(bs), (4, 4));
    mana(&mut t, P0);
    let _ = t.activate(P0, bs, 0, &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.obj_now(bs).attached_to, None, "a creature Equipment got attached");
    // Dan Lewis: a noncreature artifact is Equipment and can equip.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Dan Lewis");
    let ms = t.battlefield(P0, "Mind Stone");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert!(t.obj_now(ms).chars.has_subtype("Equipment"));
    mana(&mut t, P0);
    let equip = t
        .obj_now(ms)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .position(|a| a.text.to_lowercase().contains("equip"))
        .expect("equip");
    t.activate(P0, ms, equip, &[obj(bears)]).expect("equip");
    t.resolve_all();
    assert_eq!(t.obj_now(ms).attached_to, Some(obj(bears)));
    assert_eq!(t.pt(bears), (3, 2));
}

#[test]
fn badgermole_cub_only_sees_creatures_tapped_for_their_own_tap_mana_abilities() {
    cr!("605.1a", "106.12a");
    ruling!(
        "Badgermole Cub",
        "An ability that triggers \"whenever you tap a creature for mana\" triggers only if you activate a mana ability of a creature including {T} in its cost. Mana abilities that don't include the {T} symbol and instead say \"Tap an untapped creature you control\" or similar won't cause Badgermole Cub's second ability to trigger. Similarly, it won't trigger if you tap a creature to activate a mana ability of another object (even if that mana ability also includes {T})."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Badgermole Cub");
    let elves = t.battlefield(P0, "Llanowar Elves");
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, elves, "{G}"));
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
    // Springleaf Drum: tapping the Elves pays the Drum's cost; no extra mana.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Badgermole Cub");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let drum = t.battlefield(P0, "Springleaf Drum");
    t.answer_choose(P0, &[obj(elves)]);
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, drum, "one mana"));
    assert!(t.obj_now(elves).tapped);
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
}

#[test]
fn phasing_in_tapped_isnt_becoming_tapped() {
    cr!("702.26c", "702.26d", "603.2");
    // The Pandorica isn't supported; the rule is exercised with a direct phase-out/in.
    ruling!(
        "The Pandorica",
        "Abilities that trigger when a creature becomes tapped won't trigger when a creature phases in tapped. (This might happen if the creature in question has one or more stun counters on it.)"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Verity Circle");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1, "control: tapping triggers Verity Circle");
    t.resolve_all();
    mtg_engine::keyword_impls::phase_out(&mut t.g, vec![bears]);
    t.g.flush_events();
    t.settle();
    assert!(t.obj_now(bears).phased_out);
    mtg_engine::keyword_impls::phase_in(&mut t.g, bears);
    t.g.flush_events();
    t.settle();
    let o = t.obj_now(bears);
    assert!(!o.phased_out && o.tapped);
    assert_eq!(t.stack_len(), 0, "phasing in tapped triggered Verity Circle");
}

#[test]
fn suspends_last_counter_cast_ignores_type_timing_or_leaves_it_exiled() {
    cr!("702.62a", "702.62b");
    // Taigam isn't supported; the suspend rule is exercised with Rift Bolt.
    ruling!(
        "Taigam, Master Opportunist",
        "As the second triggered ability of suspend resolves, you may cast the card. Timing permissions based on the card’s type are ignored. If you choose not to (or can’t) cast the card, it remains exiled with no time counters on it, and it’s no longer suspended."
    );
    for cast in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::End);
        let c = t.exile(P0, "Rift Bolt");
        t.g.add_counters(obj(c), counters::TIME, 1, None);
        t.answer_yes(P0, cast);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        if cast {
            // A sorcery cast during the upkeep.
            assert_eq!(t.life(P1), 17);
            assert!(t.in_graveyard(P0, "Rift Bolt"));
        } else {
            assert_eq!(t.zone(c), Zone::Exile);
            assert_eq!(t.counters(c, counters::TIME), 0);
            // No longer suspended: nothing happens at the next upkeep.
            t.advance_to(P0, Step::Draw);
            t.set_step(P1, Step::End);
            t.advance_to(P0, Step::Upkeep);
            assert_eq!(t.stack_len(), 0);
            assert_eq!(t.zone(c), Zone::Exile);
        }
    }
}
