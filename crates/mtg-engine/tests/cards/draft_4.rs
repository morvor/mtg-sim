//! Conspiracy draft-matters cards, round 3 (`oracle/patterns/draft_4.rs`, `draft.rs`):
//! abilities that function during the draft (CR 905.2) and the information the game reads
//! of them (CR 905.2b).

use mtg_engine::card::{card, CardDef};
use mtg_engine::draft::{Draft, DraftError, DraftInfo, DraftOption, DraftStyle};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

/// Three players, one round; each pack is `cards` (by name) for P0's and fillers for the
/// others.
fn draft_with(p0_pack: &[&str]) -> Draft {
    let filler =
        |k: usize| -> Vec<Arc<CardDef>> { (0..k).map(|_| card("Grizzly Bears")).collect() };
    let pack: Vec<Arc<CardDef>> = p0_pack.iter().map(|n| card(n)).collect();
    let k = pack.len();
    Draft::new(
        vec![vec![pack], vec![filler(k)], vec![filler(k)]],
        DraftStyle::Conspiracy,
        1,
    )
}

/// P1 and P2 draft the first card of their packs.
fn others_pick(d: &mut Draft) {
    for p in [P1, P2] {
        if !d.pack(p).is_empty() {
            d.pick(p, &[0]).unwrap();
        }
    }
}

fn index_of(d: &Draft, p: PlayerId, name: &str) -> usize {
    d.pack(p).iter().position(|c| c.name == name).unwrap()
}

fn game_with(info: DraftInfo) -> TestGame {
    let mut t = TestGame::new(3);
    t.g.start.draft = info;
    t.set_step(P0, Step::PrecombatMain);
    t
}

#[test]
fn animus_of_predation_removes_a_card_and_has_its_keywords() {
    cr!("905.2", "905.2b", "905.2c");
    ruling!(
        "Animus of Predation",
        "You can remove from the draft any card you draft after Animus of Predation."
    );
    ruling!(
        "Animus of Predation",
        "this creature gains those specific variants"
    );
    assert_supported(&["Animus of Predation"]);
    let mut d = Draft::new(
        vec![vec![vec![
            card("Animus of Predation"),
            card("Serra Angel"),
            card("Grizzly Bears"),
        ]]],
        DraftStyle::Conspiracy,
        1,
    );
    d.pick(P0, &[0]).unwrap();
    assert!(d.drafted(P0)[0].face_up);
    let with = "Animus of Predation".into();
    let serra = index_of(&d, P0, "Serra Angel");
    d.pick_with(P0, &[serra], Some(DraftOption::Remove { with }))
        .unwrap();
    d.pick(P0, &[0]).unwrap();
    let (pools, info) = d.finish();
    // Serra Angel isn't in P0's card pool.
    assert!(pools[0].iter().all(|c| c.name != "Serra Angel"));
    let mut t = game_with(info);
    let animus = t.battlefield(P0, "Animus of Predation");
    let c = &t.obj_now(animus).chars;
    assert!(c.has_keyword(keywords::KeywordKind::Flying));
    assert!(c.has_keyword(keywords::KeywordKind::Vigilance));
    assert!(!c.has_keyword(keywords::KeywordKind::Lifelink));
    // Another player removed nothing with theirs.
    let other = t.battlefield(P1, "Animus of Predation");
    assert!(!t
        .obj_now(other)
        .chars
        .has_keyword(keywords::KeywordKind::Flying));
}

