//! Searches for cards with the same name as an object (CR 701.23, 201.2): "Search that
//! player's graveyard, hand, and library for all cards with the same name as the chosen
//! card and exile them" (`kw/search_same_name.rs`), "For each creature you control, you
//! may search your library for a creature card with the same name as that creature. Put
//! those cards onto the battlefield, then shuffle." (`kw/search_each_same_name.rs`).

use super::nouns::Det;
use super::players::Case;
use super::*;

impl Renderer<'_> {
    /// The object a same-name search names: a target ("that spell"), the card chosen by
    /// an earlier instruction ("the chosen card"), or each card an iteration goes over
    /// ("that card").
    fn named_object(&mut self, v: &[Effect], i: usize, object: &Sel) -> String {
        match object {
            Sel::Target(t) => {
                let noun = match self.targets.get(*t as usize).map(|s| &s.what) {
                    Some(TargetKind::Spell(_)) => "spell",
                    Some(TargetKind::Object(f)) if format!("{f:?}").contains("Creature") => {
                        "creature"
                    }
                    _ => "permanent",
                };
                format!("that {noun}")
            }
            Sel::Var(x) => {
                let chosen = v[..i].iter().any(
                    |e| matches!(e, Effect::Store { var, sel: Sel::Choose { .. } } if var == x),
                );
                if chosen {
                    "the chosen card".into()
                } else {
                    "that card".into()
                }
            }
            other => self.sel(other, Case::Obj),
        }
    }

    /// "search that player's graveyard, hand, and library for all cards with the same name
    /// as the chosen card and exile them[. Then that player shuffles]" (`v[i]` is the
    /// search).
    pub(crate) fn same_name_search(&mut self, v: &[Effect], i: usize) -> Option<(String, usize)> {
        let Effect::Custom(n) = v.get(i)? else {
            return None;
        };
        if let Some(r) = self.search_each_same_name(v, i, n) {
            return Some(r);
        }
        let s = crate::kw::search_same_name::parse_general(n)?;
        let Some(Effect::Exile {
            what: Sel::Var(found),
            face_down: false,
            ..
        }) = v.get(i + 1)
        else {
            return None;
        };
        if *found != s.var {
            return None;
        }
        let obj = self.named_object(v, i, &s.object);
        let zones = if s.all_zones {
            "graveyard, hand, and library"
        } else {
            "library"
        };
        let whose = match &s.whose {
            Some(p) => {
                let p = self.player(p, Case::Poss);
                format!("{p} {zones}")
            }
            // The zones of the object's controller: "the graveyard, hand, and library of
            // that spell's controller".
            None => format!("the {zones} of {obj}'s controller"),
        };
        let how_many = if s.any_number { "any number of" } else { "all" };
        let mut out = format!(
            "search {whose} for {how_many} cards with the same name as {obj} {{alt:and exile them|, exile those cards}}"
        );
        let mut used = 2;
        // "..., then that player shuffles": the player whose library was searched.
        if let Some(Effect::Shuffle { who }) = v.get(i + 2) {
            if format!("{who:?}") == format!("{:?}", s.player()) {
                out.push_str("{alt:. then that player shuffles|, then that player shuffles}");
                used = 3;
            }
        }
        Some((out, used))
    }

    /// "For each creature you control, you may search your library for a creature card
    /// with the same name as that creature. Put those cards onto the battlefield, then
    /// shuffle."
    fn search_each_same_name(
        &mut self,
        v: &[Effect],
        i: usize,
        name: &str,
    ) -> Option<(String, usize)> {
        let spec = crate::kw::search_each_same_name::parse(name)?;
        let (
            Some(Effect::Store {
                var: found,
                sel: Sel::Var(vars::IT),
            }),
            Some(Effect::Move {
                what: Sel::Var(moved),
                to,
            }),
        ) = (v.get(i + 1), v.get(i + 2))
        else {
            return None;
        };
        if found != moved || to.zone != ZoneKind::Battlefield {
            return None;
        }
        // The objects: "each creature you control", "each of those permanents".
        let (each, filter) = match &spec.objects {
            Sel::All(f) => {
                let n = self.for_each_noun(f);
                (n, f.clone())
            }
            Sel::Var(x) => {
                let f = v[..i].iter().find_map(|e| match e {
                    Effect::Store {
                        var,
                        sel: Sel::Choose { filter, .. },
                    } if var == x => Some(filter.clone()),
                    _ => None,
                })?;
                let n = self.noun(&f, Num::Many);
                let head = n.split(' ').next().unwrap_or("permanents").to_string();
                (format!("of those {head}"), f)
            }
            _ => return None,
        };
        let head = if format!("{filter:?}").contains("Creature") {
            "creature"
        } else {
            "permanent"
        };
        let may = if spec.optional { "you may " } else { "" };
        let lib = match &spec.whose {
            PlayerRef::You => "your library".to_string(),
            p => {
                let p = self.player(p, Case::Poss);
                format!("{p} library")
            }
        };
        let card = self.noun_det(&spec.filter, Det::A);
        let tapped = if to.tapped { " tapped" } else { "" };
        let mut out = format!(
            "for each {each}, {may}search {lib} for {card} with the same name as that {head}. put those cards onto the battlefield{tapped} {{opt:under your control}}"
        );
        let mut used = 3;
        if let Some(Effect::Shuffle { who }) = v.get(i + 3) {
            if format!("{who:?}") == format!("{:?}", spec.whose) {
                let w = match who {
                    PlayerRef::You => "shuffle".to_string(),
                    _ => "that player shuffles".to_string(),
                };
                out.push_str(&format!(", then {w}"));
                used = 4;
            }
        }
        Some((out, used))
    }
}
