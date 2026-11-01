//! Rulings batch P075 — abilities that grant protection: a target gaining protection in
//! response (CR 608.2b), what protection from a color or from colorless does (DEBT, CR
//! 702.16), protection from instants and sorceries, a missing "last chosen color", and the
//! color chosen as the spell or ability resolves.

use crate::r_p075_common::*;
use crate::r_s04_common::{ability_targets, spell_targets};
use crate::r_s21_common::legal_blocks;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn color_idx(c: Color) -> usize {
    Color::ALL.iter().position(|x| *x == c).unwrap()
}

/// Queues the color answer for Giver of Runes (option 0 is colorless) or for a plain
/// "color of your choice" (Shelter).
fn choose(t: &mut TestGame, giver: bool, c: Option<Color>) {
    let i = match (giver, c) {
        (true, None) => 0,
        (true, Some(c)) => 1 + color_idx(c),
        (false, Some(c)) => color_idx(c),
        (false, None) => unreachable!(),
    };
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
}

/// P0 gives `target` protection from `c` with Giver of Runes (an activation) or Shelter (a
/// spell), leaving it on the stack.
fn protect(t: &mut TestGame, giver: bool, target: ObjectId, c: Option<Color>) {
    choose(t, giver, c);
    if giver {
        let g = t.battlefield(P0, "Giver of Runes");
        t.activate(P0, g, 0, &[obj(target)]).expect("activates");
    } else {
        cast_card(t, P0, "Shelter", &[obj(target)]);
    }
}

#[test]
fn gaining_protection_in_response_makes_targets_illegal() {
    cr!("608.2b", "702.16b");
    ruling!(
        "Giver of Runes",
        "if all its targets are illegal, that spell or ability doesn't resolve. None of its effects happen, including effects unrelated to the target. If at least one target is still legal"
    );
    ruling!(
        "Shelter",
        "if all its targets are illegal, that spell or ability doesn’t resolve. None of its effects happen, including effects unrelated to the target. If at least one target is still legal"
    );
    supported("Shelter");
    for giver in [true, false] {
        for also_player in [false, true] {
            let mut t = TestGame::new(2);
            let bears = t.battlefield(P0, "Grizzly Bears");
            let targets = if also_player {
                vec![obj(bears), Entity::Player(P0)]
            } else {
                vec![obj(bears)]
            };
            t.answer(P1, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
            t.answer_targets(P1, &targets);
            give_mana_for(&mut t, P1, "Electrolyze");
            let e = t.hand(P1, "Electrolyze");
            t.cast(P1, e).go();
            let hand = t.hand_size(P1);
            protect(&mut t, giver, bears, Some(Color::Red));
            t.resolve_all();
            let case = format!("giver={giver} player={also_player}");
            assert_eq!(t.obj_now(bears).damage, 0, "{case}");
            if also_player {
                assert_eq!(
                    t.life(P0),
                    19,
                    "{case}: the legal target is still dealt damage"
                );
                assert_eq!(t.hand_size(P1), hand + 1, "{case}: still draws");
            } else {
                assert_eq!(t.life(P0), 20, "{case}");
                assert_eq!(t.hand_size(P1), hand, "{case}: no draw at all");
            }
        }
    }
}

/// Checks the DEBT of protection from the quality of `colored` sources on `bears`:
/// `creature` (P1's) can't block it, deal damage to it; `spell` and `attachment` can't
/// target / attach to it.
fn check_debt(
    t: &mut TestGame,
    bears: ObjectId,
    creature: &str,
    spell: &str,
    aura: Option<&str>,
    equipment: Option<&str>,
) {
    assert!(
        !spell_targets(t, P1, spell).contains(&obj(bears)),
        "{spell}"
    );
    if let Some(a) = aura {
        let aura = t.battlefield(P0, a);
        assert!(!t.g.attach(aura, obj(bears)), "{a} can't enchant it");
        t.g.move_object(
            aura,
            mtg_engine::object::Zone::Exile,
            mtg_engine::events::MoveCause::Effect,
            None,
        );
    }
    if let Some(e) = equipment {
        let eq = t.battlefield(P0, e);
        assert!(!ability_targets(t, eq, 0).contains(&obj(bears)), "{e}");
    }
    let blocker = t.battlefield(P1, creature);
    damage(t, blocker, obj(bears), 2);
    assert!(t.on_battlefield(bears), "damage from {creature} prevented");
    assert_eq!(t.obj_now(bears).damage, 0);
    to_beginning_of_combat(t, P0);
    attack_with(t, &at_p1(&[bears]));
    assert!(
        !legal_blocks(t, P1, &[(blocker, bears)]),
        "{creature} can't block"
    );
}

#[test]
fn protection_from_a_color_is_debt_and_nothing_else() {
    cr!("702.16b", "702.16c", "702.16d", "702.16e", "702.16f");
    ruling!(
        "Giver of Runes",
        "Protection from a color means that the target creature can't be blocked by creatures of that color, can't be the target of spells of that color or abilities from sources of that color, can't be enchanted or equipped by Auras or Equipment of that color, and all damage that sources of that color would deal to it is prevented. Nothing other than these events is prevented or illegal."
    );
    ruling!(
        "Shelter",
        "Protection from a color means that the target creature can’t be blocked by creatures of that color, can’t be the target of spells of that color or abilities from sources of that color, can’t be enchanted or equipped by Auras or Equipment of that color, and all damage that sources of that color would deal to it is prevented. Nothing other than these events is prevented or illegal."
    );
    for giver in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        protect(&mut t, giver, bears, Some(Color::Black));
        t.resolve_all();
        check_debt(
            &mut t,
            bears,
            "Vampire Noble",
            "Murder",
            Some("Dead Weight"),
            Some("Barbed Bloodletter"),
        );
        // Nothing else: a black sorcery that neither targets nor deals damage still kills it.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        protect(&mut t, giver, bears, Some(Color::Black));
        t.resolve_all();
        t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
        cast_card(&mut t, P1, "Infest", &[]);
        t.resolve_all();
        assert!(
            !t.on_battlefield(bears),
            "giver={giver}: Infest isn't stopped"
        );
    }
}

