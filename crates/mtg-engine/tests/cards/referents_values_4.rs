//! Referent and value grammar IV (cards3-referents-values): results "this way", referents
//! to earlier objects, computed values, last-known information (CR 608.2c, 608.2h).

use mtg_engine::decision::Decision;
use mtg_engine::object::{CastMethod, FaceState};
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

fn scried(t: &TestGame, p: PlayerId) -> bool {
    t.asked()
        .iter()
        .any(|(q, d)| *q == p && matches!(d, Decision::Scry { .. }))
}

// --- Players dealt damage this way ----------------------------------------------------

#[test]
fn play_with_fire_scries_only_if_a_player_was_dealt_damage() {
    cr!("608.2c", "120.4b");
    compiles("Play with Fire");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let fire = t.hand(P0, "Play with Fire");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, fire).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(scried(&t, P0), "a player was dealt damage");

    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let fire = t.hand(P0, "Play with Fire");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, fire).target(bear).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bear));
    assert!(!scried(&t, P0), "only a creature was dealt damage");
}

#[test]
fn sonic_shrieker_makes_the_damaged_player_discard() {
    cr!("608.2c");
    compiles("Sonic Shrieker");
    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Sonic Shrieker");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.hand_size(P1), 0, "the player dealt damage discards");

    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Object(bear)]);
    t.enter(P0, "Sonic Shrieker");
    t.resolve_all();
    assert!(!t.on_battlefield(bear));
    assert_eq!(t.hand_size(P1), 1, "no player was dealt damage");
}

#[test]
fn screaming_nemesis_stops_the_damaged_player_gaining_life() {
    cr!("119.7", "608.2c");
    ruling!(
        "Screaming Nemesis",
        "Once Screaming Nemesis's last ability causes it to deal damage to a player, that player won't be able to gain life for the rest of the game."
    );
    compiles("Screaming Nemesis");
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    let nemesis = t.battlefield(P0, "Screaming Nemesis");
    let shock = t.hand(P1, "Shock");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P1, shock).target(nemesis).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    t.g.gain_life(P1, 5);
    assert_eq!(t.life(P1), 18, "can't gain life");
    t.g.gain_life(P0, 5);
    assert_eq!(t.life(P0), 25, "only the player dealt damage");
}

// --- "another" relative to a result ----------------------------------------------------

#[test]
fn deadly_brew_returns_another_permanent_card_if_you_sacrificed_one() {
    cr!("608.2c", "400.7");
    ruling!(
        "Deadly Brew",
        "You cannot return the same permanent card that you sacrificed."
    );
    compiles("Deadly Brew");
    // Only the sacrificed card is in the graveyard: nothing to return.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let brew = t.hand(P0, "Deadly Brew");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.cast(P0, brew).go();
    t.resolve_all();
    assert!(!t.on_battlefield(mine) && !t.on_battlefield(theirs));
    assert!(
        t.in_graveyard(P0, "Grizzly Bears"),
        "not the sacrificed card"
    );
    assert!(!t.in_hand(P0, "Grizzly Bears"));

    // Another permanent card.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    let elves = t.graveyard(P0, "Llanowar Elves");
    let brew = t.hand(P0, "Deadly Brew");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(elves)]);
    t.cast(P0, brew).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Llanowar Elves"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));

    // You sacrificed nothing (an opponent did): no return.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    t.battlefield(P1, "Hill Giant");
    let elves = t.graveyard(P0, "Llanowar Elves");
    let brew = t.hand(P0, "Deadly Brew");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(elves)]);
    t.cast(P0, brew).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(
        t.in_graveyard(P0, "Llanowar Elves"),
        "you didn't sacrifice a permanent"
    );
}

#[test]
fn rise_of_the_witch_king_returns_another_permanent_card_to_the_battlefield() {
    cr!("608.2c", "400.7");
    compiles("Rise of the Witch-king");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    let elves = t.graveyard(P0, "Llanowar Elves");
    let rise = t.hand(P0, "Rise of the Witch-king");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(elves)]);
    t.cast(P0, rise).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

// --- Values ------------------------------------------------------------------------------

