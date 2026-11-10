//! Rulings batch P155 — the mana value (CR 202.3) of double-faced permanents and cards
//! (CR 202.3b, 712.8a, 712.8e), melded permanents (CR 202.3c), split cards (CR 202.3d,
//! 709.3b), Adventure spells (CR 715.3), prototype permanents (CR 702.160), tokens and
//! copies (CR 111.4, 707.2), and spells whose cost was reduced (CR 601.2f).

use crate::r_p155_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_back_face_up_permanent_has_the_mana_value_of_its_front_face() {
    cr!("202.3b", "712.8e");
    ruling!(
        "Broadside Bombardiers",
        "The back face of a double-faced card doesn't have a mana cost. A double-faced permanent with its back face up has a mana value equal to the mana value of its front face."
    );
    ruling!(
        "Summon: Bahamut",
        "The mana value of a nonmodal double-faced permanent is the mana value of its front face, no matter which face is up."
    );
    ruling!(
        "Boon of Boseiju",
        "Transforming double-faced permanents, such as those found in this set, have the mana value of their front face at all times, even while back-face-up."
    );
    supported("Broadside Bombardiers");
    supported("Boon of Boseiju");
    supported("Summon: Bahamut");
    // Broadside Bombardiers' boast sacrificing the Militia (no mana cost, front face 4):
    // 2 + 4 damage.
    let mut t = TestGame::new(2);
    let bomb = t.battlefield(P0, "Broadside Bombardiers");
    let rav = militia(&mut t, P0);
    assert_eq!(mana_value(&t, rav), 4);
    attack_with(&mut t, &[(bomb, pl(P1))]);
    t.answer_choose(P0, &[obj(rav)]);
    activate_resolve(&mut t, P0, bomb, "Sacrifice another", "", &[pl(P1)]);
    assert_eq!(t.life(P1), 14, "{}", t.dump_log());
    // Boon of Boseiju with the Militia and Grizzly Bears: X is 4.
    let mut t = TestGame::new(2);
    militia(&mut t, P0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Boon of Boseiju", "1G", &[obj(bears)]);
    assert_eq!(t.pt(bears), (6, 6));
    // Summon: Bahamut's chapter IV with the Militia, a token copy of Hill Giant (4) and a
    // plain token (0): 8 damage to P1.
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Summon: Bahamut");
    militia(&mut t, P0);
    let giant = t.battlefield(P1, "Hill Giant");
    let copies = token_copy(&mut t, P0, giant);
    assert_eq!(mana_value(&t, copies[0]), 4);
    create_token(&mut t, P0, "Saproling");
    t.answer_targets(P0, &[]);
    t.answer_targets(P0, &[]);
    put(&mut t, saga, counters::LORE, 4);
    t.resolve_all();
    assert_eq!(t.life(P1), 12, "{}", t.dump_log());
}

#[test]
fn a_double_faced_card_in_a_graveyard_has_the_mana_value_of_its_front_face() {
    cr!("712.8a", "202.3");
    ruling!(
        "Heretic's Punishment",
        "The mana value of a double-faced card in your graveyard is the mana value of the front face."
    );
    supported("Heretic's Punishment");
    let mut t = TestGame::new(2);
    let hp = t.battlefield(P0, "Heretic's Punishment");
    stack_library(
        &mut t,
        P0,
        &[
            "Delver of Secrets // Insectile Aberration",
            "Huntmaster of the Fells // Ravager of the Fells",
            "Forest",
        ],
    );
    activate_resolve(&mut t, P0, hp, "mill three", "3R", &[pl(P1)]);
    assert_eq!(
        t.life(P1),
        16,
        "Huntmaster's {{2}}{{R}}{{G}}: {}",
        t.dump_log()
    );
    let hm =
        t.g.find_in_zone(Zone::Graveyard(P0), "Huntmaster of the Fells")[0];
    assert_eq!(mana_value(&t, hm), 4);
}

#[test]
fn eldritch_evolution_with_a_transformed_a_melded_and_a_copied_creature() {
    cr!("202.3b", "202.3c", "712.8e", "712.8g", "707.2");
    ruling!(
        "Eldritch Evolution",
        "The mana value of a permanent that's a double-faced card with its back face up is the mana value of its front face. The mana value of a melded permanent is the sum of the mana values of its front faces. A creature that's a copy of either has a mana value of 0."
    );
    supported("Eldritch Evolution");
    supported("Ulamog's Crusher");
    // The library holds creature cards of mana value 2, 6 and 8.
    let lib = |t: &mut TestGame| {
        (
            t.library_top(P0, "Grizzly Bears"),
            t.library_top(P0, "Craw Wurm"),
            t.library_top(P0, "Ulamog's Crusher"),
        )
    };
    // The Militia (front face {3}{W}): X is 6: Craw Wurm, not the Crusher.
    let mut t = TestGame::new(2);
    let rav = militia(&mut t, P0);
    let (bears, wurm, crusher) = lib(&mut t);
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(rav)]);
    t.answer_choose(P0, &[obj(wurm)]);
    cast_resolve(&mut t, P0, "Eldritch Evolution", "1GG", &[]);
    assert!(was_offered(&t, P0, from, bears) && was_offered(&t, P0, from, wurm));
    assert!(!was_offered(&t, P0, from, crusher));
    assert_eq!(count_named(&t, "Craw Wurm"), 1);
    // Brisela ({5}{W}{W} + {2}{W}{W} = 11): X is 13: the Crusher (8) too.
    let mut t = TestGame::new(2);
    let b = brisela(&mut t, P0);
    assert_eq!(mana_value(&t, b), 11);
    let (_, _, crusher) = lib(&mut t);
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(b)]);
    t.answer_choose(P0, &[obj(crusher)]);
    cast_resolve(&mut t, P0, "Eldritch Evolution", "1GG", &[]);
    assert!(was_offered(&t, P0, from, crusher));
    assert_eq!(count_named(&t, "Ulamog's Crusher"), 1);
    // A Mirror Image copying the Militia has mana value 0: X is 2: only Grizzly Bears.
    let mut t = TestGame::new(2);
    let rav = militia(&mut t, P0);
    let copy = mirror_image_of(&mut t, P0, rav);
    assert_eq!(mana_value(&t, copy), 0);
    let (bears, wurm, crusher) = lib(&mut t);
    // Hill Giant (4) would be offered if the sacrificed Mirror Image card's own mana
    // value (3) were used: X 5.
    let giant = t.library_top(P0, "Hill Giant");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(copy)]);
    t.answer_choose(P0, &[obj(bears)]);
    cast_resolve(&mut t, P0, "Eldritch Evolution", "1GG", &[]);
    assert!(was_offered(&t, P0, from, bears));
    assert!(
        !was_offered(&t, P0, from, giant),
        "X is 2, not 5: {}",
        t.dump_log()
    );
    assert!(!was_offered(&t, P0, from, wurm) && !was_offered(&t, P0, from, crusher));
    assert_eq!(count_named(&t, "Grizzly Bears"), 1);
}

