//! CR 702.22 Banding and "bands with other".

use crate::common_k702_011_017::*;
use crate::common_k702_018_026::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kw::banding::{band_mates, has_banding};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn band_of(t: &TestGame, a: ObjectId) -> Option<u32> {
    t.g.combat
        .as_ref()
        .and_then(|c| c.attacker(a))
        .and_then(|x| x.band)
}

fn blockers_of(t: &TestGame, a: ObjectId) -> Vec<ObjectId> {
    t.g.combat
        .as_ref()
        .map(|c| c.blockers_of(a))
        .unwrap_or_default()
}

fn is_blocked(t: &TestGame, a: ObjectId) -> bool {
    t.g.combat.as_ref().is_some_and(|c| c.is_blocked(a))
}

/// Declares attackers with a band: `leader` bands with `others` (all attacking `target`),
/// plus any `more` attackers.
fn attack_in_band(
    t: &mut TestGame,
    leader: ObjectId,
    others: &[ObjectId],
    target: Entity,
    more: &[(ObjectId, Entity)],
) {
    let chosen: Vec<Entity> = others.iter().map(|o| Entity::Object(*o)).collect();
    t.answer_choose(P0, &chosen);
    let mut decl = vec![(leader, target)];
    decl.extend(others.iter().map(|o| (*o, target)));
    decl.extend_from_slice(more);
    attack_with(t, &decl);
}

/// "Target creature loses banding until end of turn."
fn lose_banding_spell(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let def = custom_card(
        "Disband",
        "Instant",
        None,
        "Target creature loses banding and all \"bands with other\" abilities until end of turn.",
    );
    t.custom(p, def, object::Zone::Hand(p))
}

#[test]
fn blocking_any_creature_in_a_band_blocks_the_band() {
    cr!("702.22", "702.22a", "702.22c", "702.22h");
    ruling!(
        "Nalathni Dragon",
        "Blocking any creature in a band blocks the entire band"
    );
    assert_supported("Benalish Hero");
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Benalish Hero");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_in_band(&mut t, hero, &[bears], Entity::Player(P1), &[]);
    assert!(band_of(&t, hero).is_some());
    assert_eq!(band_of(&t, hero), band_of(&t, bears));
    declare_blocks(&mut t, P1, &[(giant, hero)]);
    assert_eq!(blockers_of(&t, hero), vec![giant]);
    assert_eq!(blockers_of(&t, bears), vec![giant]);
    assert!(is_blocked(&t, bears));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn banding_doesnt_share_abilities() {
    cr!("702.22g", "702.22h");
    ruling!(
        "Banding Sliver",
        "Creatures don’t “share” abilities in a band"
    );
    let mut t = TestGame::new(2);
    let pegasus = t.battlefield(P0, "Mesa Pegasus");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_in_band(&mut t, pegasus, &[bears], Entity::Player(P1), &[]);
    // The bears don't gain flying; the pegasus keeps it and can't be blocked by the giant.
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Flying));
    assert!(t.obj_now(pegasus).has_keyword(KeywordKind::Flying));
    assert!(!t.g.can_block(giant, pegasus));
    // Blocking the bears blocks the flying pegasus too.
    declare_blocks(&mut t, P1, &[(giant, bears)]);
    assert!(is_blocked(&t, pegasus));
    assert_eq!(blockers_of(&t, pegasus), vec![giant]);
}

#[test]
fn a_band_has_at_most_one_creature_without_banding() {
    cr!("702.22c");
    ruling!(
        "Nalathni Dragon",
        "A maximum of one nonbanding creature can join an attacking band"
    );
    // Any number with banding.
    let mut t = TestGame::new(2);
    let h1 = t.battlefield(P0, "Benalish Hero");
    let h2 = t.battlefield(P0, "Benalish Hero");
    let h3 = t.battlefield(P0, "Timber Wolves");
    attack_in_band(&mut t, h1, &[h2, h3], Entity::Player(P1), &[]);
    assert!(band_of(&t, h1).is_some());
    assert_eq!(band_of(&t, h3), band_of(&t, h1));

    // A creature with "bands with other" counts as one without banding.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Adventurers' Guildhouse");
    let hero = t.battlefield(P0, "Benalish Hero");
    let ayula = t.battlefield(P0, "Ayula, Queen Among Bears");
    let fynn = t.battlefield(P0, "Fynn, the Fangbearer");
    assert!(!has_banding(&t.g, ayula));
    attack_in_band(&mut t, hero, &[ayula, fynn], Entity::Player(P1), &[]);
    assert_eq!(band_of(&t, hero), None);
    assert_eq!(band_of(&t, ayula), None);
}

