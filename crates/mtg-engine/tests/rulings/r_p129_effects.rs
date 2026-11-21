//! Rulings batch P129: what reflexive triggered abilities do as they resolve, and the other
//! abilities of the cards that have them.

use crate::r_p129_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn nevinyrral_destroys_only_after_the_reflexive_ability_resolves() {
    cr!("603.12", "603.10a");
    ruling!(
        "Nevinyrral, Urborg Tyrant",
        "If you choose to pay {1} for the last ability, a second triggered ability triggers. Players may respond"
    );
    supported("Nevinyrral, Urborg Tyrant");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let nev = t.battlefield(P0, "Nevinyrral, Urborg Tyrant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    destroy(&mut t, nev);
    resolve_into_reflexive(&mut t, &[]);
    assert!(t.on_battlefield(bears));
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn nevinyrral_has_hexproof_from_artifacts_creatures_and_enchantments() {
    cr!("702.11d");
    ruling!(
        "Nevinyrral, Urborg Tyrant",
        "Nevinyrral can't be the target of abilities of artifacts, creatures, or enchantments your opponents control."
    );
    let mut t = TestGame::new(2);
    let nev = t.battlefield(P0, "Nevinyrral, Urborg Tyrant");
    // An opponent's instant can target it.
    assert!(spell_targets(&mut t, P1, "Lightning Bolt").contains(&obj(nev)));
    // An opponent's creature's ability can't; the controller's own creature's can.
    let theirs = t.battlefield(P1, "Prodigal Sorcerer");
    assert!(!ability_targets(&mut t, theirs, 0).contains(&obj(nev)));
    let mine = t.battlefield(P0, "Prodigal Sorcerer");
    assert!(ability_targets(&mut t, mine, 0).contains(&obj(nev)));
    // An opponent's Aura spell (an enchantment spell) can't target it; its controller's
    // can.
    let aura_targets = |t: &mut TestGame, p: PlayerId| {
        let aura = t.hand(p, "Pacifism");
        t.g.recompute();
        let chars = t.g.obj(aura).chars.clone();
        let spec = mtg_engine::attach::aura_target_spec(&chars).expect("Aura without enchant");
        let ctx = mtg_engine::eval::Ctx::new(Some(aura), p);
        t.g.legal_target_candidates(&spec, &ctx, aura)
    };
    assert!(!aura_targets(&mut t, P1).contains(&obj(nev)));
    assert!(aura_targets(&mut t, P0).contains(&obj(nev)));
}

#[test]
fn nyssa_draws_that_many_cards_unless_all_targets_are_illegal() {
    cr!("603.12", "608.2b", "601.2c");
    ruling!(
        "Nyssa of Traken",
        "You can choose fewer than X targets for Nyssa's reflexive triggered ability. The number of targets doesn't change the number of cards drawn."
    );
    ruling!(
        "Nyssa of Traken",
        "If you chose at least one target and all targets are illegal as the ability tries to resolve, the ability is removed from the stack and does nothing."
    );
    supported("Nyssa of Traken");
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        let nyssa = t.battlefield(P0, "Nyssa of Traken");
        let a = t.battlefield(P0, "Ornithopter");
        let b = t.battlefield(P0, "Ornithopter");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let hand = t.hand_size(P0);
        t.answer_choose(P0, &[obj(a), obj(b)]);
        t.answer_targets(P0, &[obj(bears)]);
        attack_with(&mut t, &[(nyssa, Entity::Player(P1))]);
        resolve_into_reflexive(&mut t, &[obj(bears)]);
        if illegal {
            destroy(&mut t, bears);
        }
        t.resolve_all();
        if illegal {
            assert_eq!(t.hand_size(P0), hand);
        } else {
            assert!(t.obj_now(bears).tapped);
            assert_eq!(t.hand_size(P0), hand + 2);
        }
    }
}

#[test]
fn minsc_and_boo_draws_nothing_if_the_target_is_illegal() {
    cr!("603.12", "608.2b");
    ruling!(
        "Minsc & Boo, Timeless Heroes",
        "if the target is illegal as the ability tries to resolve, you will not draw any cards even if the sacrificed creature was a Hamster."
    );
    supported("Minsc & Boo, Timeless Heroes");
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        t.answer_yes(P0, true);
        let mb = t.enter(P0, "Minsc & Boo, Timeless Heroes");
        t.settle();
        t.resolve_all();
        let boo = with_subtype(&t, P0, "Hamster");
        assert_eq!(boo.len(), 1);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let hand = t.hand_size(P0);
        t.answer_choose(P0, &[obj(boo[0])]);
        t.answer_targets(P0, &[obj(bears)]);
        t.activate(P0, mb, 1, &[]).unwrap();
        resolve_into_reflexive(&mut t, &[obj(bears)]);
        if illegal {
            destroy(&mut t, bears);
        }
        t.resolve_all();
        if illegal {
            assert_eq!(t.hand_size(P0), hand);
        } else {
            // Boo is a 1/1: 1 damage, draw 1.
            assert_eq!(t.obj_now(bears).damage, 1);
            assert_eq!(t.hand_size(P0), hand + 1);
        }
    }
}

