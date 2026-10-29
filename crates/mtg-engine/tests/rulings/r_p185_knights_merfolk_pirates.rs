//! Rulings batch P185 — Knight, Merfolk, Pirate, Griffin, Mouse, Outlaw and Rebel typal
//! cards: simultaneous deaths (CR 603.10a, 704.3), cast triggers resolving before the
//! spell (CR 603.3), targeting with "target [type] you control" (CR 115), cost increases
//! for opponents' spells and abilities (CR 601.2f, 602.2b), intervening "if" clauses (CR
//! 603.4), "unless" checked on resolution, counts made as an ability resolves (CR 608.2h)
//! and Two-Headed Giant attacks (CR 810.7).

use crate::r_p185_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::{create_token, destroy};
use crate::r_s19_common::add_lore;
use crate::r_s25_common::cast_new;
use crate::r_s33_common::two_headed_giant;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn locthwain_lancer_triggers_for_each_nontoken_knight_dying_with_it() {
    cr!("603.10a", "603.2c");
    ruling!(
        "Locthwain Lancer",
        "If Locthwain Lancer dies at the same time as other nontoken Knights you control, its ability will trigger once for itself (assuming it’s not a token) and once for each of those other Knights."
    );
    supported("Locthwain Lancer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Locthwain Lancer");
    t.battlefield(P0, "Craven Knight");
    create_token(&mut t, P0, "Knight");
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Day of Judgment", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn venerable_knight_cant_save_a_knight_dealt_lethal_damage_with_it() {
    cr!("704.3", "603.10a");
    ruling!(
        "Venerable Knight",
        "If a Knight you control is dealt lethal damage at the same time as Venerable Knight, they die at the same time. That Knight can't receive a counter from Venerable Knight's ability in time to save it."
    );
    supported("Venerable Knight");
    supported("Pyroclasm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Venerable Knight");
    t.battlefield(P0, "Craven Knight");
    // A Knight that survives gets the counter.
    let big = t.battlefield(P0, "Kithkin Billyrider");
    cast_new(&mut t, P0, "Pyroclasm", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Venerable Knight"));
    assert!(t.in_graveyard(P0, "Craven Knight"));
    assert_eq!(t.counters(big, counters::PLUS1), 1);
}

#[test]
fn history_of_benalia_chapter_three_affects_only_knights_there_as_it_resolves() {
    cr!("611.2c", "714.2b");
    ruling!(
        "History of Benalia",
        "The final chapter ability of History of Benalia affects only Knights you control at the time it resolves. Creatures you begin to control later in the turn or that become Knights later in the turn won’t get +2/+1."
    );
    supported("History of Benalia");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "History of Benalia");
    t.g.objects[saga.0 as usize]
        .counters
        .insert(counters::LORE.into(), 2);
    let knight = t.battlefield(P0, "Craven Knight");
    let bears = t.battlefield(P0, "Grizzly Bears");
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    assert_eq!(t.pt(knight), (4, 3));
    let late = t.battlefield(P0, "Craven Knight");
    add_types(&mut t, bears, &["Knight"]);
    assert_eq!(t.pt(late), (2, 2));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn knight_and_merfolk_cast_triggers_resolve_first_even_if_the_spell_is_countered() {
    cr!("603.3", "601.2i", "603.2");
    ruling!(
        "Worthy Knight",
        "Worthy Knight's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Worthy Knight",
        "Worthy Knight's triggered ability won't trigger when you cast it because it's not on the battlefield yet."
    );
    ruling!(
        "Deeproot Waters",
        "The ability of Deeproot Waters resolves before the spell that caused it to trigger."
    );
    supported("Worthy Knight");
    supported("Deeproot Waters");
    // Casting Worthy Knight itself: no trigger.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Worthy Knight", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // A Knight spell with Worthy Knight out: the trigger resolves though the spell is
    // countered.
    t.resolve_all();
    let spell = cast_new(&mut t, P0, "Craven Knight", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.g.counter(spell, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Human").len(), 2, "Worthy Knight and a token");
    assert!(t.in_graveyard(P0, "Craven Knight"));
    // Deeproot Waters: the token exists while the Merfolk spell is still on the stack.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Deeproot Waters");
    let spell = cast_new(&mut t, P0, "Coral Commando", &[]);
    t.settle();
    t.resolve();
    assert_eq!(tokens(&t, P0).len(), 1);
    assert_eq!(t.zone(spell), Zone::Stack);
}

#[test]
fn merfolk_enters_triggers_can_target_themselves_or_the_entering_merfolk() {
    cr!("115.1", "603.3d");
    ruling!(
        "Deeproot Elite",
        "Deeproot Elite's ability can target the Merfolk that caused it to trigger. It can also target Deeproot Elite itself."
    );
    ruling!("Jade Guardian", "Jade Guardian can be the target of its own ability.");
    ruling!(
        "Vineshaper Mystic",
        "Vineshaper Mystic can be the target of its own ability."
    );
    supported("Deeproot Elite");
    supported("Jade Guardian");
    supported("Vineshaper Mystic");
    // Deeproot Elite targeting itself, then the Merfolk that entered.
    for target_self in [true, false] {
        let mut t = TestGame::new(2);
        let elite = t.battlefield(P0, "Deeproot Elite");
        let entered = t.enter(P0, "Maritime Guard");
        let m = if target_self { elite } else { entered };
        t.answer_targets(P0, &[Entity::Object(m)]);
        t.g.flush_events();
        t.settle();
        t.resolve_all();
        assert_eq!(t.counters(m, counters::PLUS1), 1, "self: {target_self}");
    }
    for name in ["Jade Guardian", "Vineshaper Mystic"] {
        let mut t = TestGame::new(2);
        let id = enter_self_targeting(&mut t, name);
        t.resolve_all();
        assert_eq!(t.counters(id, counters::PLUS1), 1, "{name}");
    }
}

/// Puts `name` onto the battlefield for P0, its enters trigger targeting itself.
fn enter_self_targeting(t: &mut TestGame, name: &str) -> ObjectId {
    let id = t.enter(P0, name);
    t.clear_answers();
    t.answer_targets(P0, &[Entity::Object(id)]);
    t.g.flush_events();
    t.settle();
    id
}

#[test]
fn vineshaper_mystic_cant_target_the_same_merfolk_twice() {
    cr!("115.3");
    ruling!(
        "Vineshaper Mystic",
        "You can’t target the same Merfolk twice to have it receive two +1/+1 counters."
    );
    supported("Vineshaper Mystic");
    let mut t = TestGame::new(2);
    let guard = t.battlefield(P0, "Maritime Guard");
    t.answer_targets(P0, &[Entity::Object(guard), Entity::Object(guard)]);
    t.enter(P0, "Vineshaper Mystic");
    t.g.flush_events();
    t.settle();
    // The trigger goes on the stack, but the Guard can't be its target twice (the
    // answer is rejected).
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.counters(guard, counters::PLUS1) < 2);
    // Two different Merfolk: one counter each.
    let mut t = TestGame::new(2);
    let guard = t.battlefield(P0, "Maritime Guard");
    let other = t.battlefield(P0, "Maritime Guard");
    t.answer_targets(P0, &[Entity::Object(guard), Entity::Object(other)]);
    t.enter(P0, "Vineshaper Mystic");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(guard, counters::PLUS1), 1);
    assert_eq!(t.counters(other, counters::PLUS1), 1);
}

#[test]
fn kopala_taxes_opponents_spells_and_activated_abilities_but_not_triggers() {
    cr!("601.2f", "602.2b", "202.3", "603.3d");
    ruling!(
        "Kopala, Warden of Waves",
        "To determine the total cost of an opponent's spell that targets a Merfolk you control, start with the mana cost or alternative cost that player is paying, add any cost increases, then apply any cost reductions. The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    ruling!(
        "Kopala, Warden of Waves",
        "Activated abilities contain a colon. They're generally written \"[Cost]: [Effect].\""
    );
    ruling!(
        "Kopala, Warden of Waves",
        "Kopala's abilities won't affect triggered abilities (starting with \"when,\" \"whenever,\" or \"at\") that target a Merfolk you control."
    );
    supported("Kopala, Warden of Waves");
    // Shock targeting a Merfolk costs {2}{R}; its mana value stays 1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kopala, Warden of Waves");
    let merfolk = t.battlefield(P0, "Headwater Sentries");
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    assert!(t.cast(P1, shock).target(merfolk).try_go().is_err());
    t.lands(P1, "Mountain", 2);
    let spell = t.cast(P1, shock).target(merfolk).go();
    assert_eq!(crate::r_s08_common::mana_value(&t, spell), 1);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P1), 3);
    // Not targeting a Merfolk: no increase.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kopala, Warden of Waves");
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(Entity::Player(P0)).go();
    // An activated ability ("{T}: ...") targeting a Merfolk costs {2} more.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kopala, Warden of Waves");
    let merfolk = t.battlefield(P0, "Headwater Sentries");
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    assert!(t.activate(P1, pyro, 0, &[Entity::Object(merfolk)]).is_err());
    t.lands(P1, "Mountain", 2);
    t.activate(P1, pyro, 0, &[Entity::Object(merfolk)]).unwrap();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P1), 2);
    // A triggered ability targeting a Merfolk costs nothing more: it just resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kopala, Warden of Waves");
    let merfolk = t.battlefield(P0, "Maritime Guard");
    enter_with(&mut t, P1, "Flametongue Kavu", &[Entity::Object(merfolk)]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Maritime Guard"));
}