#[test]
fn a_split_card_has_the_combined_mana_value_but_the_spell_only_its_half() {
    cr!("202.3d", "709.3b");
    ruling!(
        "Sunbird's Invocation",
        "The mana value of a split card, such as cards with aftermath from the Amonkhet block, is based on the combined mana cost of its two halves. The mana value of the resulting spell is based only on the half you cast."
    );
    supported("Sunbird's Invocation");
    supported("Fire // Ice");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sunbird's Invocation");
    // Two Forests on top, then two Craw Wurms: the trigger reveals the Forests (X is 2,
    // Fire's mana value), not the Wurms too (X 4, the card's).
    let cards = stack_library(&mut t, P0, &["Forest", "Forest", "Craw Wurm", "Craw Wurm"]);
    let fi = t.hand(P0, "Fire // Ice");
    assert_eq!(mana_value(&t, fi), 4, "{{1}}{{R}} and {{1}}{{U}} combined");
    mana(&mut t, P0, "1R");
    // Fire divides 2 damage: all to P1.
    t.answer(P0, DecisionKind::Divide, Answer::Default);
    let fire = t
        .cast(P0, fi)
        .method(CastMethod::Half(0))
        .targets(&[pl(P1)])
        .go();
    assert_eq!(mana_value(&t, fire), 2);
    t.settle();
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // The two Forests (not castable) went to the bottom; the Wurms are still on top.
    let lib = crate::r_s33_common::library_top_first(&t, P0);
    assert_eq!(lib.len(), 34);
    assert_eq!(&lib[..2], &cards[2..], "the Wurms on top: {}", t.dump_log());
    let bottom = &lib[32..];
    assert!(
        bottom.contains(&t.g.current(cards[0])) && bottom.contains(&t.g.current(cards[1])),
        "{}",
        t.dump_log()
    );
}

