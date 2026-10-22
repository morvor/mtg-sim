//! Token creation with full descriptions (patterns in
//! `src/oracle/patterns/tokens_copies_create.rs`): quoted abilities in the token's
//! definition ("with "This token can't block."", "It has "Sacrifice this token: Add
//! {C}.""), tokens that enter tapped and attacking, named tokens, several kinds of tokens
//! at once, and follow-up instructions about the tokens just created ("It gains haste
//! until end of turn", "then attach this Equipment to it", "Destroy it at the beginning of
//! the next end step").

use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{CardType, Color};
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn assert_line_supported(name: &str, needle: &str) {
    let bad: Vec<String> = card(name)
        .unsupported_text()
        .into_iter()
        .filter(|u| u.contains(needle))
        .map(str::to_string)
        .collect();
    assert!(bad.is_empty(), "{name} has unsupported text: {bad:?}");
}

fn tokens_named(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|id| t.g.obj(*id).is_token())
        .collect()
}

#[test]
fn described_token_cards_compile() {
    assert_compiles(&[
        "Voracious Vermin",
        "Incubator Drone",
        "Prosperity Tycoon",
        "Krenko, Baron of Tin Street",
        "Ancestral Blade",
        "Forbidden Friendship",
        "Hornet Cannon",
        "Stinging Hivemaster",
        "Chrome Dome",
    ]);
    assert_line_supported("Squee, Dubious Monarch", "tapped and attacking");
}

#[test]
fn a_quoted_restriction_is_part_of_the_tokens_definition() {
    cr!("111.3", "509.1b");
    let mut t = TestGame::new(2);
    t.enter(P0, "Voracious Vermin");
    t.resolve_all();
    let rats = tokens_named(&t, "Rat Token");
    assert_eq!(rats.len(), 1);
    let rat = rats[0];
    assert_eq!(t.pt(rat), (1, 1));
    assert!(t.g.obj(rat).chars.colors.contains(Color::Black));
    // "This token can't block": the token can't block, but the card itself can.
    assert!(!t.g.can_block_at_all(rat), "the Rat token can't block");
    let vermin = t.named_on_battlefield("Voracious Vermin")[0];
    assert!(t.g.can_block_at_all(vermin), "the card's own creature can block");
}

