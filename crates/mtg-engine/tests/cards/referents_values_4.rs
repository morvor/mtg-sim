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

/// The tokens on the battlefield with the subtype (tokens are named for it).
fn with_subtype(t: &TestGame, sub: &str) -> usize {
    t.g.battlefield
        .iter()
        .filter(|o| {
            let o = t.g.obj(**o);
            o.is_token() && o.chars.subtypes.iter().any(|s| s.as_str() == sub)
        })
        .count()
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

    // Damage to the player that's prevented isn't dealt (CR 120.4b).
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.battlefield(P1, "Energy Field");
    let fire = t.hand(P0, "Play with Fire");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, fire).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(!scried(&t, P0), "the damage was prevented");
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
    t.lands(P0, "Island", 2);
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

    // X greater than the number of lands: each land is sacrificed, a card drawn for each.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let nahiri = t.hand(P0, "Nahiri's Lithoforming");
    t.advance_to(P0, Step::PrecombatMain);
    t.g.players[P0.idx()]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::R, 4);
    let before = t.hand_size(P0);
    t.cast(P0, nahiri).x(4).go();
    t.resolve_all();
    assert!(t.named_on_battlefield("Mountain").is_empty());
    assert_eq!(t.hand_size(P0), before - 1 + 2, "two lands sacrificed");
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
    assert_eq!(with_subtype(&t, "Blood"), 2);
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
    cr!("608.2h", "500.7");
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
    t.cast(P0, murder).target(bear).go();
    t.answer_targets(P0, &[Entity::Object(card)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bear));
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
    t.cast(P0, murder).target(bear).go();
    t.answer_targets(P0, &[Entity::Object(card)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bear));
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
    t.cast(P0, spell).target(mine).target(theirs).go();
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
    t.lands(P0, "Island", 4);
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
        t.lands(P0, "Forest", n + 3);
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
        let wurm = t.battlefield(P1, "Colossal Dreadmaw");
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

        t.cast(P0, whelm).target(bear).go();
        // Keep the clash card on top; then put the creature on top of its library.
        t.answer_yes(P0, false);
        t.answer_yes(P0, true);
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
        assert_eq!(with_subtype(&t, "Goblin"), goblin as usize, "{top} {put}");
        let land = top == "Forest" && put;
        assert_eq!(t.named_on_battlefield("Forest").len(), land as usize);
    }
}

#[test]
fn worlds_within_worlds_returns_the_exiled_cards_to_hand() {
    cr!("608.2c", "400.7");
    compiles("Worlds Within Worlds");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    t.lands(P0, "Island", 1);
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
        // One target for each player (APNAP order).
        if mine {
            t.answer_targets(P0, &[Entity::Object(a)]);
        } else {
            t.answer_targets(P0, &[]);
        }
        t.answer_targets(P0, &[Entity::Object(b)]);
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
    t.lands(P0, "Forest", 6);
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
        t.lands(P0, "Mountain", 6);
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
        assert_eq!(with_subtype(&t, "Food"), done as usize);
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
    // Chapter II: the second lore counter.
    t.g.objects[saga.0 as usize]
        .counters
        .insert("lore".into(), 1);
    t.g.add_counters(Entity::Object(saga), "lore", 1, None);
    t.g.flush_events();
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
        "Each target must receive at least one +1/+1 counter."
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

    // Two targets, two of three counters: one each. Fewer than one per target isn't a
    // legal choice (the full amount is distributed instead).
    for (k, each) in [(2i64, 1u32), (1, 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Lathiel, the Bounteous Dawn");
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Hill Giant");
        t.g.gain_life(P0, 3);
        t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
        t.answer(
            P0,
            DecisionKind::Number,
            mtg_engine::decision::Answer::Number(k),
        );
        t.advance_to(P0, Step::End);
        t.resolve_all();
        let total = t.counters(a, "+1/+1") + t.counters(b, "+1/+1");
        assert!(t.counters(a, "+1/+1") >= 1 && t.counters(b, "+1/+1") >= 1);
        if each == 1 {
            assert_eq!(total, 2, "chose two");
        } else {
            assert_eq!(total, 3, "an illegal amount: all three");
        }
    }
}

#[test]
fn game_preserve_puts_the_cards_onto_the_battlefield_if_all_are_creatures() {
    cr!("608.2c", "701.20a");
    compiles("Game Preserve");
    for (theirs, all) in [("Hill Giant", true), ("Forest", false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Game Preserve");
        t.advance_to(P1, Step::End);
        t.library_top(P0, "Grizzly Bears");
        t.library_top(P1, theirs);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield("Grizzly Bears").len(),
            all as usize,
            "{theirs}"
        );
        if all {
            let giant = t.named_on_battlefield("Hill Giant");
            assert_eq!(
                t.g.obj(giant[0]).controller,
                P1,
                "under their owners' control"
            );
        }
    }
}

#[test]
fn rise_and_shine_puts_counters_on_each_artifact_that_became_a_creature() {
    cr!("608.2c", "702.96a");
    compiles("Rise and Shine");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let rock = t.battlefield(P0, "Mind Stone");
    let rise = t.hand(P0, "Rise and Shine");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, rise).target(rock).go();
    t.resolve_all();
    assert_eq!(t.counters(rock, "+1/+1"), 4);
    assert_eq!(t.pt(rock), (4, 4));

    // Overloaded: each artifact you control becomes a creature and gets the counters; an
    // opponent's artifact doesn't.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    let a = t.battlefield(P0, "Mind Stone");
    let b = t.battlefield(P0, "Sol Ring");
    let theirs = t.battlefield(P1, "Mind Stone");
    let rise = t.hand(P0, "Rise and Shine");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, rise)
        .method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Overload))
        .go();
    t.resolve_all();
    assert_eq!(t.counters(a, "+1/+1"), 4);
    assert_eq!(t.pt(a), (4, 4));
    assert_eq!(t.counters(b, "+1/+1"), 4);
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
}

#[test]
fn spry_and_mighty_uses_the_difference_between_the_chosen_powers() {
    cr!("107.1", "608.2h");
    ruling!(
        "Spry and Mighty",
        "subtract the smaller of those two numbers from the larger one"
    );
    ruling!(
        "Spry and Mighty",
        "If you don't control two creatures as Spry and Mighty resolves, the spell won't do anything."
    );
    compiles("Spry and Mighty");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let small = t.battlefield(P0, "Llanowar Elves");
    let big = t.battlefield(P0, "Craw Wurm");
    let spry = t.hand(P0, "Spry and Mighty");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(small), Entity::Object(big)]);
    let before = t.hand_size(P0);
    t.cast(P0, spry).go();
    t.resolve_all();
    // 6 - 1 = 5.
    assert_eq!(t.hand_size(P0), before - 1 + 5);
    assert_eq!(t.pt(small), (6, 6));
    assert_eq!(t.pt(big), (11, 9));

    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let only = t.battlefield(P0, "Craw Wurm");
    let spry = t.hand(P0, "Spry and Mighty");
    t.advance_to(P0, Step::PrecombatMain);
    let before = t.hand_size(P0);
    t.cast(P0, spry).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before - 1);
    assert!(!t
        .g
        .obj(only)
        .has_keyword(mtg_engine::keywords::KeywordKind::Trample));
}