#[test]
fn an_adventure_spell_has_the_mana_value_of_the_adventure() {
    cr!("715.3", "715.3a");
    ruling!(
        "Lozhan, Dragons' Legacy",
        "The mana value of an Adventure spell is the based on the mana cost of the Adventure, not the mana cost of the permanent card that has the Adventure."
    );
    supported("Lozhan, Dragons' Legacy");
    supported("Bonecrusher Giant // Stomp");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lozhan, Dragons' Legacy");
    let giant = t.hand(P0, "Bonecrusher Giant // Stomp");
    assert_eq!(
        mana_value(&t, giant),
        3,
        "the card's: Bonecrusher Giant's {{2}}{{R}}"
    );
    mana(&mut t, P0, "1R");
    t.answer_targets(P0, &[pl(P1)]);
    let stomp = t
        .cast(P0, giant)
        .method(CastMethod::Half(1))
        .target(pl(P1))
        .go();
    assert_eq!(mana_value(&t, stomp), 2, "Stomp's {{1}}{{R}}");
    t.settle();
    assert_eq!(t.stack_len(), 2, "Lozhan triggered: {}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "2 from Lozhan, 2 from Stomp");
}

#[test]
fn a_prototyped_permanent_has_the_prototype_mana_value() {
    cr!("702.160a");
    ruling!(
        "Fateful Handoff",
        "The mana value of an artifact creature that was cast as a prototype is based on the prototype mana cost rather than the card's usual mana cost."
    );
    supported("Fateful Handoff");
    supported("Phyrexian Fleshgorger");
    let mut t = TestGame::new(2);
    let fg = t.hand(P0, "Phyrexian Fleshgorger");
    assert_eq!(mana_value(&t, fg), 7);
    mana(&mut t, P0, "1BB");
    t.cast(P0, fg)
        .method(CastMethod::Keyword(KeywordKind::Prototype))
        .go();
    t.resolve_all();
    let perm = t.g.current(fg);
    assert_eq!(t.zone(perm), Zone::Battlefield);
    assert_eq!(t.pt(perm), (3, 3));
    assert_eq!(mana_value(&t, perm), 3);
    cast_resolve(&mut t, P0, "Fateful Handoff", "3B", &[obj(perm)]);
    assert_eq!(t.hand_size(P0), 3, "drew three: {}", t.dump_log());
    assert_eq!(t.obj_now(perm).controller, P1);
}

#[test]
fn reinterpret_can_cast_either_face_of_a_modal_double_faced_card_with_a_fitting_mana_value() {
    cr!("712.8f", "712.11b", "118.9");
    ruling!(
        "Reinterpret",
        "This can be used to cast either face of a modal double-faced card, but only if the face you are casting has an equal or lesser mana value than the target spell."
    );
    supported("Reinterpret");
    supported("Birgi, God of Storytelling // Harnfel, Horn of Bounty");
    const MDFC: &str = "Birgi, God of Storytelling // Harnfel, Horn of Bounty";
    // P1 casts `theirs`; P0 counters it with Reinterpret and casts the MDFC's face
    // `face` (Birgi {2}{R} = 3, Harnfel {4}{R} = 5) if offered.
    fn counter_with(theirs: &str, cost: &str, face: usize) -> (TestGame, usize, ObjectId) {
        let mut t = TestGame::new(2);
        let mdfc = t.hand(P0, MDFC);
        t.set_step(P1, Step::PrecombatMain);
        let spell = cast(&mut t, P1, theirs, cost, &[]);
        mana(&mut t, P0, "2UR");
        let r = t.hand(P0, "Reinterpret");
        let from = t.asked().len();
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(mdfc)]);
        t.answer(P0, DecisionKind::Option, Answer::Index(face));
        t.cast(P0, r).target(obj(spell)).go();
        t.resolve_all();
        assert!(t.in_graveyard(P1, theirs));
        (t, from, mdfc)
    }
    // Hill Giant (4) countered: Birgi (3) may be cast; Harnfel (5) may not.
    let (t, _, _) = counter_with("Hill Giant", "3R", 0);
    assert_eq!(
        count_named(&t, "Birgi, God of Storytelling"),
        1,
        "{}",
        t.dump_log()
    );
    let (t, from, _) = counter_with("Hill Giant", "3R", 1);
    assert_eq!(
        count_named(&t, "Harnfel, Horn of Bounty"),
        0,
        "{}",
        t.dump_log()
    );
    let offered: Vec<String> = asked_of(&t, P0, from)
        .iter()
        .filter_map(|d| match d {
            Decision::ChooseCastingMethod { options, .. } => Some(options.join("|")),
            _ => None,
        })
        .collect();
    assert!(
        offered.iter().all(|o| !o.contains("Harnfel")),
        "Harnfel offered: {offered:?}"
    );
    // Air Elemental (5) countered: Harnfel may be cast.
    let (t, _, _) = counter_with("Air Elemental", "3UU", 1);
    assert_eq!(
        count_named(&t, "Harnfel, Horn of Bounty"),
        1,
        "{}",
        t.dump_log()
    );
    // Grizzly Bears (2) countered: neither face; the card isn't offered.
    let (t, from, mdfc) = counter_with("Grizzly Bears", "1G", 0);
    assert_eq!(count_named(&t, "Birgi, God of Storytelling"), 0);
    assert_eq!(t.zone(mdfc), Zone::Hand(P0), "{}", t.dump_log());
    assert!(!last_offered_names(&t, P0, from)
        .iter()
        .any(|n| n.contains("Birgi")));
}

