//! CR 605: mana abilities.

use super::r600_common::*;
use mtg_engine::ability::*;
use mtg_engine::mana::ManaType;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::*;

fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect()
}

fn is_mana(def: &CardDef, i: usize) -> bool {
    abilities(def)[i].is_mana_ability()
}

fn enchant(t: &mut TestGame, aura: ObjectId, what: ObjectId) {
    t.g.objects[aura.0 as usize].attached_to = Some(Entity::Object(what));
    t.g.recompute();
}

#[test]
fn only_abilities_meeting_the_criteria_are_mana_abilities() {
    cr!("605.1", "605.1a", "605.1b", "605.5", "605.5a");
    // Activated: no target, could add mana, not loyalty, no library.
    assert!(is_mana(&card("Llanowar Elves"), 0));
    // A target: not a mana ability (Deathrite Shaman).
    assert!(!is_mana(&card("Deathrite Shaman"), 0));
    // A loyalty ability: not a mana ability (Chandra, Torch of Defiance's "+1: Add {R}{R}").
    let chandra = card("Chandra, Torch of Defiance");
    let plus = abilities(&chandra)
        .into_iter()
        .find(|a| a.text.contains("Add {R}{R}"))
        .unwrap();
    assert!(!plus.is_mana_ability());
    // Its cost moves a card from a library: not a mana ability (Deranged Assistant).
    assert!(!is_mana(&card("Deranged Assistant"), 0));
    // Triggered: triggers from a mana ability, no target, could add mana (Wild Growth).
    assert!(is_mana(&card("Wild Growth"), 1));
    // A triggered ability that could add mana but triggers on another event isn't.
    let birgi_like = compile_def(
        "Storyteller Test",
        "Enchantment",
        "{2}",
        "Whenever you cast a spell, add {R}.",
    );
    assert!(!is_mana(&birgi_like, 0));

    // Such abilities follow the normal rules: they use the stack.
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Deathrite Shaman");
    let land = t.graveyard(P1, "Forest");
    t.activate(P0, d, 0, &[Entity::Object(land)]).unwrap();
    assert_eq!(t.stack_len(), 1);
    assert!(pool(&t, P0).is_empty());
    t.resolve();
    assert_eq!(pool(&t, P0).len(), 1);

    let mut t = TestGame::new(2);
    t.custom(P0, birgi_like, Zone::Battlefield);
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.settle();
    // The Bears spell and the trigger.
    assert_eq!(t.stack_len(), 2);
    assert!(pool(&t, P0).is_empty());
    t.resolve();
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
}

#[test]
fn a_mana_ability_remains_one_even_if_it_cant_produce_mana() {
    cr!("605.2");
    // Gaea's Cradle: "{T}: Add {G} for each creature you control."
    let mut t = TestGame::new(2);
    let cradle = t.battlefield(P0, "Gaea's Cradle");
    assert!(t.obj(cradle).chars.abilities[0].is_mana_ability());
    // With no creatures it produces nothing, but it's still activated as a mana ability:
    // it doesn't use the stack.
    t.activate(P0, cradle, 0, &[]).unwrap();
    assert!(t.obj(cradle).tapped);
    assert_eq!(t.stack_len(), 0);
    assert!(pool(&t, P0).is_empty());
    // Tapped, it's still a mana ability.
    assert!(t.obj(cradle).chars.abilities[0].is_mana_ability());
    // With creatures it produces that much.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    let cradle = t.battlefield(P0, "Gaea's Cradle");
    t.activate(P0, cradle, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::G, ManaType::G]);
}

#[test]
fn activating_a_mana_ability_pays_its_costs_and_resolves_immediately() {
    cr!("605.3", "605.3b");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.activate(P0, elves, 0, &[]).unwrap();
    // The cost was paid ({T}) and the mana was added without using the stack, so nothing
    // could respond to it.
    assert!(t.obj(elves).tapped);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pool(&t, P0), vec![ManaType::G]);
    // Like any activated ability, its cost must be payable (CR 602.2): it can't be
    // activated again while tapped.
    assert!(t.activate(P0, elves, 0, &[]).is_err());
    // A summoning-sick creature's {T} mana ability can't be activated.
    let sick = t.battlefield_sick(P0, "Llanowar Elves");
    assert!(t.activate(P0, sick, 0, &[]).is_err());
}