#[test]
fn cogwork_grinder_enters_with_a_counter_for_each_removed_card() {
    cr!("905.2", "905.2b");
    ruling!(
        "Cogwork Grinder",
        "You can't remove cards from the draft that you've already drafted."
    );
    assert_supported(&["Cogwork Grinder"]);
    let mut d = Draft::new(
        vec![vec![vec![
            card("Lightning Bolt"),
            card("Cogwork Grinder"),
            card("Shock"),
            card("Opt"),
        ]]],
        DraftStyle::Conspiracy,
        1,
    );
    let with: smol_str::SmolStr = "Cogwork Grinder".into();
    // No Grinder drafted yet: the option can't be used.
    assert_eq!(
        d.pick_with(P0, &[0], Some(DraftOption::Remove { with: with.clone() })),
        Err(DraftError::InvalidOption)
    );
    d.pick(P0, &[0]).unwrap();
    d.pick(P0, &[0]).unwrap();
    d.pick_with(P0, &[0], Some(DraftOption::Remove { with: with.clone() }))
        .unwrap();
    d.pick_with(P0, &[0], Some(DraftOption::Remove { with }))
        .unwrap();
    let (pools, info) = d.finish();
    assert_eq!(pools[0].len(), 2);
    assert_eq!(info.removed.len(), 2);
    assert!(info.removed.iter().all(|r| !r.face_up));
    let mut t = game_with(info);
    let g = t.enter(P0, "Cogwork Grinder");
    assert_eq!(t.counters(g, counters::PLUS1), 2);
    // The controller's removals, not the owner's.
    let g2 = t.enter(P1, "Cogwork Grinder");
    assert_eq!(t.counters(g2, counters::PLUS1), 0);
}

#[test]
fn noble_banneret_notes_a_creature_name_and_pumps_those_creatures() {
    cr!("905.2", "905.2b", "905.2c");
    ruling!(
        "Noble Banneret",
        "If you gain control of a Noble Banneret, but you didn't draft one, there will be no applicable noted names."
    );
    assert_supported(&["Noble Banneret"]);
    let mut d = Draft::new(
        vec![vec![vec![
            card("Noble Banneret"),
            card("Lightning Bolt"),
            card("Grizzly Bears"),
        ]]],
        DraftStyle::Conspiracy,
        1,
    );
    d.pick(P0, &[0]).unwrap();
    let with: smol_str::SmolStr = "Noble Banneret".into();
    // Only a creature card can be revealed.
    let bolt = index_of(&d, P0, "Lightning Bolt");
    assert_eq!(
        d.pick_with(
            P0,
            &[bolt],
            Some(DraftOption::RevealAndNote { with: with.clone() })
        ),
        Err(DraftError::InvalidOption)
    );
    let bears = index_of(&d, P0, "Grizzly Bears");
    d.pick_with(P0, &[bears], Some(DraftOption::RevealAndNote { with }))
        .unwrap();
    // Turned face down.
    assert!(!d.drafted(P0)[0].face_up);
    d.pick(P0, &[0]).unwrap();
    let (_, info) = d.finish();
    let mut t = game_with(info);
    let banneret = t.battlefield(P0, "Noble Banneret");
    let base = t.pt(banneret);
    assert!(!t
        .obj_now(banneret)
        .chars
        .has_keyword(keywords::KeywordKind::Lifelink));
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(banneret), (base.0 + 1, base.1 + 1));
    assert_eq!(t.pt(bears), (3, 3));
    assert!(t
        .obj_now(bears)
        .chars
        .has_keyword(keywords::KeywordKind::Lifelink));
    assert_eq!(t.pt(other), (3, 3));
    // An opponent's Grizzly Bears isn't "those creatures".
    let theirs = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(theirs), (2, 2));
}

#[test]
fn smuggler_captain_finds_a_noted_card() {
    cr!("905.2b");
    ruling!(
        "Smuggler Captain",
        "If a Smuggler Captain enters the battlefield under your control, but you didn't draft one"
    );
    assert_supported(&["Smuggler Captain"]);
    let mut d = Draft::new(
        vec![vec![vec![card("Smuggler Captain"), card("Lightning Bolt")]]],
        DraftStyle::Conspiracy,
        1,
    );
    d.pick(P0, &[0]).unwrap();
    d.pick_with(
        P0,
        &[0],
        Some(DraftOption::RevealAndNote {
            with: "Smuggler Captain".into(),
        }),
    )
    .unwrap();
    let (_, info) = d.finish();
    assert_eq!(info.notes[0].noted_name.as_deref(), Some("Lightning Bolt"));
    let mut t = game_with(info);
    t.library_top(P0, "Shock");
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.enter(P0, "Smuggler Captain");
    t.resolve_all();
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert!(!t.in_hand(P0, "Shock"));
}