#[test]
fn eternal_flame_deals_half_x_rounded_up_to_you() {
    cr!("107.1a");
    compiles("Eternal Flame");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let flame = t.hand(P0, "Eternal Flame");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, flame).target(Entity::Player(P1)).go();
    t.resolve_all();
    // Five Mountains: 5 damage, and half of 5 rounded up is 3.
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn breakthrough_keeps_x_cards() {
    cr!("107.3a");
    compiles("Breakthrough");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let keep = t.hand(P0, "Grizzly Bears");
    let bt = t.hand(P0, "Breakthrough");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(keep)]);
    t.cast(P0, bt).x(1).go();
    t.resolve_all();
    // Drew four, kept one: the rest (four) were discarded.
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.graveyard_size(P0), 5, "four cards and Breakthrough");
}

#[test]
fn nahiris_lithoforming_lets_you_play_x_additional_lands() {
    cr!("305.2", "107.3a");
    ruling!(
        "Nahiri's Lithoforming",
        "If X is greater than the number of lands you control, you sacrifice each land you control."
    );
    compiles("Nahiri's Lithoforming");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let nahiri = t.hand(P0, "Nahiri's Lithoforming");
    t.advance_to(P0, Step::PrecombatMain);
    let before = t.hand_size(P0);
    t.cast(P0, nahiri).x(2).go();
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.named_on_battlefield("Mountain").len(), 2);
    assert_eq!(
        t.hand_size(P0),
        before - 1 + 2,
        "drew a card for each land sacrificed"
    );
    assert_eq!(t.g.player(P0).land_plays, 3, "two additional land plays");
}

#[test]
fn ancient_cellarspawn_drains_the_difference() {
    cr!("601.2f", "202.3");
    ruling!(
        "Ancient Cellarspawn",
        "The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    compiles("Ancient Cellarspawn");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ancient Cellarspawn");
    t.lands(P0, "Swamp", 4);
    // A Horror with mana value 4 costs {1} less: 3 mana spent.
    let horror = t.hand(P0, "Ancient Cellarspawn");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, horror).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // A spell without a reduction: no trigger effect.
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn odric_counts_each_listed_ability_once() {
    cr!("608.2h");
    compiles("Odric, Blood-Cursed");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Serra Angel"); // flying, vigilance
    t.battlefield(P0, "Wind Drake"); // flying again
    t.battlefield(P1, "Vampire Nighthawk"); // an opponent's: not counted
    t.advance_to(P0, Step::PrecombatMain);
    t.enter(P0, "Odric, Blood-Cursed");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Blood").len(), 2);
}

#[test]
fn tam_proliferates_once_per_planeswalker_type() {
    cr!("205.3j", "701.34a");
    compiles("Tam, the Possibility");
    let mut t = TestGame::new(2);
    let tam = t.battlefield(P0, "Tam, the Possibility");
    let jace = t.battlefield(P0, "Jace Beleren");
    t.battlefield(P0, "Jace, the Mind Sculptor");
    t.battlefield(P0, "Chandra Nalaar");
    for l in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        t.lands(P0, l, 1);
    }
    t.advance_to(P0, Step::PrecombatMain);
    let before = t.counters(jace, "loyalty");
    t.answer_choose(P0, &[Entity::Object(jace)]);
    t.answer_choose(P0, &[Entity::Object(jace)]);
    t.answer_choose(P0, &[Entity::Object(jace)]);
    t.activate(P0, tam, 0, &[]).unwrap();
    t.resolve_all();
    // Two planeswalker types (Jace, Chandra): proliferate twice.
    assert_eq!(t.counters(jace, "loyalty"), before + 2);
}

#[test]
fn lethal_exploit_counts_modified_creatures_as_it_was_cast() {
    cr!("601.2i", "608.2h");
    compiles("Lethal Exploit");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    t.g.add_counters(Entity::Object(a), "+1/+1", 1, None);
    t.g.add_counters(Entity::Object(b), "+1/+1", 1, None);
    let target = t.battlefield(P1, "Colossal Dreadmaw");
    let exploit = t.hand(P0, "Lethal Exploit");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, exploit).target(target).go();
    // A modification removed after the spell was cast doesn't change the count.
    t.g.remove_counters(Entity::Object(b), "+1/+1", 1);
    t.resolve_all();
    // -2/-2 and -1/-1 for each of the two modified creatures.
    assert_eq!(t.pt(target), (2, 2));
}

// --- Last-known information -----------------------------------------------------------