#[test]
fn tokens_have_mana_value_0_unless_they_copy_a_card_with_a_mana_cost() {
    cr!("202.3a", "707.2");
    ruling!(
        "Soul Tithe",
        "The mana value of a creature token is 0, unless that token is a copy of another creature, in which case it copies that creature's mana cost."
    );
    ruling!(
        "Ancient Ooze",
        "You add up the mana value of all other creatures you control. Remember that tokens and face-down creatures have a cost of zero."
    );
    supported("Soul Tithe");
    supported("Ancient Ooze");
    supported("Willbender");
    // Soul Tithe on P1's token copy of Hill Giant: P1 pays {4} (four Mountains tapped).
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let copy = token_copy(&mut t, P1, giant)[0];
    t.lands(P1, "Mountain", 4);
    crate::r_s06_common::attach_new(&mut t, P0, "Soul Tithe", copy);
    t.answer_yes(P1, true);
    crate::r_s04_common::next_upkeep(&mut t, P1);
    t.resolve_all();
    assert!(t.on_battlefield(copy), "{}", t.dump_log());
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P1), 4);
    // On a plain Saproling token: {0}, no land tapped.
    let mut t = TestGame::new(2);
    let sap = create_token(&mut t, P1, "Saproling");
    t.lands(P1, "Mountain", 4);
    crate::r_s06_common::attach_new(&mut t, P0, "Soul Tithe", sap);
    t.answer_yes(P1, true);
    crate::r_s04_common::next_upkeep(&mut t, P1);
    t.resolve_all();
    assert!(t.on_battlefield(sap));
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P1), 0);
    // Ancient Ooze with a Hill Giant (4), a Saproling token (0), a token copy of Hill
    // Giant (4) and a face-down Willbender (0): 8/8.
    let mut t = TestGame::new(2);
    let ooze = t.battlefield(P0, "Ancient Ooze");
    let giant = t.battlefield(P0, "Hill Giant");
    create_token(&mut t, P0, "Saproling");
    token_copy(&mut t, P0, giant);
    let wb = t.hand(P0, "Willbender");
    mana(&mut t, P0, "3");
    t.cast(P0, wb)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve_all();
    assert!(t.obj_now(wb).face_down);
    assert_eq!(t.pt(ooze), (8, 8), "{}", t.dump_log());
}

#[test]
fn cost_reductions_dont_change_a_spells_mana_value() {
    cr!("601.2f", "202.3");
    ruling!(
        "Sunderflock",
        "To determine Sunderflock's total cost, start with the mana cost (or an alternative cost if another card's effect allows you to pay one instead), add any cost increases, then apply any cost reductions. Sunderflock's mana value remains unchanged, no matter what the total cost to cast it was."
    );
    ruling!(
        "The Kenriths' Royal Funeral",
        "The last ability doesn't change the mana cost or mana value of any legendary spell you cast, only the total cost you pay."
    );
    supported("Sunderflock");
    supported("The Kenriths' Royal Funeral");
    // Sunderflock ({7}{U}{U}) with an Air Elemental (5): {2}{U}{U} is paid, the mana
    // value is 9 on the stack and on the battlefield (Manaplasm gets +9/+9).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Air Elemental");
    let plasm = t.battlefield(P0, "Manaplasm");
    let flock = cast(&mut t, P0, "Sunderflock", "2UU", &[]);
    assert_eq!(pool(&t, P0), 0, "all of {{2}}{{U}}{{U}} paid");
    assert_eq!(mana_value(&t, flock), 9);
    // Manaplasm's trigger resolves first (then Sunderflock bounces it).
    t.resolve();
    assert_eq!(t.pt(plasm), (10, 10));
    t.resolve_all();
    assert_eq!(mana_value(&t, flock), 9);
    // The Kenriths' Royal Funeral entering exiling two legendary creature cards: Jedit
    // Ojanen ({4}{W}{W}{U}) costs {2}{W}{W}{U}, mana value 7.
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, "Jedit Ojanen");
    let b = t.graveyard(P0, "Mirri, Cat Warrior");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.enter(P0, "The Kenriths' Royal Funeral");
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    let jedit = cast(&mut t, P0, "Jedit Ojanen", "2WWU", &[]);
    assert_eq!(pool(&t, P0), 0);
    assert_eq!(mana_value(&t, jedit), 7);
    assert_eq!(
        t.obj_now(jedit)
            .chars
            .mana_cost
            .as_ref()
            .map(|m| m.mana_value()),
        Some(7)
    );
}
