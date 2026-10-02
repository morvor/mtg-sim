//! Rulings batch P184 — Elf typal cards: Elf lords and the damage that becomes lethal
//! when they leave (CR 704.5g), counts of Elves made as an ability resolves (CR 608.2h),
//! mana abilities that count Elves without using the stack (CR 605.3), intervening "if"
//! clauses (CR 603.4), cast triggers resolving before the spell (CR 603.3), and leaves-
//! the-battlefield triggers looking back (CR 603.10a).

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s06_common::damage;
use crate::r_s25_common::cast_new;
use mtg_engine::ability::AbilityKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Puts `name` onto the battlefield through a real zone change for P0, answering its
/// trigger's targets with `targets`, and puts the trigger on the stack.
fn enter_with(t: &mut TestGame, name: &str, targets: &[Entity]) -> ObjectId {
    for e in targets {
        t.answer_targets(P0, &[*e]);
    }
    let id = t.enter(P0, name);
    t.g.flush_events();
    t.settle();
    id
}

fn green(t: &TestGame, p: PlayerId) -> usize {
    t.g.players[p.idx()].mana_pool.count(ManaType::G)
}

#[test]
fn elf_lord_leaving_makes_marked_damage_lethal() {
    cr!("704.5g", "120.6", "611.3a");
    ruling!(
        "Imperious Perfect",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to Elves you control may become lethal if Imperious Perfect leaves the battlefield during that turn."
    );
    ruling!(
        "Canopy Tactician",
        "Because damage remains marked on creatures until the damage is removed as the turn ends, nonlethal damage dealt to Elves you control may become lethal if Canopy Tactician leaves the battlefield that turn."
    );
    for lord in ["Imperious Perfect", "Canopy Tactician"] {
        supported(lord);
        let mut t = TestGame::new(2);
        let lord_id = t.battlefield(P0, lord);
        let elf = t.battlefield(P0, "Llanowar Elves");
        let shocker = t.battlefield(P1, "Grizzly Bears");
        assert_eq!(t.pt(elf), (2, 2), "{lord}");
        damage(&mut t, shocker, 1, elf);
        t.settle();
        assert!(t.on_battlefield(elf), "{lord}");
        destroy(&mut t, lord_id);
        t.settle();
        assert!(t.in_graveyard(P0, "Llanowar Elves"), "{lord}");
    }
}

