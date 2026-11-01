//! Rulings batch P134 — bounce ("removal-bounce"): costs (alternative costs, increases and
//! reductions, kicker), targets needed to cast, "up to" targets, legendary sorceries,
//! values and conditions checked on resolution, and the cards' own rules.

use crate::r_p133_common::{in_hand_of, kill};
use crate::r_p134_common::*;
use crate::r_s01_common::{stack_library, supported};
use crate::r_s02_common::destroy;
use crate::r_s06_common::give_control;
use crate::r_s07_common::{chosen_modes, has_kw, resolved};
use crate::r_s08_common::mana_value;
use crate::r_s11_common::spells_copied;
use crate::r_s13_common::add;
use crate::r_s25_common::lands_for_cost;
use crate::r_s34_common::alternative;
use crate::r_s35_common::{creature_type_options, pick_options};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

#[test]
fn raise_the_palisade_offers_single_existing_creature_types() {
    cr!("205.3m", "700.13");
    ruling!("Raise the Palisade", "To choose a creature type, you must choose an existing creature type, such as Halfling or Scout. You can't choose multiple creature types");
    supported("Raise the Palisade");
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let thopter = t.battlefield(P1, "Ornithopter");
    lands_for_cost(&mut t, P0, "Raise the Palisade");
    let card = t.hand(P0, "Raise the Palisade");
    let from = t.asked().len();
    pick_options(&mut t, P0, &["Elf"]);
    t.cast(P0, card).go();
    t.resolve_all();
    let opts = creature_type_options(&t, P0, from);
    assert_eq!(opts.len(), 1);
    for ok in ["Halfling", "Scout", "Elf"] {
        assert!(opts[0].iter().any(|o| o == ok), "{ok}");
    }
    for bad in ["Forest", "Equipment", "Food", "Artifact", "Halfling Scout"] {
        assert!(!opts[0].iter().any(|o| o == bad), "{bad}");
    }
    assert!(t.on_battlefield(elf));
    assert!(in_hand_of(&t, bears, P1));
    assert!(in_hand_of(&t, thopter, P1));
}