#[test]
fn plasma_caster_damages_the_blocker_if_you_lose_the_flip() {
    cr!("705.2", "608.2c");
    compiles("Plasma Caster");
    for win in [true, false] {
        let mut t = TestGame::new(2);
        let bear = t.battlefield(P0, "Grizzly Bears");
        let caster = t.battlefield(P0, "Plasma Caster");
        t.g.attach(caster, Entity::Object(bear));
        let giant = t.battlefield(P1, "Craw Wurm");
        t.advance_to(P0, Step::PrecombatMain);
        t.answer(
            P0,
            DecisionKind::Attackers,
            mtg_engine::decision::Answer::Attackers(vec![(bear, Entity::Player(P1))]),
        );
        t.answer(
            P1,
            DecisionKind::Blockers,
            mtg_engine::decision::Answer::Blockers(vec![(giant, bear)]),
        );
        t.advance_to(P0, Step::DeclareBlockers);
        // Call heads (the default); the coin comes up heads (a win) or tails.
        t.g.dice.loaded_coins.push_back(win);
        t.activate(P0, caster, 0, &[Entity::Object(giant)]).unwrap();
        t.resolve_all();
        if win {
            assert!(!t.on_battlefield(giant), "exiled");
        } else {
            assert_eq!(t.g.obj(giant).damage, 1, "1 damage to the chosen creature");
        }
    }
}

#[test]
fn anowon_draws_if_the_player_milled_a_creature_card() {
    cr!("608.2c", "701.17a");
    compiles("Anowon, the Ruin Thief");
    for (top, draws) in [("Grizzly Bears", 1usize), ("Forest", 0)] {
        let mut t = TestGame::new(2);
        let anowon = t.battlefield(P0, "Anowon, the Ruin Thief");
        // Anowon is a 2/4 Rogue: two cards milled.
        t.library_top(P1, top);
        t.library_top(P1, "Island");
        t.advance_to(P0, Step::PrecombatMain);
        let before = t.hand_size(P0);
        t.attack(&[(anowon, Entity::Player(P1))], &[]);
        assert_eq!(t.graveyard_size(P1), 2);
        assert_eq!(t.hand_size(P0), before + draws, "{top}");
    }
}

#[test]
fn optimus_prime_gives_trample_to_the_creature_bolster_chose() {
    cr!("701.39a", "608.2c");
    compiles("Optimus Prime, Hero // Optimus Prime, Autobot Leader");
    let mut t = TestGame::new(2);
    let optimus = t.battlefield(P0, "Optimus Prime, Hero // Optimus Prime, Autobot Leader");
    mtg_engine::dfc::transform(&mut t.g, optimus);
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.advance_to(P0, Step::PrecombatMain);
    t.attack(&[(bear, Entity::Player(P1))], &[]);
    assert_eq!(
        t.counters(bear, "+1/+1"),
        2,
        "the creature with the least toughness"
    );
    assert!(t
        .obj_now(bear)
        .has_keyword(mtg_engine::keywords::KeywordKind::Trample));
    assert_eq!(t.life(P1), 16);
    // "When that creature deals combat damage to a player this turn, convert ~."
    assert_eq!(t.obj_now(optimus).face, FaceState::Front);
}

#[test]
fn suppression_ray_stuns_up_to_that_many_creatures_tapped_this_way() {
    cr!("107.14", "608.2c");
    compiles("Suppression Ray // Orderly Plaza");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    t.g.add_counters(Entity::Player(P0), "energy", 3, None);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P1, "Craw Wurm");
    t.g.tap(c);
    let ray = t.hand(P0, "Suppression Ray // Orderly Plaza");
    t.advance_to(P0, Step::PrecombatMain);
    // Pay {E}{E} ("any amount" is announced like an X).
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(2));
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.cast(P0, ray).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(t.g.obj(a).tapped && t.g.obj(b).tapped);
    assert_eq!(t.counters(a, "stun"), 1);
    assert_eq!(t.counters(b, "stun"), 1);
    assert_eq!(
        t.counters(c, "stun"),
        0,
        "already tapped: not tapped this way"
    );
}

#[test]
fn scalpelexis_repeats_while_two_exiled_cards_share_a_name() {
    cr!("201.2a", "608.2c");
    compiles("Scalpelexis");
    let mut t = TestGame::new(2);
    let beast = t.battlefield(P0, "Scalpelexis");
    // Second four (exiled second): all different. First four: two Forests.
    for c in [
        "Plains",
        "Island",
        "Swamp",
        "Mountain",
        "Forest",
        "Grizzly Bears",
        "Forest",
        "Shock",
    ] {
        t.library_top(P1, c);
    }
    t.advance_to(P0, Step::PrecombatMain);
    t.attack(&[(beast, Entity::Player(P1))], &[]);
    assert_eq!(
        t.g.objects_in_zone_kind(mtg_engine::ability::ZoneKind::Exile)
            .len(),
        8,
        "two passes"
    );
}

#[test]
fn goblin_charbelcher_deals_double_if_the_land_was_a_mountain() {
    cr!("608.2c", "701.20a");
    compiles("Goblin Charbelcher");
    for (land, dmg) in [("Mountain", 4), ("Forest", 2)] {
        let mut t = TestGame::new(2);
        let belcher = t.battlefield(P0, "Goblin Charbelcher");
        t.lands(P0, "Plains", 3);
        t.library_top(P0, land);
        t.library_top(P0, "Grizzly Bears");
        t.library_top(P0, "Shock");
        t.advance_to(P0, Step::PrecombatMain);
        t.activate(P0, belcher, 0, &[Entity::Player(P1)]).unwrap();
        t.resolve_all();
        assert_eq!(t.life(P1), 20 - dmg, "{land}");
    }
}