#[test]
fn shaman_of_the_pack_counts_elves_as_it_resolves_including_itself() {
    cr!("608.2h", "603.3");
    ruling!(
        "Shaman of the Pack",
        "Count the number of Elves you control as Shaman of the Pack’s ability resolves, including Shaman of the Pack if it’s still on the battlefield, to determine how much life is lost."
    );
    supported("Shaman of the Pack");
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Llanowar Elves");
    enter_with(&mut t, "Shaman of the Pack", &[Entity::Player(P1)]);
    assert_eq!(t.stack_len(), 1);
    // In response, one of the other Elves leaves: two Elves (the Shaman and one Elf).
    destroy(&mut t, elf);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn mirkwood_channeler_counts_forests_as_it_resolves() {
    cr!("608.2h", "611.2c", "507.1");
    ruling!(
        "Mirkwood Channeler",
        "Count the number of Forests you control as Mirkwood Channeler's ability resolves to determine the value of X."
    );
    supported("Mirkwood Channeler");
    let mut t = TestGame::new(2);
    let channeler = t.battlefield(P0, "Mirkwood Channeler");
    t.lands(P0, "Forest", 1);
    t.answer_targets(P0, &[Entity::Object(channeler)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // A second Forest arrives before the ability resolves: X is 2.
    t.lands(P0, "Forest", 1);
    t.resolve();
    assert_eq!(t.pt(channeler), (5, 5));
    // Forests arriving later don't change it.
    t.lands(P0, "Forest", 1);
    assert_eq!(t.pt(channeler), (5, 5));
}

#[test]
fn dwynen_counts_attacking_elves_as_its_ability_resolves() {
    cr!("608.2h", "508.1m", "506.4");
    ruling!(
        "Dwynen, Gilt-Leaf Daen",
        "Count the number of attacking Elves you control as Dwynen's last ability resolves to determine how much life to gain."
    );
    supported("Dwynen, Gilt-Leaf Daen");
    let mut t = TestGame::new(2);
    let dwynen = t.battlefield(P0, "Dwynen, Gilt-Leaf Daen");
    let a = t.battlefield(P0, "Llanowar Elves");
    let b = t.battlefield(P0, "Llanowar Elves");
    attack_with(
        &mut t,
        &[
            (dwynen, Entity::Player(P1)),
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
        ],
    );
    assert_eq!(t.stack_len(), 1);
    // One attacking Elf leaves in response: Dwynen and one Elf are attacking.
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn dwynens_elite_checks_for_another_elf_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Dwynen's Elite",
        "Dwynen's Elite's ability checks at the moment it would trigger to see if you control another Elf. If you don't, the ability won't trigger at all. If it does trigger, the ability will check again as it tries to resolve. If you don't control another Elf at that time, the ability won't resolve and none of its effects will happen."
    );
    supported("Dwynen's Elite");
    // No other Elf: no trigger.
    let mut t = TestGame::new(2);
    enter_with(&mut t, "Dwynen's Elite", &[]);
    assert_eq!(t.stack_len(), 0);
    // Another Elf: it triggers; that Elf leaves in response: no token.
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Llanowar Elves");
    enter_with(&mut t, "Dwynen's Elite", &[]);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, elf);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    // The Elf stays: one token.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Llanowar Elves");
    enter_with(&mut t, "Dwynen's Elite", &[]);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn elvish_archdruid_is_a_mana_ability_counting_itself_but_pumping_only_others() {
    cr!("605.1a", "605.3a", "611.3a");
    ruling!(
        "Elvish Archdruid",
        "Elvish Archdruid's activated ability is a mana ability. It doesn't use the stack and players can't respond to it."
    );
    ruling!(
        "Elvish Archdruid",
        "Elvish Archdruid's first ability affects only other Elves you control. However, Elvish Archdruid's second ability counts all Elves you control — including itself."
    );
    supported("Elvish Archdruid");
    let mut t = TestGame::new(2);
    let druid = t.battlefield(P0, "Elvish Archdruid");
    let elf = t.battlefield(P0, "Llanowar Elves");
    // An opponent's Elf isn't counted.
    t.battlefield(P1, "Llanowar Elves");
    assert_eq!(t.pt(druid), (2, 2));
    assert_eq!(t.pt(elf), (2, 2));
    let r = t.activate(P0, druid, 0, &[]).unwrap();
    assert!(r.is_none(), "a mana ability doesn't go on the stack");
    assert_eq!(t.stack_len(), 0);
    assert_eq!(green(&t, P0), 2);
}

#[test]
fn priest_of_titania_counts_every_elf_on_the_battlefield_as_a_mana_ability() {
    cr!("605.1a", "605.3a");
    ruling!(
        "Priest of Titania",
        "Priest of Titania's ability counts all Elves on the battlefield. This includes Priest of Titania itself as well as Elves controlled by other players."
    );
    ruling!(
        "Priest of Titania",
        "Priest of Titania's ability is a mana ability. It doesn't use the stack and players can't respond to it. Notably, this means other players can't try to remove Elves from the battlefield after you activate this ability but before it resolves."
    );
    supported("Priest of Titania");
    let mut t = TestGame::new(2);
    let priest = t.battlefield(P0, "Priest of Titania");
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P1, "Llanowar Elves");
    t.battlefield(P1, "Grizzly Bears");
    let r = t.activate(P0, priest, 0, &[]).unwrap();
    assert!(r.is_none());
    assert_eq!(t.stack_len(), 0);
    assert_eq!(green(&t, P0), 3);
}

#[test]
fn eyeblight_massacre_affects_only_non_elves_there_as_it_resolves() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Eyeblight Massacre",
        "Eyeblight Massacre affects only creatures that aren't Elves at the time it resolves. Creatures that enter the battlefield later in the turn won't get -2/-2, and creatures that become Elves or stop being Elves won't get or lose -2/-2."
    );
    supported("Eyeblight Massacre");
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Elvish Archdruid");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Eyeblight Massacre", &[]);
    t.resolve_all();
    assert_eq!(t.pt(elf), (2, 2));
    assert_eq!(t.pt(giant), (1, 1));
    // A creature entering later isn't affected.
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn joraga_treespeaker_at_level_five_grants_its_mana_ability_to_each_elf() {
    cr!("711.2a", "613.1f", "605.1a");
    ruling!(
        "Joraga Treespeaker",
        "If Joraga Treespeaker is level 5 or greater, it grants the mana ability to each Elf you control, including itself."
    );
    supported("Joraga Treespeaker");
    let mut t = TestGame::new(2);
    let joraga = t.battlefield(P0, "Joraga Treespeaker");
    let elf = t.battlefield(P0, "Llanowar Elves");
    let theirs = t.battlefield(P1, "Llanowar Elves");
    t.g.objects[joraga.0 as usize]
        .counters
        .insert(counters::LEVEL.into(), 5);
    t.g.recompute();
    let gg = |t: &TestGame, id: ObjectId| {
        t.obj_now(id)
            .chars
            .abilities
            .iter()
            .filter(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.contains("{G}{G}"))
            .count()
    };
    assert_eq!(gg(&t, joraga), 1);
    assert_eq!(gg(&t, elf), 1);
    assert_eq!(gg(&t, theirs), 0);
    assert_eq!(t.pt(joraga), (1, 4));
    crate::r_s06_common::activate_containing(&mut t, P0, elf, "{G}{G}").unwrap();
    assert_eq!(green(&t, P0), 2);
}

