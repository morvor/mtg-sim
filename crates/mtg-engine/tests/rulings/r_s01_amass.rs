//! Rulings batch S01 — amass (CR 701.47): "If you don't control an Army creature, create a
//! 0/0 black [subtype] Army creature token. Choose an Army creature you control. Put N
//! +1/+1 counters on that creature. If it isn't a [subtype], it becomes a [subtype] in
//! addition to its other types."

use crate::r_s01_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn armies(t: &TestGame) -> Vec<ObjectId> {
    with_subtype(t, P0, "Army")
}

/// P0 casts the real spell `name` (with the mana for it) with the given targets and
/// resolves the stack.
fn cast(t: &mut TestGame, name: &str, targets: &[Entity]) {
    supported(name);
    give_mana_for(t, P0, name);
    let c = t.hand(P0, name);
    t.cast(P0, c).targets(targets).go();
    t.resolve_all();
}

fn assert_army(t: &TestGame, army: ObjectId, subtypes: &[&str], pt: (i32, i32)) {
    let o = t.obj_now(army);
    assert!(o.is_token());
    assert_eq!(o.chars.colors, ColorSet::single(Color::Black));
    assert!(o.chars.has_subtype("Army"));
    for s in subtypes {
        assert!(o.chars.has_subtype(s), "not a {s}: {:?}", o.chars.subtypes);
    }
    assert_eq!(t.pt(army), pt);
}