#[test]
fn lost_isle_calling_takes_an_extra_turn_with_seven_verse_counters() {
    cr!("608.2h", "118.3");
    compiles("Lost Isle Calling");
    for (n, extra) in [(7u32, true), (6, false)] {
        let mut t = TestGame::new(2);
        let isle = t.battlefield(P0, "Lost Isle Calling");
        t.g.add_counters(Entity::Object(isle), "verse", n, None);
        t.lands(P0, "Island", 6);
        t.advance_to(P0, Step::PrecombatMain);
        let before = t.hand_size(P0);
        t.activate(P0, isle, 0, &[]).unwrap();
        t.resolve_all();
        assert!(!t.on_battlefield(isle));
        assert_eq!(
            t.hand_size(P0),
            before + n as usize,
            "a card per verse counter"
        );
        assert_eq!(t.g.extra_turns.contains(&P0), extra, "{n} verse counters");
    }
}

#[test]
fn unforge_damages_the_creature_the_equipment_was_attached_to() {
    cr!("608.2h");
    ruling!(
        "Unforge",
        "Unforge deals damage only to the creature that the Equipment was attached to when the Equipment was destroyed."
    );
    compiles("Unforge");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let sword = t.battlefield(P1, "Bonesplitter");
    t.g.attach(sword, Entity::Object(bear));
    let unforge = t.hand(P0, "Unforge");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, unforge).target(sword).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Bonesplitter"));
    assert!(!t.on_battlefield(bear), "2 damage to the equipped creature");

    // An unattached Equipment: no damage.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let sword = t.battlefield(P1, "Bonesplitter");
    let unforge = t.hand(P0, "Unforge");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, unforge).target(sword).go();
    t.resolve_all();
    assert!(t.on_battlefield(bear));
    assert_eq!(t.g.obj(bear).damage, 0);
}

#[test]
fn dawn_evangel_looks_back_at_the_auras_on_the_creature_that_died() {
    cr!("603.10a", "608.2h");
    compiles("Dawn Evangel");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dawn Evangel");
    let bear = t.battlefield(P1, "Grizzly Bears");
    let aura = t.battlefield(P0, "Pacifism");
    t.g.attach(aura, Entity::Object(bear));
    let card = t.graveyard(P0, "Llanowar Elves");
    t.lands(P0, "Swamp", 1);
    let murder = t.hand(P0, "Disfigure");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Object(card)]);
    t.cast(P0, murder).target(bear).go();
    t.resolve_all();
    assert!(
        t.in_hand(P0, "Llanowar Elves"),
        "an Aura you controlled was attached"
    );

    // An opponent's Aura doesn't count.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dawn Evangel");
    let bear = t.battlefield(P1, "Grizzly Bears");
    let aura = t.battlefield(P1, "Rancor");
    t.g.attach(aura, Entity::Object(bear));
    let card = t.graveyard(P0, "Llanowar Elves");
    t.lands(P0, "Swamp", 1);
    let murder = t.hand(P0, "Disfigure");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Object(card)]);
    t.cast(P0, murder).target(bear).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
}

// --- Referents ----------------------------------------------------------------------------

#[test]
fn outmuscle_adamant_names_the_creature_you_control() {
    cr!("207.2c", "701.14a");
    compiles("Outmuscle");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let spell = t.hand(P0, "Outmuscle");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, spell)
        .targets(&[Entity::Object(mine), Entity::Object(theirs)])
        .go();
    t.resolve_all();
    // 3/3 Bears fight a 3/3 Giant: the Giant dies, the Bears are indestructible.
    assert!(!t.on_battlefield(theirs));
    assert!(
        t.on_battlefield(mine),
        "gained indestructible (three green mana spent)"
    );
}

#[test]
fn coveted_jewel_untaps_after_changing_control() {
    cr!("608.2c");
    compiles("Coveted Jewel");
    let mut t = TestGame::new(2);
    let jewel = t.battlefield(P0, "Coveted Jewel");
    t.g.tap(jewel);
    let attacker = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::PrecombatMain);
    t.attack(&[(attacker, Entity::Player(P0))], &[]);
    t.resolve_all();
    assert_eq!(t.g.obj(jewel).controller, P1);
    assert!(!t.g.obj(jewel).tapped, "untapped");
}

