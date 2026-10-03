//! A copy's exceptions written as Oracle text lists them (CR 707.9b): one clause per
//! exception, each with its subject — "except its name is Mishra's Warform and it's a
//! 4/4 Construct artifact creature in addition to its other types", "except it's 4/3,
//! it's a Vehicle artifact in addition to its other types, and it has flying", "except
//! it's an enchantment and loses all other card types".

use super::*;

impl Renderer<'_> {
    /// The exceptions as a list of clauses, in the ways cards write them (power and
    /// toughness with the types or apart), or `None` if a modification has no such
    /// clause.
    pub(crate) fn exception_list(&mut self, mods: &[Modification]) -> Option<Vec<String>> {
        let changeling = mods.iter().any(|m| {
            matches!(m, Modification::AddKeyword(k) if k.kind == crate::keywords::KeywordKind::Changeling)
        });
        let mut name: Option<String> = None;
        let mut not_super: Vec<String> = Vec::new();
        let mut pt: Option<String> = None;
        let mut supers: Vec<String> = Vec::new();
        let mut colors: Vec<String> = Vec::new();
        let mut added_colors = false;
        let mut subtypes: Vec<String> = Vec::new();
        let mut types: Vec<String> = Vec::new();
        let mut set_types = false;
        let mut grants: Vec<String> = Vec::new();
        for m in mods {
            match m {
                Modification::SetName(n) => name = Some(n.to_string()),
                Modification::RemoveSupertypes(s) => {
                    not_super.extend(s.iter().map(|x| nouns::supertype_word(*x).to_string()))
                }
                Modification::AddSupertypes(s) => {
                    supers.extend(s.iter().map(|x| nouns::supertype_word(*x).to_string()))
                }
                Modification::SetPT(Some(Value::Const(p)), Some(Value::Const(t))) => {
                    pt = Some(format!("{p}/{t}"))
                }
                Modification::AddColors(cs) => {
                    colors.extend(cs.iter().map(|c| c.word().to_string()));
                    added_colors = true;
                }
                Modification::AddSubtypes(s) => subtypes.extend(s.iter().map(|x| x.to_string())),
                Modification::AddTypes(t) => types.extend(t.iter().map(|x| x.word().to_string())),
                Modification::SetTypes {
                    types: t,
                    subtypes: s,
                } => {
                    set_types = true;
                    types.extend(t.iter().map(|x| x.word().to_string()));
                    subtypes.extend(s.iter().map(|x| x.to_string()));
                }
                // CR 702.73a: changeling is "is every creature type".
                Modification::AllCreatureTypes if changeling => {}
                Modification::AddAbility(a) if changeling && is_changeling_cda(a) => {}
                Modification::AddKeyword(k) => {
                    let saved = std::mem::replace(&mut self.granted_keyword, true);
                    grants.push(self.keyword_lower(k));
                    self.granted_keyword = saved;
                }
                Modification::AddAbility(a) => {
                    let s = self.nested_ability(a);
                    grants.push(format!("\"{s}\""));
                }
                Modification::AddThisAbility => grants.push("this ability".into()),
                _ => return None,
            }
        }
        let mut lead: Vec<String> = Vec::new();
        if let Some(n) = &name {
            lead.push(format!("its name is {n}"));
        }
        if !not_super.is_empty() {
            lead.push(format!("it isn't {}", join_list(&not_super, "or")));
        }
        // The kind of object it is: "a 2/2 black Zombie", "a Vehicle artifact".
        let kind = |pt: Option<&String>| -> String {
            let mut w: Vec<String> = Vec::new();
            w.extend(pt.cloned());
            w.extend(supers.iter().cloned());
            w.extend(colors.iter().cloned());
            w.extend(subtypes.iter().cloned());
            w.extend(types.iter().cloned());
            with_article(&w.join(" "))
        };
        let typed = !types.is_empty() || !subtypes.is_empty() || !supers.is_empty();
        let tail = if set_types {
            " and loses all other card types".to_string()
        } else if added_colors {
            " in addition to its other colors and types".to_string()
        } else if types.is_empty() && supers.is_empty() && !subtypes.is_empty() {
            " in addition to its other {alt:types|creature types}".to_string()
        } else {
            " in addition to its other types".to_string()
        };
        // Power and toughness with the types, or apart ("it's 4/3, it's a Vehicle
        // artifact ...").
        let mut forms: Vec<Vec<String>> = Vec::new();
        if typed || added_colors {
            forms.push(vec![format!("it's {}{tail}", kind(pt.as_ref()))]);
            if let Some(pt) = &pt {
                forms.push(vec![
                    format!("it's {pt}"),
                    format!("it's {}{tail}", kind(None)),
                ]);
            }
        } else if !colors.is_empty() {
            return None;
        } else {
            forms.push(pt.iter().map(|pt| format!("it's {pt}")).collect());
        }
        let has = (!grants.is_empty()).then(|| format!("it has {}", join_list(&grants, "and")));
        let out: Vec<String> = forms
            .into_iter()
            .map(|f| {
                let mut clauses = lead.clone();
                clauses.extend(f);
                clauses.extend(has.clone());
                join_list(&clauses, "and")
            })
            .filter(|s| !s.is_empty())
            .collect();
        (!out.is_empty()).then_some(out)
    }
}
