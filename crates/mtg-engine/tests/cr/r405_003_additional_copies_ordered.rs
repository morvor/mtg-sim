//! CR 405.3: a copy an effect makes together with the copies another effect adds to it
//! (Twinning Staff: "If you would copy a spell one or more times, instead copy it that many
//! times plus an additional time. You may choose new targets for the additional copy.") are
//! put onto the stack at the same time, in the order their controller chooses — for the
//! copies keyword abilities make in code too (demonstrate here; epic's upkeep copy in
//! `tests/keywords/k702_050_epic.rs`; casualty's copy with exceptions shares the same
//! ordering) — and the controller is asked once for all of them, also when the copies
//! are made for each of several objects (Feather, Radiant Arbiter; CR 707.10d's "for each
//! ... it could target").

use mtg_engine::card::card;
use mtg_engine::decision::Decision;
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

/// The items of each order question `p` was asked since `from`.
fn orders_asked(t: &TestGame, from: usize, p: PlayerId) -> Vec<Vec<String>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::Order { items, .. } if *q == p => Some(items.clone()),
            _ => None,
        })
        .collect()
}

/// The targets of a spell on the stack.
fn targets_of(t: &TestGame, id: ObjectId) -> Vec<Entity> {
    t.g.obj(id)
        .stack
        .as_deref()
        .map(|si| {
            si.chosen
                .iter()
                .flat_map(|m| m.targets.iter().flatten().copied())
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn demonstrate_copy_and_twinning_staffs_copy_are_ordered_by_their_controller() {
    cr!("405.3", "702.144a", "707.10c");
    for name in ["Excavation Technique", "Twinning Staff"] {
        supported(name);
    }
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Twinning Staff");
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Hill Giant");
        let c = t.battlefield(P1, "Gray Ogre");
        // Excavation Technique ("Destroy target nonland permanent. Its controller creates
        // two Treasure tokens.") at the Bears. P0 copies it (demonstrate) at the Giant, and
        // Twinning Staff's additional copy at the Ogre; P1's copy keeps the Bears.
        t.lands(P0, "Plains", 4);
        let tech = t.hand(P0, "Excavation Technique");
        // (Target answers are taken in order: the spell's first.)
        t.answer_targets(P0, &[Entity::Object(a)]);
        t.answer_yes(P0, true); // demonstrate: copy it
        t.answer_yes(P0, true); // new targets for the copy
        t.answer_targets(P0, &[Entity::Object(b)]);
        t.answer_yes(P0, true); // new targets for the additional copy
        t.answer_targets(P0, &[Entity::Object(c)]);
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.clone()));
        t.answer_yes(P1, false); // P1 keeps the Bears as its copy's target
        let spell = t.cast(P0, tech).go();
        let from = t.asked().len();
        t.resolve(); // the demonstrate trigger
        let orders = t.asked()[from..]
            .iter()
            .filter(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. }))
            .count();
        assert_eq!(orders, 1, "{order:?}");
        // The original, P0's two copies, P1's copy on top.
        assert_eq!(t.stack_len(), 4, "{order:?}");
        assert_eq!(t.g.stack[0], spell, "{order:?}");
        assert_eq!(t.obj(t.g.stack[3]).controller, P1, "{order:?}");
        // The copy listed first is put onto the stack first, so the other resolves first.
        let (first, second) = if order[0] == 0 { (b, c) } else { (c, b) };
        t.resolve(); // P1's copy: the Bears
        assert!(!t.on_battlefield(a), "{order:?}");
        t.resolve();
        assert!(!t.on_battlefield(second), "{order:?}: the upper copy resolves first");
        assert!(t.on_battlefield(first), "{order:?}");
        t.resolve();
        assert!(!t.on_battlefield(first), "{order:?}");
    }
}