#[test]
fn back_for_seconds_may_put_one_card_onto_the_battlefield_if_bargained() {
    cr!("702.166b", "608.2c");
    ruling!(
        "Back for Seconds",
        "If you bargain this spell, you can still choose not to put one of the cards onto the battlefield"
    );
    compiles("Back for Seconds");
    for bargain in [true, false] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Swamp", 5);
        let food = t.battlefield(P0, "Ornithopter");
        let a = t.graveyard(P0, "Grizzly Bears");
        let b = t.graveyard(P0, "Craw Wurm");
        let spell = t.hand(P0, "Back for Seconds");
        t.advance_to(P0, Step::PrecombatMain);
        if bargain {
            t.answer(
                P0,
                DecisionKind::OptionalCost,
                mtg_engine::decision::Answer::Bool(true),
            );
            t.answer_choose(P0, &[Entity::Object(food)]);
        } else {
            t.answer(
                P0,
                DecisionKind::OptionalCost,
                mtg_engine::decision::Answer::Bool(false),
            );
        }
        t.answer_choose(P0, &[Entity::Object(a)]);
        t.cast(P0, spell)
            .targets(&[Entity::Object(a), Entity::Object(b)])
            .go();
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield("Grizzly Bears").len(),
            bargain as usize
        );
        assert!(t.in_hand(P0, "Craw Wurm"));
        assert_eq!(t.in_hand(P0, "Grizzly Bears"), !bargain);
    }
}

#[test]
fn emeria_shepherd_returns_to_the_battlefield_if_the_land_is_a_plains() {
    cr!("608.2c", "207.2c");
    compiles("Emeria Shepherd");
    for (land, bf) in [("Plains", true), ("Forest", false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Emeria Shepherd");
        let card = t.graveyard(P0, "Grizzly Bears");
        let l = t.hand(P0, land);
        t.advance_to(P0, Step::PrecombatMain);
        t.answer_targets(P0, &[Entity::Object(card)]);
        t.answer_yes(P0, true);
        t.play_land(P0, l).unwrap();
        t.settle();
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield("Grizzly Bears").len(),
            bf as usize,
            "{land}"
        );
        assert_eq!(t.in_hand(P0, "Grizzly Bears"), !bf);
    }
}

#[test]
fn once_and_future_adamant_returns_both_cards_to_hand() {
    cr!("207.2c", "608.2c");
    compiles("Once and Future");
    for (lands, adamant) in [("Forest", true), ("Island", false)] {
        let mut t = TestGame::new(2);
        t.lands(P0, lands, 4);
        if !adamant {
            t.lands(P0, "Forest", 1);
        }
        let a = t.graveyard(P0, "Grizzly Bears");
        let b = t.graveyard(P0, "Hill Giant");
        let spell = t.hand(P0, "Once and Future");
        t.advance_to(P0, Step::PrecombatMain);
        t.cast(P0, spell).target(a).target(b).go();
        t.resolve_all();
        assert!(t.in_hand(P0, "Grizzly Bears"));
        assert_eq!(t.in_hand(P0, "Hill Giant"), adamant);
        if !adamant {
            let top = *t.g.player(P0).library.last().unwrap();
            assert_eq!(t.g.obj(top).chars.name, "Hill Giant");
        }
        assert!(t.in_exile("Once and Future"));
    }
}

#[test]
fn lethal_throwdown_draws_if_the_modified_creature_option_was_paid() {
    cr!("601.2b", "601.2h", "700.9");
    compiles("Lethal Throwdown");
    for modified in [true, false] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Swamp", 3);
        let fodder = t.battlefield(P0, "Grizzly Bears");
        if modified {
            t.g.add_counters(Entity::Object(fodder), "+1/+1", 1, None);
        }
        let target = t.battlefield(P1, "Hill Giant");
        let spell = t.hand(P0, "Lethal Throwdown");
        t.advance_to(P0, Step::PrecombatMain);
        t.answer(
            P0,
            DecisionKind::Option,
            mtg_engine::decision::Answer::Index(modified as usize),
        );
        let before = t.hand_size(P0);
        t.cast(P0, spell).target(target).go();
        t.resolve_all();
        assert!(!t.on_battlefield(fodder) && !t.on_battlefield(target));
        assert_eq!(t.hand_size(P0), before - 1 + modified as usize);
    }
}

#[test]
fn haunting_imitation_returns_to_hand_if_no_creature_was_revealed() {
    cr!("608.2c", "701.20a");
    compiles("Haunting Imitation");
    for (top, back) in [("Forest", true), ("Grizzly Bears", false)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 4);
        t.library_top(P0, "Island");
        t.library_top(P1, top);
        let spell = t.hand(P0, "Haunting Imitation");
        t.advance_to(P0, Step::PrecombatMain);
        t.cast(P0, spell).go();
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Haunting Imitation"), back, "{top}");
        assert_eq!(
            t.named_on_battlefield("Grizzly Bears").len(),
            !back as usize
        );
    }
}

