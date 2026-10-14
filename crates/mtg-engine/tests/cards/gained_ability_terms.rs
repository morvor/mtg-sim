//! Activation instructions for abilities an effect gives (gap R4, CR 113.10a): "You may
//! spend mana as though it were mana of any color to activate those abilities" (Drana and
//! Linvala, Grell Philosopher) and "You may activate each of those abilities only once each
//! turn" (Mairsil, the Pretender) become part of each gained ability (CR 602.5c, 609.4b),
//! not of the object's other abilities.

use mtg_engine::ability::AbilityKind;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// The index (among activated abilities) of `id`'s first activated ability whose text
/// contains `needle`.
fn activated_index(t: &mut TestGame, id: ObjectId, needle: &str) -> Vec<usize> {
    t.g.recompute();
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .enumerate()
        .filter(|(_, a)| a.text.contains(needle))
        .map(|(i, _)| i)
        .collect()
}

#[test]
fn drana_and_linvala_spends_any_color_only_for_gained_abilities() {
    cr!("113.10a", "609.4b", "613.1f");
    ruling!(
        "Drana and Linvala",
        "Drana and Linvala gains only activated abilities."
    );
    assert_supported(&["Drana and Linvala", "Grell Philosopher"]);
    // P1's Shivan Dragon gives D&L "{R}: This creature gets +1/+0 until end of turn.";
    // Dragon Mantle gives it the same ability, which D&L has from elsewhere.
    let mut t = TestGame::new(2);
    let dl = t.battlefield(P0, "Drana and Linvala");
    t.battlefield(P1, "Shivan Dragon");
    // Not the trample of P1's Colossal Dreadmaw: only activated abilities.
    t.battlefield(P1, "Colossal Dreadmaw");
    t.g.recompute();
    assert!(!t
        .obj_now(dl)
        .has_keyword(mtg_engine::keywords::KeywordKind::Trample));
    let mantle = t.battlefield(P0, "Dragon Mantle");
    assert!(t.g.attach(mantle, Entity::Object(dl)));
    let idx = activated_index(&mut t, dl, "+1/+0");
    assert_eq!(idx.len(), 2);
    let base = t.pt(dl);
    t.lands(P0, "Plains", 1);
    // Exactly one of the two can be paid with white mana.
    let mut ok = 0;
    for i in &idx {
        if t.activate(P0, dl, *i, &[]).is_ok() {
            ok += 1;
            t.resolve_all();
        } else {
            t.clear_answers();
        }
    }
    assert_eq!(ok, 1);
    assert_eq!(t.pt(dl), (base.0 + 1, base.1));
    // The gained one is the one that worked: with only the Dragon's ability (no Mantle),
    // white mana pays for it.
    let mut t = TestGame::new(2);
    let dl = t.battlefield(P0, "Drana and Linvala");
    t.battlefield(P1, "Shivan Dragon");
    t.lands(P0, "Plains", 1);
    let i = activated_index(&mut t, dl, "+1/+0")[0];
    t.activate(P0, dl, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(dl), (base.0 + 1, base.1));
}

#[test]
fn drana_and_linvala_activated_ability_resolves_after_the_creature_leaves() {
    cr!("113.10a", "113.7a");
    ruling!(
        "Drana and Linvala",
        "loses that ability because the opposing creature with that ability leaves the battlefield"
    );
    let mut t = TestGame::new(2);
    let dl = t.battlefield(P0, "Drana and Linvala");
    let dragon = t.battlefield(P1, "Shivan Dragon");
    t.lands(P0, "Plains", 1);
    let base = t.pt(dl);
    let i = activated_index(&mut t, dl, "+1/+0")[0];
    t.activate(P0, dl, i, &[]).unwrap();
    t.g.destroy(dragon, None);
    t.g.flush_events();
    t.resolve_all();
    assert!(activated_index(&mut t, dl, "+1/+0").is_empty());
    assert_eq!(t.pt(dl), (base.0 + 1, base.1));
}