/// Engine bug found by this ruling: a choice the targets depend on ("X target creatures of
/// the creature type of your choice") was made only as the spell resolved, so the spell
/// had no legal targets and couldn't be cast (CR 601.2c).
#[test]
fn selective_snare_choosing_one_type_lets_you_target_a_multi_type_creature() {
    cr!("205.3m", "601.2c");
    ruling!(
        "Selective Snare",
        "but choosing “Vampire” is enough to allow you to target a Vampire Knight."
    );
    supported("Selective Snare");
    let mut t = TestGame::new(2);
    let vk = t.battlefield(P1, "Bloodcrazed Paladin");
    let knight = t.battlefield(P1, "White Knight");
    assert!(t.obj(vk).chars.subtypes.iter().any(|s| s == "Vampire"));
    lands_for_cost(&mut t, P0, "Selective Snare");
    t.lands(P0, "Island", 1);
    let card = t.hand(P0, "Selective Snare");
    let from = t.asked().len();
    pick_options(&mut t, P0, &["Vampire"]);
    t.cast(P0, card).x(1).target(vk).go();
    t.resolve_all();
    assert_eq!(
        creature_type_options(&t, P0, from).len(),
        1,
        "chosen once, as it's cast"
    );
    assert!(in_hand_of(&t, vk, P1));
    assert!(t.on_battlefield(knight));
    // The same for cards in a graveyard (Aphetto Dredging): the type is chosen with the
    // targets, which must have it.
    supported("Aphetto Dredging");
    let mut t = TestGame::new(2);
    let e1 = t.graveyard(P0, "Llanowar Elves");
    let e2 = t.graveyard(P0, "Elvish Mystic");
    let bears = t.graveyard(P0, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Aphetto Dredging");
    let card = t.hand(P0, "Aphetto Dredging");
    let from = t.asked().len();
    pick_options(&mut t, P0, &["Elf"]);
    t.cast(P0, card).targets(&[ob(e1), ob(e2)]).go();
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(cands.len(), 1);
    assert!(cands[0].contains(&ob(e1)) && cands[0].contains(&ob(e2)));
    assert!(!cands[0].contains(&ob(bears)), "not an Elf");
    t.resolve_all();
    assert!(t.in_hand(P0, "Llanowar Elves") && t.in_hand(P0, "Elvish Mystic"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn total_cost_alternative_cost_plus_increases_minus_reductions_mana_value_unchanged() {
    cr!("601.2f", "118.9d", "202.3");
    ruling!("Snapback", "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying (such as the alternative cost of Snapback), add any cost increases, then apply any cost reductions.");
    ruling!("Gust of Wind", "add any cost increases, then apply any cost reductions (such as that of Gust of Wind). The mana value of the spell remains unchanged");
    supported("Snapback");
    supported("Gust of Wind");
    supported("Thalia, Guardian of Thraben");
    // Snapback for its alternative cost with Thalia out: exile a blue card plus {1}.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.hand(P0, "Counterspell");
    let snap = t.hand(P0, "Snapback");
    assert!(
        crate::r_s07_common::cast_methods(&mut t, P0, snap)
            .iter()
            .all(|m| !matches!(m, object::CastMethod::Alternative(_))),
        "the {{1}} isn't payable"
    );
    t.lands(P0, "Wastes", 1);
    let alt = alternative(&mut t, snap);
    let s = t.cast(P0, snap).method(alt).target(bears).go();
    assert!(t.in_exile("Counterspell"));
    assert_eq!(mana_value(&t, s), 2);
    t.resolve_all();
    assert!(in_hand_of(&t, bears, P1));
    // Gust of Wind ({3}{U}) with Thalia (+{1}) and a flier (-{2}): {2}{U}.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    let gust = t.hand(P0, "Gust of Wind");
    assert!(
        t.cast(P0, gust).target(bears).try_go().is_err(),
        "{{1}}{{U}} isn't enough"
    );
    t.clear_answers();
    t.lands(P0, "Wastes", 1);
    let s = t.cast(P0, gust).target(bears).go();
    assert_eq!(mana_value(&t, s), 4);
}

#[test]
fn spectral_deluge_counts_islands_as_it_resolves() {
    cr!("608.2h");
    ruling!("Spectral Deluge", "Use the number of Islands you control as Spectral Deluge resolves to determine the value of X.");
    supported("Spectral Deluge");
    // Two Islands as it's cast; a third arrives (both bounce) or one leaves (neither does).
    for (delta, bounced) in [(1i32, true), (-1, false)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        let mine = t.battlefield(P0, "Hill Giant");
        lands_for_cost(&mut t, P0, "Spectral Deluge");
        let card = t.hand(P0, "Spectral Deluge");
        t.cast(P0, card).go();
        if delta > 0 {
            t.battlefield(P0, "Island");
        } else {
            let island = t.named_on_battlefield("Island")[0];
            kill(&mut t, island);
        }
        t.resolve_all();
        assert_eq!(in_hand_of(&t, bears, P1), bounced, "{delta}");
        assert_eq!(in_hand_of(&t, giant, P1), bounced, "{delta}");
        assert!(t.on_battlefield(mine));
    }
}

#[test]
fn vronos_ultimate_sets_creature_type_and_keeps_everything_else() {
    cr!("205.1b", "613.1d", "613.1f");
    ruling!("Vronos, Masked Inquisitor", "Vronos, Masked Inquisitor's last ability causes the target artifact to lose any other creature types it has. It retains any other types, subtypes, and supertypes.");
    ruling!("Vronos, Masked Inquisitor", "Vronos, Masked Inquisitor's last ability doesn't remove any abilities the target artifact has.");
    supported("Vronos, Masked Inquisitor");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vronos, Masked Inquisitor");
    add(&mut t, v, "loyalty", 3);
    let th = t.battlefield(P0, "Ornithopter");
    t.activate(P0, v, 2, &[ob(th)]).expect("-7");
    t.resolve_all();
    let o = t.obj_now(th);
    assert_eq!(t.pt(th), (9, 9));
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    assert!(o.chars.subtypes.iter().any(|s| s == "Construct"));
    assert!(!o.chars.subtypes.iter().any(|s| s == "Thopter"));
    assert!(has_kw(&t, th, KeywordKind::Flying), "keeps flying");
    assert!(has_kw(&t, th, KeywordKind::Indestructible));
    // A legendary Equipment keeps its supertype and subtype.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vronos, Masked Inquisitor");
    add(&mut t, v, "loyalty", 3);
    let eq = t.battlefield(P0, "Shadowspear");
    t.activate(P0, v, 2, &[ob(eq)]).expect("-7");
    t.resolve_all();
    let o = t.obj_now(eq);
    assert!(o
        .chars
        .supertypes
        .iter()
        .any(|s| format!("{s:?}") == "Legendary"));
    assert!(o.chars.subtypes.iter().any(|s| s == "Equipment"));
    assert!(o.chars.subtypes.iter().any(|s| s == "Construct"));
}

#[test]
fn jace_living_guildpact_plus_one_leaves_the_other_card_on_top() {
    cr!("401.4");
    ruling!("Jace, the Living Guildpact", "When the first ability resolves, the card you don't put into your graveyard stays on top of your library.");
    supported("Jace, the Living Guildpact");
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P0, &["Island", "Forest", "Swamp"]);
    let j = t.battlefield(P0, "Jace, the Living Guildpact");
    t.answer_choose(P0, &[ob(lib[1])]);
    t.activate(P0, j, 0, &[]).expect("+1");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"));
    assert_eq!(
        t.g.players[0].library.last().copied(),
        Some(lib[0]),
        "the Island stays on top"
    );
}

#[test]
fn temporal_machinations_checks_artifacts_after_returning_the_creature() {
    cr!("608.2c");
    ruling!("Temporal Machinations", "Whether you control an artifact is checked only after returning the target creature to its owner’s hand. If the target creature is the only artifact you control, you won’t draw a card.");
    supported("Temporal Machinations");
    for other in [false, true] {
        let mut t = TestGame::new(2);
        let th = t.battlefield(P0, "Ornithopter");
        if other {
            t.battlefield(P0, "Sol Ring");
        }
        lands_for_cost(&mut t, P0, "Temporal Machinations");
        let card = t.hand(P0, "Temporal Machinations");
        t.cast(P0, card).target(th).go();
        t.resolve_all();
        assert!(in_hand_of(&t, th, P0));
        assert_eq!(
            t.hand_size(P0),
            if other { 2 } else { 1 },
            "other artifact {other}"
        );
    }
}

#[test]
fn string_of_disappearances_can_be_copied_only_once_per_resolution() {
    cr!("707.10", "608.2c");
    ruling!("String of Disappearances", "While resolving String of Disappearances, you can’t pay {U}{U} multiple times to copy it multiple times.");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.lands(P1, "Island", 4);
    lands_for_cost(&mut t, P0, "String of Disappearances");
    let card = t.hand(P0, "String of Disappearances");
    t.cast(P0, card).target(a).go();
    // P1 says yes to everything they're asked.
    for _ in 0..8 {
        t.answer_yes(P1, true);
    }
    t.answer_targets(P1, &[ob(b)]);
    t.resolve();
    assert_eq!(spells_copied(&t), 1);
    assert_eq!(t.stack_len(), 1);
    assert_eq!(crate::r_s07_common::untapped_named(&t, P1, "Island"), 2);
    t.clear_answers();
    t.resolve_all();
    assert!(in_hand_of(&t, b, P0));
}

#[test]
fn ghostly_wings_only_the_first_activation_does_anything() {
    cr!("608.2b", "303.4d");
    ruling!("Ghostly Wings", "You can activate the last ability multiple times to discard multiple cards. Only the first ability to resolve will have any effect.");
    supported("Ghostly Wings");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    let wings = crate::r_s06_common::attach_new(&mut t, P0, "Ghostly Wings", bears);
    t.hand(P0, "Island");
    t.hand(P0, "Forest");
    t.activate(P0, wings, 0, &[]).expect("first");
    t.activate(P0, wings, 0, &[]).expect("second");
    assert_eq!(t.hand_size(P0), 0, "two cards discarded");
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(in_hand_of(&t, bears, P0));
    assert!(t.on_battlefield(other));
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn castable_without_targets_for_its_trigger_or_with_up_to_targets() {
    cr!("603.3d", "115.1");
    ruling!(
        "Stern Proctor",
        "You can cast this card if there are no artifacts or enchantments on the battlefield."
    );
    supported("Stern Proctor");
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Stern Proctor");
    let card = t.hand(P0, "Stern Proctor");
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Stern Proctor").len(), 1);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn rushing_river_kicker_can_sacrifice_a_tapped_land() {
    cr!("702.33a", "118.3");
    ruling!("Rushing River", "You can sacrifice a tapped land.");
    supported("Rushing River");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let lands = t.lands(P0, "Island", 4);
    t.g.tap(lands[3]);
    let card = t.hand(P0, "Rushing River");
    t.answer_choose(P0, &[ob(lands[3])]);
    t.answer_targets(P0, &[ob(a)]);
    t.answer_targets(P0, &[ob(b)]);
    t.cast(P0, card).kicked(true).go();
    t.resolve_all();
    assert!(
        !t.on_battlefield(lands[3]),
        "the tapped Island was sacrificed"
    );
    assert!(in_hand_of(&t, a, P1) && in_hand_of(&t, b, P1));
}

#[test]
fn spells_that_need_their_targets_to_be_cast() {
    cr!("601.2c", "115.1");
    ruling!("Whiplash Trap", "You must target two creatures as you cast Whiplash Trap. If you can't (because just one creature is on the battlefield, perhaps), you can't cast the spell.");
    ruling!(
        "Whiplash Trap",
        "You can target any two creatures, not just ones that entered this turn."
    );
    ruling!(
        "Dramatic Rescue",
        "You must be able to target a creature to cast Dramatic Rescue."
    );
    ruling!("Aether Helix", "You must choose two legal targets in order to cast Aether Helix. If there are either no permanent cards in your graveyard or no permanents on the battlefield, you cannot begin to cast it.");
    for name in [
        "Whiplash Trap",
        "Dramatic Rescue",
        "Aether Helix",
        "Aether Gale",
    ] {
        supported(name);
    }
    // Whiplash Trap: one creature isn't enough; any two (one of them yours) are.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Whiplash Trap");
    let card = t.hand(P0, "Whiplash Trap");
    assert!(t.cast(P0, card).target(a).try_go().is_err());
    t.clear_answers();
    let b = t.battlefield(P0, "Hill Giant");
    t.cast(P0, card).targets(&[ob(a), ob(b)]).go();
    t.resolve_all();
    assert!(in_hand_of(&t, a, P1) && in_hand_of(&t, b, P0));
    // Dramatic Rescue with no creature.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sol Ring");
    lands_for_cost(&mut t, P0, "Dramatic Rescue");
    let card = t.hand(P0, "Dramatic Rescue");
    assert!(t.cast(P0, card).try_go().is_err());
    // Aether Helix with no permanent card in the graveyard.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    t.graveyard(P0, "Lightning Bolt");
    lands_for_cost(&mut t, P0, "Aether Helix");
    let card = t.hand(P0, "Aether Helix");
    t.answer_targets(P0, &[ob(a)]);
    assert!(t.cast(P0, card).try_go().is_err());
    t.clear_answers();
    let g = t.graveyard(P0, "Hill Giant");
    t.answer_targets(P0, &[ob(a)]);
    t.answer_targets(P0, &[ob(g)]);
    t.cast(P0, card).go();
    t.resolve_all();
    assert!(in_hand_of(&t, a, P1));
    assert!(t.in_hand(P0, "Hill Giant"));
}

#[test]
fn aether_gale_needs_six_targets_and_returns_the_legal_ones() {
    cr!("601.2c", "608.2b", "115.3");
    ruling!("Aether Gale", "You must choose six different legal targets in order to cast Aether Gale. If some, but not all, of those targets become illegal before Aether Gale resolves, the remaining legal targets will be put into their owners' hands.");
    let mut t = TestGame::new(2);
    let mut ids: Vec<ObjectId> = (0..5).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
    lands_for_cost(&mut t, P0, "Aether Gale");
    let card = t.hand(P0, "Aether Gale");
    let es: Vec<Entity> = ids.iter().map(|i| ob(*i)).collect();
    assert!(
        t.cast(P0, card).targets(&es).try_go().is_err(),
        "five aren't enough"
    );
    t.clear_answers();
    ids.push(t.battlefield(P0, "Hill Giant"));
    let es: Vec<Entity> = ids.iter().map(|i| ob(*i)).collect();
    t.cast(P0, card).targets(&es).go();
    kill(&mut t, ids[0]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    for id in &ids[1..5] {
        assert!(in_hand_of(&t, *id, P1));
    }
    assert!(in_hand_of(&t, ids[5], P0));
}

#[test]
fn legendary_sorceries_check_legendary_permanents_only_when_cast() {
    cr!("205.4d", "601.3");
    ruling!("Kamahl's Druidic Vow", "You can’t cast a legendary sorcery unless you control a legendary creature or a legendary planeswalker. Once you begin to cast a legendary sorcery, losing control of your legendary creatures and planeswalkers won’t affect that spell.");
    for name in ["Kamahl's Druidic Vow", "Karn's Temporal Sundering"] {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        lands_for_cost(&mut t, P0, name);
        let card = t.hand(P0, name);
        let cast = |t: &mut TestGame| {
            let b = t.cast(P0, card);
            if name == "Karn's Temporal Sundering" {
                b.targets(&[Entity::Player(P0)]).target(bears).try_go()
            } else {
                b.x(0).try_go()
            }
        };
        assert!(cast(&mut t).is_err(), "{name}: no legendary permanent");
        t.clear_answers();
        let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
        let s = cast(&mut t).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        destroy(&mut t, isamaru);
        t.resolve_all();
        assert!(resolved(&t, s), "{name}");
        if name == "Karn's Temporal Sundering" {
            assert!(in_hand_of(&t, bears, P1));
        }
    }
}

#[test]
fn cryptic_command_needs_two_different_modes() {
    cr!("700.2a", "700.2d", "601.2b");
    ruling!(
        "Cryptic Command",
        "You choose both modes as you cast Cryptic Command. You must choose two different modes."
    );
    supported("Cryptic Command");
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Cryptic Command");
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    let card = t.hand(P0, "Cryptic Command");
    let from = t.asked().len();
    let r = t.cast(P0, card).modes(&[3, 3]).try_go();
    let asked = t.asked();
    let d = asked[from..].iter().find_map(|(_, d)| match d {
        Decision::ChooseModes {
            min,
            max,
            allow_repeat,
            ..
        } => Some((*min, *max, *allow_repeat)),
        _ => None,
    });
    assert_eq!(d, Some((2, 2, false)));
    if let Ok(s) = r {
        let m = chosen_modes(&t, s);
        assert_eq!(m.len(), 2);
        assert_ne!(m[0], m[1], "{m:?}");
    }
}

#[test]
fn withdraw_chooses_the_payable_creature_on_cast_and_pays_on_resolution() {
    cr!("601.2c", "608.2c", "118.12");
    ruling!("Withdraw", "You choose on announcement which creature is the one that can be paid to prevent the effect. The payment itself is made on resolution (if at all).");
    supported("Withdraw");
    for pay in [false, true] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Hill Giant");
        lands_for_cost(&mut t, P0, "Withdraw");
        let card = t.hand(P0, "Withdraw");
        t.answer_targets(P0, &[ob(a)]);
        t.answer_targets(P0, &[ob(b)]);
        t.cast(P0, card).go();
        // P1's land arrives only after the spell was cast: the payment is on resolution.
        t.lands(P1, "Wastes", 1);
        t.answer_yes(P1, pay);
        t.resolve_all();
        assert!(in_hand_of(&t, a, P1));
        assert_eq!(t.on_battlefield(b), pay, "paid {pay}");
    }
}

#[test]
fn blizzard_specter_mode_chosen_when_triggered_discard_with_empty_hand() {
    cr!("603.3c", "700.2b");
    ruling!("Blizzard Specter", "You choose the mode when you put Blizzard Specter’s triggered ability on the stack. Blizzard Specter’s second mode may be chosen even if the player’s hand is empty.");
    supported("Blizzard Specter");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let s = t.battlefield(P0, "Blizzard Specter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.hand_size(P1), 0);
    answer_modes(&mut t, P0, &[1]);
    crate::r_s01_common::attack_with(&mut t, &[(s, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(chosen_modes(&t, top), vec![1]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn optional_targets_skip_or_fizzle() {
    cr!("115.1", "608.2b");
    ruling!("Teferi, Time Raveler", "You may activate Teferi's last ability without choosing any target. You'll just draw a card. However, if you do choose a target and the target permanent is an illegal target by the time Teferi's last ability tries to resolve, the ability doesn't resolve. You don't draw a card.");
    ruling!("Grasping Current", "You may cast Grasping Current without choosing any target creatures. You’ll just search for Jace, Ingenious Mind-Mage. However, if you choose any targets and all of those targets become illegal before Grasping Current resolves, the spell won’t resolve and you won’t search.");
    ruling!("Jace's Ruse", "You may cast Jace’s Ruse without choosing any target creatures. You’ll just search for Jace, Arcane Strategist. However, if you choose any targets and all of those targets become illegal before Jace’s Ruse resolves, the spell won’t resolve and you won’t search.");
    supported("Teferi, Time Raveler");
    // Teferi: no target, draw; a target that becomes illegal, no draw.
    for target in [false, true] {
        let mut t = TestGame::new(2);
        t.library_top(P0, "Island");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let tef = t.battlefield(P0, "Teferi, Time Raveler");
        let tg: Vec<Entity> = if target { vec![ob(bears)] } else { vec![] };
        t.answer_targets(P0, &tg);
        t.activate(P0, tef, 1, &[]).expect("-3");
        if target {
            kill(&mut t, bears);
        }
        t.resolve_all();
        assert_eq!(
            t.hand_size(P0),
            if target { 0 } else { 1 },
            "target {target}"
        );
    }
    for (spell, jace) in [
        ("Grasping Current", "Jace, Ingenious Mind-Mage"),
        ("Jace's Ruse", "Jace, Arcane Strategist"),
    ] {
        supported(spell);
        for target in [false, true] {
            let mut t = TestGame::new(2);
            let j = t.library_top(P0, jace);
            let bears = t.battlefield(P1, "Grizzly Bears");
            lands_for_cost(&mut t, P0, spell);
            let card = t.hand(P0, spell);
            let tg: Vec<Entity> = if target { vec![ob(bears)] } else { vec![] };
            t.answer_targets(P0, &tg);
            t.answer_yes(P0, true);
            t.answer_choose(P0, &[ob(j)]);
            t.cast(P0, card).go();
            if target {
                kill(&mut t, bears);
            }
            t.resolve_all();
            assert_eq!(t.in_hand(P0, jace), !target, "{spell} target {target}");
        }
    }
}

#[test]
fn neurok_replica_can_target_itself_but_then_fizzles() {
    cr!("601.2c", "602.2b", "608.2b");
    ruling!("Neurok Replica", "You may target Neurok Replica with its own ability. However, if you do, the ability won’t resolve for having an illegal target.");
    supported("Neurok Replica");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Neurok Replica");
    t.lands(P0, "Island", 2);
    t.activate(P0, r, 0, &[ob(r)])
        .expect("it can target itself");
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Neurok Replica"));
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn geistwave_draws_only_if_you_control_the_permanent_as_it_resolves() {
    cr!("608.2c", "608.2h");
    ruling!("Geistwave", "You won't draw a card if you don't control the target permanent as Geistwave resolves, even if you controlled that permanent at some previous point.");
    supported("Geistwave");
    for stolen in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.library_top(P0, "Island");
        lands_for_cost(&mut t, P0, "Geistwave");
        let card = t.hand(P0, "Geistwave");
        t.cast(P0, card).target(bears).go();
        if stolen {
            give_control(&mut t, bears, P1);
        }
        t.resolve_all();
        assert!(in_hand_of(&t, bears, P0));
        assert_eq!(
            t.hand_size(P0),
            if stolen { 1 } else { 2 },
            "stolen {stolen}"
        );
    }
}

#[test]
fn banishing_knack_ability_is_subject_to_summoning_sickness() {
    cr!("302.6", "113.6");
    ruling!("Banishing Knack", "“Summoning sickness” applies. The new ability can’t be activated unless the targeted creature has been under its controller’s control since the beginning of that player’s most recent turn or it has haste.");
    supported("Banishing Knack");
    for sick in [true, false] {
        let mut t = TestGame::new(2);
        let c = if sick {
            t.battlefield_sick(P0, "Grizzly Bears")
        } else {
            t.battlefield(P0, "Grizzly Bears")
        };
        let victim = t.battlefield(P1, "Hill Giant");
        lands_for_cost(&mut t, P0, "Banishing Knack");
        let card = t.hand(P0, "Banishing Knack");
        t.cast(P0, card).target(c).go();
        t.resolve_all();
        let r = t.activate(P0, c, 0, &[ob(victim)]);
        assert_eq!(r.is_ok(), !sick, "sick {sick}");
        t.resolve_all();
        assert_eq!(in_hand_of(&t, victim, P1), !sick);
    }
}
