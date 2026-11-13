//! Characteristic and static grammar II (cards2-statics-characteristics), continued:
//! in-game behaviour of the cards whose texts this item compiled that the first file
//! only checks for support (riders, self-references through pronouns and plural names,
//! keywords copied from exiled cards, granted abilities that name their granter).

use crate::basic_effects_common::{assert_supported, last_target_candidates};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::FaceState;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn is(t: &mut TestGame, id: ObjectId, ct: CardType) -> bool {
    t.g.recompute();
    t.obj_now(id).chars.card_types.contains(ct)
}

fn has_kw(t: &mut TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.g.recompute();
    t.obj_now(id).chars.has_keyword(k)
}

fn has_subtype(t: &mut TestGame, id: ObjectId, s: &str) -> bool {
    t.g.recompute();
    t.obj_now(id).chars.subtypes.iter().any(|x| x == s)
}

fn named(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.g.battlefield
        .iter()
        .copied()
        .filter(|o| t.g.obj(*o).chars.name == name)
        .collect()
}

// ---------------------------------------------------------------------------
// Riders on returned permanents
// ---------------------------------------------------------------------------

#[test]
fn fang_returns_once_as_a_spirit() {
    cr!("611.3c", "201.5");
    assert_supported("Fang, Roku's Companion");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Fang, Roku's Companion");
    t.g.destroy(f, None);
    t.resolve_all();
    let back = t.named_on_battlefield("Fang, Roku's Companion");
    assert_eq!(back.len(), 1);
    assert!(has_subtype(&mut t, back[0], "Spirit"));
    assert!(has_subtype(&mut t, back[0], "Dragon"));
    // It was a Spirit when it died: it stays in the graveyard.
    t.g.destroy(back[0], None);
    t.resolve_all();
    assert!(t.named_on_battlefield("Fang, Roku's Companion").is_empty());
    assert!(t.in_graveyard(P0, "Fang, Roku's Companion"));
}

#[test]
fn tom_bert_and_william_return_only_if_they_were_a_creature() {
    cr!("611.3c", "201.5");
    let mut t = TestGame::new(2);
    let tbw = t.battlefield(P0, "Tom, Bert, and William");
    t.g.destroy(tbw, None);
    t.resolve_all();
    let back = t.named_on_battlefield("Tom, Bert, and William");
    assert_eq!(back.len(), 1);
    assert!(is(&mut t, back[0], CardType::Artifact));
    // As a noncreature artifact, it isn't put into the graveyard by "die" again.
    t.g.destroy(back[0], None);
    t.resolve_all();
    assert!(t.named_on_battlefield("Tom, Bert, and William").is_empty());
    assert!(t.in_graveyard(P0, "Tom, Bert, and William"));
}

#[test]
fn drana_returns_a_creature_card_as_a_vampire_with_a_counter() {
    cr!("611.3c", "205.1b");
    assert_supported("Drana, the Last Bloodchief");
    let mut t = TestGame::new(2);
    let drana = t.battlefield(P0, "Drana, the Last Bloodchief");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_choose(P1, &[Entity::Object(bears)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(drana, Entity::Player(P1))], &[]);
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.counters(back[0], "+1/+1"), 1);
    assert!(has_subtype(&mut t, back[0], "Vampire"));
    assert!(has_subtype(&mut t, back[0], "Bear"));
}

#[test]
fn ghouls_night_out_returns_black_zombies_with_decayed() {
    cr!("611.3c", "105.3", "205.1b");
    assert_supported("Ghouls' Night Out");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    t.lands(P0, "Swamp", 5);
    let s = t.hand(P0, "Ghouls' Night Out");
    t.cast(P0, s).go();
    t.resolve_all();
    for name in ["Grizzly Bears", "Hill Giant"] {
        let v = t.named_on_battlefield(name);
        assert_eq!(v.len(), 1, "{name}");
        assert_eq!(t.g.obj(v[0]).controller, P0);
        assert!(has_subtype(&mut t, v[0], "Zombie"));
        assert!(has_kw(&mut t, v[0], KeywordKind::Decayed));
        t.g.recompute();
        let colors = t.obj_now(v[0]).chars.colors;
        let own = if name == "Grizzly Bears" { Color::Green } else { Color::Red };
        assert!(colors.contains(Color::Black) && colors.contains(own), "{name}");
    }
}