#[test]
fn grell_philosopher_horrors_spend_blue_mana_as_any_color() {
    cr!("113.10a", "609.4b", "608.2h");
    ruling!(
        "Grell Philosopher",
        "Grell Philosopher causes Horrors to gain only activated abilities."
    );
    // P1's Furnace Hellkite: affinity for artifacts, flying, "{R}: This creature gets
    // +1/+0 until end of turn." Grell gains only the activated ability, payable with blue mana.
    for (land, works) in [("Island", true), ("Plains", false)] {
        let mut t = TestGame::new(2);
        let hellkite = t.battlefield(P1, "Furnace Hellkite");
        t.answer_targets(P0, &[Entity::Object(hellkite)]);
        let grell = t.enter(P0, "Grell Philosopher");
        t.resolve_all();
        let i = activated_index(&mut t, grell, "+1/+0");
        assert_eq!(i.len(), 1);
        assert!(!t
            .obj_now(grell)
            .has_keyword(mtg_engine::keywords::KeywordKind::Flying));
        let base = t.pt(grell);
        t.lands(P0, land, 1);
        assert_eq!(t.activate(P0, grell, i[0], &[]).is_ok(), works, "{land}");
        t.resolve_all();
        let bonus = if works { 1 } else { 0 };
        assert_eq!(t.pt(grell), (base.0 + bonus, base.1), "{land}");
    }
}

#[test]
fn mairsil_each_caged_cards_ability_once_each_turn() {
    cr!("113.10a", "602.5b", "602.5c", "613.1f");
    ruling!(
        "Mairsil, the Pretender",
        "Mairsil will have multiple instances of that ability. Each may be activated once each turn."
    );
    ruling!(
        "Mairsil, the Pretender",
        "it will have the abilities of only cards that player owns in exile with cage counters on them"
    );
    // Two Shivan Dragons P0 owns in exile with cage counters, one without.
    let mut t = TestGame::new(2);
    let mairsil = t.battlefield(P0, "Mairsil, the Pretender");
    for caged in [true, true, false] {
        let d = t.exile(P0, "Shivan Dragon");
        if caged {
            t.g.add_counters(Entity::Object(d), "cage", 1, None);
        }
    }
    let idx = activated_index(&mut t, mairsil, "+1/+0");
    assert_eq!(idx.len(), 2);
    t.lands(P0, "Mountain", 4);
    let base = t.pt(mairsil);
    t.activate(P0, mairsil, idx[0], &[]).unwrap();
    t.resolve_all();
    assert!(t.activate(P0, mairsil, idx[0], &[]).is_err());
    t.clear_answers();
    t.activate(P0, mairsil, idx[1], &[]).unwrap();
    t.resolve_all();
    assert!(t.activate(P0, mairsil, idx[1], &[]).is_err());
    t.clear_answers();
    assert_eq!(t.pt(mairsil), (base.0 + 2, base.1));
    // Under P1's control: P1 owns no caged cards.
    let mut ctx = mtg_engine::eval::Ctx::new(None, P1);
    ctx.targets = vec![vec![Entity::Object(mairsil)]];
    t.g.exec(
        &mtg_engine::ability::Effect::GainControl {
            what: mtg_engine::ability::Sel::Target(0),
            who: mtg_engine::ability::PlayerRef::You,
            duration: mtg_engine::ability::Duration::Permanent,
        },
        &mut ctx,
    );
    assert!(activated_index(&mut t, mairsil, "+1/+0").is_empty());
}

#[test]
fn sharkey_colored_mana_pays_a_gained_lands_colorless_symbol() {
    cr!("118.14", "113.10a");
    ruling!(
        "Sharkey, Tyrant of the Shire",
        "The six types of mana are white, blue, black, red, green, and colorless."
    );
    // P1's Sea Gate Wreckage: "{2}{C}, {T}: Draw a card. Activate only if you have no
    // cards in hand." Sharkey has it; three Islands pay {2}{C}.
    let mut t = TestGame::new(2);
    let sharkey = t.battlefield(P0, "Sharkey, Tyrant of the Shire");
    t.battlefield(P1, "Sea Gate Wreckage");
    t.lands(P0, "Island", 3);
    let i = activated_index(&mut t, sharkey, "Draw a card");
    assert_eq!(i.len(), 1);
    let before = t.hand_size(P0);
    t.activate(P0, sharkey, i[0], &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before + 1);
}
