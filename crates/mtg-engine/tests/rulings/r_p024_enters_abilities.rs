//! Rulings batch P024 — a token that's a copy of a permanent has its copied "when this
//! enters" abilities, which trigger as it's created, and its copied "as this enters" and
//! "enters with" abilities apply (CR 707.2, 707.5, 603.6a, 614.1c, 614.12); triggers from
//! several tokens created by one spell wait until it's finished resolving and go on the
//! stack in APNAP order (CR 603.3, 603.3b, 101.4); a copy's haste and a delayed trigger
//! that's countered (CR 603.7, 611.2a).

use crate::r_p020_common::{entered_as_riftwatcher, RIFTWATCHER};
use crate::r_p024_common::*;
use crate::r_s24_common::choose_creature_type;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// How a card makes a token copy of P0's permanent `what`: returns the new tokens of
/// P0 (the stack resolved).
type MakeCopy = fn(&mut TestGame, ObjectId) -> Vec<ObjectId>;

/// Asserts `make` makes a token copy of each kind of enters ability: of Aven
/// Riftwatcher ("enters with" time counters and a "when this enters" life gain), and of
/// Adaptive Automaton ("As this creature enters, choose a creature type").
fn copies_enters_abilities(card: &str, make: MakeCopy) {
    supported(card);
    let mut t = TestGame::new(2);
    let rift = t.battlefield(P0, RIFTWATCHER);
    let life = t.life(P0);
    let toks = make(&mut t, rift);
    assert_eq!(toks.len(), 1, "{card}: {}", t.dump_log());
    entered_as_riftwatcher(&t, toks[0]);
    assert_eq!(t.life(P0), life + 2, "{card}: the copy's enters trigger");
    let mut t = TestGame::new(2);
    let auto = t.battlefield(P0, "Adaptive Automaton");
    let elf = t.battlefield(P0, "Llanowar Elves");
    choose_creature_type(&mut t, P0, "Elf");
    let toks = make(&mut t, auto);
    assert_eq!(toks.len(), 1, "{card}");
    assert!(has_subtype(&t, toks[0], "Elf"), "{card}: no creature type chosen");
    // "Other creatures you control of the chosen type get +1/+1": the token's lord.
    assert_eq!(t.pt(elf), (2, 2), "{card}");
}

fn here_comes_a_new_hero(t: &mut TestGame, what: ObjectId) -> Vec<ObjectId> {
    // X = 3: P0 draws three cards and copies a creature with mana value 3 or less.
    cast_spell(
        t,
        P0,
        "Here Comes a New Hero!",
        "{3}{2}{U}",
        &[Entity::Player(P0), obj(what)],
        None,
        Some(3),
    )
}

fn tempestra(t: &mut TestGame, what: ObjectId) -> Vec<ObjectId> {
    let tempestra = t.battlefield(P0, "Tempestra, Dame of Games");
    // (Adaptive Automaton is an artifact too: sacrifice the Ornithopter.)
    let orni = t.battlefield(P0, "Ornithopter");
    t.answer_choose(P0, &[obj(orni)]);
    lands_for(t, P0, "{2}{R}");
    let before = t.g.battlefield.clone();
    t.activate(P0, tempestra, 0, &[obj(what)]).expect("activation");
    t.resolve_all();
    new_tokens(t, P0, &before)
}

fn the_fire_crystal(t: &mut TestGame, what: ObjectId) -> Vec<ObjectId> {
    let crystal = t.battlefield(P0, "The Fire Crystal");
    lands_for(t, P0, "{4}{R}{R}");
    let before = t.g.battlefield.clone();
    t.activate(P0, crystal, 0, &[obj(what)]).expect("activation");
    t.resolve_all();
    new_tokens(t, P0, &before)
}

fn flash_photography(t: &mut TestGame, what: ObjectId) -> Vec<ObjectId> {
    cast_spell(t, P0, "Flash Photography", "{2}{U}{U}", &[obj(what)], None, None)
}

fn relms_sketching(t: &mut TestGame, what: ObjectId) -> Vec<ObjectId> {
    cast_spell(t, P0, "Relm's Sketching", "{2}{U}{U}", &[obj(what)], None, None)
}

#[test]
fn token_copies_have_the_copied_enters_abilities() {
    cr!("707.2", "707.5", "603.6a", "614.1c", "614.12");
    ruling!("Here Comes a New Hero!", "If the creature copied by the token had any \"when [this creature] enters\" abilities, the token also has those abilities, and they'll trigger when it's created. Similarly, any \"as [this creature] enters\" or \"[this creature] enters with\" abilities that the token has copied will also work.");
    ruling!("The Fire Crystal", "If the creature copied by the token had any \"when [this permanent] enters\" abilities, the token also has those abilities, and they'll trigger when it's created. Similarly, any \"as [this permanent] enters\" or \"[this permanent] enters with\" abilities that the token has copied will also work.");
    ruling!("Flash Photography", "If the permanent copied by the token had any \"when [this permanent] enters\" abilities, the token also has those abilities, and they'll trigger when it's created. Similarly, any \"as [this permanent] enters\" or \"[this permanent] enters with\" abilities that the token has copied will also work.");
    copies_enters_abilities("Here Comes a New Hero!", here_comes_a_new_hero);
    copies_enters_abilities("Tempestra, Dame of Games", tempestra);
    copies_enters_abilities("The Fire Crystal", the_fire_crystal);
    copies_enters_abilities("Flash Photography", flash_photography);
    copies_enters_abilities("Relm's Sketching", relms_sketching);
}

