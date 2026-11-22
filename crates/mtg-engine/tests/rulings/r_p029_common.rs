//! Shared helpers for rulings batch P029 (`r_p029_*.rs`): copies of instant and sorcery
//! spells and of abilities on the stack (CR 707.10) made by many different cards. The
//! copying cards of batch P028 (`r_p028_common`) are reused; the ones below are new.

#![allow(dead_code)]

use crate::r_p028_common::{nothing, obj, Copier};
use crate::r_p030_common::resolve_until_copies;
use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::*;
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// "Copy target instant or sorcery spell. You may choose new targets for the copy."
pub const TWINCAST: Copier = Copier {
    name: "Twincast",
    setup: nothing,
    start: |t, s| {
        cast_new(t, P0, "Twincast", &[obj(s)]);
        keep_copy_targets(t, P0);
    },
};

/// "Copy target instant or sorcery spell, except that the copy is red. You may choose new
/// targets for the copy."
pub const FORK: Copier = Copier {
    name: "Fork",
    setup: nothing,
    start: |t, s| {
        cast_new(t, P0, "Fork", &[obj(s)]);
        keep_copy_targets(t, P0);
    },
};

/// "Buyback {3}. Copy target instant or sorcery spell. You may choose new targets for the
/// copy." (Cast without buyback.)
pub const REITERATE: Copier = Copier {
    name: "Reiterate",
    setup: nothing,
    start: |t, s| {
        t.answer(
            P0,
            DecisionKind::OptionalCost,
            mtg_engine::decision::Answer::Bool(false),
        );
        cast_new(t, P0, "Reiterate", &[obj(s)]);
        keep_copy_targets(t, P0);
    },
};

/// "Flash. When Lutri enters, if you cast it, copy target instant or sorcery spell you
/// control. You may choose new targets for the copy."
pub const LUTRI: Copier = Copier {
    name: "Lutri, the Spellchaser",
    setup: nothing,
    start: |t, s| {
        cast_new(t, P0, "Lutri, the Spellchaser", &[]);
        t.answer_targets(P0, &[obj(s)]);
        keep_copy_targets(t, P0);
    },
};

/// "{X}{R}, {T}: Copy target instant or sorcery spell you control with mana value X. You
/// may choose new targets for the copy."
pub const LEAGUE_GUILDMAGE: Copier = Copier {
    name: "League Guildmage",
    setup: |t| {
        t.battlefield(P0, "League Guildmage");
    },
    start: |t, s| {
        let x = t.g.mana_value_of(s) as usize;
        t.lands(P0, "Mountain", 1);
        t.lands(P0, "Wastes", x);
        let g = t.named_on_battlefield("League Guildmage")[0];
        t.answer(
            P0,
            DecisionKind::X,
            mtg_engine::decision::Answer::Number(x as i64),
        );
        t.answer_targets(P0, &[obj(s)]);
        activate_containing(t, P0, g, "Copy target").unwrap();
        keep_copy_targets(t, P0);
    },
};

/// "If you would copy a spell one or more times, instead copy it that many times plus an
/// additional time. You may choose new targets for the additional copy. {7}, {T}: Copy
/// target instant or sorcery spell you control. You may choose new targets for the copy."
/// (Its own replacement effect applies to its ability: two copies.)
pub const TWINNING_STAFF: Copier = Copier {
    name: "Twinning Staff",
    setup: |t| {
        t.battlefield(P0, "Twinning Staff");
    },
    start: |t, s| {
        t.lands(P0, "Wastes", 7);
        let staff = t.named_on_battlefield("Twinning Staff")[0];
        t.answer_targets(P0, &[obj(s)]);
        activate_containing(t, P0, staff, "Copy target").unwrap();
        keep_copy_targets(t, P0);
        keep_copy_targets(t, P0);
    },
};

/// "Whenever you cast an instant or sorcery spell, you may sacrifice an artifact. If you
/// do, copy that spell and you may choose new targets for the copy." (An Ornithopter is
/// sacrificed.)
pub const MICA: Copier = Copier {
    name: "Mica, Reader of Ruins",
    setup: |t| {
        t.battlefield(P0, "Mica, Reader of Ruins");
        t.battlefield(P0, "Ornithopter");
    },
    start: |t, _| {
        let thopter = t.named_on_battlefield("Ornithopter")[0];
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(thopter)]);
        keep_copy_targets(t, P0);
    },
};

/// "Remove three charge counters from this artifact: When you next cast an instant or
/// sorcery spell this turn, copy it and you may choose new targets for the copy."
pub const ADAPTIVE_TRAINING_POST: Copier = Copier {
    name: "Adaptive Training Post",
    setup: |t| {
        let post = t.battlefield(P0, "Adaptive Training Post");
        t.g.add_counters(obj(post), "charge", 3, None);
        activate_containing(t, P0, post, "Remove three").unwrap();
        t.resolve_all();
    },
    start: keep_start,
};

