//! Group grants with complex subjects (`src/oracle/patterns/grant_grammar.rs`,
//! `src/kw/grant_filters.rs`): compound subjects mixing players and objects ("You and
//! Humans you control have hexproof"), unions of groups ("Saproling creatures and other
//! Treefolk creatures get +1/+1", "Auras, Equipment, and modified creatures you control
//! gain hexproof"), creatures attacking a player, creatures an enchanted player
//! controls, and group subjects with relative clauses. Also "twice X" counts and tokens
//! created tapped and attacking.

use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::types;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// Whether `p` could target `e` with a Lightning Bolt (CR 115.4).
fn bolt_can_target(t: &mut TestGame, p: PlayerId, e: Entity) -> bool {
    let spell = t.hand(p, "Lightning Bolt");
    t.g.recompute();
    let spec = t.g.spell_body(spell).targets[0].clone();
    let ctx = mtg_engine::eval::Ctx::new(Some(spell), p);
    t.g.legal_target_candidates(&spec, &ctx, spell).contains(&e)
}

fn has(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(k)
}

#[test]
fn sigarda_you_and_humans_you_control_have_hexproof() {
    cr!("702.11b", "702.11c", "611.3a");
    assert_supported(&["Sigarda, Heron's Grace"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sigarda, Heron's Grace");
    let human = t.battlefield(P0, "Elite Vanguard");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.settle();
    assert!(has(&t, human, KeywordKind::Hexproof));
    assert!(!has(&t, bears, KeywordKind::Hexproof));
    assert!(!bolt_can_target(&mut t, P1, Entity::Player(P0)), "you have hexproof");
    assert!(!bolt_can_target(&mut t, P1, Entity::Object(human)));
    assert!(bolt_can_target(&mut t, P1, Entity::Object(bears)));
    // You can still target yourself.
    assert!(bolt_can_target(&mut t, P0, Entity::Player(P0)));
}

#[test]
fn gruul_spellbreaker_during_your_turn_you_and_it_have_hexproof() {
    cr!("611.3a", "702.11c");
    assert_supported(&["Gruul Spellbreaker"]);
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gruul Spellbreaker");
    t.set_step(P0, Step::PrecombatMain);
    t.settle();
    assert!(has(&t, g, KeywordKind::Hexproof));
    assert!(!bolt_can_target(&mut t, P1, Entity::Player(P0)));
    t.set_step(P1, Step::PrecombatMain);
    t.settle();
    assert!(!has(&t, g, KeywordKind::Hexproof));
    assert!(bolt_can_target(&mut t, P1, Entity::Player(P0)));
}

#[test]
fn shalai_you_planeswalkers_and_other_creatures_have_hexproof() {
    cr!("611.3a", "702.11b");
    assert_supported(&["Shalai, Voice of Plenty"]);
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Shalai, Voice of Plenty");
    let pw = t.battlefield(P0, "Ajani Goldmane");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.settle();
    assert!(has(&t, pw, KeywordKind::Hexproof));
    assert!(has(&t, bears, KeywordKind::Hexproof));
    assert!(!has(&t, s, KeywordKind::Hexproof), "other creatures");
    assert!(!has(&t, theirs, KeywordKind::Hexproof));
}

#[test]
fn verdeloth_saprolings_and_other_treefolk_get_the_bonus() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Verdeloth the Ancient",
        "Verdeloth won't give himself +1/+1 unless he somehow becomes a Saproling"
    );
    assert_supported(&["Verdeloth the Ancient", "Masked Gorgon"]);
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Verdeloth the Ancient");
    let tree = t.battlefield(P1, "Ironroot Treefolk");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.pt(v), (4, 7));
    assert_eq!(t.pt(tree), (4, 6), "any player's Treefolk");
    assert_eq!(t.pt(bears), (2, 2));
    // Masked Gorgon: green creatures and white creatures have protection from Gorgons.
    t.battlefield(P1, "Masked Gorgon");
    let lions = t.battlefield(P0, "Savannah Lions");
    t.settle();
    assert!(has(&t, bears, KeywordKind::Protection));
    assert!(has(&t, lions, KeywordKind::Protection));
    let gorgon = t.named_on_battlefield("Masked Gorgon")[0];
    assert!(!has(&t, gorgon, KeywordKind::Protection));
}

#[test]
fn curse_of_deaths_hold_affects_the_enchanted_players_creatures() {
    cr!("303.4a", "611.3a");
    assert_supported(&["Curse of Death's Hold"]);
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let curse = t.battlefield(P0, "Curse of Death's Hold");
    t.g.attach(curse, Entity::Player(P1));
    t.settle();
    assert_eq!(t.pt(theirs), (1, 1));
    assert_eq!(t.pt(mine), (2, 2));
}