#[test]
fn aurelias_fury_taps_creatures_and_stops_players_dealt_damage() {
    cr!("601.2d", "608.2c");
    compiles("Aurelia's Fury");
    let mut t = TestGame::new(3);
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Mountain", 3);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let fury = t.hand(P0, "Aurelia's Fury");
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt2 = t.hand(P2, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.lands(P2, "Mountain", 1);
    t.advance_to(P0, Step::PrecombatMain);
    t.answer(
        P0,
        DecisionKind::Divide,
        mtg_engine::decision::Answer::Numbers(vec![2, 1]),
    );
    t.cast(P0, fury)
        .x(3)
        .targets(&[Entity::Object(wurm), Entity::Player(P1)])
        .go();
    t.resolve_all();
    assert!(
        t.g.obj(wurm).tapped,
        "a creature dealt damage this way is tapped"
    );
    assert_eq!(t.life(P1), 19);
    assert!(t
        .cast(P1, bolt)
        .target(Entity::Player(P0))
        .try_go()
        .is_err());
    // A player not dealt damage can.
    assert!(t
        .cast(P2, bolt2)
        .target(Entity::Player(P0))
        .try_go()
        .is_ok());
}

#[test]
fn hoarders_overflow_puts_a_stash_counter_on_itself() {
    cr!("603.2");
    compiles("Hoarder's Overflow");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let h = t.enter(P0, "Hoarder's Overflow");
    t.resolve_all();
    assert_eq!(t.counters(h, "stash"), 1);
}

#[test]
fn malcolm_may_cast_the_discarded_card_with_four_chorus_counters() {
    cr!("608.2c", "400.7");
    compiles("Malcolm, Alluring Scoundrel");
    for (counters, cast) in [(3u32, true), (2, false)] {
        let mut t = TestGame::new(2);
        let malcolm = t.battlefield(P0, "Malcolm, Alluring Scoundrel");
        t.g.add_counters(Entity::Object(malcolm), "chorus", counters, None);
        let bolt = t.hand(P0, "Lightning Bolt");
        t.advance_to(P0, Step::PrecombatMain);
        t.answer_choose(P0, &[Entity::Object(bolt)]);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.attack(&[(malcolm, Entity::Player(P1))], &[]);
        t.resolve_all();
        // 2 combat damage, and 3 from the Bolt cast from the graveyard.
        assert_eq!(t.life(P1), if cast { 15 } else { 18 }, "{counters}");
    }
}

#[test]
fn primo_puts_counters_equal_to_the_damage_dealt_on_the_token() {
    cr!("603.2c", "510.2");
    compiles("Primo, the Unbounded");
    let mut t = TestGame::new(2);
    let primo = t.battlefield(P0, "Primo, the Unbounded");
    t.g.add_counters(Entity::Object(primo), "+1/+1", 2, None);
    let a = t.battlefield(P0, "Ornithopter");
    let b = t.battlefield(P0, "Ornithopter");
    t.g.add_counters(Entity::Object(a), "+1/+1", 2, None);
    t.g.add_counters(Entity::Object(b), "+1/+1", 1, None);
    t.advance_to(P0, Step::PrecombatMain);
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
    let fractals: Vec<ObjectId> =
        t.g.battlefield
            .iter()
            .copied()
            .filter(|o| {
                t.g.obj(*o)
                    .chars
                    .subtypes
                    .iter()
                    .any(|s| s.as_str() == "Fractal")
                    && t.g.obj(*o).is_token()
            })
            .collect();
    assert_eq!(fractals.len(), 1);
    assert_eq!(t.counters(fractals[0], "+1/+1"), 3);
}

#[test]
fn aura_barbs_damages_controllers_and_enchanted_creatures() {
    cr!("120.3", "608.2c");
    compiles("Aura Barbs");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let aura = t.battlefield(P1, "Rancor");
    t.g.attach(aura, Entity::Object(bear));
    t.battlefield(P0, "Glorious Anthem");
    let barbs = t.hand(P0, "Aura Barbs");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, barbs).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 18);
    assert!(!t.on_battlefield(bear), "2 damage to the enchanted 4/2");
}

#[test]
fn ultra_magnus_converts_if_the_attackers_have_total_power_eight() {
    cr!("207.2c", "701.28a");
    compiles("Ultra Magnus, Tactician // Ultra Magnus, Armored Carrier");
    for (extra, converts) in [("Craw Wurm", true), ("Grizzly Bears", false)] {
        let mut t = TestGame::new(2);
        let um = t.battlefield(
            P0,
            "Ultra Magnus, Tactician // Ultra Magnus, Armored Carrier",
        );
        mtg_engine::dfc::transform(&mut t.g, um);
        let other = t.battlefield(P0, extra);
        t.advance_to(P0, Step::PrecombatMain);
        t.attack(
            &[(um, Entity::Player(P1)), (other, Entity::Player(P1))],
            &[],
        );
        assert_eq!(t.obj_now(um).face == FaceState::Front, converts, "{extra}");
    }
}

#[test]
fn grimoire_thief_counters_spells_with_the_exiled_cards_names() {
    cr!("201.2a", "607.2a");
    compiles("Grimoire Thief");
    let mut t = TestGame::new(2);
    let thief = t.battlefield(P0, "Grimoire Thief");
    t.library_top(P1, "Shock");
    t.lands(P1, "Mountain", 2);
    t.lands(P0, "Island", 1);
    let shock = t.hand(P1, "Shock");
    let bolt = t.hand(P1, "Lightning Bolt");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.tap(thief);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    // P1 casts Shock and Lightning Bolt; the Thief's ability counters only the Shock.
    t.g.turn.priority = Some(P1);
    t.cast(P1, shock).target(Entity::Player(P0)).go();
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.activate(P0, thief, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 17, "only Lightning Bolt resolved");
}

#[test]
fn holistic_wisdom_needs_a_shared_card_type_with_the_exiled_card() {
    cr!("400.7j", "205.2a");
    compiles("Holistic Wisdom");
    for (exiled, back) in [("Hill Giant", true), ("Shock", false)] {
        let mut t = TestGame::new(2);
        let wisdom = t.battlefield(P0, "Holistic Wisdom");
        t.lands(P0, "Forest", 2);
        let ex = t.hand(P0, exiled);
        let bear = t.graveyard(P0, "Grizzly Bears");
        t.advance_to(P0, Step::PrecombatMain);
        t.answer_choose(P0, &[Entity::Object(ex)]);
        t.activate(P0, wisdom, 0, &[Entity::Object(bear)]).unwrap();
        t.resolve_all();
        assert!(t.in_exile(exiled));
        assert_eq!(t.in_hand(P0, "Grizzly Bears"), back, "{exiled}");
    }
}

#[test]
fn goliath_hatchery_draws_the_chosen_creatures_total_toxic_value() {
    cr!("702.164b");
    compiles("Goliath Hatchery");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Goliath Hatchery");
    let nurse = t.battlefield(P0, "Plague Nurse");
    t.g.add_counters(Entity::Player(P1), "poison", 3, None);
    t.advance_to(P1, Step::End);
    let before = t.hand_size(P0);
    t.answer_choose(P0, &[Entity::Object(nurse)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    // Toxic 2: two cards (the draw step hasn't happened yet).
    assert_eq!(t.hand_size(P0), before + 2);
}

#[test]
fn phosphorescent_feast_counts_green_symbols_in_the_revealed_cards() {
    cr!("107.4e", "701.20a");
    compiles("Phosphorescent Feast");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let a = t.hand(P0, "Craw Wurm"); // {4}{G}{G}
    let b = t.hand(P0, "Grizzly Bears"); // {1}{G}
    let c = t.hand(P0, "Kitchen Finks"); // {1}{G/W}{G/W}
    t.hand(P0, "Llanowar Elves"); // not revealed
    let feast = t.hand(P0, "Phosphorescent Feast");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_choose(
        P0,
        &[Entity::Object(a), Entity::Object(b), Entity::Object(c)],
    );
    t.cast(P0, feast).go();
    t.resolve_all();
    assert_eq!(
        t.life(P0),
        30,
        "2 life for each of five green symbols, hybrid ones included"
    );
}

#[test]
fn sovereign_okinec_ahau_counters_equal_to_power_over_base_power() {
    cr!("608.2h");
    compiles("Sovereign Okinec Ahau");
    let mut t = TestGame::new(2);
    let ahau = t.battlefield(P0, "Sovereign Okinec Ahau");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let elf = t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Glorious Anthem");
    t.g.add_counters(Entity::Object(bear), "+1/+1", 2, None);
    t.advance_to(P0, Step::PrecombatMain);
    t.attack(&[(ahau, Entity::Player(P1))], &[]);
    // Bears: 5 power, base 2: three more counters. Elves: 2 power, base 1: one.
    assert_eq!(t.counters(bear, "+1/+1"), 5);
    assert_eq!(t.counters(elf, "+1/+1"), 1);
}

#[test]
fn colfenors_urn_returns_the_cards_exiled_with_it() {
    cr!("607.2a", "608.2c");
    compiles("Colfenor's Urn");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Colfenor's Urn");
    let wurms: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Craw Wurm")).collect();
    t.advance_to(P0, Step::PrecombatMain);
    for w in &wurms {
        t.answer_yes(P0, true);
        t.g.destroy(*w, None);
        t.settle();
        t.resolve_all();
    }
    assert!(t.named_on_battlefield("Craw Wurm").is_empty());
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Colfenor's Urn"));
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 3);
}

#[test]
fn specters_shriek_exiles_from_your_hand_if_a_nonblack_card_was_exiled() {
    cr!("608.2c");
    compiles("Specter's Shriek");
    for (card, mine) in [("Grizzly Bears", true), ("Hypnotic Specter", false)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Swamp", 1);
        let theirs = t.hand(P1, card);
        t.hand(P0, "Llanowar Elves");
        let shriek = t.hand(P0, "Specter's Shriek");
        t.advance_to(P0, Step::PrecombatMain);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(theirs)]);
        t.cast(P0, shriek).target(Entity::Player(P1)).go();
        t.resolve_all();
        assert!(t.in_exile(card));
        assert_eq!(t.in_exile("Llanowar Elves"), mine, "{card}");
    }
}