#[test]
fn bands_with_other_bands_with_creatures_of_its_quality() {
    cr!("702.22b", "702.22c");
    assert_supported("Adventurers' Guildhouse");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Adventurers' Guildhouse");
    let ayula = t.battlefield(P0, "Ayula, Queen Among Bears");
    let fynn = t.battlefield(P0, "Fynn, the Fangbearer");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    assert_eq!(keyword_count(&t, ayula, KeywordKind::Banding), 1);
    // Isamaru is legendary but not green: it doesn't have the ability, but it's a
    // legendary creature, so it can join the band.
    assert_eq!(keyword_count(&t, isamaru, KeywordKind::Banding), 0);
    attack_in_band(&mut t, ayula, &[fynn, isamaru], Entity::Player(P1), &[]);
    let band = band_of(&t, ayula);
    assert!(band.is_some());
    assert_eq!(band_of(&t, fynn), band);
    assert_eq!(band_of(&t, isamaru), band);

    // A creature that isn't legendary can't join.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Adventurers' Guildhouse");
    let ayula = t.battlefield(P0, "Ayula, Queen Among Bears");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_in_band(&mut t, ayula, &[bears], Entity::Player(P1), &[]);
    assert_eq!(band_of(&t, ayula), None);
}

#[test]
fn losing_banding_removes_bands_with_other() {
    cr!("702.22b");
    // An effect that removes only banding removes "bands with other" too.
    let def = custom_card(
        "Unband",
        "Instant",
        None,
        "Target creature loses banding until end of turn.",
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Adventurers' Guildhouse");
    let ayula = t.battlefield(P0, "Ayula, Queen Among Bears");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    assert_eq!(keyword_count(&t, ayula, KeywordKind::Banding), 1);
    assert!(mtg_engine::kw::banding::legal_band(&t.g, &[ayula, isamaru]));
    let spell = t.custom(P1, def, object::Zone::Hand(P1));
    t.cast(P1, spell).target(ayula).go();
    t.resolve_all();
    assert_eq!(keyword_count(&t, ayula, KeywordKind::Banding), 0);
    assert!(mtg_engine::kw::banding::bands_with_other(&t.g, ayula).is_empty());
    // Without its "bands with other legendary creatures", Ayula can't band with the
    // legendary Isamaru.
    assert!(!mtg_engine::kw::banding::legal_band(&t.g, &[ayula, isamaru]));
    attack_in_band(&mut t, ayula, &[isamaru], Entity::Player(P1), &[]);
    assert!(t.g.is_attacking(ayula) && t.g.is_attacking(isamaru));
    assert_eq!(band_of(&t, ayula), None);
    assert_eq!(band_of(&t, isamaru), None);

    // "Loses banding and all "bands with other" abilities" says the same.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Adventurers' Guildhouse");
    let ayula = t.battlefield(P0, "Ayula, Queen Among Bears");
    let spell = lose_banding_spell(&mut t, P1);
    t.cast(P1, spell).target(ayula).go();
    t.resolve_all();
    assert_eq!(keyword_count(&t, ayula, KeywordKind::Banding), 0);
}

#[test]
fn a_band_attacks_a_single_player_or_planeswalker() {
    cr!("702.22d");
    ruling!(
        "Nalathni Dragon",
        "Creatures in the same band must all attack the same player or planeswalker"
    );
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    let hero = t.battlefield(P0, "Benalish Hero");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    attack_with(
        &mut t,
        &[(hero, Entity::Object(jace)), (bears, Entity::Player(P1))],
    );
    assert_eq!(band_of(&t, hero), None);
    assert_eq!(band_of(&t, bears), None);
}

#[test]
fn a_band_lasts_even_if_banding_is_lost() {
    cr!("702.22e");
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Benalish Hero");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_in_band(&mut t, hero, &[bears], Entity::Player(P1), &[]);
    let spell = lose_banding_spell(&mut t, P1);
    t.cast(P1, spell).target(hero).go();
    t.resolve_all();
    assert!(!t.obj_now(hero).has_keyword(KeywordKind::Banding));
    declare_blocks(&mut t, P1, &[(giant, bears)]);
    assert!(is_blocked(&t, hero));
    assert_eq!(blockers_of(&t, hero), vec![giant]);
}

#[test]
fn a_creature_removed_from_combat_leaves_its_band() {
    cr!("702.22f");
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Benalish Hero");
    let h2 = t.battlefield(P0, "Benalish Hero");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_in_band(&mut t, hero, &[h2, bears], Entity::Player(P1), &[]);
    assert_eq!(band_mates(&t.g, bears).len(), 2);
    mtg_engine::combat::remove_from_combat(&mut t.g, hero);
    assert_eq!(band_mates(&t.g, bears), vec![h2]);
    declare_blocks(&mut t, P1, &[(giant, bears)]);
    assert!(is_blocked(&t, h2));
    assert!(!t.g.is_attacking(hero));
    assert!(!t.g.combat.as_ref().unwrap().blocking(giant).contains(&hero));
}

#[test]
fn an_effect_blocking_one_member_blocks_the_band() {
    cr!("702.22i");
    assert_supported("Curtain of Light");
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Benalish Hero");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_in_band(&mut t, hero, &[bears], Entity::Player(P1), &[]);
    declare_blocks(&mut t, P1, &[]);
    t.lands(P1, "Plains", 2);
    let curtain = t.hand(P1, "Curtain of Light");
    t.cast(P1, curtain).target(bears).go();
    t.resolve_all();
    assert!(is_blocked(&t, bears));
    assert!(is_blocked(&t, hero));
    t.advance_to(P0, Step::EndOfCombat);
    // Blocked creatures with no blockers deal no combat damage.
    assert_eq!(t.life(P1), 20);
}

#[test]
fn an_effect_blocking_a_second_target_blocks_its_band() {
    cr!("702.22i");
    let def = custom_card(
        "Ambush Order",
        "Instant",
        None,
        "Target creature gets +1/+1 until end of turn. Target unblocked attacking creature becomes blocked.",
    );
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Benalish Hero");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Hill Giant");
    attack_in_band(&mut t, hero, &[bears], Entity::Player(P1), &[]);
    declare_blocks(&mut t, P1, &[]);
    let spell = t.custom(P1, def, object::Zone::Hand(P1));
    t.cast(P1, spell).target(wall).target(hero).go();
    t.resolve_all();
    assert_eq!(t.pt(wall), (4, 4));
    assert!(is_blocked(&t, hero));
    assert!(is_blocked(&t, bears));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn defending_player_assigns_damage_of_a_creature_blocked_by_banding() {
    cr!("702.22j");
    ruling!(
        "Nalathni Dragon",
        "If a creature in combat has banding, its controller assigns damage for creatures blocking or blocked by it"
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let hero = t.battlefield(P1, "Benalish Hero");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // P1 puts all 3 damage on the Hero, saving the Bears.
    assign_damage(&mut t, P1, &[3, 0]);
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(hero, giant), (bears, giant)]);
    let d = damage_decisions(&t);
    assert!(d.iter().any(|(p, c, _)| *p == P1 && *c == giant));
    assert!(!t.on_battlefield(hero));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn defending_player_assigns_a_tramplers_damage_among_banding_blockers() {
    cr!("702.22j");
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let hero = t.battlefield(P1, "Benalish Hero");
    attack_with(&mut t, &[(maw, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(hero, maw)]);
    // All of it is divided among the blocking creatures.
    assert!(!t.on_battlefield(hero));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn defending_player_assigns_against_a_bands_with_other_group() {
    cr!("702.22j");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Adventurers' Guildhouse");
    let ayula = t.battlefield(P1, "Ayula, Queen Among Bears");
    let fynn = t.battlefield(P1, "Fynn, the Fangbearer");
    // Fynn (1/3) takes all 3; Ayula (2/2) survives.
    assign_damage(&mut t, P1, &[0, 3]);
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(ayula, giant), (fynn, giant)]);
    let d = damage_decisions(&t);
    assert!(d.iter().any(|(p, c, _)| *p == P1 && *c == giant));
    assert!(t.on_battlefield(ayula));
    assert!(!t.on_battlefield(fynn));
}

#[test]
fn active_player_assigns_damage_of_a_creature_blocking_a_band() {
    cr!("702.22k");
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Benalish Hero");
    let hill = t.battlefield(P0, "Hill Giant");
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    attack_in_band(&mut t, hero, &[hill], Entity::Player(P1), &[]);
    declare_blocks(&mut t, P1, &[(maw, hill)]);
    // The Dreadmaw blocks the whole band; P0 assigns its 6 damage, all to the Hero.
    let blocking = t.g.combat.as_ref().unwrap().blocking(maw);
    assert_eq!(blocking.len(), 2);
    let amounts: Vec<i64> = blocking
        .iter()
        .map(|a| if *a == hero { 6 } else { 0 })
        .collect();
    assign_damage(&mut t, P0, &amounts);
    t.advance_to(P0, Step::EndOfCombat);
    let d = damage_decisions(&t);
    assert!(d.iter().any(|(p, c, _)| *p == P0 && *c == maw));
    assert!(!t.on_battlefield(hero));
    assert!(t.on_battlefield(hill));
    assert_eq!(t.obj_now(hill).damage, 0);
}

#[test]
fn multiple_instances_of_banding_are_redundant() {
    cr!("702.22m");
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Benalish Hero");
    let coop = t.battlefield(P0, "Cooperation");
    t.g.attach(coop, Entity::Object(hero));
    t.g.recompute();
    assert_eq!(keyword_count(&t, hero, KeywordKind::Banding), 2);
    let b1 = t.battlefield(P0, "Grizzly Bears");
    let b2 = t.battlefield(P0, "Hill Giant");
    // Still only one creature without banding may join.
    attack_in_band(&mut t, hero, &[b1, b2], Entity::Player(P1), &[]);
    assert_eq!(band_of(&t, hero), None);

    // Two "bands with other legendary creatures" abilities work like one.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Adventurers' Guildhouse");
    t.battlefield(P0, "Adventurers' Guildhouse");
    let ayula = t.battlefield(P0, "Ayula, Queen Among Bears");
    let fynn = t.battlefield(P0, "Fynn, the Fangbearer");
    assert_eq!(keyword_count(&t, ayula, KeywordKind::Banding), 2);
    attack_in_band(&mut t, ayula, &[fynn], Entity::Player(P1), &[]);
    assert!(band_of(&t, ayula).is_some());
    assert_eq!(band_of(&t, fynn), band_of(&t, ayula));
}
