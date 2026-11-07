//! "For each creature you control, you may search your library for a creature card with
//! the same name as that creature." (Doubling Chant), "For each of those permanents, ..."
//! (Clarion Ultimatum), "For each tapped nonland permanent target opponent controls,
//! search that player's library for a card with the same name as that permanent."
//! (Dichotomancy): one search (CR 701.23, "You make just one search", Clarion
//! Ultimatum's ruling) for a card for each of the objects, each with that object's name
//! as the search happens (CR 201.2; a copy has the name it copies, a face-down permanent
//! has none). A card is found for at most one of the objects. The cards found stay where
//! they are (`vars::IT`) for the instruction that follows. Parsed in
//! `oracle/patterns/search_grammar_2.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::Entity;
use serde::{Deserialize, Serialize};

const PREFIX: &str = "search each same name:";

/// Variables holding the object each part of the search is for.
const PART_OBJECT: Var = vars::USER + 2320;

/// What the search is for.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Spec {
    /// The objects whose names are searched for.
    pub objects: Sel,
    /// Whose library is searched.
    pub whose: PlayerRef,
    /// What else the cards must be ("a creature card").
    pub filter: Filter,
    /// "you may search".
    pub optional: bool,
}

/// The instruction for `spec`.
pub fn effect(spec: &Spec) -> Option<Effect> {
    let json = serde_json::to_string(spec).ok()?;
    Some(Effect::Custom(format!("{PREFIX}{json}").into()))
}

/// The spec of an [`effect`] instruction.
pub fn parse(name: &str) -> Option<Spec> {
    serde_json::from_str(name.strip_prefix(PREFIX)?).ok()
}

pub struct SearchEachSameName;

impl KeywordRules for SearchEachSameName {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(spec) = parse(name) else {
            return false;
        };
        let objects = g.resolve_objects(&spec.objects, ctx);
        let mut parts = Vec::new();
        for (i, o) in objects.iter().enumerate() {
            let var = PART_OBJECT + i as Var;
            ctx.set_var(var, vec![Entity::Object(*o)]);
            parts.push(SearchPart {
                filter: Filter::and(vec![
                    spec.filter.clone(),
                    Filter::SameNameAs(Box::new(Sel::Var(var))),
                ]),
                count: Value::c(1),
                up_to: false,
                all: false,
            });
        }
        let search = SearchSpec {
            who: PlayerRef::You,
            whose: spec.whose.clone(),
            zones: vec![ZoneKind::Library],
            zones_optional: false,
            parts,
            distinct_names: false,
            optional: spec.optional,
            reveal: false,
            dests: vec![],
            shuffle: SearchShuffle::No,
        };
        crate::search_rules::perform(g, &search, ctx);
        true
    }
}

inventory::submit! { KeywordRegistration(&SearchEachSameName) }