#[test]
fn zack_fair_passes_its_counters_and_equipment_on() {
    cr!("122.8", "608.2h");
    compiles("Zack Fair");
    let mut t = TestGame::new(2);
    let zack = t.battlefield(P0, "Zack Fair");
    t.g.add_counters(Entity::Object(zack), "+1/+1", 2, None);
    let sword = t.battlefield(P0, "Bonesplitter");
    t.g.attach(sword, Entity::Object(zack));
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(sword)]);
    t.activate(P0, zack, 0, &[Entity::Object(bear)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(zack));
    assert_eq!(t.counters(bear, "+1/+1"), 2);
    assert_eq!(t.g.obj(sword).attached_to, Some(Entity::Object(bear)));
    assert!(t
        .obj_now(bear)
        .has_keyword(mtg_engine::keywords::KeywordKind::Indestructible));
}

#[test]
fn tainted_specter_deals_damage_only_if_the_player_discarded() {
    cr!("118.12", "608.2c");
    compiles("Tainted Specter");
    for (put, damage) in [(false, true), (true, false)] {
        let mut t = TestGame::new(2);
        let specter = t.battlefield(P0, "Tainted Specter");
        t.lands(P0, "Swamp", 3);
        t.hand(P1, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        t.advance_to(P0, Step::PrecombatMain);
        t.answer_yes(P1, put);
        t.activate(P0, specter, 0, &[Entity::Player(P1)]).unwrap();
        t.resolve_all();
        assert_eq!(t.hand_size(P1), 0);
        let hurt = if damage { 1 } else { 0 };
        assert_eq!(t.life(P1), 20 - hurt, "{put}");
        assert_eq!(t.life(P0), 20 - hurt);
        assert_eq!(t.g.obj(giant).damage, hurt as u32);
    }
}

#[test]
fn cerebral_eruption_uses_the_revealed_cards_mana_value() {
    cr!("701.20a", "608.2c");
    compiles("Cerebral Eruption");
    for (top, dmg, back) in [("Hill Giant", 4, false), ("Forest", 0, true)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Mountain", 4);
        t.library_top(P1, top);
        let bear = t.battlefield(P1, "Grizzly Bears");
        let mine = t.battlefield(P0, "Llanowar Elves");
        let spell = t.hand(P0, "Cerebral Eruption");
        t.advance_to(P0, Step::PrecombatMain);
        t.cast(P0, spell).target(Entity::Player(P1)).go();
        t.resolve_all();
        assert_eq!(t.life(P1), 20 - dmg, "{top}");
        assert_eq!(t.on_battlefield(bear), dmg < 2);
        assert!(t.on_battlefield(mine), "only that player's creatures");
        assert_eq!(t.in_hand(P0, "Cerebral Eruption"), back);
    }
}

#[test]
fn froghemoth_exiles_up_to_that_many_cards_from_the_damaged_players_graveyard() {
    cr!("510.2", "608.2c");
    compiles("Froghemoth");
    let mut t = TestGame::new(2);
    let frog = t.battlefield(P0, "Froghemoth");
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Shock");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.attack(&[(frog, Entity::Player(P1))], &[]);
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Shock"));
    assert_eq!(t.counters(frog, "+1/+1"), 1, "one creature card");
    assert_eq!(t.life(P0), 21, "one noncreature card");
}

#[test]
fn zoyowas_justice_lets_the_owner_discover_the_mana_value() {
    cr!("701.57a", "608.2h");
    compiles("Zoyowa's Justice");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    // P1's library: lands and a Hill Giant (mana value 4).
    t.g.player_mut(P1).library.clear();
    t.library_top(P1, "Hill Giant");
    for _ in 0..5 {
        t.library_top(P1, "Forest");
    }
    let bear = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Zoyowa's Justice");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, spell).target(bear).go();
    t.resolve_all();
    // Discover 2 (the Bears' mana value): the Giant is skipped, the Bears found again.
    let bears_back = t.on_battlefield(t.g.current(bear))
        || !t.named_on_battlefield("Grizzly Bears").is_empty()
        || t.in_hand(P1, "Grizzly Bears");
    assert!(bears_back, "{}", t.dump_log());
    assert!(t.named_on_battlefield("Hill Giant").is_empty() && !t.in_hand(P1, "Hill Giant"));
}

#[test]
fn haphazard_bombardment_destroys_one_of_those_permanents_at_random() {
    cr!("603.4", "701.8a");
    ruling!(
        "Haphazard Bombardment",
        "select the permanent destroyed at random from among the permanents with aim counters that don’t have indestructible"
    );
    compiles("Haphazard Bombardment");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Haphazard Bombardment");
    let bear = t.battlefield(P1, "Grizzly Bears");
    let myr = t.battlefield(P1, "Darksteel Myr");
    let other = t.battlefield(P1, "Hill Giant");
    t.g.add_counters(Entity::Object(bear), "aim", 1, None);
    t.g.add_counters(Entity::Object(myr), "aim", 1, None);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(
        !t.on_battlefield(bear),
        "the only one that can be destroyed"
    );
    assert!(t.on_battlefield(myr) && t.on_battlefield(other));
}

#[test]
fn evolving_door_searches_for_one_more_color() {
    cr!("105.2", "608.2h");
    compiles("Evolving Door");
    let mut t = TestGame::new(2);
    let door = t.battlefield(P0, "Evolving Door");
    t.lands(P0, "Forest", 1);
    let bear = t.battlefield(P0, "Grizzly Bears"); // green
    t.g.player_mut(P0).library.clear();
    t.library_top(P0, "Craw Wurm"); // one color: not a match
    let two = t.library_top(P0, "Kitchen Finks"); // two colors
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(bear)]);
    t.answer_choose(P0, &[Entity::Object(two)]);
    t.activate(P0, door, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Kitchen Finks") || !t.named_on_battlefield("Kitchen Finks").is_empty());
    assert!(!t.in_exile("Craw Wurm"));
}