#[test]
fn paliano_vanguard_pumps_creatures_of_a_noted_type() {
    cr!("905.2b");
    assert_supported(&["Paliano Vanguard"]);
    let mut d = Draft::new(
        vec![vec![vec![card("Paliano Vanguard"), card("Grizzly Bears")]]],
        DraftStyle::Conspiracy,
        1,
    );
    d.pick(P0, &[0]).unwrap();
    d.pick_with(
        P0,
        &[0],
        Some(DraftOption::RevealAndNote {
            with: "Paliano Vanguard".into(),
        }),
    )
    .unwrap();
    let (_, info) = d.finish();
    assert_eq!(
        info.notes[0].noted_types,
        vec![smol_str::SmolStr::new("Bear")]
    );
    let mut t = game_with(info);
    let vanguard = t.battlefield(P0, "Paliano Vanguard");
    let base = t.pt(vanguard);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(t.pt(giant), (3, 3));
    assert_eq!(t.pt(vanguard), base);
}

#[test]
fn paliano_and_regicide_use_the_colors_chosen_as_they_were_drafted() {
    cr!("905.2", "905.2b");
    interp!("draft-three-colors-default");
    // Without preset colors: three different colors.
    let mut d = Draft::new(
        vec![vec![vec![card("Regicide")]]],
        DraftStyle::Conspiracy,
        7,
    );
    d.pick(P0, &[0]).unwrap();
    let mut c = d.info.notes[0].colors.clone();
    assert_eq!(c.len(), 3);
    c.sort();
    c.dedup();
    assert_eq!(c.len(), 3);
    ruling!("Regicide", "Regicide can't target colorless creatures.");
    ruling!(
        "Regicide",
        "The target creature can be colors that weren't chosen as long as it's also one or more colors that were."
    );
    ruling!(
        "Paliano, the High City",
        "Notably, Paliano won't produce colorless mana."
    );
    assert_supported(&["Paliano, the High City", "Regicide"]);
    let mut d = draft_with(&["Regicide", "Grizzly Bears", "Grizzly Bears"]);
    d.preset_colors(P0, [Color::Red, Color::White, Color::Green]);
    d.pick(P0, &[0]).unwrap();
    assert!(d.can_see(P1, "Regicide"));
    while !d.is_complete() {
        for p in [P0, P1, P2] {
            if !d.pack(p).is_empty() {
                let _ = d.pick(p, &[0]);
            }
        }
    }
    let (_, info) = d.finish();
    let mut t = game_with(info);
    t.lands(P0, "Swamp", 2);
    // Only a blue creature and a colorless one: no legal target.
    let blue = t.battlefield(P1, "Merfolk Looter");
    let golem = t.battlefield(P1, "Ornithopter");
    let r = t.hand(P0, "Regicide");
    assert!(t.cast(P0, r).target(blue).try_go().is_err());
    assert!(t.cast(P0, r).target(golem).try_go().is_err());
    // White and black: white was chosen.
    let wb = t.battlefield(P1, "Tidehollow Sculler");
    t.cast(P0, r).target(wb).go();
    t.resolve();
    assert!(!t.on_battlefield(wb));
    assert!(t.on_battlefield(blue));
    // Paliano, the High City: one mana of a color chosen as P0 drafted cards with its
    // name — none were, so no mana.
    let paliano = t.battlefield(P0, "Paliano, the High City");
    let _ = t.activate(P0, paliano, 0, &[]);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    let mut d = Draft::new(
        vec![vec![vec![card("Paliano, the High City")]]],
        DraftStyle::Conspiracy,
        1,
    );
    d.preset_colors(P0, [Color::Blue, Color::Black, Color::Red]);
    d.pick(P0, &[0]).unwrap();
    let (_, info) = d.finish();
    let mut t = game_with(info);
    let paliano = t.battlefield(P0, "Paliano, the High City");
    t.activate(P0, paliano, 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    let pool = &t.g.player(P0).mana_pool;
    assert!(
        pool.count(mana::ManaType::U)
            + pool.count(mana::ManaType::B)
            + pool.count(mana::ManaType::R)
            == 1
    );
}

#[test]
fn aether_searcher_casts_the_card_noted_after_it() {
    cr!("905.2b");
    ruling!(
        "Aether Searcher",
        "If Aether Searcher is the last card you draft, there won't be a noted name."
    );
    assert_supported(&["Aether Searcher"]);
    let mut d = Draft::new(
        vec![vec![vec![card("Aether Searcher"), card("Lightning Bolt")]]],
        DraftStyle::Conspiracy,
        1,
    );
    d.pick(P0, &[0]).unwrap();
    d.pick(P0, &[0]).unwrap();
    let (_, info) = d.finish();
    assert_eq!(info.notes[0].noted_name.as_deref(), Some("Lightning Bolt"));
    let mut t = game_with(info);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Aether Searcher");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn cogwork_librarian_drafts_two_cards_and_goes_back_into_the_pack() {
    cr!("905.2", "905.2c");
    ruling!(
        "Cogwork Librarian",
        "Cogwork Librarian essentially lets you draft two cards from a booster pack in exchange for putting Cogwork Librarian back into that booster pack."
    );
    assert_supported(&["Cogwork Librarian"]);
    let mut d = draft_with(&["Cogwork Librarian", "Lightning Bolt", "Shock", "Opt"]);
    d.pick(P0, &[0]).unwrap();
    others_pick(&mut d);
    // P0 now drafts from P2's pack (fillers): two cards, Librarian goes in.
    let before = d.pack(P0).len();
    d.pick_with(
        P0,
        &[0],
        Some(DraftOption::Additional {
            with: "Cogwork Librarian".into(),
            extra: 1,
        }),
    )
    .unwrap();
    assert_eq!(d.drafted(P0).len(), 2);
    assert!(d.drafted(P0).iter().all(|c| c.card.name == "Grizzly Bears"));
    let _ = before;
}

#[test]
fn leovolds_operative_skips_the_next_pack() {
    cr!("905.2", "905.2c");
    assert_supported(&["Leovold's Operative"]);
    let mut d = draft_with(&[
        "Leovold's Operative",
        "Lightning Bolt",
        "Shock",
        "Opt",
        "Duress",
    ]);
    d.pick(P0, &[0]).unwrap();
    others_pick(&mut d);
    d.pick_with(
        P0,
        &[0],
        Some(DraftOption::Additional {
            with: "Leovold's Operative".into(),
            extra: 1,
        }),
    )
    .unwrap();
    assert_eq!(d.drafted(P0).len(), 3);
    assert!(!d.drafted(P0)[0].face_up);
    others_pick(&mut d);
    // The next pack is passed without drafting from it.
    assert_eq!(d.pick(P0, &[0]), Err(DraftError::InvalidPick));
    d.pick(P0, &[]).unwrap();
    assert_eq!(d.drafted(P0).len(), 3);
}

#[test]
fn spire_phantasm_guesses_the_next_card() {
    cr!("905.2", "905.2b");
    assert_supported(&["Spire Phantasm"]);
    for (guess, draw) in [("Lightning Bolt", true), ("Shock", false)] {
        let mut d = draft_with(&["Spire Phantasm", "Lightning Bolt", "Shock"]);
        d.pick(P0, &[0]).unwrap();
        d.guess(P0, guess).unwrap();
        others_pick(&mut d);
        // P1 now drafts from P0's pack; its first card is Lightning Bolt.
        assert_eq!(d.pack(P1)[0].name, "Lightning Bolt");
        d.pick(P1, &[0]).unwrap();
        assert!(d.can_see(P0, "Lightning Bolt"));
        let (_, info) = d.clone().finish();
        let mut t = game_with(info);
        t.library_top(P0, "Island");
        let hand = t.hand_size(P0);
        t.enter(P0, "Spire Phantasm");
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + draw as usize, "{guess}");
    }
}

