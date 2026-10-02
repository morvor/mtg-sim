//! Temporary debugging probe (removed before landing).

/// PS_DISABLE set: this item's new patterns are off (to diff compiled cards).
pub fn disabled() -> bool {
    std::env::var("PS_DISABLE").is_ok()
}

#[cfg(test)]
mod probe {
    use crate::oracle::*;

    fn sentences_report(
        eff: &str,
        ctx: &CompileContext,
        trig: Option<(&crate::ability::Sel, &crate::ability::PlayerRef)>,
    ) {
        for s in crate::oracle::effects::split_sentences(eff) {
            let r = match trig {
                Some((it, p)) => {
                    effects::parse_trigger_body(&s, ctx, it.clone(), p.clone()).is_some()
                }
                None => effects::parse_body(&s, ctx).is_some(),
            };
            eprintln!("     {} {s}", if r { "ok  " } else { "FAIL" });
        }
    }

    #[test]
    fn probe_dump_all() {
        let Ok(f) = std::env::var("PS_DUMP") else {
            return;
        };
        let mut out = String::new();
        for c in mtg_data::cards().iter() {
            if !c.is_playable_card() || !c.is_legal_somewhere() {
                continue;
            }
            let def = crate::card::CardDef::from_scryfall(c);
            let mut line = format!("{}\t{}", c.name, def.is_fully_supported());
            for fd in &def.faces {
                for a in &fd.chars.abilities {
                    line.push_str(&format!("\t{:?}", a.kind));
                }
            }
            out.push_str(&line);
            out.push('\n');
        }
        std::fs::write(f, out).unwrap();
    }

    #[test]
    fn probe_ps() {
        let Ok(f) = std::env::var("PS_PROBE") else {
            return;
        };
        let verbose = std::env::var("PS_VERBOSE").is_ok();
        for l in std::fs::read_to_string(f).unwrap().lines() {
            let l = l.trim();
            if l.is_empty() {
                continue;
            }
            if let Some(x) = l.strip_prefix("U:") {
                let cd = crate::card::card(x);
                let u = cd.unsupported_text();
                eprintln!("U {x}: {}", if u.is_empty() { "OK".to_string() } else { u.join(" | ") });
                continue;
            }
            if let Some(x) = l.strip_prefix("P:") {
                eprintln!("P {x:?} => {:?}", crate::oracle::phrases::parse_object_phrase(x));
                continue;
            }
            if let Some(x) = l.strip_prefix("T:") {
                eprintln!("T {x:?} => {:?}", crate::oracle::phrases::parse_target(x));
                continue;
            }
            if let Some(x) = l.strip_prefix("C:") {
                let tl = crate::types::TypeLine::parse("Creature — Elf");
                let ctx = CompileContext { card_name: "Probe", full_name: "Probe", type_line: &tl, layout: crate::card::Layout::Normal, face_index: 0, keywords: &[], power: Some("2"), toughness: Some("2") };
                eprintln!("C {x:?} => {:?}", crate::oracle::statics::parse_condition(x, &ctx));
                continue;
            }
            if let Some(x) = l.strip_prefix("E:") {
                let tl = crate::types::TypeLine::parse("Sorcery");
                let ctx = CompileContext { card_name: "Probe", full_name: "Probe", type_line: &tl, layout: crate::card::Layout::Normal, face_index: 0, keywords: &[], power: None, toughness: None };
                eprintln!("E {x:?} => {:?}", crate::oracle::effects::parse_body(x, &ctx));
                continue;
            }
            let Some(def) = crate::card::CardDb::global().get(l) else {
                eprintln!("?? {l}");
                continue;
            };
            let full = def.name.to_string();
            let src = mtg_data::cards().iter().find(|c| c.name == full);
            for (i, fd) in def.faces.iter().enumerate() {
                let tl_s = src
                    .and_then(|c| {
                        c.card_faces
                            .as_ref()
                            .and_then(|fs| fs.get(i))
                            .and_then(|f| f.type_line.clone())
                            .or_else(|| c.type_line.clone())
                    })
                    .unwrap_or_default();
                let tl = crate::types::TypeLine::parse(&tl_s);
                let name = fd.chars.name.to_string();
                let ctx = CompileContext {
                    card_name: &name,
                    full_name: &full,
                    type_line: &tl,
                    layout: def.layout,
                    face_index: i,
                    keywords: &[],
                    power: Some("2"),
                    toughness: Some("2"),
                };
                if verbose {
                    for a in &fd.chars.abilities {
                        eprintln!("  {:?}\n      <- {:?}", a.kind, a.text);
                    }
                }
                for u in &fd.unsupported {
                    eprintln!("=== {full}: {u}");
                    let t = strip_ability_word(u);
                    if let Some((_c, e)) = split_cost(t) {
                        let (e2, ..) = crate::oracle::costs::split_activation_restrictions(e);
                        sentences_report(e2, &ctx, None);
                        continue;
                    }
                    let lower = t.to_lowercase();
                    if lower.starts_with("when") || lower.starts_with("at ") {
                        let mut done = false;
                        for (ix, _) in t.match_indices(", ") {
                            let (c, e) = (&t[..ix], &t[ix + 2..]);
                            if let Some((_, it, p)) =
                                crate::oracle::triggers::parse_trigger_condition(&c.to_lowercase())
                            {
                                eprintln!("   trigger ok: {c}");
                                sentences_report(e, &ctx, Some((&it, &p)));
                                done = true;
                                break;
                            }
                        }
                        if !done {
                            eprintln!("   trigger FAIL");
                        }
                        continue;
                    }
                    sentences_report(t, &ctx, None);
                }
            }
        }
    }
}
