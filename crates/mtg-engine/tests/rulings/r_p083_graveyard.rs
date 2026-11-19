//! Rulings batch P083 — graveyard hate and spells cast from exile: Eradicate and a
//! face-down creature (CR 708.2, 201.2), Spellweaver Helix and split card names (CR
//! 709.4a), casting copies (CR 707.12), Lochmere Serpent's costs (CR 205.3i, 509.1h),
//! Watchers of the Dead, Memory's Journey and Flesh // Blood with illegal targets
//! (CR 608.2b), Armored Scrapgorger's tap trigger (CR 603.2e) and Extirpate against
//! flashback (CR 601.2a).

use crate::r_p083_common::*;
use crate::r_s01_common::{attack_with, watch};
use crate::r_s21_common::blocks_now;
use mtg_engine::decision::Decision;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn named_in(t: &TestGame, zone: Zone, name: &str) -> usize {
    let ids: Vec<ObjectId> = match zone {
        Zone::Exile => t.g.exile.clone(),
        Zone::Graveyard(p) => t.g.player(p).graveyard.clone(),
        Zone::Hand(p) => t.g.player(p).hand.clone(),
        Zone::Library(p) => t.g.player(p).library.clone(),
        _ => vec![],
    };
    ids.iter()
        .filter(|id| t.g.obj(**id).chars.name == name)
        .count()
}