#[test]
fn giver_of_runes_protection_from_colorless() {
    cr!("702.16b", "702.16d", "702.16e", "702.16f", "105.2c");
    ruling!(
        "Giver of Runes",
        "protection from colorless means that the target creature can't be blocked by colorless creatures, can't be the target of colorless spells or abilities of colorless sources"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    protect(&mut t, true, bears, None);
    t.resolve_all();
    check_debt(
        &mut t,
        bears,
        "Ornithopter",
        "Spatial Contortion",
        None,
        Some("Bonesplitter"),
    );
}

#[test]
fn devoted_caretaker_protection_from_instants_and_sorceries() {
    cr!("702.16b", "702.16e");
    ruling!(
        "Devoted Caretaker",
        "can’t be targeted by instant and sorcery spells and (if it’s a creature) all damage from instant and sorcery spells will be prevented"
    );
    supported("Devoted Caretaker");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Devoted Caretaker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    t.activate(P0, c, 0, &[obj(bears)]).expect("activates");
    t.resolve_all();
    // Shock (an instant) can't target it once it's a spell being cast.
    let from = t.asked().len();
    t.answer_targets(P1, &[obj(other)]);
    cast_card(&mut t, P1, "Shock", &[]);
    let candidates = asked_since(&t, from)
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        });
    let candidates = candidates.expect("Shock's target chosen");
    assert!(!candidates.contains(&obj(bears)) && candidates.contains(&obj(other)));
    t.resolve_all();
    let other = t.battlefield(P0, "Grizzly Bears");
    // A creature's ability can still target it.
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    assert!(ability_targets(&mut t, pyro, 0).contains(&obj(bears)));
    // Pyroclasm (a sorcery) deals no damage to it.
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    cast_card(&mut t, P1, "Pyroclasm", &[]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(!t.on_battlefield(other));
}

