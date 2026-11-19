//! One instruction's copies of a spell are one copy event (register
//! `copy-for-each-one-copy-event`): Twinning Staff ("If you would copy a spell one or more
//! times, instead copy it that many times plus an additional time. You may choose new
//! targets for the additional copy.") adds its copy once to "for each of those creatures,
//! copy that spell" (Feather, Radiant Arbiter) and to "for each other opponent, ..., copy
//! that spell" (Zevlor, Elturel Exile), as to "copy that spell twice" — N copies become
//! N + 1, not 2N (CR 608.2f, 614.5, 700.1). Copying a card isn't copying a spell (Mizzix's
//! Mastery, "For each card exiled this way, copy it").

use mtg_engine::card::card;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, ObjKind};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

fn spell_copies_on_stack(t: &TestGame) -> usize {
    t.g.stack
        .iter()
        .filter(|c| t.g.obj(**c).kind == ObjKind::SpellCopy)
        .count()
}

/// How many times `p` was asked since `from` whether to choose new targets for a copy.
fn new_target_questions(t: &TestGame, from: usize, p: PlayerId) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(q, d)| {
            *q == p && matches!(d, Decision::YesNo { prompt, .. } if prompt.contains("new targets"))
        })
        .count()
}

/// The sizes of the order questions `p` was asked since `from`.
fn orders_asked(t: &TestGame, from: usize, p: PlayerId) -> Vec<usize> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::Order { items, .. } if *q == p => Some(items.len()),
            _ => None,
        })
        .collect()
}

fn grown(pt: (i32, i32), times: i32) -> (i32, i32) {
    (pt.0 + 3 * times, pt.1 + 3 * times)
}

#[test]
fn feathers_copies_for_each_creature_are_one_copy_event() {
    cr!("608.2f", "614.5", "707.10e");
    interp!("copy-for-each-one-copy-event");
    ruling!(
        "Twinning Staff",
        "If you have a second Twinning Staff, copying a spell results in two additional copies. If you have a third, you get three additional copies, and so on."
    );
    ruling!(
        "Twinning Staff",
        "You may choose new targets for the additional copy, even if you can't choose new targets for the original copy or copies."
    );
    ruling!(
        "Feather, Radiant Arbiter",
        "You control all the copies, and you choose the order the copies are put on the stack. The original spell will be on the stack beneath those copies and will resolve last."
    );
    for name in ["Feather, Radiant Arbiter", "Twinning Staff"] {
        supported(name);
    }
    for staves in 0..3 {
        let mut t = TestGame::new(2);
        for _ in 0..staves {
            t.battlefield(P0, "Twinning Staff");
        }
        let feather = t.battlefield(P0, "Feather, Radiant Arbiter");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P0, "Hill Giant");
        let ogre = t.battlefield(P0, "Gray Ogre");
        let pts: Vec<(i32, i32)> = [feather, bears, giant, ogre]
            .iter()
            .map(|c| t.pt(*c))
            .collect();
        t.lands(P0, "Forest", 5);
        // Giant Growth on Feather; P0 chooses the Bears and the Giant and pays {4}: "for
        // each of those creatures, copy that spell. The copy targets that creature." The
        // first additional copy is pointed at the Ogre; any other keeps Feather. (Answers
        // are taken in order of kind: the spell's target first.)
        t.answer_targets(P0, &[Entity::Object(feather)]);
        t.answer_yes(P0, true); // you may choose creatures
        t.answer_choose(P0, &[Entity::Object(bears), Entity::Object(giant)]);
        t.answer_yes(P0, true); // pay {4}
        t.answer_yes(P0, true); // new targets for the first additional copy
        t.answer_targets(P0, &[Entity::Object(ogre)]);
        let gg = t.hand(P0, "Giant Growth");
        let spell = t.cast(P0, gg).go();
        t.settle();
        let from = t.asked().len();
        // Feather's trigger: one copy event of two copies, so one additional copy for each
        // Staff, not one for each Staff and each creature.
        t.resolve();
        let copies = 2 + staves;
        assert_eq!(
            spell_copies_on_stack(&t),
            copies as usize,
            "{staves} staves"
        );
        assert_eq!(t.g.stack[0], spell, "{staves} staves");
        // Only the additional copies may get new targets (Feather's copies target their
        // creatures, CR 707.10e).
        assert_eq!(
            new_target_questions(&t, from, P0),
            staves as usize,
            "{staves} staves"
        );
        // All of them are put onto the stack at once, in an order P0 chooses (asked once).
        assert_eq!(
            orders_asked(&t, from, P0),
            vec![copies as usize],
            "{staves} staves"
        );
        t.resolve_all();
        // The original and every additional copy but the first grow Feather.
        let feather_growth = 1 + (staves - 1).max(0);
        assert_eq!(
            t.pt(feather),
            grown(pts[0], feather_growth),
            "{staves} staves"
        );
        assert_eq!(t.pt(bears), grown(pts[1], 1), "{staves} staves");
        assert_eq!(t.pt(giant), grown(pts[2], 1), "{staves} staves");
        let ogre_growth = if staves > 0 { 1 } else { 0 };
        assert_eq!(t.pt(ogre), grown(pts[3], ogre_growth), "{staves} staves");
    }
}