#[test]
fn feathers_copies_and_twinning_staffs_copies_are_ordered_once() {
    // Feather, Radiant Arbiter: "If you do, for each of those creatures, copy that spell.
    // The copy targets that creature." With Twinning Staff, its copy comes with an
    // additional one; all of them are put onto the stack at the same time, and their
    // controller is asked for their order once (not also for each copy and its additional
    // one, a question the last one overrode).
    cr!("405.3", "707.10c", "707.10e");
    for name in ["Feather, Radiant Arbiter", "Twinning Staff"] {
        supported(name);
    }
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Twinning Staff");
        let feather = t.battlefield(P0, "Feather, Radiant Arbiter");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P0, "Hill Giant");
        let (bears_pt, giant_pt) = (t.pt(bears), t.pt(giant));
        t.lands(P0, "Forest", 3);
        // Giant Growth on Feather; P0 chooses the Bears and pays {2}: a copy at the Bears,
        // and Twinning Staff's additional copy, which P0 points at the Hill Giant. (Target
        // answers are taken in order: the spell's first.)
        t.answer_targets(P0, &[Entity::Object(feather)]);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(bears)]);
        t.answer_yes(P0, true);
        t.answer_yes(P0, true); // new targets for the additional copy
        t.answer_targets(P0, &[Entity::Object(giant)]);
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.clone()));
        let gg = t.hand(P0, "Giant Growth");
        let spell = t.cast(P0, gg).go();
        t.settle();
        let from = t.asked().len();
        t.resolve(); // Feather's trigger
        let asked = orders_asked(&t, from, P0);
        assert_eq!(asked.len(), 1, "{order:?}: one question, {asked:?}");
        assert_eq!(asked[0].len(), 2, "{order:?}");
        assert_eq!(t.stack_len(), 3, "{order:?}");
        assert_eq!(t.g.stack[0], spell, "{order:?}");
        // The copy listed first (the Bears') is put onto the stack first, so the other one
        // resolves first.
        t.resolve();
        if order[0] == 0 {
            assert_eq!(t.pt(giant), (giant_pt.0 + 3, giant_pt.1 + 3), "{order:?}");
            assert_eq!(t.pt(bears), bears_pt, "{order:?}");
        } else {
            assert_eq!(t.pt(bears), (bears_pt.0 + 3, bears_pt.1 + 3), "{order:?}");
            assert_eq!(t.pt(giant), giant_pt, "{order:?}");
        }
        t.resolve_all();
        assert_eq!(t.pt(bears), (bears_pt.0 + 3, bears_pt.1 + 3), "{order:?}");
        assert_eq!(t.pt(giant), (giant_pt.0 + 3, giant_pt.1 + 3), "{order:?}");
    }
}

#[test]
fn copies_for_each_creature_it_could_target_and_twinning_staffs_copy_are_ordered_once() {
    // CR 707.10d: "copy the spell for each other creature that spell could target"
    // (Ink-Treader Nephilim) puts the copies onto the stack in the order their controller
    // chooses; Twinning Staff's additional copy goes there with them, anywhere among them,
    // and the controller is asked once for all of them (not for the copies and then again
    // for all of them).
    cr!("405.3", "707.10d");
    ruling!(
        "Twinning Staff",
        "You choose whether the additional copy is put onto the stack before or after the original copy. If a spell is being copied more than once, you can even have the additional copy put onto the stack in between the original copies."
    );
    for name in ["Ink-Treader Nephilim", "Twinning Staff"] {
        supported(name);
    }
    for order in [vec![0, 2, 1], vec![2, 1, 0], vec![1, 0, 2]] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Twinning Staff");
        let ink = t.battlefield(P0, "Ink-Treader Nephilim");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        let pts: Vec<(i32, i32)> = [ink, bears, giant].iter().map(|c| t.pt(*c)).collect();
        t.lands(P0, "Forest", 1);
        // Giant Growth on the Nephilim: a copy for the Bears and one for the Hill Giant,
        // and Twinning Staff's additional copy, which keeps the original's target.
        t.answer_yes(P0, false);
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.clone()));
        let gg = t.hand(P0, "Giant Growth");
        let spell = t.cast(P0, gg).target(ink).go();
        t.settle();
        let from = t.asked().len();
        t.resolve(); // the Nephilim's trigger
        let asked = orders_asked(&t, from, P0);
        assert_eq!(asked.len(), 1, "{order:?}: one question, {asked:?}");
        let items = &asked[0];
        assert_eq!(items.len(), 3, "{order:?}");
        assert_eq!(t.stack_len(), 4, "{order:?}");
        assert_eq!(t.g.stack[0], spell, "{order:?}");
        // Bottom first, the copies are the items in the order answered: the first listed
        // is put onto the stack first. The additional copy, made last (item 2), is the one
        // at the Nephilim, wherever the order put it.
        let copies: Vec<ObjectId> = t.g.stack[1..].to_vec();
        for (c, i) in copies.iter().zip(&order) {
            assert_eq!(t.obj(*c).controller, P0, "{order:?}");
            let target = format!("{:?}", targets_of(&t, *c)[0]);
            assert!(items[*i].contains(&target), "{order:?}: {} vs {target}", items[*i]);
        }
        let extra = order.iter().position(|i| *i == 2).unwrap();
        assert_eq!(targets_of(&t, copies[extra]), vec![Entity::Object(ink)], "{order:?}");
        // The top one resolves first.
        let top = targets_of(&t, copies[2])[0];
        t.resolve();
        for (c, pt) in [ink, bears, giant].iter().zip(&pts) {
            let grown = Entity::Object(*c) == top;
            let want = if grown { (pt.0 + 3, pt.1 + 3) } else { *pt };
            assert_eq!(t.pt(*c), want, "{order:?}");
        }
        t.resolve_all();
        // The original and the additional copy on the Nephilim, a copy on each other one.
        assert_eq!(t.pt(ink), (pts[0].0 + 6, pts[0].1 + 6), "{order:?}");
        assert_eq!(t.pt(bears), (pts[1].0 + 3, pts[1].1 + 3), "{order:?}");
        assert_eq!(t.pt(giant), (pts[2].0 + 3, pts[2].1 + 3), "{order:?}");
    }
}