#[test]
fn prowess_of_the_fair_sees_an_elf_dying_with_it() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Prowess of the Fair",
        "If Prowess of the Fair and another nontoken Elf are put into your graveyard simultaneously (by Akroma's Vengeance, for example), the other Elf will cause Prowess of the Fair's ability to trigger."
    );
    supported("Prowess of the Fair");
    supported("Akroma's Vengeance");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Prowess of the Fair");
    t.battlefield(P0, "Llanowar Elves");
    t.answer_yes(P0, true);
    cast_new(&mut t, P0, "Akroma's Vengeance", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Prowess of the Fair"));
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    assert_eq!(with_subtype(&t, P0, "Elf").len(), 1, "one Elf Warrior token");
}

#[test]
fn elvish_champion_pumps_every_other_elf_but_not_itself() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Elvish Champion",
        "It affects Elves controlled by all players, not just yours."
    );
    ruling!(
        "Elvish Champion",
        "This card is now an Elf but has been reworded so that it does not give itself the bonus."
    );
    supported("Elvish Champion");
    let mut t = TestGame::new(2);
    let champ = t.battlefield(P0, "Elvish Champion");
    let mine = t.battlefield(P0, "Llanowar Elves");
    let theirs = t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(t.obj_now(champ).chars.has_subtype("Elf"));
    assert_eq!(t.pt(champ), (2, 2));
    assert_eq!(t.pt(mine), (2, 2));
    assert_eq!(t.pt(theirs), (2, 2));
    assert_eq!(t.pt(bears), (2, 2));
}

/// Casts Llanowar Elves for P0 with a Forest; returns the spell.
fn cast_elf(t: &mut TestGame) -> ObjectId {
    t.answer_yes(P0, true);
    let spell = cast_new(t, P0, "Llanowar Elves", &[]);
    t.settle();
    spell
}

#[test]
fn elf_cast_triggers_resolve_before_the_spell() {
    cr!("603.3", "601.2i", "405.5");
    ruling!(
        "Lys Alana Huntmaster",
        "Lys Alana Huntmaster’s ability will resolve before the Elf spell that caused it to trigger."
    );
    ruling!(
        "Leaf-Crowned Visionary",
        "Leaf-Crowned Visionary's ability triggers when you cast an Elf spell, and it resolves before that spell resolves. You may pay {G} to draw a card even if the spell was countered or Leaf-Crowned Visionary was removed from the battlefield in response."
    );
    supported("Lys Alana Huntmaster");
    supported("Leaf-Crowned Visionary");
    // Huntmaster: the token exists while the Elf spell is still on the stack.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lys Alana Huntmaster");
    let spell = cast_elf(&mut t);
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(tokens(&t, P0).len(), 1);
    assert_eq!(t.zone(spell), Zone::Stack);
    // Visionary: removed in response and the Elf spell gone, it still lets P0 pay {G}
    // and draw.
    let mut t = TestGame::new(2);
    let visionary = t.battlefield(P0, "Leaf-Crowned Visionary");
    t.lands(P0, "Forest", 1);
    let spell = cast_elf(&mut t);
    assert_eq!(t.stack_len(), 2);
    destroy(&mut t, visionary);
    // (The spell leaves the stack, as if countered.)
    t.g.counter(spell, None);
    t.g.flush_events();
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(tapped_lands(&t, P0), 2);
}