#[test]
fn mana_abilities_can_be_activated_while_paying_during_resolution() {
    cr!("605.3a");
    // Casting a spell: mana abilities are activated during the payment.
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Forest", 5);
    let bears = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, bears).go();
    assert_eq!(lands.iter().filter(|l| t.obj(**l).tapped).count(), 2);
    // An effect asks for a mana payment while it resolves (Mana Leak): the player
    // activates mana abilities in the middle of its resolution, without priority.
    t.lands(P1, "Island", 2);
    let leak = t.hand(P1, "Mana Leak");
    t.cast(P1, leak).target(spell).go();
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(lands.iter().filter(|l| t.obj(**l).tapped).count(), 5);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn a_mana_ability_cant_be_activated_again_until_it_has_resolved() {
    cr!("605.3c");
    // "{1}: Add {R}{R}." It can't be activated again to pay for its own cost.
    let filter = CB::new("Doubling Filter")
        .artifact()
        .ability({
            let mut a = ActivatedAbility::new(
                mana_cost("{1}"),
                Body::effect(Effect::AddMana {
                    who: PlayerRef::You,
                    mana: ManaProduction::Fixed(vec![ManaType::R, ManaType::R]),
                    restriction: None,
                }),
            );
            a.is_mana_ability = true;
            act_from(a)
        })
        .build();
    let mut t = TestGame::new(2);
    let f = t.custom(P0, filter.clone(), Zone::Battlefield);
    assert!(t.activate(P0, f, 0, &[]).is_err());
    assert!(pool(&t, P0).is_empty());
    // Paid with other mana, it works (and resolves before it could be activated again).
    t.lands(P0, "Mountain", 1);
    t.activate(P0, f, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::R, ManaType::R]);
    t.activate(P0, f, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::R, ManaType::R, ManaType::R]);
}

#[test]
fn triggered_mana_abilities_resolve_immediately_after_the_mana_ability() {
    cr!("605.4", "605.4a");
    // Wild Growth: "Whenever enchanted land is tapped for mana, its controller adds an
    // additional {G}."
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let wg = t.battlefield(P0, "Wild Growth");
    enchant(&mut t, wg, forest);
    t.activate(P0, forest, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::G, ManaType::G]);
    assert_eq!(t.stack_len(), 0);
    // While casting a spell, the additional mana can pay for it: one Forest pays for
    // Grizzly Bears ({1}{G}).
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let wg = t.battlefield(P0, "Wild Growth");
    enchant(&mut t, wg, forest);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    assert_eq!(t.stack_len(), 1);
    assert!(pool(&t, P0).is_empty());
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // It follows the rules for triggered abilities: it triggers only on its trigger event.
    // Tapping the land another way doesn't trigger it.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let wg = t.battlefield(P0, "Wild Growth");
    enchant(&mut t, wg, forest);
    t.g.tap(forest);
    t.settle();
    assert!(pool(&t, P0).is_empty());
    assert_eq!(t.stack_len(), 0);
    // Mana Flare: "Whenever a player taps a land for mana, that player adds one mana of
    // any type that land produced." Its controller doesn't matter for who gets the mana.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mana Flare");
    let m = t.battlefield(P1, "Mountain");
    t.activate(P1, m, 0, &[]).unwrap();
    assert_eq!(pool(&t, P1), vec![ManaType::R, ManaType::R]);
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn a_spell_that_adds_mana_is_not_a_mana_ability() {
    cr!("605.5b");
    // Dark Ritual is cast and resolves like any other spell: it uses the stack and can be
    // countered.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let dr = t.hand(P0, "Dark Ritual");
    let spell = t.cast(P0, dr).go();
    assert_eq!(t.stack_len(), 1);
    assert!(pool(&t, P0).is_empty());
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    t.resolve_all();
    assert!(pool(&t, P0).is_empty());
    assert!(t.in_graveyard(P0, "Dark Ritual"));
    // Uncountered, it adds its mana as it resolves.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let dr = t.hand(P0, "Dark Ritual");
    t.cast(P0, dr).go();
    t.resolve();
    assert_eq!(pool(&t, P0), vec![ManaType::B; 3]);
}