#[test]
fn stomping_slabs_deals_damage_if_a_card_named_it_was_revealed() {
    cr!("608.2c", "201.5");
    compiles("Stomping Slabs");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    t.library_top(P0, "Stomping Slabs");
    let slabs = t.hand(P0, "Stomping Slabs");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, slabs).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 13);

    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let slabs = t.hand(P0, "Stomping Slabs");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, slabs).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn cryptic_gateway_needs_a_type_shared_with_each_tapped_creature() {
    cr!("205.3m");
    compiles("Cryptic Gateway");
    let mut t = TestGame::new(2);
    let gate = t.battlefield(P0, "Cryptic Gateway");
    let elf = t.battlefield(P0, "Llanowar Elves"); // Elf Druid
    let bear = t.battlefield(P0, "Grizzly Bears"); // Bear
    let only_elf = t.hand(P0, "Elvish Mystic"); // Elf Druid: no type in common with Bears
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(elf), Entity::Object(bear)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(only_elf)]);
    t.activate(P0, gate, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.g.obj(elf).tapped && t.g.obj(bear).tapped);
    assert!(
        t.in_hand(P0, "Elvish Mystic"),
        "doesn't share a type with the Bears"
    );
}

// --- Results of earlier instructions ------------------------------------------------------

#[test]
fn fold_into_aether_lets_the_countered_spells_controller_put_a_creature() {
    cr!("701.6a", "608.2c");
    compiles("Fold into Aether");
    let mut t = TestGame::new(2);
    t.lands(P1, "Forest", 2);
    t.lands(P0, "Island", 3);
    let bears = t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Hill Giant");
    let fold = t.hand(P0, "Fold into Aether");
    t.advance_to(P1, Step::PrecombatMain);
    t.cast(P1, bears).go();
    t.answer_yes(P1, true);
    t.cast(P0, fold).target(bears).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(
        t.g.obj(t.named_on_battlefield("Hill Giant")[0]).controller,
        P1
    );
}

#[test]
fn long_rest_needs_eight_cards_returned() {
    cr!("608.2c", "119.1");
    compiles("Long Rest");
    let names = [
        "Crookshank Kobolds",
        "Kraken Hatchling",
        "Leopard-Spotted Jiao",
        "Coral Commando",
        "Dross Crocodile",
        "Yargle, Goliath of Otaria",
        "Minotaur Abomination",
        "Sivitri Scarzam",
    ];
    for n in [8usize, 7] {
        let mut t = TestGame::new(2);
        t.g.player_mut(P0).life = 5;
        t.lands(P0, "Forest", n + 2);
        let cards: Vec<Entity> = names[..n]
            .iter()
            .map(|c| Entity::Object(t.graveyard(P0, c)))
            .collect();
        let rest = t.hand(P0, "Long Rest");
        t.advance_to(P0, Step::PrecombatMain);
        t.cast(P0, rest).x(n as i64).targets(&cards).go();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), n);
        assert_eq!(
            t.life(P0),
            if n == 8 { 20 } else { 5 },
            "{n} cards returned"
        );
        assert!(t.in_exile("Long Rest"));
    }
}

#[test]
fn flood_of_tears_counts_your_nontoken_permanents_returned() {
    cr!("608.2c", "608.2h");
    compiles("Flood of Tears");
    for (n, put) in [(4usize, true), (3, false)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 6);
        for _ in 0..n {
            t.battlefield(P0, "Grizzly Bears");
        }
        t.battlefield(P1, "Hill Giant");
        let giant = t.hand(P0, "Craw Wurm");
        let flood = t.hand(P0, "Flood of Tears");
        t.advance_to(P0, Step::PrecombatMain);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(giant)]);
        t.cast(P0, flood).go();
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 0);
        assert_eq!(
            t.named_on_battlefield("Craw Wurm").len(),
            put as usize,
            "{n}"
        );
    }
}

#[test]
fn pollen_lullaby_keeps_the_clashed_players_creatures_tapped() {
    cr!("701.30a", "701.30d", "502.3");
    compiles("Pollen Lullaby");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    t.library_top(P0, "Colossal Dreadmaw");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bear);
    let mine = t.battlefield(P0, "Llanowar Elves");
    t.g.tap(mine);
    let lullaby = t.hand(P0, "Pollen Lullaby");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, lullaby).go();
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert!(
        t.g.obj(bear).tapped,
        "doesn't untap during that player's next untap step"
    );
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.g.obj(mine).tapped);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.g.obj(bear).tapped, "only that untap step");
}