#[test]
fn quag_feast_destroys_the_chosen_permanent_if_its_mana_value_is_low_enough() {
    cr!("608.2c", "701.17a");
    compiles("Quag Feast");
    for (graveyard, destroyed) in [(3usize, true), (0, false)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Swamp", 2);
        // Mill two: the graveyard then has 2 + the cards already there (+ Quag Feast is
        // still on the stack).
        for _ in 0..graveyard {
            t.graveyard(P0, "Grizzly Bears");
        }
        let target = t.battlefield(P1, "Serra Angel"); // mana value 5
        let spell = t.hand(P0, "Quag Feast");
        t.advance_to(P0, Step::PrecombatMain);
        t.cast(P0, spell).target(target).go();
        t.resolve_all();
        assert_eq!(!t.on_battlefield(target), destroyed, "{graveyard}");
    }
}

#[test]
fn vaevictis_asmadi_each_player_who_sacrificed_reveals_and_may_get_a_permanent() {
    cr!("701.21a", "608.2c");
    compiles("Vaevictis Asmadi, the Dire");
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Vaevictis Asmadi, the Dire");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.library_top(P0, "Forest");
    t.library_top(P1, "Shock");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Object(mine)]);
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    t.attack(&[(dragon, Entity::Player(P1))], &[]);
    assert!(t.in_graveyard(P0, "Grizzly Bears") && t.in_graveyard(P1, "Hill Giant"));
    // P0 revealed a Forest (a permanent card): onto the battlefield. P1 revealed Shock.
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    assert!(t
        .g
        .player(P1)
        .library
        .last()
        .is_some_and(|c| t.g.obj(*c).chars.name == "Shock"));
}

#[test]
fn spoils_of_the_vault_loses_life_for_each_exiled_card() {
    cr!("701.20a", "608.2c");
    compiles("Spoils of the Vault");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.library_top(P0, "Forest");
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Shock");
    let spoils = t.hand(P0, "Spoils of the Vault");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer(
        P0,
        DecisionKind::Name,
        mtg_engine::decision::Answer::Text("Forest".into()),
    );
    t.cast(P0, spoils).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Forest"));
    assert!(t.in_exile("Shock") && t.in_exile("Grizzly Bears"));
    assert_eq!(t.life(P0), 18);
}

#[test]
fn mysterious_limousine_returns_the_other_exiled_cards() {
    cr!("607.2a", "608.2c");
    compiles("Mysterious Limousine");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Craw Wurm");
    let crew = t.battlefield(P0, "Grizzly Bears");
    let limo = t.battlefield(P0, "Mysterious Limousine");
    for (turn, target) in [(0, a), (1, b)] {
        t.advance_to(P0, Step::PrecombatMain);
        t.answer_choose(P0, &[Entity::Object(crew)]);
        t.activate(P0, limo, 0, &[]).unwrap();
        t.resolve_all();
        t.answer_targets(P0, &[Entity::Object(target)]);
        t.attack(&[(limo, Entity::Player(P1))], &[]);
        if turn == 0 {
            assert!(t.in_exile("Hill Giant"));
            t.advance_to(P1, Step::End);
        }
    }
    // The Wurm is exiled; the Giant, exiled with it before, returns.
    assert!(t.in_exile("Craw Wurm"));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn cait_gets_counters_if_you_discarded_the_greatest_mana_value() {
    cr!("608.2c", "701.9a");
    compiles("Cait, Cage Brawler");
    for (mine, theirs, grows) in [
        ("Hill Giant", "Grizzly Bears", true),
        ("Grizzly Bears", "Hill Giant", false),
        ("Grizzly Bears", "Grizzly Bears", true),
    ] {
        let mut t = TestGame::new(2);
        let cait = t.battlefield(P0, "Cait, Cage Brawler");
        t.advance_to(P0, Step::PrecombatMain);
        t.library_top(P0, mine);
        t.library_top(P1, theirs);
        t.attack(&[(cait, Entity::Player(P1))], &[]);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), 0);
        assert_eq!(t.hand_size(P1), 0);
        let n = if grows { 2 } else { 0 };
        assert_eq!(t.counters(cait, "+1/+1"), n, "{mine} vs {theirs}");
    }
}

// --- Batch 5 ---------------------------------------------------------------------------

thread_local! {
    static PICK: std::cell::RefCell<Vec<&'static str>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Answers entity choices among cards by name (the cards' ids change as they move): the
/// candidates named in `PICK`.
fn pick_named(g: &mtg_engine::game::Game, d: &Decision) -> Option<mtg_engine::decision::Answer> {
    let Decision::ChooseEntities { candidates, .. } = d else {
        return None;
    };
    let want = PICK.with(|w| w.borrow().clone());
    if want.is_empty() {
        return None;
    }
    let picks: Vec<Entity> = candidates
        .iter()
        .filter(|e| {
            e.object()
                .is_some_and(|o| want.contains(&g.obj(o).chars.name.as_str()))
        })
        .copied()
        .collect();
    (!picks.is_empty()).then_some(mtg_engine::decision::Answer::Entities(picks))
}