#[test]
fn cogwork_spy_looks_at_the_next_card_drafted_from_the_pack() {
    cr!("905.1c", "905.2");
    ruling!(
        "Cogwork Spy",
        "If you are the next person to draft from the booster pack (perhaps because of Cogwork Librarian), the first ability doesn't do anything."
    );
    assert_supported(&["Cogwork Spy"]);
    let mut d = draft_with(&["Cogwork Spy", "Lightning Bolt", "Shock"]);
    d.pick(P0, &[0]).unwrap();
    others_pick(&mut d);
    assert!(!d.can_see(P0, "Lightning Bolt"));
    d.pick(P1, &[0]).unwrap();
    assert!(d.can_see(P0, "Lightning Bolt"));
    assert!(!d.can_see(P2, "Lightning Bolt"));
}

#[test]
fn arcane_savant_and_volatile_chimera_use_cards_exiled_before_the_game() {
    cr!("607.2n", "905.2b");
    ruling!(
        "Volatile Chimera",
        "After Volatile Chimera copies a creature card, that card remains in the pool of cards you can choose with the last ability."
    );
    assert_supported(&["Arcane Savant", "Volatile Chimera"]);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let side =
        t.g.add_to_sideboard(P0, vec![card("Lightning Bolt"), card("Shock")]);
    let bolt =
        t.g.move_object(side[0], Zone::Exile, events::MoveCause::Effect, Some(P0));
    let bolt = bolt.unwrap();
    t.g.named_exiles.push((P0, "Arcane Savant".into(), bolt));
    t.lands(P0, "Mountain", 1);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Arcane Savant");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // The card stays exiled.
    assert_eq!(t.zone(bolt), Zone::Exile);
    // Volatile Chimera: a creature card exiled with cards named Volatile Chimera.
    let side = t.g.add_to_sideboard(P0, vec![card("Craw Wurm")]);
    let wurm =
        t.g.move_object(side[0], Zone::Exile, events::MoveCause::Effect, Some(P0))
            .unwrap();
    t.g.named_exiles.push((P0, "Volatile Chimera".into(), wurm));
    let chimera = t.battlefield(P0, "Volatile Chimera");
    t.lands(P0, "Mountain", 2);
    t.activate(P0, chimera, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(chimera).chars.name, "Craw Wurm");
    assert_eq!(t.pt(chimera), (6, 4));
    // It keeps the ability: it can become a copy again.
    t.lands(P0, "Mountain", 2);
    assert!(t.activate(P0, chimera, 0, &[]).is_ok());
}

