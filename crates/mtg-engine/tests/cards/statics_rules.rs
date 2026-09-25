//! Static abilities that change control, goad, restrict attacks on a player, change how
//! combat damage is assigned, and stop spells from being countered or abilities from
//! being activated (`oracle/patterns/statics.rs`, `statics_rules.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

// ---------------------------------------------------------------------------
// Control-changing Auras
// ---------------------------------------------------------------------------

#[test]
fn you_control_enchanted_creature() {
    cr!("613.1b", "302.6");
    ruling!(
        "Mind Control",
        "cause you gain control of any Auras or Equipment attached to it"
    );
    compiles("Mind Control");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let theirs = t.battlefield(P1, "Rancor");
    t.attach(theirs, Entity::Object(giant));
    let mc = t.battlefield(P0, "Mind Control");
    t.attach(mc, Entity::Object(giant));
    t.settle();
    assert_eq!(t.obj_now(giant).controller, P0);
    // The Aura attached to it stays under its controller's control.
    assert_eq!(t.obj_now(theirs).controller, P1);
    // CR 302.6: it hasn't been under P0's control since P0's turn began.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!t.can_attack(giant));
    // Without the Aura, the effect ends and control reverts.
    t.g.destroy(mc, None);
    t.settle();
    assert_eq!(t.obj_now(giant).controller, P1);
}

#[test]
fn you_control_enchanted_permanent_and_artifact() {
    cr!("613.1b");
    compiles("Take Possession");
    compiles("Steal Artifact");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    let ring = t.battlefield(P1, "Sol Ring");
    let tp = t.battlefield(P0, "Take Possession");
    t.attach(tp, Entity::Object(land));
    let sa = t.battlefield(P0, "Steal Artifact");
    t.attach(sa, Entity::Object(ring));
    t.settle();
    assert_eq!(t.obj_now(land).controller, P0);
    assert_eq!(t.obj_now(ring).controller, P0);
}

// ---------------------------------------------------------------------------
// Goad as a static ability
// ---------------------------------------------------------------------------

#[test]
fn enchanted_creature_is_goaded_by_the_auras_controller() {
    cr!("701.15b", "508.1d");
    ruling!(
        "Ghoulish Impetus",
        "it must attack a player other than the controller of the spell or ability that goaded it if able"
    );
    compiles("Ghoulish Impetus");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = t.battlefield(P0, "Ghoulish Impetus");
    t.attach(aura, Entity::Object(bears));
    t.settle();
    assert_eq!(t.pt(bears), (3, 3));
    // P1 tries not to attack; the goaded Bears must attack, and not P0.
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[], &[]);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P2), 17);
}

#[test]
fn goaded_creature_attacks_the_goader_when_no_one_else_can_be_attacked() {
    cr!("701.15b");
    compiles("Ghoulish Impetus");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = t.battlefield(P0, "Ghoulish Impetus");
    t.attach(aura, Entity::Object(bears));
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[], &[]);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn enchanted_creature_must_be_blocked_if_able() {
    cr!("509.1c");
    ruling!(
        "Predatory Impetus",
        "Only one creature is required to block the enchanted creature."
    );
    compiles("Predatory Impetus");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Predatory Impetus");
    t.attach(aura, Entity::Object(bears));
    let wall = t.battlefield(P1, "Wall of Stone");
    let wall2 = t.battlefield(P1, "Wall of Stone");
    t.settle();
    assert_eq!(t.pt(bears), (5, 5));
    t.set_step(P0, Step::BeginningOfCombat);
    // The defending player declares no blocks; a Wall must block anyway, but only one.
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
    assert!(t.on_battlefield(wall) && t.on_battlefield(wall2));
    let damaged = [wall, wall2]
        .iter()
        .filter(|w| t.obj_now(**w).damage > 0)
        .count();
    assert_eq!(damaged, 1);
}

// ---------------------------------------------------------------------------
// "can't attack you"
// ---------------------------------------------------------------------------

#[test]
fn creatures_cant_attack_you_but_can_attack_your_planeswalkers() {
    cr!("508.1c");
    ruling!("Blazing Archon", "can still attack a planeswalker you control");
    compiles("Blazing Archon");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blazing Archon");
    let jace = t.battlefield(P0, "Jace Beleren");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(t.can_attack(bears));
    assert!(!t.can_attack_target(bears, Entity::Player(P0)));
    assert!(t.can_attack_target(bears, Entity::Object(jace)));
    // An attack on P0 is illegal and undone; no damage is dealt.
    t.attack(&[(bears, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn enchanted_creature_cant_attack_you_or_your_planeswalkers() {
    cr!("508.1c");
    compiles("Vow of Flight");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = t.battlefield(P0, "Vow of Flight");
    t.attach(aura, Entity::Object(bears));
    let jace = t.battlefield(P0, "Jace Beleren");
    t.settle();
    assert_eq!(t.pt(bears), (4, 4));
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!t.can_attack_target(bears, Entity::Player(P0)));
    assert!(!t.can_attack_target(bears, Entity::Object(jace)));
    assert!(t.can_attack_target(bears, Entity::Player(P2)));
}

#[test]
fn creatures_with_small_power_cant_attack_you() {
    cr!("508.1c");
    compiles("Reverence");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Reverence");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    // "Creatures with power 2 or less can't attack you."
    assert!(!t.can_attack_target(bears, Entity::Player(P0)));
    assert!(t.can_attack_target(giant, Entity::Player(P0)));
}

// ---------------------------------------------------------------------------
// Combat damage equal to toughness
// ---------------------------------------------------------------------------

#[test]
fn each_creature_assigns_combat_damage_equal_to_its_toughness() {
    cr!("510.1a");
    ruling!(
        "Doran, the Siege Tower",
        "a 2/3 creature will assign 3 damage in combat instead of 2"
    );
    compiles("Doran, the Siege Tower");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Doran, the Siege Tower");
    let ox = t.battlefield(P0, "Pillarfield Ox");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(ox, Entity::Player(P1))], &[]);
    // The 2/4 Ox assigns 4; its power itself doesn't change.
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.pt(ox), (2, 4));
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Doran, the Siege Tower");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(elves, Entity::Player(P1))], &[(wall, elves)]);
    // The 0/8 Wall deals 8 damage to the 1/1 Elves; the Elves deal 1.
    assert!(!t.on_battlefield(elves));
    assert_eq!(t.obj_now(wall).damage, 1);
}