#[test]
fn brilliance_unleashed_returns_a_noncreature_artifact_as_a_flying_robot() {
    cr!("611.3c", "700.2");
    assert_supported("Brilliance Unleashed");
    for (card, robot) in [("Mind Stone", true), ("Ornithopter", false)] {
        let mut t = TestGame::new(2);
        let a = t.graveyard(P0, card);
        t.lands(P0, "Island", 3);
        t.lands(P0, "Mountain", 3);
        let s = t.hand(P0, "Brilliance Unleashed");
        t.cast(P0, s).modes(&[1]).target(a).go();
        t.resolve_all();
        let v = t.named_on_battlefield(card);
        assert_eq!(v.len(), 1, "{card}");
        assert!(is(&mut t, v[0], CardType::Creature));
        assert_eq!(has_subtype(&mut t, v[0], "Robot"), robot, "{card}");
        if robot {
            assert_eq!(t.pt(v[0]), (3, 3));
            assert!(has_kw(&mut t, v[0], KeywordKind::Flying));
        } else {
            assert_eq!(t.pt(v[0]), (0, 2));
        }
    }
}

#[test]
fn the_master_returns_a_milled_creature_as_a_green_mutant() {
    cr!("611.3c", "613.4b", "105.3");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "The Master, Transcendent");
    let giant = t.library_top(P1, "Serra Angel");
    // A creature card that wasn't milled isn't a legal target.
    let other = t.graveyard(P1, "Hill Giant");
    t.g.mill(P1, 1);
    t.settle();
    let giant = t.g.current(giant);
    t.activate(P0, m, 0, &[Entity::Object(giant)]).unwrap();
    let cands = last_target_candidates(&t, P0);
    assert!(cands.contains(&Entity::Object(giant)) && !cands.contains(&Entity::Object(other)));
    t.resolve_all();
    let v = t.named_on_battlefield("Serra Angel");
    assert_eq!(v.len(), 1);
    assert_eq!(t.g.obj(v[0]).controller, P0);
    assert!(has_subtype(&mut t, v[0], "Mutant"));
    assert!(!has_subtype(&mut t, v[0], "Angel"));
    assert_eq!(t.pt(v[0]), (3, 3));
    t.g.recompute();
    assert_eq!(t.obj_now(v[0]).chars.colors, ColorSet::single(Color::Green));
    assert!(has_kw(&mut t, v[0], KeywordKind::Flying));
}

#[test]
fn woodcaller_automaton_prototype_animates_a_land() {
    cr!("613.4b", "205.1b");
    assert_supported("Woodcaller Automaton");
    let mut t = TestGame::new(2);
    let forests = t.lands(P0, "Forest", 4);
    let wc = t.hand(P0, "Woodcaller Automaton");
    t.answer_targets(P0, &[Entity::Object(forests[0])]);
    t.cast(P0, wc).method(mtg_engine::object::CastMethod::Keyword(KeywordKind::Prototype)).go();
    t.resolve_all();
    let land = forests[0];
    assert!(!t.g.obj(land).tapped);
    assert!(is(&mut t, land, CardType::Creature));
    assert!(is(&mut t, land, CardType::Land));
    assert!(has_subtype(&mut t, land, "Treefolk"));
    assert!(has_subtype(&mut t, land, "Forest"));
    assert!(has_kw(&mut t, land, KeywordKind::Haste));
    assert_eq!(t.pt(land), (3, 3));
}

#[test]
fn cavernous_maw_becomes_a_creature_that_is_still_a_cave() {
    cr!("205.1b");
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Cavernous Maw");
    t.lands(P0, "Wastes", 2);
    // Two Caves: not enough.
    t.battlefield(P0, "Cavernous Maw");
    t.graveyard(P0, "Cavernous Maw");
    assert!(t.activate(P0, maw, 1, &[]).is_err());
    t.graveyard(P0, "Cavernous Maw");
    t.activate(P0, maw, 1, &[]).unwrap();
    t.resolve_all();
    assert!(is(&mut t, maw, CardType::Creature));
    assert!(is(&mut t, maw, CardType::Land));
    assert!(has_subtype(&mut t, maw, "Cave"));
    assert!(has_subtype(&mut t, maw, "Elemental"));
    assert_eq!(t.pt(maw), (3, 3));
}

