//! Shared helpers for rulings batch P028 (`r_p028_*.rs`): copies of instant and sorcery
//! spells (CR 707.10) made by many different cards.

#![allow(dead_code)]

use crate::r_s01_common::attack_with;
use crate::r_s06_common::activate_containing;
use crate::r_s19_common::add_lore;
use crate::r_s25_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// What sets up the copying card before the original spell is cast, and what (after the
/// original is cast) puts the copy effect on the stack and queues P0's answers for it.
pub struct Copier {
    pub name: &'static str,
    pub setup: fn(&mut TestGame),
    pub start: fn(&mut TestGame, ObjectId),
}

// --- The copying cards -------------------------------------------------------------------

pub fn nothing(_: &mut TestGame) {}

pub fn keep(t: &mut TestGame, _: ObjectId) {
    keep_copy_targets(t, P0);
}

/// "Copy target instant or sorcery spell. You may choose new targets for the copy."
pub const REVERBERATE: Copier = Copier {
    name: "Reverberate",
    setup: nothing,
    start: |t, s| {
        cast_new(t, P0, "Reverberate", &[obj(s)]);
        keep_copy_targets(t, P0);
    },
};

/// "You may choose new targets for target instant or sorcery spell. Then copy that spell.
/// You may choose new targets for the copy."
pub const WILD_RICOCHET: Copier = Copier {
    name: "Wild Ricochet",
    setup: nothing,
    start: |t, s| {
        cast_new(t, P0, "Wild Ricochet", &[obj(s)]);
        t.answer_yes(P0, false);
        t.answer_yes(P0, false);
    },
};

/// "• Copy target instant or sorcery spell. You may choose new targets for the copy."
pub const INSIDIOUS_WILL: Copier = Copier {
    name: "Insidious Will",
    setup: nothing,
    start: |t, s| {
        lands_for_cost(t, P0, "Insidious Will");
        let card = t.hand(P0, "Insidious Will");
        t.cast(P0, card).modes(&[2]).target(s).go();
        keep_copy_targets(t, P0);
    },
};

/// "Copy any number of target instant and/or sorcery spells."
pub const DISPLAY_OF_POWER: Copier = Copier {
    name: "Display of Power",
    setup: nothing,
    start: |t, s| {
        cast_new(t, P0, "Display of Power", &[obj(s)]);
        keep_copy_targets(t, P0);
    },
};

/// "When this creature enters, copy target instant or sorcery spell."
pub const DUALCASTER_MAGE: Copier = Copier {
    name: "Dualcaster Mage",
    setup: nothing,
    start: |t, s| {
        cast_new(t, P0, "Dualcaster Mage", &[]);
        t.answer_targets(P0, &[obj(s)]);
        keep_copy_targets(t, P0);
    },
};

/// "When Naru Meha enters, copy target instant or sorcery spell you control."
pub const NARU_MEHA: Copier = Copier {
    name: "Naru Meha, Master Wizard",
    setup: nothing,
    start: |t, s| {
        cast_new(t, P0, "Naru Meha, Master Wizard", &[]);
        t.answer_targets(P0, &[obj(s)]);
        keep_copy_targets(t, P0);
    },
};

/// Level 2: "{U}{U}, {T}: Copy target instant or sorcery spell."
pub const ECHO_MAGE: Copier = Copier {
    name: "Echo Mage",
    setup: |t| {
        let mage = t.battlefield(P0, "Echo Mage");
        t.g.add_counters(obj(mage), "level", 2, None);
        t.g.recompute();
    },
    start: |t, s| {
        t.lands(P0, "Island", 2);
        let mage = t.named_on_battlefield("Echo Mage")[0];
        t.answer_targets(P0, &[obj(s)]);
        activate_containing(t, P0, mage, "Copy target").unwrap();
        keep_copy_targets(t, P0);
    },
};

/// "{2}{U}, {T}: The controller of target instant or sorcery spell copies it."
pub const MELETIS_CHARLATAN: Copier = Copier {
    name: "Meletis Charlatan",
    setup: |t| {
        t.battlefield(P0, "Meletis Charlatan");
    },
    start: |t, s| {
        t.lands(P0, "Island", 1);
        t.lands(P0, "Wastes", 2);
        let c = t.named_on_battlefield("Meletis Charlatan")[0];
        t.answer_targets(P0, &[obj(s)]);
        activate_containing(t, P0, c, "copies it").unwrap();
        keep_copy_targets(t, P0);
    },
};