/// Routes `p`'s entity choices through [`pick_named`].
fn pick_by_name(t: &mut TestGame, p: PlayerId, names: &[&'static str]) {
    struct Picker {
        inner: Box<dyn mtg_engine::decision::Agent>,
    }
    impl mtg_engine::decision::Agent for Picker {
        fn decide(
            &mut self,
            g: &mtg_engine::game::Game,
            p: PlayerId,
            d: &Decision,
        ) -> mtg_engine::decision::Answer {
            pick_named(g, d).unwrap_or_else(|| self.inner.decide(g, p, d))
        }
    }
    PICK.with(|w| *w.borrow_mut() = names.to_vec());
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(
        &mut agents[p.idx()],
        Box::new(mtg_engine::decision::PassiveAgent),
    );
    agents[p.idx()] = Box::new(Picker { inner });
}

#[test]
fn shriekwood_devourer_untaps_up_to_the_greatest_attacking_power() {
    cr!("701.26b", "608.2h");
    compiles("Shriekwood Devourer");
    for (big, picked, untapped) in [(false, 3, 2), (true, 5, 5)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Shriekwood Devourer");
        let bear = t.battlefield(P0, "Grizzly Bears");
        let maw = t.battlefield(P0, "Colossal Dreadmaw");
        let lands = t.lands(P0, "Forest", 5);
        t.advance_to(P0, Step::PrecombatMain);
        for l in &lands {
            t.g.tap(*l);
        }
        let ents: Vec<Entity> = lands[..picked].iter().map(|l| Entity::Object(*l)).collect();
        t.answer_choose(P0, &ents);
        let mut attackers = vec![(bear, Entity::Player(P1))];
        if big {
            attackers.push((maw, Entity::Player(P1)));
        }
        t.attack(&attackers, &[]);
        let n = lands.iter().filter(|l| !t.g.obj(**l).tapped).count();
        // With only the Bears attacking, X is 2: choosing three lands isn't allowed.
        assert!(n <= untapped, "{big}: {n}");
        if big {
            assert_eq!(n, 5);
        }
    }
}

#[test]
fn taeko_grows_when_another_creature_leaves_without_dying() {
    cr!("603.4", "700.4");
    compiles("Taeko, the Patient Avalanche");
    for (spell, grows) in [("Unsummon", true), ("Lightning Bolt", false)] {
        let mut t = TestGame::new(2);
        let taeko = t.battlefield(P0, "Taeko, the Patient Avalanche");
        let bear = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, if grows { "Island" } else { "Mountain" }, 1);
        let s = t.hand(P0, spell);
        t.advance_to(P0, Step::PrecombatMain);
        t.cast(P0, s).target(bear).go();
        t.resolve_all();
        assert!(!t.on_battlefield(bear));
        assert_eq!(t.counters(taeko, "+1/+1"), grows as u32, "{spell}");
        assert_eq!(scried(&t, P0), grows);
    }
}

#[test]
fn arcane_artisan_exiles_the_tokens_it_made_when_it_leaves() {
    cr!("607.2a", "603.7c");
    compiles("Arcane Artisan");
    let mut t = TestGame::new(2);
    let artisan = t.battlefield(P0, "Arcane Artisan");
    t.lands(P0, "Island", 3);
    t.library_top(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Llanowar Elves");
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, artisan, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1, "the copy");
    t.g.destroy(artisan, None);
    t.settle();
    t.advance_to_step(Step::End);
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.on_battlefield(other), "only the tokens it created");
}

#[test]
fn veterans_voice_targets_a_creature_other_than_the_one_it_taps() {
    cr!("602.2", "602.2b");
    compiles("Veteran's Voice");
    // The enchanted creature alone: no legal target.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let voice = t.battlefield(P0, "Veteran's Voice");
    t.g.attach(voice, Entity::Object(bear));
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t.activate(P0, voice, 0, &[Entity::Object(bear)]).is_err());
    assert!(!t.g.obj(bear).tapped);

    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let elf = t.battlefield(P0, "Llanowar Elves");
    let voice = t.battlefield(P0, "Veteran's Voice");
    t.g.attach(voice, Entity::Object(bear));
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, voice, 0, &[Entity::Object(elf)]).unwrap();
    t.resolve_all();
    assert!(t.g.obj(bear).tapped);
    assert_eq!(t.pt(elf), (3, 2));
}

#[test]
fn bringer_of_the_last_gift_returns_only_cards_that_were_already_there() {
    cr!("701.21a", "400.7");
    compiles("Bringer of the Last Gift");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 8);
    let bringer = t.hand(P0, "Bringer of the Last Gift");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let elf = t.battlefield(P1, "Llanowar Elves");
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P1, "Craw Wurm");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, bringer).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bear) && !t.on_battlefield(elf));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    let giant = t.named_on_battlefield("Hill Giant");
    let wurm = t.named_on_battlefield("Craw Wurm");
    assert_eq!((giant.len(), wurm.len()), (1, 1));
    assert_eq!(t.g.obj(giant[0]).controller, P0);
    assert_eq!(t.g.obj(wurm[0]).controller, P1);
}

#[test]
fn heart_shaped_herb_returns_the_sacrificed_creature_card() {
    cr!("701.21a", "725.1");
    compiles("Heart-Shaped Herb");
    let mut t = TestGame::new(2);
    let herb = t.battlefield(P0, "Heart-Shaped Herb");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.activate(P0, herb, 0, &[]).unwrap();
    t.resolve_all();
    assert_ne!(t.g.current(bear), bear, "sacrificed, then returned");
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.counters(back[0], "+1/+1"), 3);
    assert_eq!(t.g.monarch, Some(P0));
}

#[test]
fn cache_grab_makes_food_for_a_squirrel_controlled_or_returned() {
    cr!("608.2c", "111.10b");
    compiles("Cache Grab");
    for (control, pick, food) in [
        (false, "Grizzly Bears", 0),
        (false, "Squirrel Mob", 1),
        (true, "Grizzly Bears", 1),
    ] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 2);
        for c in ["Hill Giant", "Squirrel Mob", "Grizzly Bears", "Forest"] {
            t.library_top(P0, c);
        }
        if control {
            t.battlefield(P0, "Squirrel Mob");
        }
        let grab = t.hand(P0, "Cache Grab");
        t.advance_to(P0, Step::PrecombatMain);
        pick_by_name(&mut t, P0, &[pick]);
        t.answer_yes(P0, true);
        t.cast(P0, grab).go();
        t.resolve_all();
        assert!(t.in_hand(P0, pick), "{pick}");
        assert_eq!(with_subtype(&t, "Food"), food, "{control} {pick}");
    }
}