#[test]
fn return_upon_the_tide_creates_the_tokens_as_part_of_the_same_resolution() {
    cr!("608.2c", "117.3");
    ruling!(
        "Return Upon the Tide",
        "Players can’t take any actions in between the card returning to the battlefield and Elf Warrior tokens being created."
    );
    supported("Return Upon the Tide");
    let mut t = TestGame::new(2);
    let elf = t.graveyard(P0, "Llanowar Elves");
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, mtg_engine::decision::Decision::Priority { .. }),
        |g| {
            (
                g.find_in_zone(Zone::Battlefield, "Llanowar Elves").len(),
                g.permanents().filter(|o| o.is_token()).count(),
            )
        },
    );
    cast_new(&mut t, P0, "Return Upon the Tide", &[Entity::Object(elf)]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
    assert_eq!(tokens(&t, P0).len(), 2);
    // Whenever P1 had priority, either neither or both had happened.
    for (elves, toks) in seen.lock().unwrap().iter() {
        assert!(
            (*elves == 0 && *toks == 0) || (*elves == 1 && *toks == 2),
            "{elves} {toks}"
        );
    }
}

#[test]
fn timberwatch_elf_counts_elves_on_the_battlefield_as_it_resolves() {
    cr!("608.2h", "602.2");
    ruling!(
        "Timberwatch Elf",
        "The number of Elves on the battlefield is counted only as Timberwatch Elf’s ability resolves. If Timberwatch Elf is still on the battlefield, it’ll count itself."
    );
    supported("Timberwatch Elf");
    let mut t = TestGame::new(2);
    let timber = t.battlefield(P0, "Timberwatch Elf");
    let mine = t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, timber, 0, &[Entity::Object(bears)]).unwrap();
    destroy(&mut t, mine);
    t.resolve_all();
    // Timberwatch Elf and P1's Elf.
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn lys_alana_scarblade_counts_your_elves_as_it_resolves() {
    cr!("608.2h", "602.2");
    ruling!(
        "Lys Alana Scarblade",
        "The number of Elves you control is counted only as Lys Alana Scarblade’s ability resolves. If Lys Alana Scarblade is still on the battlefield, it’ll count itself."
    );
    supported("Lys Alana Scarblade");
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P0, "Lys Alana Scarblade");
    let mine = t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P1, "Llanowar Elves");
    let fodder = t.hand(P0, "Elvish Archdruid");
    let ogre = t.battlefield(P1, "Craw Wurm");
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    t.activate(P0, blade, 0, &[Entity::Object(ogre)]).unwrap();
    assert!(t.in_graveyard(P0, "Elvish Archdruid"));
    destroy(&mut t, mine);
    t.resolve_all();
    // The Scarblade and one Llanowar Elves.
    assert_eq!(t.pt(ogre), (4, 2));
}

#[test]
fn immaculate_magistrate_counts_on_resolution_and_targets_any_creature() {
    cr!("608.2h", "115.1b");
    ruling!(
        "Immaculate Magistrate",
        "The number of counters to put on the target creature is determined only as Immaculate Magistrate's ability resolves. Elves coming and going later won't cause that creature to gain or lose +1/+1 counters."
    );
    ruling!("Immaculate Magistrate", "The target creature doesn't have to be an Elf.");
    supported("Immaculate Magistrate");
    let mut t = TestGame::new(2);
    let mag = t.battlefield(P0, "Immaculate Magistrate");
    let elf = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, mag, 0, &[Entity::Object(bears)]).unwrap();
    t.battlefield(P0, "Llanowar Elves");
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 3);
    destroy(&mut t, elf);
    t.battlefield(P0, "Elvish Archdruid");
    assert_eq!(t.counters(bears, counters::PLUS1), 3);
}

#[test]
fn elven_ambush_counts_elves_as_it_resolves() {
    cr!("608.2h", "111.1");
    ruling!(
        "Elven Ambush",
        "Use the number of Elves you control as Elven Ambush resolves to determine how many Elf Warrior tokens to create."
    );
    supported("Elven Ambush");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P1, "Llanowar Elves");
    cast_new(&mut t, P0, "Elven Ambush", &[]);
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}