#[test]
fn cogwork_tracker_attacks_the_noted_player() {
    cr!("905.2b", "508.1d");
    ruling!(
        "Cogwork Tracker",
        "If the last ability doesn't apply, you choose which player or planeswalker Cogwork Tracker attacks."
    );
    assert_supported(&["Cogwork Tracker"]);
    // P2 passed P0 a Cogwork Tracker.
    let mut d2 = Draft::new(
        vec![
            vec![vec![card("Grizzly Bears"), card("Grizzly Bears")]],
            vec![vec![card("Grizzly Bears"), card("Grizzly Bears")]],
            vec![vec![card("Grizzly Bears"), card("Cogwork Tracker")]],
        ],
        DraftStyle::Conspiracy,
        1,
    );
    for p in [P0, P1, P2] {
        d2.pick(p, &[0]).unwrap();
    }
    // Packs go left: P0 gets P2's pack.
    assert_eq!(d2.pack(P0)[0].name, "Cogwork Tracker");
    d2.pick(P0, &[0]).unwrap();
    let (_, info) = d2.finish();
    assert_eq!(info.notes[0].noted_player, Some(P2));
    let mut t = game_with(info);
    let tracker = t.battlefield(P0, "Cogwork Tracker");
    // Declared attacking P1: the requirement to attack P2 isn't obeyed, so the
    // declaration isn't legal; P0 attacks P2.
    t.attack(&[(tracker, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P2), 16);
}