#[test]
fn depala_reveals_the_top_x_cards_paid_for() {
    cr!("107.3", "701.20a");
    compiles("Depala, Pilot Exemplar");
    let mut t = TestGame::new(2);
    let depala = t.battlefield(P0, "Depala, Pilot Exemplar");
    t.lands(P0, "Mountain", 2);
    t.library_top(P0, "Hill Giant");
    t.library_top(P0, "Dwarven Trader");
    t.library_top(P0, "Grizzly Bears");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(2));
    t.attack(&[(depala, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Dwarven Trader"), "a Dwarf among the top two");
    assert!(!t.in_hand(P0, "Grizzly Bears") && !t.in_hand(P0, "Hill Giant"));
}

#[test]
fn knickknack_ouphe_reveals_the_top_x_cards() {
    cr!("107.3m", "303.4f");
    compiles("Knickknack Ouphe");
    for x in [1i64, 2] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 3);
        t.library_top(P0, "Rancor");
        t.library_top(P0, "Hill Giant");
        let ouphe = t.hand(P0, "Knickknack Ouphe");
        t.advance_to(P0, Step::PrecombatMain);
        pick_by_name(&mut t, P0, &["Rancor"]);
        t.cast(P0, ouphe).x(x).go();
        t.resolve_all();
        let rancor = t.named_on_battlefield("Rancor");
        assert_eq!(rancor.len(), (x == 2) as usize, "X = {x}");
        if let Some(r) = rancor.first() {
            assert_eq!(
                t.g.obj(*r).attached_to,
                Some(Entity::Object(t.g.current(ouphe))),
                "enchants the chosen creature"
            );
        }
    }
}

#[test]
fn dragonhawk_deals_damage_for_the_cards_still_exiled() {
    cr!("603.7c", "400.7");
    compiles("Dragonhawk, Fate's Tempest");
    for play in [false, true] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Mountain", 5);
        t.library_top(P0, "Forest");
        let hawk = t.hand(P0, "Dragonhawk, Fate's Tempest");
        t.advance_to(P0, Step::PrecombatMain);
        t.cast(P0, hawk).go();
        t.resolve_all();
        let exiled = t.g.find_in_zone(mtg_engine::object::Zone::Exile, "Forest");
        assert_eq!(exiled.len(), 1, "X is 1: Dragonhawk itself");
        if play {
            t.play_land(P0, exiled[0]).unwrap();
        }
        t.advance_to_step(Step::End);
        t.resolve_all();
        assert_eq!(t.life(P1), if play { 20 } else { 18 }, "played: {play}");
    }
}

#[test]
fn toralf_deals_the_excess_damage_to_another_target() {
    cr!("120.10");
    compiles("Toralf, God of Fury");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Toralf, God of Fury");
    t.lands(P0, "Mountain", 1);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, bolt).target(bear).go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bear));
    assert_eq!(t.life(P1), 19, "3 damage to a 2-toughness creature: 1 excess");
}

#[test]
fn lady_loki_deals_the_difference_between_the_spell_and_the_card_found() {
    cr!("608.2h", "701.20a");
    compiles("Lady Loki, Agent of Chaos");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lady Loki, Agent of Chaos");
    t.lands(P0, "Mountain", 1);
    t.library_top(P0, "Hill Giant");
    t.library_top(P0, "Forest");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_yes(P0, false);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    // The Bolt is exiled before it resolves; Hill Giant (4) against Lightning Bolt (1).
    assert_eq!(t.life(P1), 17);
    assert!(t.in_exile("Lightning Bolt") && t.in_exile("Hill Giant"));
}

#[test]
fn last_rites_chooses_a_card_for_each_card_discarded() {
    cr!("701.9a", "608.2c");
    compiles("Last Rites");
    for (discard, chosen) in [(2usize, 2usize), (1, 1)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Swamp", 3);
        let rites = t.hand(P0, "Last Rites");
        let forest = t.hand(P0, "Forest");
        let island = t.hand(P0, "Island");
        for c in ["Grizzly Bears", "Hill Giant", "Mountain"] {
            t.hand(P1, c);
        }
        t.advance_to(P0, Step::PrecombatMain);
        let mine = [Entity::Object(forest), Entity::Object(island)];
        t.answer_choose(P0, &mine[..discard]);
        pick_by_name(&mut t, P0, &["Grizzly Bears", "Hill Giant"]);
        t.cast(P0, rites).target(Entity::Player(P1)).go();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), 2 - discard);
        // As many nonland cards as were discarded (the Mountain is never chosen).
        assert_eq!(t.hand_size(P1), 3 - chosen, "{discard}");
        assert!(t.in_hand(P1, "Mountain"));
    }
}

#[test]
fn extract_brain_casts_one_of_the_chosen_cards() {
    cr!("608.2c", "601.2");
    compiles("Extract Brain");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 2);
    let brain = t.hand(P0, "Extract Brain");
    let bears = t.hand(P1, "Grizzly Bears");
    let giant = t.hand(P1, "Hill Giant");
    t.hand(P1, "Lightning Bolt");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_choose(P1, &[Entity::Object(bears), Entity::Object(giant)]);
    pick_by_name(&mut t, P0, &["Hill Giant"]);
    t.answer_yes(P0, true);
    t.cast(P0, brain).x(2).target(Entity::Player(P1)).go();
    t.resolve_all();
    let g = t.named_on_battlefield("Hill Giant");
    assert_eq!(g.len(), 1);
    assert_eq!(t.g.obj(g[0]).controller, P0);
    assert!(t.in_hand(P1, "Grizzly Bears") && t.in_hand(P1, "Lightning Bolt"));
}

#[test]
fn azors_gateway_counts_different_mana_values_exiled_with_it() {
    cr!("607.2a", "202.3");
    compiles("Azor's Gateway");
    let mut t = TestGame::new(2);
    let gate = t.battlefield(P0, "Azor's Gateway");
    t.lands(P0, "Island", 1);
    for _ in 0..12 {
        t.library_top(P0, "Forest");
        t.library_top(P1, "Forest");
    }
    let cards: Vec<ObjectId> = ["Island", "Llanowar Elves", "Grizzly Bears", "Hill Giant", "Craw Wurm"]
        .iter()
        .map(|c| t.hand(P0, c))
        .collect();
    for (i, c) in cards.iter().enumerate() {
        t.advance_to(P0, Step::PrecombatMain);
        t.answer_choose(P0, &[Entity::Object(*c)]);
        t.activate(P0, gate, 0, &[]).unwrap();
        t.resolve_all();
        // Mana values 0, 1, 2, 4, 6: the fifth one makes five different values.
        assert_eq!(t.life(P0), if i == 4 { 25 } else { 20 }, "after {}", i + 1);
        // (Drawn Forests leave the hand, so nothing is discarded to hand size.)
        for f in t.g.find_in_zone(mtg_engine::object::Zone::Hand(P0), "Forest") {
            t.g.move_object(
                f,
                mtg_engine::object::Zone::Graveyard(P0),
                mtg_engine::events::MoveCause::Effect,
                None,
            );
        }
        t.advance_to(P1, Step::PrecombatMain);
    }
}