#[test]
fn faerie_artisans_token_has_the_copied_enters_abilities() {
    cr!("707.2", "707.5", "603.6a", "614.1c", "614.12", "707.9b");
    ruling!("Faerie Artisans", "If the permanent copied by the token had any “when [this permanent] enters the battlefield” abilities, then the token also has those abilities and will trigger them when it’s created. Similarly, any “as [this permanent] enters the battlefield” or “[this permanent] enters the battlefield with” abilities that the token has copied will also work.");
    supported("Faerie Artisans");
    // An opponent's Aven Riftwatcher enters: P0's artifact copy enters with time counters
    // and gains P0 2 life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Faerie Artisans");
    let (mine, theirs) = (t.life(P0), t.life(P1));
    let before = t.g.battlefield.clone();
    t.enter(P1, RIFTWATCHER);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    entered_as_riftwatcher(&t, toks[0]);
    assert!(is_artifact(&t, toks[0]));
    assert_eq!((t.life(P0), t.life(P1)), (mine + 2, theirs + 2));
    // An opponent's Adaptive Automaton: P0 chooses a type for its copy.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Faerie Artisans");
    let elf = t.battlefield(P0, "Llanowar Elves");
    choose_creature_type(&mut t, P1, "Goblin");
    choose_creature_type(&mut t, P0, "Elf");
    let before = t.g.battlefield.clone();
    t.enter(P1, "Adaptive Automaton");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert!(has_subtype(&t, toks[0], "Elf"));
    assert_eq!(t.pt(elf), (2, 2));
}

#[test]
fn dual_nature_token_has_the_copied_enters_abilities() {
    cr!("707.2", "707.5", "603.6a", "614.1c");
    ruling!("Dual Nature", "If the original creature has any \"enters\" abilities, the copy will also have those.");
    supported("Dual Nature");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dual Nature");
    let life = t.life(P0);
    let before = t.g.battlefield.clone();
    t.enter(P0, RIFTWATCHER);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    entered_as_riftwatcher(&t, toks[0]);
    // The original's and the token's enters triggers.
    assert_eq!(t.life(P0), life + 4);
}

#[test]
fn tempt_with_reflections_enters_triggers_wait_and_go_on_the_stack_in_apnap_order() {
    cr!("603.3", "603.3b", "101.4", "707.5");
    ruling!("Tempt with Reflections", "If the target creature has an enters-the-battlefield triggered ability, the abilities of all the copies will trigger as Tempt with Reflections resolves. All of those abilities will be put on the stack after Tempt with Reflections finishes resolving. Abilities controlled by the active player will be put on the stack first (in an order of their choice), followed by abilities controlled by each other player in turn order. The last ability to be put on the stack will resolve first, and so on.");
    supported("Tempt with Reflections");
    let mut t = TestGame::new(3);
    let rift = t.battlefield(P0, RIFTWATCHER);
    lands_for(&mut t, P0, "{3}{U}");
    let card = t.hand(P0, "Tempt with Reflections");
    t.answer_yes(P1, true);
    t.answer_yes(P2, true);
    let spell = t.cast(P0, card).target(obj(rift)).go();
    t.resolve();
    assert!(!t.g.stack.contains(&spell));
    // Five tokens: P0's, P1's, P2's and two more of P0's (one per opponent who did).
    let tokens: Vec<_> = (0..3)
        .map(|p| crate::r_s01_common::tokens(&t, PlayerId(p)).len())
        .collect();
    assert_eq!(tokens, vec![3, 1, 1], "{}", t.dump_log());
    // Every enters trigger is on the stack, none resolved yet: P0's at the bottom, then
    // P1's, then P2's on top.
    let controllers: Vec<PlayerId> = t
        .g
        .stack
        .iter()
        .filter(|s| {
            matches!(
                t.g.obj(**s).stack.as_ref().map(|i| &i.kind),
                Some(StackKind::Triggered { .. })
            )
        })
        .map(|s| t.g.obj(*s).controller)
        .collect();
    assert_eq!(controllers, vec![P0, P0, P0, P1, P2]);
    let lives = |t: &TestGame| (t.life(P0), t.life(P1), t.life(P2));
    assert_eq!(lives(&t), (20, 20, 20));
    // P2's resolves first.
    t.resolve();
    assert_eq!(lives(&t), (20, 20, 22));
    t.resolve();
    assert_eq!(lives(&t), (20, 22, 22));
    t.resolve_all();
    assert_eq!(lives(&t), (26, 22, 22));
}