// ---------------------------------------------------------------------------
// Self-references: pronouns, plural names, names that are subtypes
// ---------------------------------------------------------------------------

#[test]
fn kaito_is_a_creature_with_hexproof_only_during_your_turn() {
    cr!("201.5", "205.1a");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kaito, Bane of Nightmares");
    // He stops being a planeswalker while he's a creature (CR 205.1a).
    assert!(is(&mut t, k, CardType::Creature));
    assert!(!is(&mut t, k, CardType::Planeswalker));
    assert!(has_subtype(&mut t, k, "Ninja"));
    assert!(has_kw(&mut t, k, KeywordKind::Hexproof));
    assert_eq!(t.pt(k), (3, 4));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!is(&mut t, k, CardType::Creature));
    assert!(is(&mut t, k, CardType::Planeswalker));
    assert!(!has_kw(&mut t, k, KeywordKind::Hexproof));
    // Without loyalty counters, not even during your turn.
    t.set_step(P0, Step::PrecombatMain);
    t.g.objects[k.0 as usize].counters.insert("loyalty".into(), 0);
    t.g.dirty = true;
    assert!(!is(&mut t, k, CardType::Creature));
}

#[test]
fn garruk_relentless_transforms_at_two_or_fewer_loyalty() {
    cr!("201.5", "603.8");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Garruk Relentless");
    let giant = t.battlefield(P1, "Hill Giant");
    assert_eq!(t.counters(g, "loyalty"), 3);
    t.settle();
    assert_eq!(t.g.obj(g).face, FaceState::Front);
    // "Garruk deals 3 damage to target creature. That creature deals damage equal to its
    // power to him."
    t.activate(P0, g, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.counters(g, "loyalty"), 0);
    // With 0 loyalty, he's put into the graveyard before the trigger resolves.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Garruk Relentless");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let _ = bears;
    t.g.objects[g.0 as usize].counters.insert("loyalty".into(), 2);
    t.resolve_all();
    assert_eq!(t.g.obj(g).face, FaceState::Back);
    t.g.recompute();
    assert_eq!(t.obj_now(g).chars.name, "Garruk, the Veil-Cursed");
}