#[test]
fn quoted_keyword_and_restriction_together() {
    cr!("111.3", "702.164c");
    let mut t = TestGame::new(2);
    let hive = t.battlefield(P0, "Stinging Hivemaster");
    t.g.destroy(hive, None);
    t.resolve_all();
    let mites = tokens_named(&t, "Phyrexian Mite Token");
    assert_eq!(mites.len(), 1);
    let mite = mites[0];
    let o = t.g.obj(mite);
    assert!(o.chars.is(CardType::Artifact) && o.chars.is(CardType::Creature));
    assert!(!t.g.can_block_at_all(mite), "\"This token can't block.\"");
    // Toxic 1: its combat damage to a player also gives a poison counter.
    t.g.objects[mite.0 as usize].summoning_sick = false;
    t.attack(&[(mite, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.g.player(P1).counter("poison"), 1);
}

#[test]
fn it_has_adds_a_mana_ability_to_the_created_token() {
    cr!("111.3", "605.1a", "605.3b");
    let mut t = TestGame::new(2);
    t.enter(P0, "Incubator Drone");
    t.resolve_all();
    let scions = tokens_named(&t, "Eldrazi Scion Token");
    assert_eq!(scions.len(), 1);
    let scion = scions[0];
    assert!(t.g.obj(scion).chars.colors.is_colorless());
    // "Sacrifice this token: Add {C}." is a mana ability: no stack.
    let r = t.activate(P0, scion, 0, &[]).unwrap();
    assert!(r.is_none(), "mana abilities don't use the stack");
    assert_eq!(t.g.players[0].mana_pool.count(ManaType::C), 1);
    assert!(!t.on_battlefield(scion), "the token was sacrificed");
    // The Drone itself doesn't have that ability.
    let drone = t.named_on_battlefield("Incubator Drone")[0];
    assert!(!t
        .g
        .obj(drone)
        .chars
        .abilities
        .iter()
        .any(|a| matches!(a.kind, ability::AbilityKind::Activated(_))));
}

#[test]
fn a_quoted_activated_ability_is_the_tokens_own() {
    cr!("111.3", "602.5d");
    let mut t = TestGame::new(2);
    t.enter(P0, "Prosperity Tycoon");
    t.resolve_all();
    let merc = tokens_named(&t, "Mercenary Token")[0];
    assert_eq!(t.pt(merc), (1, 1));
    // The token's controller activates it; it isn't summoning sick here.
    t.g.objects[merc.0 as usize].summoning_sick = false;
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, merc, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    assert_eq!(t.pt(bears), (3, 2), "+1/+0 until end of turn");
    assert!(t.g.obj(merc).tapped);
    // "Activate only as a sorcery": not during combat.
    let mut t = TestGame::new(2);
    t.enter(P0, "Prosperity Tycoon");
    t.resolve_all();
    let merc = tokens_named(&t, "Mercenary Token")[0];
    t.g.objects[merc.0 as usize].summoning_sick = false;
    t.set_step(P0, Step::BeginningOfCombat);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.activate(P0, merc, 0, &[Entity::Object(bears)]).is_err());
}

#[test]
fn a_token_created_tapped_and_attacking() {
    cr!("508.4", "111.1");
    let mut t = TestGame::new(2);
    let squee = t.battlefield(P0, "Squee, Dubious Monarch");
    t.attack(&[(squee, Entity::Player(P1))], &[]);
    let goblins = tokens_named(&t, "Goblin Token");
    assert_eq!(goblins.len(), 1);
    let goblin = goblins[0];
    assert!(t.g.obj(goblin).tapped, "created tapped");
    // Squee (2) and the attacking Goblin token (1) both dealt combat damage.
    assert_eq!(t.life(P1), 17);
}

#[test]
fn it_gains_haste_names_the_new_token_not_the_trigger_object() {
    cr!("611.2a", "514.2");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.battlefield(P0, "Krenko, Baron of Tin Street");
    let thopter = t.battlefield(P0, "Ornithopter");
    t.answer_yes(P0, true);
    t.g.destroy(thopter, None);
    t.resolve_all();
    let goblins = tokens_named(&t, "Goblin Token");
    assert_eq!(goblins.len(), 1, "paid {{R}}: one Goblin");
    let goblin = goblins[0];
    assert!(t.g.obj(goblin).chars.has_keyword(KeywordKind::Haste));
    t.advance_to(P1, Step::Upkeep);
    assert!(
        !t.g.obj(goblin).chars.has_keyword(KeywordKind::Haste),
        "only until end of turn"
    );
}

#[test]
fn equipment_attaches_itself_to_the_token_it_created() {
    cr!("301.5", "701.3a");
    let mut t = TestGame::new(2);
    let blade = t.enter(P0, "Ancestral Blade");
    t.resolve_all();
    let soldier = tokens_named(&t, "Soldier Token")[0];
    assert_eq!(t.g.obj(blade).attached_to, Some(Entity::Object(soldier)));
    assert_eq!(t.pt(soldier), (2, 2), "1/1 equipped with +1/+1");
}

#[test]
fn several_kinds_of_tokens_in_one_instruction() {
    cr!("111.1", "111.4");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let ff = t.hand(P0, "Forbidden Friendship");
    t.cast(P0, ff).go();
    t.resolve();
    let dino = tokens_named(&t, "Dinosaur Token");
    let human = tokens_named(&t, "Human Soldier Token");
    assert_eq!((dino.len(), human.len()), (1, 1));
    assert!(t.g.obj(dino[0]).chars.has_keyword(KeywordKind::Haste));
    assert!(!t.g.obj(human[0]).chars.has_keyword(KeywordKind::Haste));
    assert!(t.g.obj(human[0]).chars.colors.contains(Color::White));
}

#[test]
fn a_named_token_is_destroyed_at_the_next_end_step() {
    cr!("111.4", "603.7a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let cannon = t.battlefield(P0, "Hornet Cannon");
    t.activate(P0, cannon, 0, &[]).unwrap();
    t.resolve();
    let hornets = tokens_named(&t, "Hornet");
    assert_eq!(hornets.len(), 1, "the token is named Hornet");
    let hornet = hornets[0];
    let o = t.g.obj(hornet);
    assert!(o.chars.has_keyword(KeywordKind::Flying) && o.chars.has_keyword(KeywordKind::Haste));
    assert!(o.chars.has_subtype("Insect"));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(hornet), "destroyed at the beginning of the end step");
}

#[test]
fn that_token_gains_haste_and_is_sacrificed_later() {
    cr!("707.2", "603.7a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    t.battlefield(P0, "Chrome Dome");
    let thopter = t.battlefield(P0, "Ornithopter");
    let dome = t.named_on_battlefield("Chrome Dome")[0];
    t.activate(P0, dome, 0, &[Entity::Object(thopter)]).unwrap();
    t.resolve();
    let copies: Vec<ObjectId> = tokens_named(&t, "Ornithopter");
    assert_eq!(copies.len(), 1);
    let copy = copies[0];
    assert!(t.g.obj(copy).chars.has_keyword(KeywordKind::Haste));
    assert!(
        !t.g.obj(thopter).chars.has_keyword(KeywordKind::Haste),
        "the original doesn't gain haste"
    );
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(copy), "the copy was sacrificed");
    assert!(t.on_battlefield(thopter), "the original stays");
}

#[test]
fn a_tokens_name_can_contain_the_cards_own_name() {
    cr!("111.4");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let keep = t.battlefield(P0, "Kher Keep");
    t.activate(P0, keep, 1, &[]).unwrap();
    t.resolve();
    let kobolds = tokens_named(&t, "Kobolds of Kher Keep");
    assert_eq!(kobolds.len(), 1, "named after the card, not \"Kobold Token\"");
    assert_eq!(t.pt(kobolds[0]), (0, 1));
    assert!(t.g.obj(kobolds[0]).chars.has_subtype("Kobold"));
}

#[test]
fn a_named_token_keeps_the_quoted_ability_after_its_name() {
    cr!("111.3", "111.4");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let horror = t.battlefield(P0, "Pink Horror");
    t.g.destroy(horror, None);
    t.resolve_all();
    let blues = tokens_named(&t, "Blue Horror");
    assert_eq!(blues.len(), 2);
    for b in &blues {
        let o = t.g.obj(*b);
        assert!(o.chars.colors.contains(Color::Blue) && o.chars.colors.contains(Color::Red));
        assert!(o.chars.has_subtype("Demon") && o.chars.has_subtype("Horror"));
    }
    // "Whenever you cast an instant or sorcery spell, this token deals 1 damage to any
    // target": two triggers, plus the Shock itself.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 2 - 1 - 1);
}

#[test]
fn a_named_token_that_is_all_colors_with_a_follow_up_ability_list() {
    cr!("111.3", "111.4");
    let mut t = TestGame::new(2);
    for land in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        t.lands(P0, land, 1);
    }
    t.lands(P0, "Wastes", 3);
    let relic = t.battlefield(P0, "Dragonbroods' Relic");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, relic, 1, &[]).unwrap();
    t.resolve_all();
    let dragons = tokens_named(&t, "Reliquary Dragon");
    assert_eq!(dragons.len(), 1);
    let o = t.g.obj(dragons[0]);
    assert_eq!(o.chars.colors, types::ColorSet::ALL, "that's all colors");
    assert_eq!(t.pt(dragons[0]), (4, 4));
    assert!(o.chars.has_keyword(KeywordKind::Flying) && o.chars.has_keyword(KeywordKind::Lifelink));
    assert_eq!(t.life(P1), 17, "\"When this token enters, it deals 3 damage to any target.\"");
}

