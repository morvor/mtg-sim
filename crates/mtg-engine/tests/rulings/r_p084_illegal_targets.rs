//! Rulings batch P084 — graveyard hate whose targets become illegal (CR 608.2b): a spell
//! or ability whose every target is illegal doesn't resolve and none of its effects
//! happen; one with some legal targets does as much as it can.

use crate::r_p084_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 casts `name` (paid with `cost`, modes `modes`) targeting `target`, a card in P1's
/// graveyard, which leaves the graveyard before the spell resolves: nothing happens
/// beyond the spell going to the graveyard.
/// (With the card still there, the spell does change the outcome.)
fn spell_fizzles(name: &str, cost: &str, modes: &[usize], target_card: &str) {
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        let card = t.graveyard(P1, target_card);
        let spell = cast_modes(&mut t, P0, name, cost, modes, &[&[obj(card)]]);
        let before = outcome(&t);
        if gone {
            exile_now(&mut t, card);
        }
        t.resolve_all();
        assert_eq!(outcome(&t) == before, gone, "{name} gone={gone}");
        assert!(t.stack.is_empty(), "{name}");
        assert_eq!(t.zone(spell), Zone::Graveyard(P0), "{name}");
    }
}

#[test]
fn spells_with_a_single_graveyard_target_do_nothing_if_it_is_gone() {
    cr!("608.2b", "115.1");
    ruling!(
        "Beckon Apparition",
        "If the target card becomes an illegal target for Beckon Apparition, the spell doesn’t resolve. You don’t create a token."
    );
    ruling!(
        "Shamble Back",
        "If the creature card becomes an illegal target, Shamble Back doesn’t resolve. None of its effects happen."
    );
    ruling!(
        "Vile Rebirth",
        "If the creature card is an illegal target when Vile Rebirth tries to resolve, it won’t resolve and none of its effects will happen. You won’t put a Zombie token onto the battlefield."
    );
    ruling!(
        "Cling to Dust",
        "If the target card is an illegal target by the time Cling to Dust tries to resolve, the spell won’t resolve. You won’t gain 3 life or draw a card."
    );
    ruling!(
        "Cremate",
        "If the target card is no longer in the graveyard when Cremate tries to resolve, it won't resolve and none of its effects will happen. You won't draw a card."
    );
    ruling!(
        "Morbid Bloom",
        "If the targeted card is removed from the graveyard before Morbid Bloom resolves, the spell doesn’t resolve. You won’t get any Saproling tokens."
    );
    ruling!(
        "Shadowfeed",
        "If the targeted card is removed from the graveyard before Shadowfeed resolves, Shadowfeed doesn’t resolve. You won’t gain 3 life."
    );
    ruling!(
        "Glorious Decay",
        "If the last mode of Glorious Decay is chosen but the target is illegal as Glorious Decay tries to resolve, it won't resolve and none of its effects will happen. You won't draw a card."
    );
    for (name, cost, modes, card) in [
        ("Beckon Apparition", "B", &[][..], "Lightning Bolt"),
        ("Shamble Back", "B", &[], "Grizzly Bears"),
        ("Vile Rebirth", "B", &[], "Grizzly Bears"),
        // A creature card (life) and a noncreature card (draw): neither happens.
        ("Cling to Dust", "B", &[], "Grizzly Bears"),
        ("Cling to Dust", "B", &[], "Lightning Bolt"),
        ("Cremate", "B", &[], "Lightning Bolt"),
        ("Morbid Bloom", "4BG", &[], "Grizzly Bears"),
        ("Shadowfeed", "B", &[], "Lightning Bolt"),
        ("Glorious Decay", "1G", &[2], "Lightning Bolt"),
    ] {
        spell_fizzles(name, cost, modes, card);
    }
}

#[test]
fn turn_the_earth_with_only_illegal_targets_gains_no_life() {
    cr!("608.2b", "115.1");
    ruling!(
        "Turn the Earth",
        "If you choose target cards in graveyards and all of those targets are somehow illegal as Turn the Earth tries to resolve (perhaps because your opponent responded with a Turn the Earth of their own), the spell will not resolve and you will gain no life."
    );
    // P1 responds with a Turn the Earth of their own, shuffling both targets away first.
    let mut t = TestGame::new(2);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    let mine = cast_modes(&mut t, P0, "Turn the Earth", "G", &[], &[&[obj(a), obj(b)]]);
    cast_modes(&mut t, P1, "Turn the Earth", "G", &[], &[&[obj(a), obj(b)]]);
    t.resolve_all();
    assert_eq!(t.life(P1), 22);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.zone(mine), Zone::Graveyard(P0));
    // With one target still legal, it resolves: that card is shuffled in, P0 gains 2.
    let mut t = TestGame::new(2);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    cast_modes(&mut t, P0, "Turn the Earth", "G", &[], &[&[obj(a), obj(b)]]);
    exile_now(&mut t, a);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert!(matches!(t.zone(b), Zone::Library(P0)));
}