#[test]
fn sarkhan_the_mad_deals_damage_to_himself() {
    cr!("201.5", "306.8");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sarkhan the Mad");
    t.library_top(P0, "Hill Giant");
    t.activate(P0, s, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    // 7 loyalty, 0 cost, 4 damage from Hill Giant's mana value.
    assert_eq!(t.counters(s, "loyalty"), 3);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn notary_hobbits_create_two_nonlegendary_token_copies() {
    cr!("201.5", "707.9b");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let h = t.hand(P0, "The Notary Hobbits");
    t.cast(P0, h).go();
    t.resolve_all();
    let all = t.named_on_battlefield("The Notary Hobbits");
    assert_eq!(all.len(), 3);
    let tokens: Vec<_> = all.iter().filter(|o| t.g.obj(**o).is_token()).copied().collect();
    assert_eq!(tokens.len(), 2);
    t.g.recompute();
    for tok in tokens {
        assert!(!t.obj_now(tok).chars.supertypes.contains(Supertype::Legendary));
    }
    // A token copy entering doesn't make more ("if they're not a token").
    assert_eq!(t.named_on_battlefield("The Notary Hobbits").len(), 3);
}

#[test]
fn garruk_cursed_huntsman_wolves_feed_each_garruk() {
    cr!("201.5");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Garruk, Cursed Huntsman");
    let other = t.battlefield(P0, "Garruk Relentless");
    t.activate(P0, g, 0, &[]).unwrap();
    t.resolve_all();
    let wolves: Vec<ObjectId> = t
        .g
        .battlefield
        .clone()
        .into_iter()
        .filter(|o| t.g.obj(*o).chars.subtypes.iter().any(|s| s == "Wolf"))
        .collect();
    assert_eq!(wolves.len(), 2);
    t.g.destroy(wolves[0], None);
    t.resolve_all();
    assert_eq!(t.counters(g, "loyalty"), 6);
    assert_eq!(t.counters(other, "loyalty"), 4);
}

#[test]
fn icingdeath_creates_its_frost_tongue_token() {
    cr!("201.5", "111.4");
    let mut t = TestGame::new(2);
    let i = t.battlefield(P0, "Icingdeath, Frost Tyrant");
    t.g.destroy(i, None);
    t.resolve_all();
    let tok = named(&t, "Icingdeath, Frost Tongue");
    assert_eq!(tok.len(), 1);
    assert!(is(&mut t, tok[0], CardType::Artifact));
    assert!(has_subtype(&mut t, tok[0], "Equipment"));
    t.g.recompute();
    assert!(t.obj_now(tok[0]).chars.supertypes.contains(Supertype::Legendary));
}

#[test]
fn gideon_champion_of_justice_is_a_creature_with_power_equal_to_loyalty() {
    cr!("201.5", "615.1");
    assert_supported("Gideon, Champion of Justice");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gideon, Champion of Justice");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.activate(P0, g, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(g, "loyalty"), 4 + 1 + 2);
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gideon, Champion of Justice");
    t.activate(P0, g, 1, &[]).unwrap();
    t.resolve_all();
    assert!(is(&mut t, g, CardType::Creature));
    assert!(is(&mut t, g, CardType::Planeswalker));
    assert_eq!(t.pt(g), (4, 4));
    assert!(has_kw(&mut t, g, KeywordKind::Indestructible));
    // Damage to him is prevented.
    let src = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(src, Entity::Object(g), 3, false);
    t.settle();
    assert_eq!(t.counters(g, "loyalty"), 4);
}

#[test]
fn jace_reality_sculptor_empowers_by_islands_and_counts_jaces() {
    cr!("201.5", "107.1");
    assert_supported("Jace, Reality Sculptor");
    let mut t = TestGame::new(2);
    let j = t.battlefield(P0, "Jace, Reality Sculptor");
    t.lands(P0, "Island", 3);
    // The 0 ability needs twenty-five loyalty counters among Jaces.
    assert!(t.activate(P0, j, 2, &[]).is_err());
    t.g.objects[j.0 as usize].counters.insert("loyalty".into(), 25);
    t.g.dirty = true;
    t.activate(P0, j, 2, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.library_size(P1), 1);
}

// ---------------------------------------------------------------------------
// Keywords copied from exiled cards
// ---------------------------------------------------------------------------

#[test]
fn urborg_scavengers_has_keywords_of_cards_it_exiled() {
    cr!("702.1c", "607.2a");
    let mut t = TestGame::new(2);
    let drake = t.graveyard(P1, "Wind Drake");
    t.lands(P0, "Swamp", 3);
    let us = t.hand(P0, "Urborg Scavengers");
    t.answer_targets(P0, &[Entity::Object(drake)]);
    let us_spell = t.cast(P0, us).go();
    let _ = us_spell;
    t.resolve_all();
    let us = t.named_on_battlefield("Urborg Scavengers")[0];
    assert!(t.in_exile("Wind Drake"));
    assert!(has_kw(&mut t, us, KeywordKind::Flying));
    assert!(!has_kw(&mut t, us, KeywordKind::Trample));
    assert_eq!(t.counters(us, "+1/+1"), 1);
    // A card exiled by something else doesn't count.
    let mut t = TestGame::new(2);
    let us = t.battlefield(P0, "Urborg Scavengers");
    t.exile(P1, "Wind Drake");
    assert!(!has_kw(&mut t, us, KeywordKind::Flying));
}

#[test]
fn eater_of_virtue_gives_the_equipped_creature_keywords_of_exiled_cards() {
    cr!("702.1c", "607.2a");
    let mut t = TestGame::new(2);
    let eater = t.battlefield(P0, "Eater of Virtue");
    let drake = t.battlefield(P0, "Wind Drake");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.attach(eater, Entity::Object(drake));
    t.g.destroy(drake, None);
    t.resolve_all();
    assert!(t.in_exile("Wind Drake"));
    t.g.attach(eater, Entity::Object(bears));
    assert!(has_kw(&mut t, bears, KeywordKind::Flying));
    assert_eq!(t.pt(bears), (4, 2));
}

#[test]
fn rayami_has_keywords_of_exiled_cards_with_blood_counters() {
    cr!("702.1c");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Rayami, First of the Fallen");
    let drake = t.battlefield(P1, "Wind Drake");
    assert!(!has_kw(&mut t, r, KeywordKind::Flying));
    t.g.destroy(drake, None);
    t.resolve_all();
    assert!(t.in_exile("Wind Drake"));
    assert!(has_kw(&mut t, r, KeywordKind::Flying));
    // An exiled card without a blood counter doesn't count.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Rayami, First of the Fallen");
    t.exile(P1, "Wind Drake");
    assert!(!has_kw(&mut t, r, KeywordKind::Flying));
}

#[test]
fn escaped_shapeshifter_ignores_creatures_named_escaped_shapeshifter() {
    cr!("702.1c");
    let mut t = TestGame::new(2);
    let es = t.battlefield(P0, "Escaped Shapeshifter");
    let theirs = t.battlefield(P1, "Escaped Shapeshifter");
    t.battlefield(P0, "Wind Drake");
    // My Wind Drake gives their Shapeshifter flying; that one doesn't give mine flying.
    assert!(has_kw(&mut t, theirs, KeywordKind::Flying));
    assert!(!has_kw(&mut t, es, KeywordKind::Flying));
}

// ---------------------------------------------------------------------------
// Other keyword statics
// ---------------------------------------------------------------------------

#[test]
fn armament_of_nyx_double_strike_or_prevention() {
    cr!("613.1f", "615.1");
    let mut t = TestGame::new(2);
    let aura = t.battlefield(P0, "Armament of Nyx");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.attach(aura, Entity::Object(bears));
    assert!(!has_kw(&mut t, bears, KeywordKind::DoubleStrike));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
    // An enchantment creature has double strike.
    let mut t = TestGame::new(2);
    let aura = t.battlefield(P0, "Armament of Nyx");
    let nyx = t.battlefield(P0, "Nyxborn Rollicker");
    t.g.attach(aura, Entity::Object(nyx));
    assert!(has_kw(&mut t, nyx, KeywordKind::DoubleStrike));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(nyx, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn doors_of_durin_attacker_gains_keywords_by_dwarves_and_elves() {
    cr!("613.1f", "611.2c");
    assert_supported("Doors of Durin");
    for (dwarf, elf) in [(true, false), (false, true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Doors of Durin");
        let attacker = t.battlefield(P0, "Hill Giant");
        if dwarf {
            t.battlefield(P0, "Dwarven Trader");
        }
        if elf {
            t.battlefield(P0, "Llanowar Elves");
        }
        t.library_top(P0, "Serra Angel");
        t.answer(P0, DecisionKind::Scry, Answer::Default);
        t.answer_yes(P0, true);
        t.set_step(P0, Step::BeginningOfCombat);
        t.answer(
            P0,
            DecisionKind::Attackers,
            Answer::Attackers(vec![(attacker, Entity::Player(P1))]),
        );
        t.advance_to_step(Step::DeclareBlockers);
        t.resolve_all();
        let angel = t.named_on_battlefield("Serra Angel");
        assert_eq!(angel.len(), 1, "dwarf {dwarf}");
        assert_eq!(has_kw(&mut t, angel[0], KeywordKind::Trample), dwarf);
        assert_eq!(has_kw(&mut t, angel[0], KeywordKind::Hexproof), elf);
    }
}

#[test]
fn pride_of_hull_clade_gives_a_defender_card_draw_and_lets_it_attack() {
    cr!("702.3b", "613.1f");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Wood");
    let p = t.battlefield(P0, "The Pride of Hull Clade");
    t.lands(P0, "Island", 4);
    t.activate(P0, p, 0, &[Entity::Object(wall)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(wall), (1, 3));
    let hand = t.hand_size(P0);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(wall, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn llanowar_greenwidow_returns_and_is_exiled_when_it_would_leave() {
    cr!("614.1a", "602.2b");
    let mut t = TestGame::new(2);
    let w = t.graveyard(P0, "Llanowar Greenwidow");
    t.lands(P0, "Forest", 6);
    t.battlefield(P0, "Plains");
    // {7}{G} minus 2 for two basic land types.
    t.activate(P0, w, 0, &[]).unwrap();
    t.resolve_all();
    let v = t.named_on_battlefield("Llanowar Greenwidow");
    assert_eq!(v.len(), 1);
    assert!(t.g.obj(v[0]).tapped);
    t.g.destroy(v[0], None);
    t.resolve_all();
    assert!(t.in_exile("Llanowar Greenwidow"));
}

#[test]
fn diligent_farmhand_counts_for_muscle_burst() {
    cr!("201.2a", "113.6b");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Diligent Farmhand");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let mb = t.hand(P0, "Muscle Burst");
    t.cast(P0, mb).target(bears).go();
    t.resolve_all();
    // 3 plus the Farmhand.
    assert_eq!(t.pt(bears), (2 + 4, 2 + 4));
    // On the battlefield it doesn't count.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Diligent Farmhand");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let mb = t.hand(P0, "Muscle Burst");
    t.cast(P0, mb).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (2 + 3, 2 + 3));
}

#[test]
fn sound_the_call_wolves_count_cards_named_sound_the_call() {
    cr!("201.2");
    assert_supported("Sound the Call");
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Sound the Call");
    t.lands(P0, "Forest", 3);
    let s = t.hand(P0, "Sound the Call");
    t.cast(P0, s).go();
    t.resolve_all();
    let wolf: Vec<ObjectId> = t
        .g
        .battlefield
        .clone()
        .into_iter()
        .filter(|o| t.g.obj(*o).chars.subtypes.iter().any(|s| s == "Wolf"))
        .collect();
    assert_eq!(wolf.len(), 1);
    // The one in P1's graveyard and the resolved one.
    assert_eq!(t.pt(wolf[0]), (3, 3));
}

#[test]
fn ms_marvel_base_power_is_hand_size() {
    cr!("613.4b", "604.3a");
    assert_supported("Ms. Marvel, Kamala Khan");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Ms. Marvel, Kamala Khan");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(m).go();
    t.resolve_all();
    // Drew a card: 3 in hand, +3/+3 from Giant Growth.
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.pt(m), (3 + 3, 4 + 3));
    t.hand(P0, "Grizzly Bears");
    assert_eq!(t.pt(m), (4 + 3, 4 + 3));
}

#[test]
fn the_irencrag_becomes_everflame() {
    cr!("613.1f", "205.1a");
    assert_supported("The Irencrag");
    let mut t = TestGame::new(2);
    let i = t.battlefield(P0, "The Irencrag");
    t.answer_yes(P0, true);
    t.enter(P0, "Isamaru, Hound of Konda");
    t.resolve_all();
    t.g.recompute();
    let c = t.obj_now(i).chars.clone();
    assert_eq!(c.name, "Everflame, Heroes' Legacy");
    assert!(c.subtypes.iter().any(|s| s == "Equipment"));
    assert!(c.supertypes.contains(Supertype::Legendary));
    // Only equip and the pump: no mana ability.
    assert!(!c.abilities.iter().any(|a| matches!(&a.kind, ability::AbilityKind::Activated(x) if x.is_mana_ability)));
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.attach(i, Entity::Object(bears));
    t.g.recompute();
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn mirror_golem_has_protection_from_the_exiled_cards_types() {
    cr!("702.16a", "607.2a");
    assert_supported("Mirror Golem");
    let mut t = TestGame::new(2);
    let bolt = t.graveyard(P1, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Object(bolt)]);
    t.answer_yes(P0, true);
    let g = t.enter(P0, "Mirror Golem");
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt"));
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let b2 = t.hand(P1, "Lightning Bolt");
    t.cast(P1, b2).target(P0).go();
    // Protection from instants: not a legal target for Lightning Bolt.
    assert!(!last_target_candidates(&t, P1).contains(&Entity::Object(g)));
    assert!(t.g.protected_from(g, b2));
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(!t.g.protected_from(g, bears));
}

// ---------------------------------------------------------------------------
// Craft, granted keywords, modes as bundles, flavor words
// ---------------------------------------------------------------------------

/// The uid of `src`'s activated ability whose text starts with `prefix`.
fn ability_uid(t: &mut TestGame, src: ObjectId, prefix: &str) -> u64 {
    t.g.recompute();
    t.g.obj(t.g.current(src))
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, ability::AbilityKind::Activated(_)) && a.text.starts_with(prefix))
        .map(|a| a.uid)
        .unwrap_or_else(|| panic!("no ability {prefix:?}"))
}

/// P0 activates `src`'s craft ability exiling `materials`, and it resolves.
fn craft(t: &mut TestGame, src: ObjectId, materials: &[ObjectId]) {
    let es: Vec<Entity> = materials.iter().map(|o| Entity::Object(*o)).collect();
    t.answer_choose(P0, &es);
    let uid = ability_uid(t, src, "Craft");
    t.g.turn.priority = Some(P0);
    t.g.activate_ability(P0, src, uid).expect("craft");
    t.clear_answers();
    t.resolve_all();
}

#[test]
fn wretched_bonemass_has_keywords_of_the_cards_used_to_craft_it() {
    cr!("702.167c", "702.1c");
    let mut t = TestGame::new(2);
    let altar = t.battlefield(P0, "Altar of the Wretched // Wretched Bonemass");
    let drake = t.graveyard(P0, "Wind Drake");
    t.lands(P0, "Swamp", 4);
    craft(&mut t, altar, &[drake]);
    let b = t.named_on_battlefield("Wretched Bonemass");
    assert_eq!(b.len(), 1);
    assert!(has_kw(&mut t, b[0], KeywordKind::Flying));
    assert!(!has_kw(&mut t, b[0], KeywordKind::Trample));
    assert_eq!(t.pt(b[0]), (2, 2));
}

#[test]
fn locus_of_enlightenment_activates_each_gained_ability_once_each_turn() {
    cr!("702.167c", "113.10a");
    let mut t = TestGame::new(2);
    let jewel = t.battlefield(P0, "The Enigma Jewel // Locus of Enlightenment");
    let mats = [
        t.battlefield(P0, "Llanowar Elves"),
        t.battlefield(P0, "Birds of Paradise"),
        t.battlefield(P0, "Shivan Dragon"),
        t.graveyard(P0, "Prodigal Sorcerer"),
    ];
    t.lands(P0, "Island", 7);
    craft(&mut t, jewel, &mats);
    let locus = t.g.current(jewel);
    assert_eq!(t.g.obj(locus).chars.name, "Locus of Enlightenment");
    t.lands(P0, "Mountain", 3);
    let firebreathing = ability_uid(&mut t, locus, "{R}");
    t.g.turn.priority = Some(P0);
    t.g.activate_ability(P0, locus, firebreathing).expect("first activation");
    t.resolve_all();
    t.g.turn.priority = Some(P0);
    assert!(t.g.activate_ability(P0, locus, firebreathing).is_err());
    // Next turn, again.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    assert!(t.g.activate_ability(P0, locus, firebreathing).is_ok());
}

#[test]
fn outlaws_merriment_creates_one_of_the_listed_tokens() {
    cr!("700.2", "111.4");
    ruling!("Outlaws' Merriment", "you choose a mode at random");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Outlaws' Merriment");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let toks: Vec<ObjectId> = t
        .g
        .battlefield
        .clone()
        .into_iter()
        .filter(|o| t.g.obj(*o).is_token())
        .collect();
    assert_eq!(toks.len(), 1);
    let tok = toks[0];
    t.g.recompute();
    let c = t.obj_now(tok).chars.clone();
    assert!(c.colors.contains(Color::Red) && c.colors.contains(Color::White));
    assert!(c.subtypes.iter().any(|s| s == "Human"));
    assert!(c.has_keyword(KeywordKind::Haste));
    let pt = t.pt(tok);
    if c.subtypes.iter().any(|s| s == "Warrior") {
        assert_eq!(pt, (3, 1));
        assert!(c.has_keyword(KeywordKind::Trample));
    } else if c.subtypes.iter().any(|s| s == "Cleric") {
        assert_eq!(pt, (2, 1));
        assert!(c.has_keyword(KeywordKind::Lifelink));
    } else {
        assert!(c.subtypes.iter().any(|s| s == "Rogue"));
        assert_eq!(pt, (1, 2));
        assert_eq!(t.life(P1), 19);
    }
}

#[test]
fn balduvian_shaman_gives_cumulative_upkeep() {
    cr!("702.24a", "113.10");
    ruling!("Balduvian Shaman", "Can target a card with no appropriate words on it");
    assert_supported("Balduvian Shaman");
    let mut t = TestGame::new(2);
    let shaman = t.battlefield(P0, "Balduvian Shaman");
    let anthem = t.battlefield(P0, "Glorious Anthem");
    let theirs = t.battlefield(P1, "Glorious Anthem");
    let pacifism = t.battlefield(P0, "Pacifism");
    let _ = pacifism;
    t.activate(P0, shaman, 0, &[Entity::Object(anthem)]).unwrap();
    // Only white enchantments you control.
    let cands = last_target_candidates(&t, P0);
    assert!(!cands.contains(&Entity::Object(theirs)));
    t.resolve_all();
    assert!(has_kw(&mut t, anthem, KeywordKind::CumulativeUpkeep));
    // At the next upkeep it gets an age counter; not paying sacrifices it.
    t.answer_yes(P0, false);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(anthem));
    // An enchantment that has cumulative upkeep isn't a legal target.
    let mut t = TestGame::new(2);
    let shaman = t.battlefield(P0, "Balduvian Shaman");
    let a = t.battlefield(P0, "Glorious Anthem");
    t.activate(P0, shaman, 0, &[Entity::Object(a)]).unwrap();
    t.resolve_all();
    t.g.objects[shaman.0 as usize].tapped = false;
    let b = t.battlefield(P0, "Glorious Anthem");
    t.activate(P0, shaman, 0, &[Entity::Object(b)]).unwrap();
    let cands = last_target_candidates(&t, P0);
    assert!(cands.contains(&Entity::Object(b)) && !cands.contains(&Entity::Object(a)));
}

#[test]
fn aloy_flavor_word_with_a_comma_discovers() {
    cr!("207.2d", "701.57a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aloy, Savior of Meridian");
    let sable = t.battlefield(P0, "Bronze Sable");
    t.library_top(P0, "Grizzly Bears");
    t.answer_yes(P0, false);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(sable, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears") || !t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn cosima_voyages_and_returns_with_counters() {
    cr!("113.6b", "113.10");
    assert_supported("Cosima, God of the Voyage // The Omenkeel");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Cosima, God of the Voyage // The Omenkeel");
    t.answer_yes(P0, true);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let exiled = t.g.current(c);
    assert_eq!(t.zone(exiled), mtg_engine::object::Zone::Exile);
    // A land enters: put a voyage counter on it.
    t.answer_yes(P0, true);
    t.enter(P0, "Forest");
    t.resolve_all();
    assert_eq!(t.counters(exiled, "voyage"), 1);
    t.answer_yes(P0, true);
    t.enter(P0, "Forest");
    t.resolve_all();
    assert_eq!(t.counters(exiled, "voyage"), 2);
    // The next time, don't: it returns with two +1/+1 counters and you draw two.
    let hand = t.hand_size(P0);
    t.answer_yes(P0, false);
    t.enter(P0, "Forest");
    t.resolve_all();
    let back = t.named_on_battlefield("Cosima, God of the Voyage");
    assert_eq!(back.len(), 1);
    assert_eq!(t.counters(back[0], "+1/+1"), 2);
    assert_eq!(t.hand_size(P0), hand + 2);
    // A land entering now doesn't trigger the ability it had in exile.
    t.enter(P0, "Forest");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Cosima, God of the Voyage").len(), 1);
}

// ---------------------------------------------------------------------------
// Rule-modifying statics: choices an opponent's effect offers
// ---------------------------------------------------------------------------

#[test]
fn tamiyo_you_cant_choose_to_discard_for_an_opponents_punisher() {
    cr!("701.9a", "101.2");
    ruling!(
        "Tamiyo, Collector of Tales",
        "you can’t choose to sacrifice or discard"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tamiyo, Collector of Tales");
    t.battlefield(P1, "Painful Quandary");
    t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    // P0 would rather discard, but can't: they lose 5 life.
    t.answer_yes(P0, true);
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.life(P0), 15);
    // Without Tamiyo, the discard is allowed.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Painful Quandary");
    t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn ward_triggers_without_nowhere_to_run() {
    cr!("702.21a");
    // The counterpart of the Nowhere to Run test: here ward triggers and counters the
    // spell whose cost isn't paid.
    let mut t = TestGame::new(2);
    let warded = t.battlefield(P1, "Tomakul Honor Guard");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(warded).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(t.on_battlefield(warded));
}