#[test]
fn where_x_is_after_a_tokens_name_sets_how_many() {
    cr!("107.3c", "111.4");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    // Three permanent cards and an instant in the graveyard; the sacrificed enchantment
    // is a fourth permanent card there by the time the ability resolves.
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Mind Stone");
    t.graveyard(P0, "Forest");
    t.graveyard(P0, "Lightning Bolt");
    let rmm = t.battlefield(P0, "Return of the Mole Man");
    t.activate(P0, rmm, 0, &[]).unwrap();
    t.resolve();
    let moloids = tokens_named(&t, "Moloid");
    assert_eq!(moloids.len(), 4);
    for m in &moloids {
        assert_eq!(t.pt(*m), (1, 1));
        assert!(t.g.obj(*m).chars.has_subtype("Minion"));
        assert!(t
            .g
            .obj(*m)
            .chars
            .abilities
            .iter()
            .any(|a| matches!(a.kind, ability::AbilityKind::Triggered(_))));
    }
}

#[test]
fn another_player_creates_a_described_token_with_quoted_abilities() {
    cr!("111.2", "111.3");
    let mut t = TestGame::new(2);
    let robber = t.battlefield(P0, "Relic Robber");
    t.attack(&[(robber, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
    let goblins = tokens_named(&t, "Goblin Construct Token");
    assert_eq!(goblins.len(), 1);
    let g = goblins[0];
    assert_eq!(t.g.obj(g).controller, P1, "the player who creates it controls it");
    assert_eq!(t.g.obj(g).owner, P1);
    assert!(!t.g.can_block_at_all(g), "\"This token can't block\"");
    // "At the beginning of your upkeep, this token deals 1 damage to you": "you" is the
    // token's controller.
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 20);
}