#[test]
fn shower_of_coals_deals_four_with_threshold() {
    cr!("608.2c", "207.2c");
    compiles("Shower of Coals");
    for threshold in [true, false] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Mountain", 5);
        if threshold {
            for _ in 0..7 {
                t.graveyard(P0, "Grizzly Bears");
            }
        }
        let giant = t.battlefield(P1, "Hill Giant");
        let wurm = t.battlefield(P1, "Craw Wurm");
        let shower = t.hand(P0, "Shower of Coals");
        t.advance_to(P0, Step::PrecombatMain);
        t.cast(P0, shower)
            .targets(&[
                Entity::Player(P1),
                Entity::Object(giant),
                Entity::Object(wurm),
            ])
            .go();
        t.resolve_all();
        let n = if threshold { 4 } else { 2 };
        assert_eq!(t.life(P1), 20 - n);
        assert_eq!(t.on_battlefield(giant), !threshold);
        assert_eq!(t.g.obj(t.g.current(wurm)).damage, n as u32);
    }
}

#[test]
fn whirlpool_whelm_puts_the_creature_on_top_if_you_win_the_clash() {
    cr!("701.30d", "608.2c");
    compiles("Whirlpool Whelm");
    for win in [true, false] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 2);
        if win {
            t.library_top(P0, "Colossal Dreadmaw");
        } else {
            t.library_top(P1, "Colossal Dreadmaw");
        }
        let bear = t.battlefield(P1, "Grizzly Bears");
        let whelm = t.hand(P0, "Whirlpool Whelm");
        t.advance_to(P0, Step::PrecombatMain);
        t.answer_yes(P0, true);
        t.cast(P0, whelm).target(bear).go();
        t.resolve_all();
        let now = t.g.current(bear);
        if win {
            assert_eq!(
                t.g.player(P1).library.last(),
                Some(&now),
                "on top of the library"
            );
        } else {
            assert!(t.in_hand(P1, "Grizzly Bears"));
        }
    }
}

#[test]
fn mr_foxglove_puts_a_creature_if_it_drew_no_cards() {
    cr!("608.2h", "107.1b");
    compiles("Mr. Foxglove");
    let mut t = TestGame::new(2);
    let fox = t.battlefield(P0, "Mr. Foxglove");
    let bear = t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    t.hand(P1, "Llanowar Elves");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bear)]);
    t.attack(&[(fox, Entity::Player(P1))], &[]);
    assert_eq!(
        t.named_on_battlefield("Grizzly Bears").len(),
        1,
        "drew no cards"
    );

    let mut t = TestGame::new(2);
    let fox = t.battlefield(P0, "Mr. Foxglove");
    let bear = t.hand(P0, "Grizzly Bears");
    for _ in 0..4 {
        t.hand(P1, "Llanowar Elves");
    }
    t.advance_to(P0, Step::PrecombatMain);
    let before = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bear)]);
    t.attack(&[(fox, Entity::Player(P1))], &[]);
    assert_eq!(t.hand_size(P0), before + 3, "4 - 1 cards");
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn puppet_master_returns_itself_if_the_creature_card_was_returned() {
    cr!("608.2c", "603.10a");
    compiles("Puppet Master");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let pm = t.battlefield(P0, "Puppet Master");
    t.g.attach(pm, Entity::Object(bear));
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.g.destroy(bear, None);
    t.settle();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Puppet Master"), "paid {{U}}{{U}}{{U}}");
}

#[test]
fn rulik_mons_makes_a_goblin_unless_it_put_the_land_onto_the_battlefield() {
    cr!("608.2c");
    compiles("Rulik Mons, Warren Chief");
    for (top, put, goblin) in [
        ("Forest", true, false),
        ("Forest", false, true),
        ("Grizzly Bears", true, true),
    ] {
        let mut t = TestGame::new(2);
        let rulik = t.battlefield(P0, "Rulik Mons, Warren Chief");
        t.library_top(P0, top);
        t.advance_to(P0, Step::PrecombatMain);
        t.answer_yes(P0, put);
        t.attack(&[(rulik, Entity::Player(P1))], &[]);
        assert_eq!(
            t.named_on_battlefield("Goblin").len(),
            goblin as usize,
            "{top} {put}"
        );
        let land = top == "Forest" && put;
        assert_eq!(t.named_on_battlefield("Forest").len(), land as usize);
    }
}