#[test]
fn sanctuary_blade_without_a_chosen_color_grants_no_protection() {
    cr!("607.2d", "702.16a");
    ruling!(
        "Sanctuary Blade",
        "If Sanctuary Blade is somehow attached to a creature without a color having been chosen for it, it doesn't grant a protection ability to the creature. The creature doesn't gain protection from colorless."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blade = t.battlefield(P0, "Sanctuary Blade");
    // Attached without the "as it becomes attached" choice having been made.
    t.g.objects[blade.0 as usize].attached_to = Some(obj(bears));
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 2), "attached");
    assert!(!has(&t, bears, KeywordKind::Protection));
    let thopter = t.battlefield(P1, "Ornithopter");
    assert!(!t.g.protected_from(bears, thopter));
}

#[test]
fn sword_of_truth_and_justice_proliferates_both_kinds_before_sbas() {
    cr!("701.34a", "704.5q");
    ruling!(
        "Sword of Truth and Justice",
        "you'll put another of each of those counters on it before state-based actions remove them"
    );
    supported("Sword of Truth and Justice");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let sword = t.battlefield(P0, "Sword of Truth and Justice");
    assert!(t.g.attach(sword, obj(wurm)));
    put_counters(&mut t, wurm, "-1/-1", 2);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[wurm]));
    // The +1/+1 counter goes on the Wurm, then the Wurm is chosen while proliferating.
    t.answer_choose(P0, &[obj(wurm)]);
    t.answer_choose(P0, &[obj(wurm)]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    // (+1: 1, -1: 2) -> proliferate -> (2, 3) -> SBA -> one -1/-1 counter left.
    assert_eq!((plus(&t, wurm), minus(&t, wurm)), (0, 1));
}

#[test]
fn shelter_keeps_drawing_after_protection_from_its_own_color() {
    cr!("608.2b", "105.1");
    ruling!(
        "Shelter",
        "If Shelter gives the targeted creature protection from white, you’ll still draw a card."
    );
    ruling!(
        "Shelter",
        "You can’t choose “artifact” or “colorless” as Shelter asks you to choose a color"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    protect(&mut t, false, bears, Some(Color::White));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    let lions = t.battlefield(P1, "Savannah Lions");
    assert!(t.g.protected_from(bears, lions));
    let options = asked_since(&t, from)
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options),
            _ => None,
        });
    let options = options.expect("a color was chosen");
    assert_eq!(options.len(), 5, "{options:?}");
    assert!(options
        .iter()
        .all(|o| !o.contains("olorless") && !o.contains("rtifact")));
}

#[test]
fn color_is_chosen_as_it_resolves() {
    cr!("608.2d", "601.2c");
    ruling!(
        "Kabira Evangel",
        "You choose a color (or choose not to) as the triggered ability resolves."
    );
    ruling!(
        "Emerge Unscathed",
        "You choose the color as Emerge Unscathed resolves. Once you choose a color, it’s too late for players to respond."
    );
    ruling!(
        "Feat of Resistance",
        "You choose the color as Feat of Resistance resolves."
    );
    ruling!(
        "Stave Off",
        "You choose the target as part of casting the spell. You choose what color the target gains protection from when the spell resolves."
    );
    for name in [
        "Kabira Evangel",
        "Emerge Unscathed",
        "Feat of Resistance",
        "Stave Off",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let target = if name == "Kabira Evangel" {
            t.answer_yes(P0, true);
            let k = t.enter(P0, name);
            t.settle();
            k
        } else {
            let bears = t.battlefield(P0, "Grizzly Bears");
            cast_card(&mut t, P0, name, &[obj(bears)]);
            bears
        };
        let from = t.asked().len();
        assert_eq!(t.stack_len(), 1, "{name}");
        assert!(
            !t.asked()
                .iter()
                .any(|(_, d)| matches!(d, Decision::ChooseOption { .. })),
            "{name}: no color chosen yet"
        );
        choose_color(&mut t, P0, Color::Green);
        t.resolve_all();
        assert_eq!(
            count_since(&t, from, |d| matches!(d, Decision::ChooseOption { .. })),
            1,
            "{name}"
        );
        let elf = t.battlefield(P1, "Llanowar Elves");
        assert!(t.g.protected_from(target, elf), "{name}");
    }
}
