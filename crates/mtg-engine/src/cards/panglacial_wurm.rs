//! Panglacial Wurm: "While you're searching your library, you may cast ~ from your
//! library." As a search of your own library begins (before any card is found), you may
//! cast it, paying its costs; it's cast during the resolution of the searching spell or
//! ability, can't be found by that search, and several may be cast (rulings).

use super::{marker, ManualAbility};
use crate::ability::{AbilityKind, FunctionZone};
use crate::object::CastMethod;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::types::{ObjectId, PlayerId};

const CAST_WHILE_SEARCHING: &str = "card:Panglacial Wurm:may be cast from your library while you search it";
const TEXT: &str = "While you're searching your library, you may cast ~ from your library.";

inventory::submit! { ManualAbility {
    card: "Panglacial Wurm",
    face: 0,
    text: TEXT,
    build: |_| {
        let mut m = marker(CAST_WHILE_SEARCHING, TEXT);
        if let AbilityKind::Static(s) = &mut std::sync::Arc::make_mut(&mut m).kind {
            s.zone = FunctionZone::Library;
        }
        vec![m]
    },
    reason: "may be cast from your library in the middle of a search: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn searching_library(&self, g: &mut Game, searcher: PlayerId) {
        let wurms: Vec<ObjectId> = g
            .player(searcher)
            .library
            .iter()
            .copied()
            .filter(|c| super::has_marker(g, *c, CAST_WHILE_SEARCHING))
            .collect();
        for w in wurms {
            if !g.is_live(w) || g.obj(w).zone != Zone::Library(searcher) {
                continue;
            }
            let name = g.obj(w).chars.name.clone();
            if !g.ask_yes_no(searcher, Some(w), &format!("Cast {name} from your library?"), false)
            {
                continue;
            }
            let _ = crate::casting::cast_during_resolution_as(
                g,
                searcher,
                w,
                CastMethod::Normal,
                None,
            );
        }
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