#[test]
fn zevlors_copies_for_each_other_opponent_are_one_copy_event() {
    cr!("608.2f", "614.5", "707.10e");
    interp!("copy-for-each-one-copy-event");
    ruling!(
        "Zevlor, Elturel Exile",
        "When the delayed triggered ability of Zevlor, Elturel Exile resolves, it may create one or more copies of a spell. You control each of the copies."
    );
    for name in ["Zevlor, Elturel Exile", "Twinning Staff"] {
        supported(name);
    }
    for staves in 0..2 {
        let mut t = TestGame::new(4);
        for _ in 0..staves {
            t.battlefield(P0, "Twinning Staff");
        }
        let z = t.battlefield(P0, "Zevlor, Elturel Exile");
        t.lands(P0, "Wastes", 2);
        t.activate(P0, z, 0, &[]).unwrap();
        t.resolve_all();
        // Lightning Bolt at P1: "for each other opponent, choose that player or a permanent
        // they control, copy that spell, and the copy targets the chosen player or
        // permanent" — P2 and P3. The additional copy keeps the Bolt's target, P1.
        t.answer_choose(P0, &[Entity::Player(P2)]);
        t.answer_choose(P0, &[Entity::Player(P3)]);
        t.lands(P0, "Mountain", 1);
        let bolt = t.hand(P0, "Lightning Bolt");
        t.cast(P0, bolt).target(P1).go();
        t.settle();
        let from = t.asked().len();
        t.resolve(); // Zevlor's delayed trigger
        let copies = 2 + staves;
        assert_eq!(
            spell_copies_on_stack(&t),
            copies as usize,
            "{staves} staves"
        );
        // "You control each of the copies": Zevlor's and Twinning Staff's alike.
        for c in t.g.stack[1..].iter() {
            assert_eq!(t.g.obj(*c).kind, ObjKind::SpellCopy, "{staves} staves");
            assert_eq!(t.g.obj(*c).controller, P0, "{staves} staves");
        }
        assert_eq!(
            new_target_questions(&t, from, P0),
            staves as usize,
            "{staves} staves"
        );
        assert_eq!(
            orders_asked(&t, from, P0),
            vec![copies as usize],
            "{staves} staves"
        );
        t.resolve_all();
        assert_eq!(t.life(P1), 17 - 3 * staves, "{staves} staves");
        assert_eq!((t.life(P2), t.life(P3)), (17, 17), "{staves} staves");
    }
}