/// "{3}, {T}: Copy target instant or sorcery spell you control."
pub const LITHOFORM_ENGINE: Copier = Copier {
    name: "Lithoform Engine",
    setup: |t| {
        t.battlefield(P0, "Lithoform Engine");
    },
    start: |t, s| {
        t.lands(P0, "Wastes", 3);
        let e = t.named_on_battlefield("Lithoform Engine")[0];
        t.answer_targets(P0, &[obj(s)]);
        activate_containing(t, P0, e, "instant or sorcery spell").unwrap();
        keep_copy_targets(t, P0);
    },
};

/// "Whenever you cast your first instant or sorcery spell each turn, copy that spell."
pub const DOUBLE_VISION: Copier = Copier {
    name: "Double Vision",
    setup: |t| {
        t.battlefield(P0, "Double Vision");
    },
    start: keep,
};

/// "Whenever you cast an instant or sorcery spell, you may pay {3}. If you do, copy that
/// spell."
pub const MIRARI: Copier = Copier {
    name: "Mirari",
    setup: |t| {
        t.battlefield(P0, "Mirari");
        t.lands(P0, "Wastes", 3);
    },
    start: |t, _| {
        t.answer_yes(P0, true);
        keep_copy_targets(t, P0);
    },
};

/// "Whenever you cast an instant or sorcery spell, you may copy that spell."
pub const SWARM_INTELLIGENCE: Copier = Copier {
    name: "Swarm Intelligence",
    setup: |t| {
        t.battlefield(P0, "Swarm Intelligence");
    },
    start: |t, _| {
        t.answer_yes(P0, true);
        keep_copy_targets(t, P0);
    },
};

/// III — "Until end of turn, whenever you cast an instant or sorcery spell, copy it."
pub const MIRARI_CONJECTURE: Copier = Copier {
    name: "The Mirari Conjecture",
    setup: |t| {
        let saga = t.battlefield(P0, "The Mirari Conjecture");
        add_lore(t, saga, 3);
        t.resolve_all();
    },
    start: keep,
};

/// "Whenever you cast your first instant spell each turn, if Kalamax is tapped, copy that
/// spell."
pub const KALAMAX: Copier = Copier {
    name: "Kalamax, the Stormsire",
    setup: |t| {
        let k = t.battlefield(P0, "Kalamax, the Stormsire");
        t.g.tap(k);
    },
    start: keep,
};

/// "Whenever you cast an artifact, instant, or sorcery spell, copy that spell. ... This
/// ability triggers only once each turn."
pub const JIN_GITAXIAS: Copier = Copier {
    name: "Jin-Gitaxias, Progress Tyrant",
    setup: |t| {
        t.battlefield(P0, "Jin-Gitaxias, Progress Tyrant");
    },
    start: keep,
};

/// "Whenever you cast a multicolored instant or sorcery spell, you may pay {1}. If you do,
/// copy that spell."
pub const CLOVEN_CASTING: Copier = Copier {
    name: "Cloven Casting",
    setup: |t| {
        t.battlefield(P0, "Cloven Casting");
        t.lands(P0, "Wastes", 1);
    },
    start: |t, _| {
        t.answer_yes(P0, true);
        keep_copy_targets(t, P0);
    },
};

/// "Whenever you cast an instant or sorcery spell, copy it for each other instant and
/// sorcery spell you've cast before it this turn." (One spell, Opt, was cast before.)
pub const THOUSAND_YEAR_STORM: Copier = Copier {
    name: "Thousand-Year Storm",
    setup: |t| {
        t.battlefield(P0, "Thousand-Year Storm");
        cast_new(t, P0, "Opt", &[]);
        t.resolve_all();
    },
    start: keep,
};