#[test]
fn eradicate_on_a_face_down_creature_finds_no_name() {
    cr!("708.2", "201.2a", "701.23a");
    ruling!(
        "Eradicate",
        "A face-down creature has no name, so no card can possibly share a name with it (not even other cards with no name). If you Eradicate a face-down creature you will still search its controller’s library, but you won’t be able to exile any cards from that library."
    );
    supported("Eradicate");
    let mut t = TestGame::new(2);
    let fd = crate::r_s11_common::manifest_card(&mut t, P1, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    t.graveyard(P1, "Grizzly Bears");
    t.library_top(P1, "Grizzly Bears");
    let lib = t.library_size(P1);
    cast_resolve(&mut t, P0, "Eradicate", &[obj(fd)]);
    assert!(!t.on_battlefield(fd));
    assert_eq!(
        named_in(&t, Zone::Exile, "Grizzly Bears"),
        1,
        "only the creature"
    );
    assert_eq!(named_in(&t, Zone::Hand(P1), "Grizzly Bears"), 1);
    assert_eq!(named_in(&t, Zone::Graveyard(P1), "Grizzly Bears"), 1);
    assert_eq!(t.library_size(P1), lib, "nothing exiled from the library");
}

#[test]
fn spellweaver_helix_matches_either_name_of_a_split_card() {
    cr!("709.4a", "707.12", "603.4");
    ruling!(
        "Spellweaver Helix",
        "A split card has the same name as a spell if either of its names is that spell's name."
    );
    supported("Spellweaver Helix");
    supported("Flesh // Blood");
    // The Helix exiles Flesh // Blood and Lava Spike; P0 then casts Blood (the half of
    // another Flesh // Blood): it has the same name as the exiled split card, so P0 may
    // copy Lava Spike and cast the copy.
    let mut t = TestGame::new(2);
    let fb = t.graveyard(P1, "Flesh // Blood");
    let spike = t.graveyard(P1, "Lava Spike");
    t.answer_targets(P0, &[obj(fb), obj(spike)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Spellweaver Helix");
    t.resolve_all();
    assert_eq!(named_in(&t, Zone::Exile, "Flesh // Blood"), 1);
    assert_eq!(named_in(&t, Zone::Exile, "Lava Spike"), 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    let blood = t.hand(P0, "Flesh // Blood");
    t.cast(P0, blood)
        .method(CastMethod::Half(1))
        .target(obj(bears))
        .target(player(P1))
        .go();
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 3 - 2, "Lava Spike's copy and Blood");
}

#[test]
fn flawless_forgery_leaves_the_card_exiled() {
    cr!("707.12");
    ruling!(
        "Flawless Forgery",
        "After Flawless Forgery resolves, the exiled card remains exiled."
    );
    supported("Flawless Forgery");
    let mut t = TestGame::new(2);
    let bolt = t.graveyard(P1, "Lightning Bolt");
    cast_new(&mut t, P0, "Flawless Forgery", &[obj(bolt)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "the copy was cast");
    assert_eq!(named_in(&t, Zone::Exile, "Lightning Bolt"), 1);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(named_in(&t, Zone::Exile, "Lightning Bolt"), 1);
}

#[test]
fn mnemonic_deluge_casts_sorcery_copies_while_resolving() {
    cr!("707.12", "707.12a", "307.1");
    ruling!(
        "Mnemonic Deluge",
        "Because you're casting the spells during the resolution of Mnemonic Deluge, ignore any timing restrictions based on card type. In other words, you can cast copies of a sorcery card this way."
    );
    supported("Mnemonic Deluge");
    let mut t = TestGame::new(2);
    let spike = t.graveyard(P1, "Lava Spike");
    cast_new(&mut t, P0, "Mnemonic Deluge", &[obj(spike)]);
    for _ in 0..3 {
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[player(P1)]);
    }
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 9);
    assert!(t.in_exile("Mnemonic Deluge"));
}

// ---------------------------------------------------------------------------------
// Lochmere Serpent: "{U}, Sacrifice an Island: This creature can't be blocked this turn.
// {B}, Sacrifice a Swamp: You gain 1 life and draw a card."
// ---------------------------------------------------------------------------------

#[test]
fn lochmere_serpent_after_blockers_stays_blocked() {
    cr!("509.1h");
    ruling!(
        "Lochmere Serpent",
        "Activating Lochmere Serpent's first activated ability after it has become blocked won't cause it to become unblocked."
    );
    supported("Lochmere Serpent");
    let mut t = TestGame::new(2);
    let serpent = t.battlefield(P0, "Lochmere Serpent");
    let wall = t.battlefield(P1, "Giant Spider");
    t.lands(P0, "Island", 2);
    crate::r_s03_common::to_blockers(&mut t, &[(serpent, player(P1))], &[(wall, serpent)]);
    crate::r_s06_common::activate_containing(&mut t, P0, serpent, "can't be blocked").unwrap();
    t.resolve_all();
    assert_eq!(blocks_now(&t), vec![(wall, serpent)]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn lochmere_serpent_needs_lands_with_the_subtype() {
    cr!("205.3i", "305.6", "118.3");
    ruling!(
        "Lochmere Serpent",
        "An Island is a land with the subtype Island, not any land that could produce blue mana. For example, Mystic Sanctuary is an Island, but Castle Vantress isn't. Similarly, a Swamp is a land with the subtype Swamp, not any land that could produce black mana."
    );
    supported("Mystic Sanctuary");
    supported("Castle Vantress");
    for (land, ok) in [("Castle Vantress", false), ("Mystic Sanctuary", true)] {
        let mut t = TestGame::new(2);
        let serpent = t.battlefield(P0, "Lochmere Serpent");
        let l = t.battlefield(P0, land);
        let _ = crate::r_s06_common::activate_containing(&mut t, P0, serpent, "can't be blocked");
        t.resolve_all();
        assert_eq!(t.on_battlefield(l), !ok, "{land}");
    }
    // A Swamp: Bojuka Bog taps for {B} but isn't a Swamp.
    supported("Bojuka Bog");
    let mut t = TestGame::new(2);
    let serpent = t.battlefield(P0, "Lochmere Serpent");
    let bog = t.battlefield(P0, "Bojuka Bog");
    t.lands(P0, "Swamp", 0);
    let hand = t.hand_size(P0);
    let _ = crate::r_s06_common::activate_containing(&mut t, P0, serpent, "gain 1 life");
    t.resolve_all();
    assert!(t.on_battlefield(bog));
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P0), 20);
}

// ---------------------------------------------------------------------------------
// Graveyard spells and illegal targets.
// ---------------------------------------------------------------------------------

#[test]
fn watchers_of_the_dead_leaves_two_cards() {
    cr!("608.2c");
    ruling!(
        "Watchers of the Dead",
        "An opponent with one or two cards in their graveyard won’t exile any cards."
    );
    supported("Watchers of the Dead");
    for n in [1usize, 2, 4] {
        let mut t = TestGame::new(2);
        for _ in 0..n {
            t.graveyard(P1, "Grizzly Bears");
        }
        let w = t.battlefield(P0, "Watchers of the Dead");
        t.activate(P0, w, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.graveyard_size(P1), n.min(2), "{n} cards");
    }
}

#[test]
fn memorys_journey_skips_illegal_targets() {
    cr!("608.2b", "601.2c");
    ruling!(
        "Memory's Journey",
        "Any of the targeted cards that are illegal targets by the time Memory's Journey resolves aren't shuffled into their owner's library."
    );
    supported("Memory's Journey");
    let mut t = TestGame::new(2);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Craw Wurm");
    let c = t.graveyard(P1, "Llanowar Elves");
    let lib = t.library_size(P1);
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Memory's Journey");
    let mj = t.hand(P0, "Memory's Journey");
    t.answer_targets(P0, &[player(P1)]);
    t.answer_targets(P0, &[obj(a), obj(b), obj(c)]);
    t.cast(P0, mj).go();
    // In response, P1 exiles the Wurm from the graveyard (Extirpate).
    cast_resolve(&mut t, P1, "Extirpate", &[obj(b)]);
    assert_eq!(t.library_size(P1), lib + 2);
    assert!(t.in_exile("Craw Wurm"));
    assert_eq!(t.graveyard_size(P1), 1, "only Extirpate");
}

#[test]
fn blood_with_one_illegal_target_does_nothing() {
    cr!("608.2b", "709.3");
    ruling!(
        "Flesh // Blood",
        "As Blood is resolving, if only one of the targets is legal, Blood will still resolve but will have no effect: If the first target is illegal, it can't deal damage to anything. If the second target is illegal, it can't be dealt damage."
    );
    for first_gone in [true, false] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Craw Wurm");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Mountain", 1);
        t.lands(P0, "Forest", 1);
        let blood = t.hand(P0, "Flesh // Blood");
        t.cast(P0, blood)
            .method(CastMethod::Half(1))
            .target(obj(mine))
            .target(obj(theirs))
            .go();
        let gone = if first_gone { mine } else { theirs };
        cast_resolve(&mut t, P1, "Unsummon", &[obj(gone)]);
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Flesh // Blood"));
        let survivor = if first_gone { theirs } else { mine };
        assert_eq!(damage_on(&t, survivor), 0, "first gone = {first_gone}");
        assert_eq!(t.life(P1), 20);
    }
}

