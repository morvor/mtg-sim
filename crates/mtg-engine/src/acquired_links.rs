//! Links of the abilities continuously reevaluated effects give (CR 607.5): "If an object
//! acquires a pair of linked abilities as part of the same effect, the abilities will be
//! similarly linked to one another on that object even though they weren't printed on that
//! object. They can't be linked to any other ability, regardless of what other abilities
//! the object may currently have or may have had in the past."
//!
//! A static ability's effect (or another continuously reevaluated one) gives its abilities
//! anew each time characteristics are computed, so the links it gives must be the same
//! each time. They're allocated per object that acquires them: the first time an object
//! acquires ability link `l` from a given source (and, for "has all activated abilities
//! of", a given granter), it gets the object's next link from [`ACQUIRED_LINK_BASE`]
//! (0x8000 to 0xbfff), which it keeps for as long as it's that object (CR 400.7: a new
//! object starts over). Abilities acquired together with the same link stay linked to
//! each other; acquired from another source, or by another effect, they get another link.
//! Unlike the hashes they replace, two such grants to one object can't share a link, and
//! the numbers depend only on the game (they're part of it, copied with it by a snapshot
//! or a reversal).
//!
//! The ranges of the other links: printed links below 0x4000, links of copied abilities
//! 0x4000 to 0x7ffd (`layers::copied_link`), haunt's and the pregame links 0x7ffe and
//! 0x7fff, the links resolving effects give from 0xc000 on (`ability_grants::FreshLinks`).

use crate::game::Game;
use crate::types::ObjectId;
use std::collections::BTreeMap;
use std::sync::Mutex;

/// The first link an object acquires from a continuously reevaluated effect.
pub const ACQUIRED_LINK_BASE: u16 = 0x8000;
/// How many such links there are for one object (0x8000 to 0xbfff).
const ACQUIRED_LINK_COUNT: u16 = 0x4000;

/// What identifies an acquired link on the object acquiring it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LinkKey {
    /// An ability with link `link` that an effect of `source` gives
    /// (`Modification::AddAbility`, `layers::acquired_ability`).
    Given { source: ObjectId, link: u16 },
    /// An ability with link `link` of `from` that an effect of `granter` gives ("has all
    /// activated abilities of", `ability_grants`); `granter` is `None` for an effect with
    /// no source.
    Gained {
        granter: Option<ObjectId>,
        from: ObjectId,
        link: u16,
    },
}

#[derive(Clone, Debug, Default)]
struct Table {
    /// The link each object has for each key.
    links: BTreeMap<(ObjectId, LinkKey), u16>,
    /// How many links each object has been given.
    issued: BTreeMap<ObjectId, u16>,
}

/// The links objects acquired from continuously reevaluated effects, by object (part of
/// the game). Allocated while characteristics are computed, from a shared reference to the
/// game, hence the lock.
#[derive(Default)]
pub struct AcquiredLinks(Mutex<Table>);

impl Clone for AcquiredLinks {
    fn clone(&self) -> Self {
        AcquiredLinks(Mutex::new(self.table().clone()))
    }
}

impl std::fmt::Debug for AcquiredLinks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.table().links.iter()).finish()
    }
}

impl AcquiredLinks {
    fn table(&self) -> std::sync::MutexGuard<'_, Table> {
        // A panic while the lock was held leaves the table as it was: still usable.
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// The link `target` has for `key` (see the module docs), allocated the first time.
pub fn link(g: &Game, target: ObjectId, key: LinkKey) -> u16 {
    let mut t = g.acquired_links.table();
    if let Some(l) = t.links.get(&(target, key)) {
        return *l;
    }
    let n = t.issued.get(&target).copied().unwrap_or(0);
    let l = if n < ACQUIRED_LINK_COUNT {
        t.issued.insert(target, n + 1);
        ACQUIRED_LINK_BASE + n
    } else {
        // Every link has been given to this object once (16,384 distinct grants): one it
        // no longer has an ability with or keeps data under is given again, and forgotten
        // for the key that had it.
        let used = in_use(g, target);
        let free = (ACQUIRED_LINK_BASE..ACQUIRED_LINK_BASE + ACQUIRED_LINK_COUNT)
            .find(|l| !used.contains(l))
            .unwrap_or(ACQUIRED_LINK_BASE);
        t.links.retain(|(o, _), v| *o != target || *v != free);
        free
    };
    t.links.insert((target, key), l);
    l
}

/// The links `id` has an ability with or keeps data under.
fn in_use(g: &Game, id: ObjectId) -> std::collections::BTreeSet<u16> {
    let o = g.obj(id);
    let mut used: std::collections::BTreeSet<u16> =
        o.chars.abilities.iter().map(|a| a.link).collect();
    used.extend(o.linked_choices.keys().copied());
    used.extend(o.linked.keys().copied());
    used
}