#[test]
fn only_creatures_with_toughness_greater_than_power_use_toughness() {
    cr!("510.1a");
    compiles("Bark of Doran");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Stone");
    let bark = t.battlefield(P0, "Bark of Doran");
    t.attach(bark, Entity::Object(wall));
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.settle();
    // 0/9 with Bark of Doran: it blocks and assigns 9.
    assert_eq!(t.pt(wall), (0, 9));
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P0))], &[(wall, bears)]);
    assert!(!t.on_battlefield(bears));
}

// ---------------------------------------------------------------------------
// Spells that can't be countered
// ---------------------------------------------------------------------------

#[test]
fn creature_spells_you_control_cant_be_countered() {
    cr!("101.2", "701.6a");
    ruling!(
        "Prowling Serpopard",
        "A spell or ability that counters spells can still target a creature spell you control."
    );
    compiles("Prowling Serpopard");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Prowling Serpopard");
    t.lands(P0, "Forest", 2);
    t.lands(P1, "Island", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    let counter = t.hand(P1, "Counterspell");
    t.set_step(P0, Step::PrecombatMain);
    let spell = t.cast(P0, bears).go();
    // Counterspell can target it, but it isn't countered.
    t.cast(P1, counter).target(spell).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.in_graveyard(P1, "Counterspell"));
}

#[test]
fn instant_and_sorcery_spells_you_control_cant_be_countered() {
    cr!("101.2", "701.6a");
    compiles("Sphinx of the Final Word");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sphinx of the Final Word");
    t.lands(P0, "Mountain", 1);
    t.lands(P1, "Island", 4);
    let bolt = t.hand(P0, "Lightning Bolt");
    let counter = t.hand(P1, "Counterspell");
    let counter2 = t.hand(P1, "Counterspell");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    let spell = t.cast(P0, bolt).target(bears).go();
    t.cast(P1, counter).target(spell).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    // A creature spell isn't an instant or sorcery spell: it can be countered.
    t.lands(P0, "Forest", 2);
    let elves = t.hand(P0, "Llanowar Elves");
    let spell = t.cast(P0, elves).go();
    t.cast(P1, counter2).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
}

// ---------------------------------------------------------------------------
// Activated abilities that can't be activated
// ---------------------------------------------------------------------------

#[test]
fn activated_abilities_of_artifacts_cant_be_activated() {
    cr!("602.5", "605.1a");
    ruling!(
        "Collector Ouphe",
        "No abilities of artifacts can be activated, including mana abilities."
    );
    compiles("Collector Ouphe");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Sol Ring");
    t.battlefield(P1, "Collector Ouphe");
    t.set_step(P0, Step::PrecombatMain);
    assert!(t.activate(P0, ring, 0, &[]).is_err());
    // Its mana can't pay for a spell either.
    let ornithopter = t.hand(P0, "Walking Ballista");
    assert!(t.cast(P0, ornithopter).x(1).try_go().is_err());
}

#[test]
fn enchanted_creatures_mana_abilities_cant_be_activated() {
    cr!("602.5", "605.1a");
    ruling!(
        "Stupefying Touch",
        "last ability stops mana abilities from being activated"
    );
    compiles("Stupefying Touch");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let other = t.battlefield(P0, "Llanowar Elves");
    let aura = t.battlefield(P1, "Stupefying Touch");
    t.attach(aura, Entity::Object(elves));
    t.settle();
    t.set_step(P0, Step::PrecombatMain);
    assert!(t.activate(P0, elves, 0, &[]).is_err());
    assert!(t.activate(P0, other, 0, &[]).is_ok());
}

#[test]
fn non_mana_abilities_of_artifacts_and_creatures_cant_be_activated() {
    cr!("602.5", "605.1a");
    compiles("Damping Matrix");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Damping Matrix");
    // "Activated abilities of artifacts and creatures can't be activated unless they're
    // mana abilities."
    let elves = t.battlefield(P0, "Llanowar Elves");
    let ballista = t.battlefield(P0, "Walking Ballista");
    t.set_step(P0, Step::PrecombatMain);
    assert!(t.activate(P0, elves, 0, &[]).is_ok());
    t.lands(P0, "Forest", 4);
    assert!(t.activate(P0, ballista, 0, &[]).is_err());
}