#[test]
fn mount_velus_manticore_counts_card_types_of_the_discarded_card() {
    cr!("603.12", "205.2a", "205.4a");
    ruling!(
        "Mount Velus Manticore",
        "If you do not discard a card, Mount Velus Manticore's reflexive triggered ability won't trigger."
    );
    ruling!(
        "Mount Velus Manticore",
        "Legendary, basic, and snow are supertypes, not card types"
    );
    supported("Mount Velus Manticore");
    // No discard: no reflexive ability.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mount Velus Manticore");
    t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.hand_size(P0), 1);
    // (discarded card, damage): a legendary artifact creature has two card types; a
    // kindred enchantment has two.
    for (card, dmg) in [
        ("Armix, Filigree Thrasher", 2),
        ("Bitterblossom", 2),
        ("Dryad Arbor", 2),
        ("Lightning Bolt", 1),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Mount Velus Manticore");
        let c = t.hand(P0, card);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(c)]);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.settle();
        t.resolve_all();
        assert_eq!(t.life(P1), 20 - dmg, "{card}");
    }
}

#[test]
fn savai_thundermane_reflexive_ability() {
    cr!("603.12", "608.2b", "603.10a");
    ruling!(
        "Savai Thundermane",
        "If you pay {2} while Savai Thundermane’s triggered ability is resolving, the reflexive triggered ability triggers and will resolve separately."
    );
    ruling!(
        "Savai Thundermane",
        "If the target creature is an illegal target by the time Savai Thundermane’s reflexive triggered ability tries to resolve, the ability won’t resolve. You won’t gain 2 life."
    );
    ruling!(
        "Savai Thundermane",
        "While resolving Savai Thundermane’s triggered ability, you can’t pay {2} more than once"
    );
    ruling!(
        "Savai Thundermane",
        "You can pay {2} even if Savai Thundermane leaves the battlefield before its triggered ability resolves."
    );
    supported("Savai Thundermane");
    // (Savai leaves first, the target becomes illegal)
    for (leaves, illegal) in [(false, false), (true, false), (false, true)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 6);
        let savai = t.battlefield(P0, "Savai Thundermane");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let card = t.hand(P0, "Lonely Sandbar");
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[obj(bears)]);
        cycle(&mut t, P0, card, 0).unwrap();
        t.settle();
        if leaves {
            destroy(&mut t, savai);
        }
        resolve_into_reflexive(&mut t, &[obj(bears)]);
        // {U} for cycling, {2} once.
        assert_eq!(
            t.g.permanents()
                .filter(|o| o.controller == P0 && o.is(CardType::Land) && o.tapped)
                .count(),
            3
        );
        assert_eq!(t.life(P0), 20);
        if illegal {
            destroy(&mut t, bears);
        }
        t.resolve_all();
        if illegal {
            assert_eq!(t.life(P0), 20);
        } else {
            assert!(!t.on_battlefield(bears), "leaves: {leaves}");
            assert_eq!(t.life(P0), 22);
        }
    }
}