#[test]
fn erebos_intervention_first_mode_illegal_target_gains_no_life() {
    cr!("608.2b", "700.2");
    ruling!(
        "Erebos's Intervention",
        "If you choose the first mode and the target creature is an illegal target by the time Erebos’s Intervention tries to resolve, the spell won’t resolve. You won’t gain X life."
    );
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        supported("Erebos's Intervention");
        let spell = t.hand(P0, "Erebos's Intervention");
        mana(&mut t, P0, "2B");
        t.cast(P0, spell).x(2).modes(&[0]).target(bears).go();
        if gone {
            exile_now(&mut t, bears);
        }
        t.resolve_all();
        assert_eq!(t.life(P0), if gone { 20 } else { 22 }, "gone={gone}");
        if !gone {
            assert!(t.in_graveyard(P1, "Grizzly Bears"));
        }
    }
}

#[test]
fn thought_distortion_with_an_illegal_player_target_does_nothing() {
    // Not resolving (608.2b) isn't being countered: "can't be countered" doesn't keep the
    // spell from being removed from the stack with no effect.
    cr!("608.2b", "702.11c");
    ruling!(
        "Thought Distortion",
        "If the target player is an illegal target by the time Thought Distortion tries to resolve, the spell won’t resolve and will be put into its owner’s graveyard with no effect. This is true even though it can’t be countered."
    );
    for hexproof in [false, true] {
        let mut t = TestGame::new(2);
        t.hand(P1, "Lightning Bolt");
        t.graveyard(P1, "Shock");
        let spell = cast_modes(&mut t, P0, "Thought Distortion", "4BB", &[], &[&[pl(P1)]]);
        if hexproof {
            // P1 gains hexproof in response (Leyline of Sanctity: "You have hexproof.").
            t.battlefield(P1, "Leyline of Sanctity");
        }
        t.resolve_all();
        assert_eq!(t.zone(spell), Zone::Graveyard(P0), "hexproof={hexproof}");
        assert_eq!(t.in_hand(P1, "Lightning Bolt"), hexproof);
        assert_eq!(t.in_graveyard(P1, "Shock"), hexproof);
    }
}

/// P0 activates the ability of `src` containing `needle` (paid with `cost`) targeting
/// `card`, which leaves the graveyard before the ability resolves: nothing happens, and
/// the source gets no counter of kind `counter`.
/// With `gone` false, the card stays and the ability does change the outcome.
fn ability_fizzles(
    t: &mut TestGame,
    src: ObjectId,
    needle: &str,
    cost: &str,
    card: ObjectId,
    counter: &str,
    gone: bool,
) {
    activate(t, P0, src, needle, cost, &[obj(card)]);
    t.settle();
    assert_eq!(t.stack_len(), 1, "{needle}");
    // The card is exiled on resolution, not as a cost.
    assert!(matches!(t.zone(card), Zone::Graveyard(_)), "{needle}");
    let before = outcome(t);
    if gone {
        exile_now(t, card);
    }
    t.resolve_all();
    let changed = outcome(t) != before || t.counters(src, counter) > 0;
    assert_eq!(changed, !gone, "{needle} gone={gone}");
}

#[test]
fn activated_graveyard_hate_with_an_illegal_target_does_nothing() {
    cr!("608.2b", "602.2");
    ruling!(
        "Grave Robbers",
        "If the target is not there on resolution, you do not gain the life."
    );
    ruling!(
        "Conversion Chamber",
        "If the targeted artifact card is no longer in the graveyard when the first ability tries to resolve, it won’t resolve and none of its effects will happen. You won’t put a charge counter on Conversion Chamber."
    );
    ruling!(
        "Necrogenesis",
        "If the targeted card is removed from the graveyard before the ability resolves, the ability doesn’t resolve. You won’t get a Saproling token."
    );
    ruling!(
        "Cemetery Reaper",
        "The creature card in a graveyard is exiled as part of the activated ability's effect, not as a cost. If that card has left the graveyard by the time the ability would resolve, the ability doesn't resolve and you don't get a Zombie token."
    );
    for (src, needle, cost, card, counter) in [
        (
            "Grave Robbers",
            "Exile target artifact",
            "B",
            "Ornithopter",
            "-",
        ),
        (
            "Conversion Chamber",
            "Exile target artifact",
            "2",
            "Ornithopter",
            "charge",
        ),
        (
            "Necrogenesis",
            "Exile target creature",
            "2",
            "Grizzly Bears",
            "-",
        ),
        (
            "Cemetery Reaper",
            "Exile target creature",
            "2B",
            "Grizzly Bears",
            "-",
        ),
    ] {
        supported(src);
        for gone in [false, true] {
            let mut t = TestGame::new(2);
            let s = t.battlefield(P0, src);
            let c = t.graveyard(P1, card);
            ability_fizzles(&mut t, s, needle, cost, c, counter, gone);
        }
    }
}