#[test]
fn flesh_with_an_illegal_target() {
    cr!("608.2b", "709.3");
    ruling!(
        "Flesh // Blood",
        "As Flesh is resolving, if the creature is an illegal target but the creature card is still a legal target, the creature card will be exiled but no counters will be put onto the creature. If the creature is the only legal target, Flesh will still resolve but will have no effect as you didn't exile a card."
    );
    for gone in [Some(true), Some(false), None] {
        let mut t = TestGame::new(2);
        let wurm = t.graveyard(P1, "Craw Wurm");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Swamp", 1);
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Wastes", 3);
        let flesh = t.hand(P0, "Flesh // Blood");
        t.cast(P0, flesh)
            .method(CastMethod::Half(0))
            .target(obj(wurm))
            .target(obj(bears))
            .go();
        match gone {
            Some(true) => cast_resolve(&mut t, P1, "Unsummon", &[obj(bears)]),
            Some(false) => cast_resolve(&mut t, P1, "Extirpate", &[obj(wurm)]),
            None => t.resolve_all(),
        }
        match gone {
            Some(true) => assert!(t.in_exile("Craw Wurm"), "the card is still exiled"),
            Some(false) => {
                assert_eq!(t.counters(bears, "+1/+1"), 0, "no card exiled: X is 0")
            }
            None => {
                assert!(t.in_exile("Craw Wurm"));
                assert_eq!(t.counters(bears, "+1/+1"), 6, "both legal");
            }
        }
    }
}

#[test]
fn armored_scrapgorger_triggers_however_it_becomes_tapped() {
    cr!("603.2e", "508.1f");
    ruling!(
        "Armored Scrapgorger",
        "Armored Scrapgorger's last ability triggers whenever it becomes tapped for any reason, not just due to its mana ability."
    );
    supported("Armored Scrapgorger");
    // Attacking taps it.
    let mut t = TestGame::new(2);
    let sg = t.battlefield(P0, "Armored Scrapgorger");
    let card = t.graveyard(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(card)]);
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(sg, player(P1))]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.counters(sg, "oil"), 1);
    // An opponent's effect taps it.
    supported("Fire // Ice");
    let mut t = TestGame::new(2);
    let sg = t.battlefield(P0, "Armored Scrapgorger");
    let card = t.graveyard(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(card)]);
    t.lands(P1, "Island", 1);
    t.lands(P1, "Wastes", 1);
    let fi = t.hand(P1, "Fire // Ice");
    t.cast(P1, fi)
        .method(CastMethod::Half(1))
        .targets(&[obj(sg)])
        .go();
    t.resolve_all();
    assert!(t.obj_now(sg).tapped);
    assert_eq!(t.counters(sg, "oil"), 1);
}

#[test]
fn extirpate_cant_stop_a_flashback_spell_being_cast() {
    cr!("601.2a", "702.34a");
    ruling!(
        "Extirpate",
        "Because the first step of casting a spell is to move it from the zone it's in, Extirpate can't be used to stop a spell with flashback from being cast after its controller has announced that they're casting it."
    );
    // While P0 chooses Memory's Journey's targets (casting it with flashback), the card is
    // already on the stack, not in the graveyard.
    let mut t = TestGame::new(2);
    let mj = t.graveyard(P0, "Memory's Journey");
    t.graveyard(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseTargets { .. }),
        |g| {
            g.stack
                .iter()
                .any(|id| g.obj(*id).chars.name == "Memory's Journey")
                && g.player(PlayerId(0)).graveyard.is_empty()
        },
    );
    t.answer_targets(P0, &[player(P1)]);
    let spell = t
        .cast(P0, mj)
        .method(CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Flashback,
        ))
        .go();
    assert!(!seen.lock().unwrap().is_empty());
    assert!(seen.lock().unwrap().iter().all(|x| *x));
    // Extirpate can't target it now.
    assert!(!spell_targets(&mut t, P1, "Extirpate").contains(&obj(spell)));
    assert_eq!(t.g.obj(spell).zone, Zone::Stack);
}