#[test]
fn amass_orcs_creates_an_orc_army_and_an_existing_army_becomes_an_orc() {
    cr!("701.47a");
    ruling!(
        "Orcish Medicine",
        "To amass Orcs N, if you don't control an Army creature, create a 0/0 black Orc Army creature token. Then you choose an Army creature you control and put N +1/+1 counters on it. If that Army isn't already an Orc, it becomes an Orc in addition to its other types."
    );
    ruling!(
        "Orcish Medicine",
        "Amass Zombies works the same way, except you create a 0/0 black Zombie Army creature token if you don't control an Army. If the Army creature you chose isn't already a Zombie, it becomes a Zombie in addition to its other types. By combining cards with amass Orcs and amass Zombies, you can end up with an Orc Zombie Army."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // "Target creature gains your choice of lifelink or indestructible until end of turn.
    // Amass Orcs 1."
    cast(&mut t, "Orcish Medicine", &[Entity::Object(bears)]);
    let army = armies(&t);
    assert_eq!(army.len(), 1);
    let orc = army[0];
    assert_army(&t, orc, &["Orc"], (1, 1));
    assert!(!t.obj(orc).chars.has_subtype("Zombie"));
    // Amass Zombies with that Army: it gets the counter and becomes a Zombie too.
    let target = t.battlefield(P1, "Grizzly Bears");
    cast(&mut t, "Callous Dismissal", &[Entity::Object(target)]);
    assert_eq!(armies(&t), vec![orc]);
    assert_army(&t, orc, &["Orc", "Zombie"], (2, 2));
    // In a game without an Army, amass Zombies creates a Zombie Army.
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Grizzly Bears");
    cast(&mut t, "Callous Dismissal", &[Entity::Object(target)]);
    let army = armies(&t);
    assert_eq!(army.len(), 1);
    assert_army(&t, army[0], &["Zombie"], (1, 1));
    assert!(!t.obj(army[0]).chars.has_subtype("Orc"));
}

#[test]
fn amass_goblins_creates_a_goblin_army_and_can_make_a_goblin_orc_army() {
    cr!("701.47a");
    ruling!(
        "Rage into the Valley",
        "To amass Goblins N, if you don't control an Army creature, create a 0/0 black Goblin Army creature token. Then you choose an Army creature you control and put N +1/+1 counters on it. If that Army isn't already a Goblin, it becomes a Goblin in addition to its other types."
    );
    ruling!(
        "Rage into the Valley",
        "Amass Orcs works the same way, except you create a 0/0 black Orc Army creature token if you don't control an Army. And if the Army creature you chose isn't already an Orc, it becomes an Orc in addition to its other types. By combining cards with amass Orcs and amass Goblins, you can end up with a Goblin Orc Army."
    );
    let mut t = TestGame::new(2);
    // "You draw a card and lose 1 life. Amass Goblins 2."
    cast(&mut t, "Rage into the Valley", &[]);
    let army = armies(&t);
    assert_eq!(army.len(), 1);
    let goblin = army[0];
    assert_army(&t, goblin, &["Goblin"], (2, 2));
    assert!(!t.obj(goblin).chars.has_subtype("Orc"));
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast(&mut t, "Orcish Medicine", &[Entity::Object(bears)]);
    assert_eq!(armies(&t), vec![goblin]);
    assert_army(&t, goblin, &["Goblin", "Orc"], (3, 3));
}

/// Mentor of the Meek's trigger ("Whenever another creature you control with power 2 or
/// less enters, you may pay {1}. If you do, draw a card.").
const MENTOR: &str = "power 2 or less enters";

#[test]
fn a_new_orc_army_enters_as_a_0_0_before_getting_counters() {
    cr!("701.47a", "603.6a");
    ruling!(
        "Saruman, the White Hand",
        "If you don't control an Army, the Orc Army token you create enters the battlefield as a 0/0 creature before receiving counters. Any abilities that trigger when a creature with a certain power enters the battlefield, such as that of Mentor of the Meek, will see the token enter as a 0/0 creature before it gets +1/+1 counters."
    );
    supported("Saruman, the White Hand");
    supported("Mentor of the Meek");
    let mut t = TestGame::new(2);
    // "Whenever you cast a noncreature spell, amass Orcs X, where X is that spell's mana
    // value."
    t.battlefield(P0, "Saruman, the White Hand");
    t.battlefield(P0, "Mentor of the Meek");
    t.lands(P0, "Island", 3);
    let div = t.hand(P0, "Divination");
    t.cast(P0, div).go();
    // The amass trigger resolves: a 3/3 Army, which Mentor of the Meek saw enter as 0/0.
    t.resolve();
    let army = armies(&t);
    assert_eq!(army.len(), 1);
    assert_eq!(t.pt(army[0]), (3, 3));
    assert_eq!(triggers_on_stack(&t, MENTOR), 1);
}

#[test]
fn a_new_zombie_army_enters_as_a_0_0_before_getting_counters() {
    cr!("701.47a", "603.6a");
    ruling!(
        "Commence the Endgame",
        "If you don't control an Army, the Zombie Army token you create enters the battlefield as a 0/0 creature before receiving counters."
    );
    supported("Commence the Endgame");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mentor of the Meek");
    for _ in 0..3 {
        t.hand(P0, "Island");
    }
    // "Draw two cards, then amass Zombies X, where X is the number of cards in your
    // hand." Five cards in hand: a 5/5 Army.
    give_mana_for(&mut t, P0, "Commence the Endgame");
    let c = t.hand(P0, "Commence the Endgame");
    t.cast(P0, c).go();
    t.resolve();
    let army = armies(&t);
    assert_eq!(army.len(), 1);
    assert_eq!(t.pt(army[0]), (5, 5));
    assert_eq!(triggers_on_stack(&t, MENTOR), 1);
}

#[test]
fn a_new_goblin_army_enters_as_a_0_0_before_getting_counters() {
    cr!("701.47a", "603.6a");
    ruling!(
        "Fearsome Goblin Pair",
        "If you don't control an Army, the Goblin Army token you create enters the battlefield as a 0/0 creature before receiving counters."
    );
    supported("Fearsome Goblin Pair");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mentor of the Meek");
    // "When this creature dies, amass Goblins 4."
    let pair = t.battlefield(P0, "Fearsome Goblin Pair");
    t.g.destroy(pair, None);
    t.resolve();
    let army = armies(&t);
    assert_eq!(army.len(), 1);
    assert_eq!(t.pt(army[0]), (4, 4));
    assert_eq!(triggers_on_stack(&t, MENTOR), 1);
}

#[test]
fn with_several_armies_you_choose_which_gets_the_counters() {
    cr!("701.47a");
    ruling!(
        "Orcish Medicine",
        "In the rare case that you control multiple Army creatures (perhaps because you played a creature with changeling) while you amass Orcs, you choose which of your Army creatures to put the +1/+1 counters on. If that creature isn't an Orc, it becomes an Orc in addition to its other types."
    );
    ruling!(
        "Rage into the Valley",
        "In the rare case that you control multiple Army creatures (perhaps because you control a creature with changeling) while you amass Goblins, you choose which of your Army creatures to put the +1/+1 counters on. If that creature isn't a Goblin, it becomes a Goblin in addition to its other types."
    );
    let mut t = TestGame::new(2);
    // A Zombie Army, and a changeling (every creature type, so also an Army).
    let target = t.battlefield(P1, "Grizzly Bears");
    cast(&mut t, "Callous Dismissal", &[Entity::Object(target)]);
    let zombie = armies(&t)[0];
    let changeling = t.battlefield(P0, "Changeling Outcast");
    assert_eq!(armies(&t).len(), 2);
    // Amass Orcs on the Zombie Army: it becomes an Orc.
    t.answer_choose(P0, &[Entity::Object(zombie)]);
    cast(&mut t, "Orcish Medicine", &[Entity::Object(changeling)]);
    assert_army(&t, zombie, &["Zombie", "Orc"], (2, 2));
    assert_eq!(t.counters(changeling, "+1/+1"), 0);
    // Amass Goblins on the changeling.
    t.answer_choose(P0, &[Entity::Object(changeling)]);
    cast(&mut t, "Rage into the Valley", &[]);
    assert_eq!(t.counters(changeling, "+1/+1"), 2);
    assert_eq!(t.pt(zombie), (2, 2));
    assert!(!t.obj(zombie).chars.has_subtype("Goblin"));
}

#[test]
fn an_amass_orcs_spell_whose_target_is_illegal_doesnt_amass() {
    cr!("608.2b");
    ruling!(
        "Orcish Medicine",
        "Some spells and abilities that amass Orcs may require targets. If each target chosen is an illegal target as that spell or ability tries to resolve, it won't resolve. You won't amass Orcs."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Orcish Medicine");
    let c = t.hand(P0, "Orcish Medicine");
    t.cast(P0, c).target(bears).go();
    // In response, the target leaves the battlefield.
    t.lands(P1, "Island", 1);
    let unsummon = t.hand(P1, "Unsummon");
    t.cast(P1, unsummon).target(bears).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(armies(&t).is_empty());
    assert!(t.in_graveyard(P0, "Orcish Medicine"));
}

#[test]
fn an_amass_zombies_spell_whose_target_is_illegal_doesnt_amass() {
    cr!("608.2b");
    ruling!(
        "Callous Dismissal",
        "Some spells and abilities that amass Zombies may require targets. If each target chosen is an illegal target as that spell or ability tries to resolve, it won't resolve. You won't amass Zombies."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Callous Dismissal");
    // "Return target nonland permanent to its owner's hand. Amass Zombies 1."
    let c = t.hand(P0, "Callous Dismissal");
    t.cast(P0, c).target(bears).go();
    t.lands(P1, "Island", 1);
    let unsummon = t.hand(P1, "Unsummon");
    t.cast(P1, unsummon).target(bears).go();
    t.resolve_all();
    assert!(armies(&t).is_empty());
}

#[test]
fn an_amass_goblins_ability_whose_target_is_illegal_doesnt_amass() {
    cr!("608.2b");
    ruling!(
        "Rage into the Valley",
        "Some spells and abilities that amass Goblins may require targets. If each target chosen is an illegal target as that spell or ability tries to resolve, it won't resolve. You won't amass Goblins."
    );
    // No supported card has a targeted amass Goblins ability (Azog, Moria's Ruin and
    // Bolg of the North aren't compiled yet): a custom spell with one.
    let bolt = custom_card(
        "Goblin Muster",
        "Instant",
        "{R}",
        None,
        "Target creature gets +1/+0 until end of turn. Amass Goblins 2.",
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let c = t.custom(P0, bolt.clone(), mtg_engine::object::Zone::Hand(P0));
    t.cast(P0, c).target(bears).go();
    t.lands(P1, "Island", 1);
    let unsummon = t.hand(P1, "Unsummon");
    t.cast(P1, unsummon).target(bears).go();
    t.resolve_all();
    assert!(armies(&t).is_empty());
    // With a legal target, it amasses Goblins.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let c = t.custom(P0, bolt, mtg_engine::object::Zone::Hand(P0));
    t.cast(P0, c).target(bears).go();
    t.resolve_all();
    let army = armies(&t);
    assert_eq!(army.len(), 1);
    assert_army(&t, army[0], &["Goblin"], (2, 2));
}

#[test]
fn zombie_token_bonuses_apply_to_every_zombie_token() {
    cr!("111.1", "613.1f");
    ruling!(
        "Vizier of the Scorpion",
        "Some cards that cause you to amass Zombies also provide bonuses to \"Zombie tokens.\" These affect any token that happens to be a Zombie, not just a Zombie Army you've amassed."
    );
    supported("Vizier of the Scorpion");
    supported("Moan of the Unhallowed");
    let mut t = TestGame::new(2);
    // "Zombie tokens you control have deathtouch."
    let vizier = t.battlefield(P0, "Vizier of the Scorpion");
    // Two 2/2 black Zombie tokens (not Armies), and two Soldier tokens.
    cast(&mut t, "Moan of the Unhallowed", &[]);
    cast(&mut t, "Raise the Alarm", &[]);
    let zombies = with_subtype(&t, P0, "Zombie");
    let zombie_tokens: Vec<ObjectId> = zombies
        .iter()
        .copied()
        .filter(|z| t.obj(*z).is_token())
        .collect();
    assert_eq!(zombie_tokens.len(), 2);
    for z in &zombie_tokens {
        assert!(!t.obj(*z).chars.has_subtype("Army"));
        assert!(t.obj(*z).chars.has_keyword(KeywordKind::Deathtouch));
    }
    for s in with_subtype(&t, P0, "Soldier") {
        assert!(!t.obj(s).chars.has_keyword(KeywordKind::Deathtouch));
    }
    // The Vizier is a Zombie, but not a token.
    assert!(!t.obj(vizier).chars.has_keyword(KeywordKind::Deathtouch));
}