#[test]
fn grave_robbers_targets_on_activation_exiles_on_resolution() {
    cr!("602.2b", "601.2c", "608.2b");
    ruling!(
        "Grave Robbers",
        "This exiles the artifact on resolution, but you choose it as a target when activating the ability."
    );
    supported("Grave Robbers");
    let mut t = TestGame::new(2);
    let robbers = t.battlefield(P0, "Grave Robbers");
    let thopter = t.graveyard(P1, "Ornithopter");
    activate(&mut t, P0, robbers, "Exile target", "B", &[obj(thopter)]);
    t.settle();
    // Targeted on activation, still in the graveyard while the ability waits.
    assert_eq!(t.zone(thopter), Zone::Graveyard(P1));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_exile("Ornithopter"));
    assert_eq!(t.life(P0), 22);
}

#[test]
fn deathrite_shaman_abilities_do_nothing_with_an_illegal_target() {
    cr!("608.2b", "605.1a", "602.2");
    ruling!(
        "Deathrite Shaman",
        "If the target of any of Deathrite Shaman's three abilities is an illegal target when that ability tries to resolve, it won't resolve and none of its effects will happen. You won't add mana, no opponent will lose life, or you won't gain life, as appropriate."
    );
    supported("Deathrite Shaman");
    for (needle, cost, card) in [
        ("land card", "", "Forest"),
        ("instant or sorcery", "B", "Lightning Bolt"),
        ("creature card", "G", "Grizzly Bears"),
    ] {
        for gone in [false, true] {
            let mut t = TestGame::new(2);
            let s = t.battlefield(P0, "Deathrite Shaman");
            let c = t.graveyard(P1, card);
            // The first ability has a target, so it isn't a mana ability: it uses the
            // stack.
            ability_fizzles(&mut t, s, needle, cost, c, "-", gone);
        }
    }
}

#[test]
fn scavenging_ooze_twice_on_the_same_card_only_the_first_does_anything() {
    cr!("608.2b");
    ruling!(
        "Scavenging Ooze",
        "If the target card is an illegal target when the ability tries to resolve, it won't resolve and none of its effects will happen. No +1/+1 counter will be put on Scavenging Ooze and you won't gain life. Notably, this means that if you activate Scavenging Ooze's ability multiple times targeting the same creature card, only the first instance of the ability to resolve will have any effect."
    );
    supported("Scavenging Ooze");
    let mut t = TestGame::new(2);
    let ooze = t.battlefield(P0, "Scavenging Ooze");
    let bears = t.graveyard(P1, "Grizzly Bears");
    activate(&mut t, P0, ooze, "Exile target", "G", &[obj(bears)]);
    activate(&mut t, P0, ooze, "Exile target", "G", &[obj(bears)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.counters(ooze, "+1/+1"), 1);
    assert_eq!(t.life(P0), 21);
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn armored_scrapgorger_trigger_with_an_illegal_target_adds_no_oil() {
    cr!("608.2b", "603.2");
    ruling!(
        "Armored Scrapgorger",
        "If the card chosen as the target of the last ability is an illegal target as the ability tries to resolve, most likely because it left the graveyard some other way, the ability will fail to resolve and none of its effects will happen. Armored Scrapgorger won't get an oil counter."
    );
    supported("Armored Scrapgorger");
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        let gorger = t.battlefield(P0, "Armored Scrapgorger");
        let card = t.graveyard(P1, "Lightning Bolt");
        t.answer_targets(P0, &[obj(card)]);
        t.g.tap(gorger);
        t.g.flush_events();
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if gone {
            exile_now(&mut t, card);
        }
        t.resolve_all();
        assert_eq!(
            t.counters(gorger, "oil"),
            if gone { 0 } else { 1 },
            "gone={gone}"
        );
    }
}

#[test]
fn carrion_beetles_missing_targets_dont_stop_the_others() {
    cr!("608.2b", "115.1");
    ruling!(
        "Carrion Beetles",
        "If any of the targets are not there on resolution, the others are still affected."
    );
    supported("Carrion Beetles");
    let mut t = TestGame::new(2);
    let beetles = t.battlefield(P0, "Carrion Beetles");
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Hill Giant");
    let c = t.graveyard(P1, "Lightning Bolt");
    t.answer_targets(P0, &[obj(a), obj(b), obj(c)]);
    activate(&mut t, P0, beetles, "Exile up to three", "2B", &[]);
    t.settle();
    // Hill Giant leaves the graveyard (shuffled away) before the ability resolves.
    let b_lib = move_to_library(&mut t, b);
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(c), Zone::Exile);
    assert_eq!(t.zone(b_lib), Zone::Library(P1));
}

#[test]
fn verdant_command_does_what_it_can_with_one_illegal_target() {
    cr!("608.2b", "700.2");
    ruling!(
        "Verdant Command",
        "If the target for one mode has become illegal as Verdant Command tries to resolve, but it still has at least one legal target, it will resolve and do as much as it can to the legal targets."
    );
    let mut t = TestGame::new(2);
    let card = t.graveyard(P1, "Grizzly Bears");
    // Exile target card from a graveyard; target player gains 3 life.
    cast_modes(
        &mut t,
        P0,
        "Verdant Command",
        "1G",
        &[2, 3],
        &[&[obj(card)], &[pl(P0)]],
    );
    exile_now(&mut t, card);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}