/// Gandalf the Grey's third mode: "Copy target instant or sorcery spell you control. You
/// may choose new targets for the copy."
pub const GANDALF: Copier = Copier {
    name: "Gandalf the Grey",
    setup: |t| {
        t.battlefield(P0, "Gandalf the Grey");
    },
    start: |t, s| {
        t.answer(
            P0,
            DecisionKind::Modes,
            mtg_engine::decision::Answer::Indices(vec![2]),
        );
        t.answer_targets(P0, &[obj(s)]);
        keep_copy_targets(t, P0);
    },
};

/// "Tempting offer — Choose target instant or sorcery spell. Each opponent may copy that
/// spell ... You copy that spell once plus an additional time for each opponent who copied
/// the spell this way." (P1 declines: one copy, P0's.)
pub const TEMPT_WITH_MAYHEM: Copier = Copier {
    name: "Tempt with Mayhem",
    setup: nothing,
    start: |t, s| {
        t.answer_yes(P1, false);
        cast_new(t, P0, "Tempt with Mayhem", &[obj(s)]);
        keep_copy_targets(t, P0);
    },
};

/// "{2}, Return two lands you control to their owner's hand: Copy target instant or
/// sorcery spell. You may choose new targets for the copy."
pub const UYO: Copier = Copier {
    name: "Uyo, Silent Prophet",
    setup: |t| {
        t.battlefield(P0, "Uyo, Silent Prophet");
    },
    start: |t, s| {
        let lands = t.lands(P0, "Wastes", 4);
        let uyo = t.named_on_battlefield("Uyo, Silent Prophet")[0];
        t.answer_choose(P0, &[obj(lands[2]), obj(lands[3])]);
        t.answer_targets(P0, &[obj(s)]);
        activate_containing(t, P0, uyo, "Copy target").unwrap();
        keep_copy_targets(t, P0);
    },
};

/// "{1}{U/R}{U/R}: Copy target instant or sorcery spell that targets you. You may choose
/// new targets for the copy." (Mirror Sheen is set up for P0.)
pub const MIRROR_SHEEN: Copier = Copier {
    name: "Mirror Sheen",
    setup: |t| {
        t.battlefield(P0, "Mirror Sheen");
    },
    start: |t, s| {
        t.lands(P0, "Wastes", 1);
        t.lands(P0, "Island", 2);
        let sheen = t.named_on_battlefield("Mirror Sheen")[0];
        t.answer_targets(P0, &[obj(s)]);
        activate_containing(t, P0, sheen, "Copy target").unwrap();
        keep_copy_targets(t, P0);
    },
};

pub fn keep_start(t: &mut TestGame, _: ObjectId) {
    keep_copy_targets(t, P0);
}

/// The copies of spells ever put on the stack while it resolves completely, in the order
/// they appeared.
pub fn resolve_collecting_copies(t: &mut TestGame) -> Vec<ObjectId> {
    t.settle();
    let mut seen = vec![];
    for _ in 0..200 {
        for c in spell_copies(t) {
            if !seen.contains(&c) {
                seen.push(c);
            }
        }
        if t.g.stack.is_empty() {
            break;
        }
        t.resolve();
    }
    seen
}

/// `caster` casts Divination ("Draw two cards."); `c` copies it (`copies` times); the
/// copies are `controller`'s and draw that player two cards each.
pub fn copies_divination(c: &Copier, caster: PlayerId, controller: PlayerId, copies: usize) {
    supported(c.name);
    let mut t = TestGame::new(2);
    if caster != P0 {
        t.set_step(caster, Step::PrecombatMain);
    }
    (c.setup)(&mut t);
    let div = cast_new(&mut t, caster, "Divination", &[]);
    (c.start)(&mut t, div);
    let made = resolve_until_copies(&mut t);
    assert_eq!(made.len(), copies, "{}", c.name);
    for m in &made {
        assert_eq!(t.obj(*m).controller, controller, "{}", c.name);
    }
    let h = [t.hand_size(P0), t.hand_size(P1)];
    t.resolve_all();
    for p in [P0, P1] {
        let want = h[p.idx()]
            + if p == caster { 2 } else { 0 }
            + if p == controller { 2 * copies } else { 0 };
        assert_eq!(t.hand_size(p), want, "{}: {p:?}'s hand", c.name);
    }
}

/// The source of the ability `id` on the stack.
pub fn ability_source(t: &TestGame, id: ObjectId) -> Option<ObjectId> {
    match t.g.obj(id).stack.as_deref().map(|s| &s.kind) {
        Some(StackKind::Activated { source, .. } | StackKind::Triggered { source, .. }) => {
            Some(*source)
        }
        _ => None,
    }
}