#[test]
fn might_of_the_meek_with_an_illegal_target_draws_nothing() {
    cr!("608.2b");
    ruling!(
        "Might of the Meek",
        "If the target creature is an illegal target as Might of the Meek tries to resolve, it won’t resolve and none of its effects will happen. You won’t draw a card."
    );
    supported("Might of the Meek");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Might of the Meek", &[Entity::Object(bears)]);
    destroy(&mut t, bears);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.in_graveyard(P0, "Might of the Meek"));
}

#[test]
fn hellspur_posse_boss_leaving_doesnt_remove_attacking_outlaws() {
    cr!("506.4", "508.1");
    ruling!(
        "Hellspur Posse Boss",
        "Once an outlaw that came under your control this turn has legally attacked, causing it to lose haste by removing Hellspur Posse Boss won’t cause that outlaw to stop attacking."
    );
    supported("Hellspur Posse Boss");
    let mut t = TestGame::new(2);
    let boss = t.battlefield(P0, "Hellspur Posse Boss");
    let rogue = t.battlefield_sick(P0, "Bane Alley Blackguard");
    attack_with(&mut t, &[(rogue, Entity::Player(P1))]);
    destroy(&mut t, boss);
    assert!(t.g.is_attacking(rogue));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn dire_fleet_captain_counts_other_attacking_pirates_as_it_resolves() {
    cr!("608.2h", "611.2c", "508.1m");
    ruling!(
        "Dire Fleet Captain",
        "Count the number of other attacking Pirates as the ability resolves to determine the size of the bonus."
    );
    ruling!(
        "Dire Fleet Captain",
        "Once the ability resolves, the bonus won’t change later in the turn, even if the number of other attacking Pirates does."
    );
    supported("Dire Fleet Captain");
    let mut t = TestGame::new(2);
    let cap = t.battlefield(P0, "Dire Fleet Captain");
    let a = t.battlefield(P0, "Swab Goblin");
    let b = t.battlefield(P0, "Swab Goblin");
    let c = t.battlefield(P0, "Talas Merchant");
    attack_with(
        &mut t,
        &[
            (cap, Entity::Player(P1)),
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
            (c, Entity::Player(P1)),
        ],
    );
    // One Pirate leaves before the trigger resolves: two others are attacking.
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(t.pt(cap), (4, 4));
    destroy(&mut t, b);
    assert_eq!(t.pt(cap), (4, 4));
}

#[test]
fn dire_fleet_captain_counts_a_teammates_attacking_pirates() {
    cr!("810.7", "608.2h");
    ruling!(
        "Dire Fleet Captain",
        "In a Two-Headed Giant game, attacking Pirates controlled by your teammate will count for Dire Fleet Captain’s ability."
    );
    let mut t = two_headed_giant();
    let cap = t.battlefield(P0, "Dire Fleet Captain");
    let mate = t.battlefield(P1, "Swab Goblin");
    attack_with(
        &mut t,
        &[(cap, Entity::Player(P2)), (mate, Entity::Player(P2))],
    );
    t.resolve_all();
    assert_eq!(t.pt(cap), (3, 3));
}

#[test]
fn dreamcaller_siren_needs_another_pirate_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Dreamcaller Siren",
        "Dreamcaller Siren's last ability won't trigger if you don't control another Pirate as it enters the battlefield. If you lose control of each other Pirate before that ability resolves, it won't do anything."
    );
    supported("Dreamcaller Siren");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    enter_with(&mut t, P0, "Dreamcaller Siren", &[Entity::Object(bears)]);
    assert_eq!(t.stack_len(), 0);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let pirate = t.battlefield(P0, "Swab Goblin");
    enter_with(&mut t, P0, "Dreamcaller Siren", &[Entity::Object(bears)]);
    assert_eq!(t.stack_len(), 1);
    crate::r_s06_common::give_control(&mut t, pirate, P1);
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Swab Goblin");
    enter_with(&mut t, P0, "Dreamcaller Siren", &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn fathom_fleet_boarder_always_triggers_and_checks_for_a_pirate_on_resolution() {
    cr!("603.2", "608.2c");
    ruling!(
        "Fathom Fleet Boarder",
        "Fathom Fleet Boarder’s ability triggers regardless of whether you control another Pirate. Whether you control another Pirate is checked only as the ability resolves."
    );
    supported("Fathom Fleet Boarder");
    // A Pirate at first, gone before it resolves: lose 2.
    let mut t = TestGame::new(2);
    let pirate = t.battlefield(P0, "Swab Goblin");
    enter_with(&mut t, P0, "Fathom Fleet Boarder", &[]);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, pirate);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    // No Pirate at first, one arrives before it resolves: lose nothing.
    let mut t = TestGame::new(2);
    enter_with(&mut t, P0, "Fathom Fleet Boarder", &[]);
    assert_eq!(t.stack_len(), 1);
    t.battlefield(P0, "Swab Goblin");
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn griffin_canyon_can_target_an_untapped_griffin() {
    cr!("701.26b", "115.1");
    ruling!(
        "Griffin Canyon",
        "This can target an already untapped Griffin, and gives the bonus regardless."
    );
    supported("Griffin Canyon");
    let mut t = TestGame::new(2);
    let canyon = t.battlefield(P0, "Griffin Canyon");
    let griffin = t.battlefield(P0, "Sunspire Griffin");
    t.activate(P0, canyon, 1, &[Entity::Object(griffin)]).unwrap();
    t.resolve_all();
    assert!(!t.obj_now(griffin).tapped);
    assert_eq!(t.pt(griffin), (3, 4));
}

#[test]
fn ramosian_revivalist_returns_a_rebel_permanent_card() {
    cr!("115.1");
    ruling!(
        "Ramosian Revivalist",
        "A “permanent card” is an artifact card, a creature card, an enchantment card, a planeswalker card, or a land card. Most “Rebel permanent cards” are creature cards."
    );
    supported("Ramosian Revivalist");
    let mut t = TestGame::new(2);
    let rr = t.battlefield(P0, "Ramosian Revivalist");
    let rider = t.graveyard(P0, "Jhovall Rider");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 6);
    let from = t.asked().len();
    t.activate(P0, rr, 0, &[Entity::Object(rider)]).unwrap();
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(cands.iter().all(|c| !c.contains(&Entity::Object(bears))));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Jhovall Rider").len(), 1);
}