// --- Haste, and tokens that aren't exiled ----------------------------------------------------

/// Advances to P0's end step and counters the delayed trigger on top of the stack with
/// Stifle; then advances to P0's next turn.
fn stifle_the_delayed_trigger(t: &mut TestGame) {
    supported("Stifle");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    let trig = *t.g.stack.last().unwrap();
    t.lands(P0, "Island", 1);
    let stifle = t.hand(P0, "Stifle");
    t.cast(P0, stifle).target(obj(trig)).go();
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
    t.advance_to(P0, Step::PrecombatMain);
}

#[test]
fn mimic_vat_token_isnt_exiled_if_the_delayed_trigger_is_countered() {
    cr!("603.7c", "611.2a", "701.6b");
    ruling!("Mimic Vat", "If the token isn't exiled when the delayed triggered ability resolves (due to Stifle, perhaps), it remains on the battlefield indefinitely. It continues to have haste.");
    supported("Mimic Vat");
    let mut t = TestGame::new(2);
    let vat = t.battlefield(P0, "Mimic Vat");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    kill(&mut t, giant);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    t.lands(P0, "Wastes", 3);
    let before = t.g.battlefield.clone();
    activate_containing(&mut t, P0, vat, "Create a token").expect("activation");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    stifle_the_delayed_trigger(&mut t);
    assert!(t.on_battlefield(toks[0]));
    assert!(has_kw(&t, toks[0], KeywordKind::Haste));
}

#[test]
fn splinter_twin_token_isnt_exiled_if_the_delayed_trigger_is_countered() {
    cr!("603.7c", "707.9a", "701.6b");
    ruling!("Splinter Twin", "If the token isn't exiled when the delayed triggered ability resolves (due to Stifle, perhaps), it remains on the battlefield indefinitely. It continues to have haste.");
    supported("Splinter Twin");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    crate::r_s06_common::attach_new(&mut t, P0, "Splinter Twin", obj(giant));
    let before = t.g.battlefield.clone();
    activate_containing(&mut t, P0, giant, "Create a token").expect("activation");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    stifle_the_delayed_trigger(&mut t);
    assert!(t.on_battlefield(toks[0]));
    assert!(has_kw(&t, toks[0], KeywordKind::Haste));
}

#[test]
fn flameshadow_conjuring_token_isnt_exiled_if_the_delayed_trigger_is_countered() {
    cr!("603.7c", "611.2a", "701.6b");
    ruling!("Flameshadow Conjuring", "If the token isn't exiled at the beginning of the next end step (perhaps because the delayed triggered ability is countered), it remains on the battlefield indefinitely. It continues to have haste.");
    supported("Flameshadow Conjuring");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Flameshadow Conjuring");
    t.lands(P0, "Mountain", 1);
    t.answer_yes(P0, true);
    let before = t.g.battlefield.clone();
    t.enter(P0, "Hill Giant");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    assert!(has_kw(&t, toks[0], KeywordKind::Haste));
    stifle_the_delayed_trigger(&mut t);
    assert!(t.on_battlefield(toks[0]));
    assert!(has_kw(&t, toks[0], KeywordKind::Haste));
}

#[test]
fn mimic_vat_token_copy_of_a_noncreature_card_has_haste() {
    cr!("707.2", "611.2a", "702.10a");
    ruling!("Mimic Vat", "If the token is a copy of a noncreature card, it will still have haste, though that won't matter unless that token somehow becomes a creature.");
    supported("Mimic Vat");
    supported("Mutavault");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mimic Vat");
    // Mutavault, a 2/2 creature until end of turn, dies: the Vat exiles the land card.
    let vault = t.battlefield(P0, "Mutavault");
    t.lands(P0, "Wastes", 1);
    activate_containing(&mut t, P0, vault, "becomes a 2/2").expect("animate");
    t.resolve_all();
    assert!(is_creature(&t, vault));
    t.answer_yes(P0, true);
    kill(&mut t, vault);
    t.resolve_all();
    assert!(t.in_exile("Mutavault"), "{}", t.dump_log());
    let vat = t.named_on_battlefield("Mimic Vat")[0];
    t.lands(P0, "Wastes", 3);
    let before = t.g.battlefield.clone();
    activate_containing(&mut t, P0, vat, "Create a token").expect("activation");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    let tok = toks[0];
    assert_eq!(name_of(&t, tok), "Mutavault");
    assert!(!is_creature(&t, tok));
    assert!(has_kw(&t, tok, KeywordKind::Haste));
    // It becomes a creature: it can attack (and tap for its ability) right away.
    t.lands(P0, "Wastes", 1);
    activate_containing(&mut t, P0, tok, "becomes a 2/2").expect("animate");
    t.resolve_all();
    assert!(is_creature(&t, tok) && has_kw(&t, tok, KeywordKind::Haste));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(crate::r_s02_common::can_attack(&mut t, tok));
}