#[test]
fn worlds_within_worlds_returns_the_exiled_cards_to_hand() {
    cr!("608.2c", "400.7");
    compiles("Worlds Within Worlds");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let elves = t.hand(P0, "Llanowar Elves");
    let www = t.hand(P0, "Worlds Within Worlds");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(elves)]);
    t.cast(P0, www).go();
    t.resolve_all();
    let _ = (a, b);
    assert!(t.in_hand(P0, "Grizzly Bears") && t.in_hand(P1, "Hill Giant"));
    assert_eq!(
        t.named_on_battlefield("Llanowar Elves").len(),
        1,
        "not exiled this way"
    );
    assert!(t.in_exile("Worlds Within Worlds"));
}

#[test]
fn demonic_junker_checks_whether_your_creature_was_destroyed() {
    cr!("608.2c", "608.2h");
    compiles("Demonic Junker");
    for mine in [true, false] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P1, "Hill Giant");
        t.advance_to(P0, Step::PrecombatMain);
        let targets: Vec<Entity> = if mine {
            vec![Entity::Object(a), Entity::Object(b)]
        } else {
            vec![Entity::Object(b)]
        };
        t.answer_targets(P0, &targets);
        let junker = t.enter(P0, "Demonic Junker");
        t.resolve_all();
        assert!(!t.on_battlefield(b));
        assert_eq!(t.counters(junker, "+1/+1"), if mine { 2 } else { 0 });
    }
}

#[test]
fn rowdy_crew_checks_two_discarded_cards_share_a_type() {
    cr!("608.2c", "205.2a");
    compiles("Rowdy Crew");
    // The hand is the three cards drawn; two of them are discarded at random.
    for (cards, counters) in [
        (["Grizzly Bears", "Hill Giant", "Craw Wurm"], 2),
        (["Grizzly Bears", "Shock", "Forest"], 0),
    ] {
        let mut t = TestGame::new(2);
        for c in cards {
            t.library_top(P0, c);
        }
        t.advance_to(P0, Step::PrecombatMain);
        assert_eq!(t.hand_size(P0), 0);
        let crew = t.enter(P0, "Rowdy Crew");
        t.resolve_all();
        assert_eq!(t.hand_size(P0), 1);
        assert_eq!(t.counters(crew, "+1/+1"), counters, "{cards:?}");
    }
}

#[test]
fn sparring_dummy_gains_life_if_a_lesson_was_milled() {
    cr!("608.2c");
    compiles("Sparring Dummy");
    for (top, life) in [("Introduction to Prophecy", 22), ("Grizzly Bears", 20)] {
        let mut t = TestGame::new(2);
        let dummy = t.battlefield(P0, "Sparring Dummy");
        t.library_top(P0, top);
        t.advance_to(P0, Step::PrecombatMain);
        t.activate(P0, dummy, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.life(P0), life, "{top}");
    }
    // A land milled may go to hand; a Lesson milled still gains life after another
    // instruction.
    let mut t = TestGame::new(2);
    let dummy = t.battlefield(P0, "Sparring Dummy");
    t.library_top(P0, "Forest");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.activate(P0, dummy, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Forest"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn lifestream_blessing_uses_the_greatest_power_as_it_was_cast() {
    cr!("601.2i", "608.2h", "107.3c");
    compiles("Lifestream's Blessing");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.battlefield(P0, "Grizzly Bears");
    let blessing = t.hand(P0, "Lifestream's Blessing");
    t.advance_to(P0, Step::PrecombatMain);
    let before = t.hand_size(P0);
    t.cast(P0, blessing).go();
    // The Wurm leaves after the spell was cast: X is still 6.
    t.g.destroy(wurm, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before - 1 + 6);
    assert_eq!(t.life(P0), 20, "not cast from exile");
}

#[test]
fn lukka_draws_two_instead_if_a_creature_card_was_discarded() {
    cr!("608.2c");
    compiles("Mila, Crafty Companion // Lukka, Wayward Bonder");
    for (card, drawn) in [("Grizzly Bears", 2), ("Shock", 1)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Mountain", 5);
        let lukka = t.hand(P0, "Mila, Crafty Companion // Lukka, Wayward Bonder");
        let discard = t.hand(P0, card);
        t.advance_to(P0, Step::PrecombatMain);
        t.cast(P0, lukka).method(CastMethod::Half(1)).go();
        t.resolve_all();
        let before = t.hand_size(P0);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(discard)]);
        t.activate(P0, lukka_now(&t), 0, &[]).unwrap();
        t.resolve_all();
        assert!(t.in_graveyard(P0, card));
        assert_eq!(t.hand_size(P0), before - 1 + drawn, "{card}");
    }
}