#[test]
fn separate_copy_instructions_are_separate_copy_events() {
    // The entry's other half: copies that separate instructions make are separate copy
    // events, each given Twinning Staff's copy (CR 614.5: one opportunity per event). Two
    // Reverberates ("Copy target instant or sorcery spell. You may choose new targets for
    // the copy.") on one Lightning Bolt make two copies and two additional ones, also right
    // after a "for each" copy event (Feather's) has ended: none is left open to fold
    // later copies into.
    cr!("614.5", "707.10");
    interp!("copy-for-each-one-copy-event");
    for name in ["Feather, Radiant Arbiter", "Twinning Staff", "Reverberate"] {
        supported(name);
    }
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Twinning Staff");
    let feather = t.battlefield(P0, "Feather, Radiant Arbiter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let (feather_pt, bears_pt) = (t.pt(feather), t.pt(bears));
    t.lands(P0, "Forest", 3);
    // Giant Growth on Feather, the Bears chosen ({2}): Feather's copy at the Bears and
    // Twinning Staff's, which keeps Feather as its target.
    t.answer_targets(P0, &[Entity::Object(feather)]);
    t.answer_yes(P0, true); // you may choose creatures
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true); // pay {2}
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).go();
    t.settle();
    t.resolve(); // Feather's trigger
    assert_eq!(spell_copies_on_stack(&t), 2);
    t.resolve_all();
    assert_eq!(t.pt(feather), grown(feather_pt, 2));
    assert_eq!(t.pt(bears), grown(bears_pt, 1));
    // Lightning Bolt at P1, then two Reverberates on it.
    t.lands(P0, "Mountain", 5);
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(P1).go();
    for _ in 0..2 {
        let rev = t.hand(P0, "Reverberate");
        t.cast(P0, rev).target(spell).go();
    }
    t.settle();
    t.resolve(); // the second Reverberate: its copy and Twinning Staff's
    assert_eq!(spell_copies_on_stack(&t), 2);
    t.resolve_all();
    // Five Bolts: the original, and two from each Reverberate.
    assert_eq!(t.life(P1), 20 - 3 * 5);
}

#[test]
fn copying_a_spell_twice_is_one_copy_event() {
    // "Copy that spell or ability twice" (Repeated Reverberation): Twinning Staff applies
    // once to the copying, not to each copy.
    cr!("614.5", "707.10");
    interp!("copy-for-each-one-copy-event");
    for name in ["Repeated Reverberation", "Twinning Staff"] {
        supported(name);
    }
    for staves in 0..3 {
        let mut t = TestGame::new(2);
        for _ in 0..staves {
            t.battlefield(P0, "Twinning Staff");
        }
        t.lands(P0, "Mountain", 5);
        let rr = t.hand(P0, "Repeated Reverberation");
        t.cast(P0, rr).go();
        t.resolve_all();
        let bolt = t.hand(P0, "Lightning Bolt");
        t.cast(P0, bolt).target(P1).go();
        t.settle();
        t.resolve(); // the delayed trigger
        assert_eq!(
            spell_copies_on_stack(&t),
            2 + staves as usize,
            "{staves} staves"
        );
        t.resolve_all();
        assert_eq!(t.life(P1), 20 - 3 * (3 + staves), "{staves} staves");
    }
}

#[test]
fn mizzixs_mastery_copies_cards_so_twinning_staff_adds_none() {
    // "For each card exiled this way, copy it, and you may cast the copy": a copy of a card
    // (CR 707.12), cast, is not a copied spell.
    cr!("707.12");
    ruling!(
        "Twinning Staff",
        "If an effect copies a card rather than a spell (such as that of God-Eternal Kefnet), this doesn't cause an additional copy to be created."
    );
    for name in ["Mizzix's Mastery", "Twinning Staff"] {
        supported(name);
    }
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Twinning Staff");
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P0, "Shock");
    t.lands(P0, "Mountain", 8);
    let m = t.hand(P0, "Mizzix's Mastery");
    t.cast(P0, m)
        .method(CastMethod::Keyword(KeywordKind::Overload))
        .go();
    for _ in 0..2 {
        t.answer_yes(P0, true); // cast the copy
        t.answer_targets(P0, &[Entity::Player(P1)]);
    }
    t.resolve();
    // Two copies of cards, both cast; no copy of a spell.
    assert_eq!(t.stack_len(), 2);
    assert_eq!(spell_copies_on_stack(&t), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
}