#[test]
fn creatures_attacking_your_opponents_not_their_planeswalkers() {
    cr!("506.2", "611.3a");
    ruling!(
        "Blast-Furnace Hellkite",
        "applies to any creature that is attacking one of your opponents"
    );
    assert_supported(&["Blast-Furnace Hellkite", "Watchdog"]);
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Blast-Furnace Hellkite");
    // A creature an opponent controls attacking another opponent has double strike.
    let a = t.battlefield(P1, "Grizzly Bears");
    let pw = t.battlefield(P2, "Ajani Goldmane");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.g.combat = None;
    t.attack(&[(a, Entity::Player(P2)), (b, Entity::Object(pw))], &[]);
    // 2 double strike damage to P2; the one attacking the planeswalker has no double
    // strike.
    assert_eq!(t.life(P2), 16);
    // Watchdog: creatures attacking you get -1/-0 while it's untapped.
    let mut t = TestGame::new(2);
    let dog = t.battlefield(P0, "Watchdog");
    // (A flier: Watchdog blocks each combat if able.)
    let x = t.battlefield(P1, "Wind Drake");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(x, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 19);
    assert!(t.on_battlefield(dog));
}

#[test]
fn silkguard_auras_equipment_and_modified_creatures_you_control() {
    cr!("611.2c", "700.9");
    ruling!(
        "Silkguard",
        "A creature with a counter on it is considered modified no matter what kind of counter it is"
    );
    assert_supported(&["Silkguard", "Rise of the Hobgoblins", "Sapling Nursery"]);
    let mut t = TestGame::new(2);
    let target = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    let stunned = t.battlefield(P0, "Savannah Lions");
    t.g.add_counters(Entity::Object(stunned), "stun", 1, None);
    let gear = t.battlefield(P0, "Bonesplitter");
    let theirs = t.battlefield(P1, "Bonesplitter");
    t.lands(P0, "Forest", 2);
    let s = t.hand(P0, "Silkguard");
    t.cast(P0, s).x(1).target(target).go();
    t.resolve();
    assert!(has(&t, target, KeywordKind::Hexproof));
    assert!(has(&t, stunned, KeywordKind::Hexproof));
    assert!(has(&t, gear, KeywordKind::Hexproof));
    assert!(!has(&t, other, KeywordKind::Hexproof));
    assert!(!has(&t, theirs, KeywordKind::Hexproof));
}

#[test]
fn rise_of_the_hobgoblins_red_and_white_creatures_you_control() {
    cr!("611.2c");
    assert_supported(&["Rise of the Hobgoblins"]);
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Rise of the Hobgoblins");
    let red = t.battlefield(P0, "Raging Goblin");
    let white = t.battlefield(P0, "Savannah Lions");
    let green = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Raging Goblin");
    t.lands(P0, "Mountain", 1);
    t.activate(P0, r, 0, &[]).unwrap();
    t.resolve();
    assert!(has(&t, red, KeywordKind::FirstStrike));
    assert!(has(&t, white, KeywordKind::FirstStrike));
    assert!(!has(&t, green, KeywordKind::FirstStrike));
    assert!(!has(&t, theirs, KeywordKind::FirstStrike));
}

#[test]
fn pallid_mycoderm_each_creature_you_control_thats_a_fungus_or_saproling() {
    cr!("611.2c");
    assert_supported(&["Pallid Mycoderm"]);
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Pallid Mycoderm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let tok = t.custom(
        P0,
        CardDef::custom({
            let mut c = Characteristics::default();
            c.name = "Saproling".into();
            c.card_types.insert(types::CardType::Creature);
            c.subtypes.push("Saproling".into());
            c.power = Some(1);
            c.toughness = Some(1);
            c
        }),
        Zone::Battlefield,
    );
    t.settle();
    t.activate(P0, m, 1, &[]).unwrap();
    t.resolve();
    assert!(!t.on_battlefield(tok));
    assert_eq!(t.pt(m), (3, 5));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn twice_x_tokens_and_life() {
    cr!("107.3a", "111.1");
    ruling!(
        "Pest Infestation",
        "The number of Pests created is equal to twice the chosen value of X"
    );
    ruling!("Sanguine Sacrament", "as a single life-gain event");
    assert_supported(&["Pest Infestation", "Sanguine Sacrament"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let s = t.hand(P0, "Pest Infestation");
    t.cast(P0, s).x(2).targets(&[]).go();
    t.resolve();
    let pests = t
        .g
        .battlefield
        .iter()
        .filter(|id| t.obj_now(**id).chars.has_subtype("Pest"))
        .count();
    assert_eq!(pests, 4);
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let s = t.hand(P0, "Sanguine Sacrament");
    t.cast(P0, s).x(3).go();
    t.resolve();
    assert_eq!(t.life(P0), 26);
}

#[test]
fn pugnacious_pugilist_creates_a_tapped_and_attacking_devil() {
    cr!("508.4", "111.1");
    assert_supported(&["Pugnacious Pugilist"]);
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Pugnacious Pugilist");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(p, Entity::Player(P1))], &[]);
    // The Devil attacked too: 4 + 1 damage.
    assert_eq!(t.life(P1), 15);
}
