//! Rulings batch S13 — populate (CR 701.36a): "To populate means to choose a creature
//! token you control and create a token that's a copy of that creature token." The copy is
//! created like any token copy (CR 707.2, 111.10): it copies the copiable values, including
//! "enters with" and "as enters" abilities, and enters like any permanent.

use crate::r_s01_common::*;
use crate::r_s13_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Casts Cackling Counterpart ("Create a token that's a copy of target creature you
/// control.") on `of`, and returns the token.
fn counterpart(t: &mut TestGame, of: ObjectId) -> ObjectId {
    let before = tokens(t, P0);
    let cc = t.hand(P0, "Cackling Counterpart");
    give_mana_for(t, P0, "Cackling Counterpart");
    t.cast(P0, cc).target(of).go();
    t.resolve_all();
    tokens(t, P0)
        .into_iter()
        .find(|x| !before.contains(x))
        .expect("token copy")
}

/// The tokens `p` controls that aren't in `before`.
fn new_tokens(t: &TestGame, p: PlayerId, before: &[ObjectId]) -> Vec<ObjectId> {
    tokens(t, p)
        .into_iter()
        .filter(|x| !before.contains(x))
        .collect()
}

/// Queues `p`'s choice of creature type for an "as enters, choose a creature type"
/// ability.
fn pick_type(t: &mut TestGame, p: PlayerId, ty: &str) {
    let i = subtype_lists()
        .creature
        .iter()
        .position(|s| s == ty)
        .unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

/// Activates Trostani, Selesnya's Voice's "{1}{G}{W}, {T}: Populate." choosing `token`.
fn trostani_populates(t: &mut TestGame, trostani: ObjectId, token: ObjectId) -> Vec<ObjectId> {
    t.g.objects[trostani.0 as usize].tapped = false;
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Plains", 1);
    let before = tokens(t, P0);
    t.answer_choose(P0, &[Entity::Object(token)]);
    t.activate(P0, trostani, 0, &[]).expect("Trostani");
    t.resolve_all();
    new_tokens(t, P0, &before)
}

#[test]
fn the_new_token_is_the_token_its_creating_effect_described() {
    cr!("701.36a", "111.3", "707.2");
    ruling!(
        "Trostani, Selesnya's Voice",
        "The new creature token copies the characteristics of the original token as stated by the effect that created the original token."
    );
    supported("Trostani, Selesnya's Voice");
    supported("Muster the Departed");
    let mut t = TestGame::new(2);
    let trostani = t.battlefield(P0, "Trostani, Selesnya's Voice");
    // Muster the Departed: "When this enchantment enters, create a 1/1 white Spirit
    // creature token with flying."
    t.enter(P0, "Muster the Departed");
    t.resolve_all();
    let spirit = tokens(&t, P0)[0];
    // The Spirit is changed by noncopy effects: a +1/+1 counter and Giant Growth.
    add(&mut t, spirit, counters::PLUS1, 1);
    let growth = t.hand(P0, "Giant Growth");
    give_mana_for(&mut t, P0, "Giant Growth");
    t.cast(P0, growth).target(spirit).go();
    t.resolve_all();
    assert_eq!(t.pt(spirit), (5, 5));
    let new = trostani_populates(&mut t, trostani, spirit);
    assert_eq!(new.len(), 1);
    let o = t.obj(new[0]);
    assert!(o.is_token());
    assert_eq!(o.chars.name, t.obj(spirit).chars.name);
    assert_eq!(o.chars.colors, ColorSet::single(Color::White));
    assert!(o.chars.is_creature() && o.chars.has_subtype("Spirit"));
    assert!(o.chars.has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    assert_eq!(t.pt(new[0]), (1, 1));
}

#[test]
fn populating_a_token_copy_copies_what_that_token_copies() {
    cr!("701.36a", "707.2", "707.9b");
    ruling!(
        "Rootborn Defenses",
        "If you choose to copy a creature token that's a copy of another creature, the new creature token will copy the characteristics of whatever the original token is copying."
    );
    supported("Rootborn Defenses");
    supported("Cackling Counterpart");
    supported("Croaking Counterpart");
    // A token copy of Elvish Visionary (Cackling Counterpart), populated with Rootborn
    // Defenses ("Populate. Creatures you control gain indestructible until end of turn.").
    let mut t = TestGame::new(2);
    let visionary = t.battlefield(P0, "Elvish Visionary");
    let token = counterpart(&mut t, visionary);
    assert_eq!(t.obj(token).chars.name, "Elvish Visionary");
    let before = tokens(&t, P0);
    let rd = t.hand(P0, "Rootborn Defenses");
    give_mana_for(&mut t, P0, "Rootborn Defenses");
    t.answer_choose(P0, &[Entity::Object(token)]);
    t.cast(P0, rd).go();
    t.resolve_all();
    let new = new_tokens(&t, P0, &before);
    assert_eq!(new.len(), 1);
    let o = t.obj(new[0]);
    assert_eq!(o.chars.name, "Elvish Visionary");
    assert!(o.chars.has_subtype("Elf") && o.chars.has_subtype("Shaman"));
    assert_eq!(o.chars.colors, ColorSet::single(Color::Green));
    assert_eq!(t.pt(new[0]), (1, 1));
    // A token copy of Grizzly Bears "except it's a 1/1 green Frog" (Croaking
    // Counterpart): the exception is part of what the token copies.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let cc = t.hand(P0, "Croaking Counterpart");
    give_mana_for(&mut t, P0, "Croaking Counterpart");
    t.cast(P0, cc).target(bears).go();
    t.resolve_all();
    let frog = tokens(&t, P0)[0];
    let rd = t.hand(P0, "Rootborn Defenses");
    give_mana_for(&mut t, P0, "Rootborn Defenses");
    t.answer_choose(P0, &[Entity::Object(frog)]);
    t.cast(P0, rd).go();
    t.resolve_all();
    let new = new_tokens(&t, P0, &[frog]);
    assert_eq!(new.len(), 1);
    let o = t.obj(new[0]);
    assert_eq!(o.chars.name, "Grizzly Bears");
    assert!(o.chars.has_subtype("Frog") && !o.chars.has_subtype("Bear"));
    assert_eq!(t.pt(new[0]), (1, 1));
}

#[test]
fn the_new_tokens_enters_abilities_work() {
    cr!("701.36a", "707.2", "614.1c", "603.6a");
    ruling!(
        "Druid's Deliverance",
        "Any enters-the-battlefield abilities of the copied token will trigger when the new token enters the battlefield. Any \"as [this creature] enters the battlefield\" or \"[this creature] enters the battlefield with\" abilities of the copied token will also work."
    );
    supported("Druid's Deliverance");
    supported("Patrolling Peacemaker");
    // Token copies of Elvish Visionary ("When this creature enters, draw a card.") and
    // Patrolling Peacemaker ("This creature enters with two +1/+1 counters on it.").
    let mut t = TestGame::new(2);
    let visionary = t.battlefield(P0, "Elvish Visionary");
    let peacemaker = t.battlefield(P0, "Patrolling Peacemaker");
    add(&mut t, peacemaker, counters::PLUS1, 2);
    let v_token = counterpart(&mut t, visionary);
    let p_token = counterpart(&mut t, peacemaker);
    assert_eq!(t.counters(p_token, counters::PLUS1), 2);
    // Druid's Deliverance: "Prevent all combat damage that would be dealt to you this
    // turn. Populate."
    let hand = t.hand_size(P0);
    let dd = t.hand(P0, "Druid's Deliverance");
    give_mana_for(&mut t, P0, "Druid's Deliverance");
    let before = tokens(&t, P0);
    t.answer_choose(P0, &[Entity::Object(v_token)]);
    t.cast(P0, dd).go();
    t.resolve_all();
    assert_eq!(new_tokens(&t, P0, &before).len(), 1);
    // The new Visionary's enters ability drew a card.
    assert_eq!(t.hand_size(P0), hand + 1);
    // The new Peacemaker enters with its two +1/+1 counters (it would die as a 0/0
    // otherwise).
    let dd = t.hand(P0, "Druid's Deliverance");
    give_mana_for(&mut t, P0, "Druid's Deliverance");
    let before = tokens(&t, P0);
    t.answer_choose(P0, &[Entity::Object(p_token)]);
    t.cast(P0, dd).go();
    t.resolve_all();
    let new = new_tokens(&t, P0, &before);
    assert_eq!(new.len(), 1);
    assert_eq!(t.obj(new[0]).chars.name, "Patrolling Peacemaker");
    assert_eq!(t.counters(new[0], counters::PLUS1), 2);
    assert_eq!(t.pt(new[0]), (2, 2));
}

#[test]
fn the_new_tokens_as_enters_and_enters_with_abilities_work() {
    cr!("701.36a", "707.2", "614.1c", "614.12");
    ruling!(
        "Cayth, Famed Mechanist",
        "Any \"as [this creature] enters the battlefield\" or \"[this creature] enters the battlefield with\" abilities of the new token will work."
    );
    supported("Cayth, Famed Mechanist");
    supported("Adaptive Automaton");
    // Cayth: "{2}, {T}: Choose one — • Populate. • Proliferate."
    let cayth_populates = |t: &mut TestGame, cayth: ObjectId, token: ObjectId| {
        t.g.objects[cayth.0 as usize].tapped = false;
        t.lands(P0, "Wastes", 2);
        let before = tokens(t, P0);
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
        t.answer_choose(P0, &[Entity::Object(token)]);
        t.activate(P0, cayth, 0, &[]).expect("Cayth");
        t.resolve_all();
        new_tokens(t, P0, &before)
    };
    // Adaptive Automaton: "As this creature enters, choose a creature type. This creature
    // is the chosen type in addition to its other types. Other creatures you control of
    // the chosen type get +1/+1." The token copy chose Elf; the populated token chooses
    // Goblin as it enters.
    let mut t = TestGame::new(2);
    let cayth = t.battlefield(P0, "Cayth, Famed Mechanist");
    let elf = t.battlefield(P0, "Llanowar Elves");
    let goblin = t.battlefield(P0, "Raging Goblin");
    let automaton = t.battlefield(P0, "Adaptive Automaton");
    pick_type(&mut t, P0, "Elf");
    let a_token = counterpart(&mut t, automaton);
    assert_eq!(
        t.obj(a_token).choices.creature_type.as_deref(),
        Some("Elf")
    );
    pick_type(&mut t, P0, "Goblin");
    let new = cayth_populates(&mut t, cayth, a_token);
    assert_eq!(new.len(), 1);
    assert_eq!(t.obj(new[0]).choices.creature_type.as_deref(), Some("Goblin"));
    assert!(t.obj(new[0]).chars.has_subtype("Goblin"));
    assert_eq!(t.pt(goblin), (2, 2));
    assert_eq!(t.pt(elf), (2, 2));
    // A token copy of Patrolling Peacemaker: the populated token enters with two +1/+1
    // counters.
    let peacemaker = t.battlefield(P0, "Patrolling Peacemaker");
    add(&mut t, peacemaker, counters::PLUS1, 2);
    let p_token = counterpart(&mut t, peacemaker);
    let new = cayth_populates(&mut t, cayth, p_token);
    assert_eq!(new.len(), 1);
    assert_eq!(t.counters(new[0], counters::PLUS1), 2);
    assert!(t.on_battlefield(new[0]));
}
