//! CR 702.16 Protection.
//!
//! A protection keyword carries its quality as a filter (`Keyword::filter`; `Filter::Any`
//! for "protection from everything", 702.16j; `ControlledBy(..)` for protection from a
//! player, 702.16k). "From A and from B" and "from each color" compile to one keyword per
//! quality (702.16g–i). Players get protection through `PlayerModification::ProtectionFrom`.
//!
//! Each part of protection ("DEBT") is enforced where the action happens:
//! * **D**amage (702.16e) — here, as a prevention effect (CR 615) of the permanent or
//!   player with protection;
//! * **E**nchanting/**E**quipping/fortifying (702.16c–d) — `attach.rs` and the SBAs of
//!   CR 704.5m–n;
//! * **B**locking (702.16f) — `combat.rs`;
//! * **T**argeting (702.16b) — `stack.rs`.
//!
//! The quality is matched against the source's characteristics; a card type, subtype, or
//! supertype quality matches permanents and objects in other zones alike (702.16a).

use super::{KeywordRegistration, KeywordRules, KeywordShield};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::*;
use crate::types::*;

pub struct Protection;

impl KeywordRules for Protection {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Protection]
    }

    /// CR 702.16e: "Any damage that would be dealt by sources that have the stated
    /// quality to a permanent or player with protection is prevented." Multiple
    /// instances from the same quality are redundant (702.16m), so one effect per
    /// permanent or player is enough.
    fn damage_prevention(&self, g: &Game, source: ObjectId, target: Entity) -> Vec<KeywordShield> {
        match target {
            Entity::Object(o) => {
                let ob = g.obj(o);
                if ob.zone != Zone::Battlefield {
                    return vec![];
                }
                let ctx = Ctx::new(Some(o), ob.controller);
                ob.chars
                    .abilities
                    .iter()
                    .find_map(|a| match &a.kind {
                        AbilityKind::Keyword(k)
                            if k.kind == KeywordKind::Protection
                                && k.filter.as_ref().is_none_or(|f| g.matches(source, f, &ctx)) =>
                        {
                            Some(KeywordShield {
                                holder: target,
                                id: a.uid,
                                controller: ob.controller,
                                text: format!("{}: {}", ob.chars.name, a.text),
                            })
                        }
                        _ => None,
                    })
                    .into_iter()
                    .collect()
            }
            Entity::Player(p) => g
                .player(p)
                .mods
                .iter()
                .enumerate()
                .find_map(|(i, m)| match m {
                    PlayerModification::ProtectionFrom(f)
                        if g.matches(source, f, &Ctx::new(None, p)) =>
                    {
                        Some(KeywordShield {
                            holder: target,
                            id: i as u64,
                            controller: p,
                            text: format!("{p}: protection"),
                        })
                    }
                    _ => None,
                })
                .into_iter()
                .collect(),
        }
    }
}

inventory::submit! { KeywordRegistration(&Protection) }