/// "Until end of turn, whenever a player casts an instant or sorcery spell, that player
/// copies it and may choose new targets for the copy."
pub const BONUS_ROUND: Copier = Copier {
    name: "Bonus Round",
    setup: |t| {
        cast_new(t, P0, "Bonus Round", &[]);
        t.resolve_all();
    },
    start: keep,
};

/// "−2: When you next cast an instant or sorcery spell this turn, copy that spell."
pub const CHANDRA: Copier = Copier {
    name: "Chandra, the Firebrand",
    setup: |t| {
        let c = t.battlefield(P0, "Chandra, the Firebrand");
        activate_containing(t, P0, c, "When you next cast").unwrap();
        t.resolve_all();
    },
    start: keep,
};

/// "When you next cast an instant or sorcery spell this turn, copy that spell."
pub const HOWL_OF_THE_HORDE: Copier = Copier {
    name: "Howl of the Horde",
    setup: |t| {
        cast_new(t, P0, "Howl of the Horde", &[]);
        t.resolve_all();
    },
    start: keep,
};

/// "When you next cast an instant or sorcery spell this turn, copy that spell X times."
/// (X = 1.)
pub const STORM_KINGS_THUNDER: Copier = Copier {
    name: "Storm King's Thunder",
    setup: |t| {
        t.lands(P0, "Mountain", 3);
        t.lands(P0, "Wastes", 1);
        let c = t.hand(P0, "Storm King's Thunder");
        t.cast(P0, c).x(1).go();
        t.resolve_all();
    },
    start: keep,
};

/// "• When you cast your next instant or sorcery spell this turn, copy that spell."
pub const TWINFERNO: Copier = Copier {
    name: "Twinferno",
    setup: |t| {
        lands_for_cost(t, P0, "Twinferno");
        let c = t.hand(P0, "Twinferno");
        t.cast(P0, c).modes(&[0]).go();
        t.resolve_all();
    },
    start: keep,
};

/// "Whenever Najal attacks, you may pay {2}. If you do, when you next cast an instant or
/// sorcery spell this turn, copy it." (The original is cast in the declare attackers
/// step.)
pub const NAJAL: Copier = Copier {
    name: "Najal, the Storm Runner",
    setup: |t| {
        let n = t.battlefield(P0, "Najal, the Storm Runner");
        t.lands(P0, "Wastes", 2);
        t.answer_yes(P0, true);
        attack_with(t, &[(n, Entity::Player(P1))]);
        t.resolve_all();
    },
    start: keep,
};


/// "When you next cast an instant or sorcery spell this turn, copy that spell."
pub const GALVANIC_ITERATION: Copier = Copier {
    name: "Galvanic Iteration",
    setup: |t| {
        cast_new(t, P0, "Galvanic Iteration", &[]);
        t.resolve_all();
    },
    start: keep,
};

/// "When you next cast an instant or sorcery spell this turn, copy that spell."
pub const DOUBLECAST: Copier = Copier {
    name: "Doublecast",
    setup: |t| {
        cast_new(t, P0, "Doublecast", &[]);
        t.resolve_all();
    },
    start: keep,
};

/// Level 4+: "{U}{U}, {T}: Copy target instant or sorcery spell twice."
pub const ECHO_MAGE_4: Copier = Copier {
    name: "Echo Mage",
    setup: |t| {
        let mage = t.battlefield(P0, "Echo Mage");
        t.g.add_counters(obj(mage), "level", 4, None);
        t.g.recompute();
    },
    start: |t, s| {
        t.lands(P0, "Island", 2);
        let mage = t.named_on_battlefield("Echo Mage")[0];
        t.answer_targets(P0, &[obj(s)]);
        activate_containing(t, P0, mage, "Copy target").unwrap();
    },
};

/// Spree "+ {1} — Copy target instant spell, sorcery spell, activated ability, or
/// triggered ability. You may choose new targets for the copy."
pub const RETURN_THE_FAVOR: Copier = Copier {
    name: "Return the Favor",
    setup: nothing,
    start: |t, s| {
        t.lands(P0, "Mountain", 2);
        t.lands(P0, "Wastes", 1);
        let card = t.hand(P0, "Return the Favor");
        t.cast(P0, card).modes(&[0]).target(s).go();
        keep_copy_targets(t, P0);
    },
};