#[test]
fn escape_protocol_returns_a_new_object() {
    cr!("603.12", "400.7", "704.5m", "704.5n");
    ruling!(
        "Escape Protocol",
        "When the card returns to the battlefield, it will be a new object with no connection to the card that was exiled."
    );
    ruling!(
        "Escape Protocol",
        "While resolving Escape Protocol’s triggered ability, you can’t pay {1} multiple times to exile multiple permanents you control."
    );
    supported("Escape Protocol");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    t.battlefield(P0, "Escape Protocol");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, counters::PLUS1, 1);
    let aura = t.battlefield(P0, "Holy Strength");
    t.g.attach(aura, obj(bears));
    let sword = t.battlefield(P0, "Bonesplitter");
    t.g.attach(sword, obj(bears));
    t.g.recompute();
    let card = t.hand(P0, "Lonely Sandbar");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    t.resolve_all();
    let now = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(now.len(), 1);
    assert_ne!(now[0], bears);
    assert_eq!(t.counters(now[0], counters::PLUS1), 0);
    assert!(t.in_graveyard(P0, "Holy Strength"));
    assert!(t.on_battlefield(sword));
    assert_eq!(t.obj_now(sword).attached_to, None);
    // {U} for cycling, {1} once.
    assert_eq!(
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.is(CardType::Land) && !o.tapped)
            .count(),
        2
    );
}

#[test]
fn back_for_more_fight_needs_both_creatures() {
    cr!("603.12", "701.14b", "608.2b");
    ruling!(
        "Back for More",
        "If the creature you put onto the battlefield is no longer on the battlefield or no longer a creature, the target creature won't deal or be dealt damage."
    );
    supported("Back for More");
    // The returned creature leaves before the reflexive ability resolves.
    let mut t = TestGame::new(2);
    let wurm = t.graveyard(P0, "Craw Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Back for More", &[obj(wurm)]);
    t.answer_targets(P0, &[obj(bears)]);
    resolve_into_reflexive(&mut t, &[obj(bears)]);
    let w = t.named_on_battlefield("Craw Wurm");
    assert_eq!(w.len(), 1);
    destroy(&mut t, w[0]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
    // The target becomes illegal: the ability doesn't resolve.
    let mut t = TestGame::new(2);
    let wurm = t.graveyard(P0, "Craw Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Back for More", &[obj(wurm)]);
    t.answer_targets(P0, &[obj(bears)]);
    resolve_into_reflexive(&mut t, &[obj(bears)]);
    destroy(&mut t, bears);
    t.resolve_all();
    let w = t.named_on_battlefield("Craw Wurm");
    assert_eq!(t.obj_now(w[0]).damage, 0);
}

#[test]
fn back_for_more_reflexive_ability_triggers_with_the_creatures_enters_ability() {
    cr!("603.12", "603.3b");
    ruling!(
        "Back for More",
        "The reflexive triggered ability from Back for More is put onto the stack at the same time as any other triggered abilities"
    );
    let mut t = TestGame::new(2);
    let elf = t.graveyard(P0, "Elvish Visionary");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Back for More", &[obj(elf)]);
    t.answer_targets(P0, &[obj(bears)]);
    let before = t.asked().len();
    t.resolve();
    // Both triggered abilities wait on the stack; their controller ordered them.
    assert_eq!(t.stack_len(), 2);
    assert!(t.g.stack.iter().all(|s| is_trigger(&t, *s)));
    assert!(asked_since(&t, before)
        .iter()
        .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::Order { .. })));
}

#[test]
fn curse_of_the_werefox_old_role_goes_before_the_fight() {
    cr!("603.12", "704.5y", "303.7");
    ruling!(
        "Curse of the Werefox",
        "that Role will be put into its owner's graveyard as a state-based action after the Monster Role is attached to it but before the reflexive triggered ability resolves."
    );
    supported("Curse of the Werefox");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Curse of the Werefox", &[obj(bears)]);
    t.answer_targets(P0, &[]);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Role").len(), 1);
    let first = with_subtype(&t, P0, "Role")[0];
    cast_new(&mut t, P0, "Curse of the Werefox", &[obj(bears)]);
    t.answer_targets(P0, &[obj(giant)]);
    resolve_into_reflexive(&mut t, &[obj(giant)]);
    let roles = with_subtype(&t, P0, "Role");
    assert_eq!(roles.len(), 1);
    assert_ne!(roles[0], first);
    assert_eq!(t.pt(bears), (3, 3));
    t.resolve_all();
    // A 3/3 fights the 3/3: both die.
    assert!(!t.on_battlefield(giant));
}