fn lukka_now(t: &TestGame) -> ObjectId {
    t.named_on_battlefield("Lukka, Wayward Bonder")[0]
}

#[test]
fn sphinxs_tutelage_repeats_while_two_nonland_cards_share_a_color() {
    cr!("608.2c", "105.2");
    compiles("Sphinx's Tutelage");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sphinx's Tutelage");
    // Top of P1's library: two red cards, then two cards of different colors.
    for c in ["Grizzly Bears", "Counterspell", "Shock", "Lightning Bolt"] {
        t.library_top(P1, c);
    }
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.draw_cards(P0, 1);
    t.settle();
    t.resolve_all();
    // Bolt and Shock share red: mill again (Counterspell and Bears don't share a color).
    assert_eq!(t.graveyard_size(P1), 4);
}

#[test]
fn capricious_hellraiser_copies_a_noncreature_nonland_card_among_those_exiled() {
    cr!("707.12", "608.2c");
    compiles("Capricious Hellraiser");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Forest");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Capricious Hellraiser");
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt"));
    assert_eq!(t.life(P1), 17, "cast the copy of Lightning Bolt");
}

#[test]
fn mysterious_stranger_copies_one_only_if_two_or_more_were_exiled() {
    cr!("707.12", "608.2c");
    compiles("Mysterious Stranger");
    let mut t = TestGame::new(2);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let shock = t.graveyard(P1, "Shock");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Object(bolt)]);
    t.answer_targets(P0, &[Entity::Object(shock)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Mysterious Stranger");
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt") && t.in_exile("Shock"));
    assert!(matches!(t.life(P1), 17 | 18), "one of them copied and cast");

    let mut t = TestGame::new(2);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Object(bolt)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Mysterious Stranger");
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt"));
    assert_eq!(t.life(P1), 20, "only one card exiled");
}

#[test]
fn author_of_shadows_lets_you_cast_the_chosen_card() {
    cr!("608.2c", "609.4b");
    compiles("Author of Shadows");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Forest");
    t.lands(P0, "Swamp", 2);
    t.advance_to(P0, Step::PrecombatMain);
    t.enter(P0, "Author of Shadows");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Forest"));
    let now = t.g.current(bears);
    t.cast(P0, now).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn sidequest_catch_a_fish_transforms_if_it_put_a_card_into_your_hand() {
    cr!("608.2c", "701.28a");
    compiles("Sidequest: Catch a Fish // Cooking Campsite");
    for (top, take, done) in [
        ("Grizzly Bears", true, true),
        ("Grizzly Bears", false, false),
        ("Forest", true, false),
    ] {
        let mut t = TestGame::new(2);
        let quest = t.battlefield(P0, "Sidequest: Catch a Fish // Cooking Campsite");
        t.library_top(P0, top);
        t.answer_yes(P0, take);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert_eq!(t.in_hand(P0, top), done, "{top} {take}");
        assert_eq!(t.named_on_battlefield("Food").len(), done as usize);
        assert_eq!(t.obj_now(quest).face == FaceState::Back, done);
    }
}

#[test]
fn vault_101_attaches_an_equipment_put_onto_the_battlefield_this_way() {
    cr!("714.2b", "608.2c");
    compiles("Vault 101: Birthday Party");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Vault 101: Birthday Party");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let sword = t.hand(P0, "Bonesplitter");
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(sword)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bear)]);
    t.g.add_counters(Entity::Object(saga), "lore", 1, None);
    t.settle();
    t.resolve_all();
    let s = t.named_on_battlefield("Bonesplitter");
    assert_eq!(s.len(), 1);
    assert_eq!(t.g.obj(s[0]).attached_to, Some(Entity::Object(bear)));
}

#[test]
fn lathiel_may_distribute_fewer_counters_than_the_life_gained() {
    cr!("601.2d");
    ruling!(
        "Lathiel, the Bounteous Dawn",
        "You may choose no targets if you want."
    );
    compiles("Lathiel, the Bounteous Dawn");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lathiel, the Bounteous Dawn");
    let a = t.battlefield(P0, "Grizzly Bears");
    t.g.gain_life(P0, 3);
    t.answer_targets(P0, &[Entity::Object(a)]);
    t.answer(
        P0,
        DecisionKind::Number,
        mtg_engine::decision::Answer::Number(1),
    );
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(a, "+1/+1"), 1, "up to that many");
}
