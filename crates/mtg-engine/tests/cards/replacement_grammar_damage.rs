//! Damage replacement and prevention effects compiled by the replacement grammar
//! (`src/oracle/patterns/replacement_grammar.rs`): event patterns with sources,
//! recipients, amounts and conditions; static, "this turn", "the next time" and "the next
//! N damage" scopes; and prevention, redirection, amount changes and "instead"
//! instructions (CR 614.1a, 614.9, 615).

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

fn compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn damage_grammar_cards_compile() {
    compiles(&[
        "Ajani Steadfast",
        "Undead Alchemist",
        "Lashknife Barrier",
        "Battletide Alchemist",
        "Hyperion, Supreme Hero",
        "Fated Firepower",
        "Hawkeye, Young Avenger",
        "Forethought Amulet",
        "Angel of Suffering",
        "Force Bubble",
        "Undergrowth Champion",
        "Polukranos, Unchained",
        "Ugin's Conjurant",
        "Sekki, Seasons' Guide",
        "Lichenthrope",
        "Dralnu, Lich Lord",
        "Panther Habit",
        "Unbreathing Horde",
        "Szadek, Lord of Secrets",
        "Prismatic Ward",
        "Gideon's Intervention",
        "Ajani's Aid",
        "Sanctum Guardian",
        "Harm's Way",
        "Shining Shoal",
        "Awe Strike",
        "Morningtide's Light",
        "Gatta and Luzzu",
        "Karona's Zealot",
        "Ascent of the Worthy",
        "Circle of Protection: Shadow",
        "Aegis of Honor",
        "Mercenaries",
        "Mirrorwood Treefolk",
        "Ward of Piety",
        "Circle of Solace",
        "Glarecaster",
        "Shield Dancer",
        "Dark Sphere",
        "Opal-Eye, Konda's Yojimbo",
        "Pilgrim of Virtue",
        "Pilgrim of Justice",
        "Hazduhr the Abbot",
        "Gideon's Sacrifice",
        "Heroic Sacrifice",
        "Reverberation",
        "Questing Beast",
        "The Rollercrusher Ride",
        "Absorbing Man and Titania",
        "Overblaze",
        "Neriv, Heart of the Storm",
        "Ghosts of the Innocent",
        "Sawhorn Nemesis",
        "Benevolent Unicorn",
        "Equal Treatment",
        "Purity",
        "Honorable Passage",
        "Shadowbane",
        "Rankle and Torbran",
        "Old Fat Spider Can't See Me",
        "Surge of Salvation",
        "Lithomancer's Focus",
        "Circle of Despair",
        "Opal-Eye, Konda's Yojimbo",
    ]);
}

#[test]
fn phytohydra_gets_counters_instead_of_damage_even_if_it_cant_be_prevented() {
    cr!("614.1a", "615.12");
    ruling!(
        "Phytohydra",
        "Phytohydra's ability doesn't prevent damage. Damage that can't be prevented will still be replaced"
    );
    compiles(&["Phytohydra"]);
    let mut t = TestGame::new(2);
    let hydra = t.battlefield(P0, "Phytohydra");
    let src = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(src, Entity::Object(hydra), 3, false);
    t.settle();
    assert_eq!(t.obj_now(hydra).damage, 0);
    assert_eq!(t.counters(hydra, "+1/+1"), 3);
    // Frenzied Baloth: combat damage can't be prevented; it's still replaced.
    t.battlefield(P1, "Frenzied Baloth");
    t.g.deal_damage(src, Entity::Object(hydra), 2, true);
    t.settle();
    assert_eq!(t.obj_now(hydra).damage, 0);
    assert_eq!(t.counters(hydra, "+1/+1"), 5);
}

#[test]
fn oathsworn_knight_prevents_damage_only_while_it_has_a_counter_and_removes_one() {
    cr!("615.1a", "615.5");
    ruling!(
        "Oathsworn Knight",
        "Oathsworn Knight's last ability removes only one counter from it each time it would be dealt damage"
    );
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Oathsworn Knight");
    t.g.objects[knight.0 as usize]
        .counters
        .insert("+1/+1".into(), 2);
    let src = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(src, Entity::Object(knight), 3, false);
    t.settle();
    assert_eq!(t.obj_now(knight).damage, 0);
    assert_eq!(t.counters(knight, "+1/+1"), 1);
}

#[test]
fn polukranos_removes_that_many_counters() {
    cr!("615.5");
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Polukranos, Unchained");
    t.g.objects[p.0 as usize].counters.insert("+1/+1".into(), 6);
    let src = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(src, Entity::Object(p), 4, false);
    t.settle();
    assert_eq!(t.obj_now(p).damage, 0);
    assert_eq!(t.counters(p, "+1/+1"), 2);
}

#[test]
fn callous_giant_prevents_only_small_damage() {
    cr!("614.1a", "615.1a");
    compiles(&["Callous Giant"]);
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Callous Giant");
    let src = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(src, Entity::Object(g), 3, false);
    t.settle();
    assert_eq!(t.obj_now(g).damage, 0);
    // 4 damage isn't prevented: it's lethal.
    t.g.deal_damage(src, Entity::Object(g), 4, false);
    t.settle();
    assert!(!t.on_battlefield(g));
}

#[test]
fn divine_presence_caps_large_damage_at_three() {
    cr!("614.1a");
    compiles(&["Divine Presence"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Divine Presence");
    let src = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(src, Entity::Player(P0), 5, false);
    t.settle();
    assert_eq!(t.life(P0), 17);
    t.g.deal_damage(src, Entity::Player(P0), 2, false);
    t.settle();
    assert_eq!(t.life(P0), 15);
}

#[test]
fn gisela_doubles_damage_to_opponents_and_halves_damage_to_you() {
    cr!("614.1a", "615.1a", "616.1");
    ruling!(
        "Gisela, Blade of Goldnight",
        "Gisela doubles damage dealt to opponents and permanents your opponents control from any source"
    );
    compiles(&["Gisela, Blade of Goldnight"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gisela, Blade of Goldnight");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    // Damage from an opponent's own source to that opponent is doubled too.
    t.g.deal_damage(theirs, Entity::Player(P1), 2, false);
    t.settle();
    assert_eq!(t.life(P1), 16);
    // Prevent half, rounded up: 3 damage becomes 1.
    t.g.deal_damage(theirs, Entity::Player(P0), 3, false);
    t.settle();
    assert_eq!(t.life(P0), 19);
    t.g.deal_damage(theirs, Entity::Object(mine), 1, false);
    t.settle();
    assert_eq!(t.obj_now(mine).damage, 0);
}

#[test]
fn hyperion_prevents_all_but_one_damage() {
    cr!("615.1a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hyperion, Supreme Hero");
    let src = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(src, Entity::Player(P0), 5, false);
    t.settle();
    assert_eq!(t.life(P0), 19);
}

#[test]
fn soul_scar_mage_turns_noncombat_damage_to_opposing_creatures_into_counters() {
    cr!("614.1a", "120.2b");
    compiles(&["Soul-Scar Mage"]);
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Soul-Scar Mage");
    let foe = t.battlefield(P1, "Colossal Dreadmaw");
    t.g.deal_damage(mage, Entity::Object(foe), 2, false);
    t.settle();
    assert_eq!(t.obj_now(foe).damage, 0);
    assert_eq!(t.counters(foe, "-1/-1"), 2);
    // Combat damage isn't replaced.
    t.g.deal_damage(mage, Entity::Object(foe), 1, true);
    t.settle();
    assert_eq!(t.obj_now(foe).damage, 1);
}

#[test]
fn frenzied_baloth_combat_damage_cant_be_prevented() {
    cr!("615.12", "120.2a");
    ruling!(
        "Frenzied Baloth",
        "Protection prevents damage, so protection will be unable to prevent combat damage"
    );
    compiles(&["Frenzied Baloth", "Morningtide's Light"]);
    let mut t = TestGame::new(2);
    let baloth = t.battlefield(P1, "Frenzied Baloth");
    t.lands(P0, "Plains", 4);
    let light = t.hand(P0, "Morningtide's Light");
    t.cast(P0, light).go();
    t.resolve();
    // Noncombat damage is still prevented; combat damage isn't.
    t.g.deal_damage(baloth, Entity::Player(P0), 3, false);
    t.settle();
    assert_eq!(t.life(P0), 20);
    t.g.deal_damage(baloth, Entity::Player(P0), 3, true);
    t.settle();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn harsh_judgment_redirects_spell_damage_of_the_chosen_color_to_its_controller() {
    cr!("614.9", "614.12");
    let mut t = TestGame::new(2);
    // Red is the fourth color option.
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.enter(P0, "Harsh Judgment");
    t.resolve_all();
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.resolve();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn kill_suit_cultist_destroys_the_creature_instead_of_damaging_it() {
    cr!("614.1a");
    let mut t = TestGame::new(2);
    let cultist = t.battlefield(P0, "Kill-Suit Cultist");
    let foe = t.battlefield(P1, "Colossal Dreadmaw");
    t.lands(P0, "Swamp", 1);
    t.activate(P0, cultist, 0, &[Entity::Object(foe)]).unwrap();
    t.resolve();
    let src = t.battlefield(P0, "Grizzly Bears");
    t.g.deal_damage(src, Entity::Object(foe), 1, false);
    t.settle();
    assert!(!t.on_battlefield(foe));
}

#[test]
fn reflect_damage_deals_the_damage_to_the_sources_controller() {
    cr!("614.9", "609.7a");
    compiles(&["Reflect Damage"]);
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(P0).go();
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Mountain", 3);
    let reflect = t.hand(P0, "Reflect Damage");
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.cast(P0, reflect).go();
    t.resolve();
    t.resolve();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn prismatic_strands_prevents_damage_from_sources_of_the_chosen_color() {
    cr!("615.1a", "609.7b");
    compiles(&["Prismatic Strands"]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let red = t.battlefield(P1, "Goblin Piker");
    t.lands(P0, "Plains", 3);
    let strands = t.hand(P0, "Prismatic Strands");
    // Green.
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    t.cast(P0, strands).go();
    t.resolve();
    t.g.deal_damage(bears, Entity::Player(P0), 2, true);
    t.g.deal_damage(red, Entity::Player(P0), 2, true);
    t.settle();
    assert_eq!(t.life(P0), 18);
}

#[test]
fn kithkin_armor_shields_the_creature_it_enchanted_after_being_sacrificed() {
    cr!("609.7a", "615.7");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let armor = t.battlefield(P0, "Kithkin Armor");
    t.g.obj_mut(armor).attached_to = Some(Entity::Object(bears));
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.activate(P0, armor, 0, &[]).unwrap();
    t.resolve();
    t.g.deal_damage(giant, Entity::Object(bears), 3, true);
    t.settle();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn martyrs_cause_shields_a_target_player() {
    cr!("609.7a", "115.4");
    let mut t = TestGame::new(2);
    let cause = t.battlefield(P0, "Martyr's Cause");
    let fodder = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.activate(P0, cause, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve();
    t.g.deal_damage(giant, Entity::Player(P0), 3, true);
    t.settle();
    assert_eq!(t.life(P0), 20);
    // Used up.
    t.g.deal_damage(giant, Entity::Player(P0), 3, true);
    t.settle();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn deflecting_palm_deals_the_prevented_damage_to_the_sources_controller() {
    cr!("615.5", "609.7a");
    compiles(&["Deflecting Palm"]);
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Mountain", 1);
    let palm = t.hand(P0, "Deflecting Palm");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.cast(P0, palm).go();
    t.resolve();
    t.g.deal_damage(giant, Entity::Player(P0), 3, true);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn swans_of_bryn_argoll_make_the_sources_controller_draw() {
    cr!("615.5");
    ruling!(
        "Swans of Bryn Argoll",
        "If the source of the damage is a permanent, Swans of Bryn Argoll checks who that permanent"
    );
    compiles(&["Swans of Bryn Argoll"]);
    let mut t = TestGame::new(2);
    let swans = t.battlefield(P0, "Swans of Bryn Argoll");
    let giant = t.battlefield(P1, "Hill Giant");
    for _ in 0..3 {
        t.library_top(P1, "Island");
    }
    let before = t.hand_size(P1);
    t.g.deal_damage(giant, Entity::Object(swans), 3, true);
    t.settle();
    t.resolve_all();
    assert_eq!(t.obj_now(swans).damage, 0);
    assert_eq!(t.hand_size(P1), before + 3);
}

#[test]
fn lightning_army_of_one_doubles_damage_to_that_player_and_their_permanents() {
    cr!("614.1a", "611.2c");
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Lightning, Army of One");
    let foe = t.battlefield(P1, "Colossal Dreadmaw");
    let mine = t.battlefield(P0, "Hill Giant");
    t.g.deal_damage(l, Entity::Player(P1), 1, true);
    t.settle();
    t.resolve_all();
    let life = t.life(P1);
    t.g.deal_damage(mine, Entity::Object(foe), 2, false);
    t.g.deal_damage(mine, Entity::Player(P1), 2, false);
    t.settle();
    assert_eq!(t.obj_now(foe).damage, 4);
    assert_eq!(t.life(P1), life - 4);
    // Not you.
    let mine = t.life(P0);
    t.g.deal_damage(foe, Entity::Player(P0), 2, false);
    t.settle();
    assert_eq!(t.life(P0), mine - 2);
}

#[test]
fn decorated_griffin_prevents_the_next_combat_damage_only() {
    cr!("615.7", "120.2a");
    let mut t = TestGame::new(2);
    let griffin = t.battlefield(P0, "Decorated Griffin");
    t.lands(P0, "Plains", 2);
    t.activate(P0, griffin, 0, &[]).unwrap();
    t.resolve();
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(giant, Entity::Player(P0), 2, false);
    t.settle();
    assert_eq!(t.life(P0), 18);
    t.g.deal_damage(giant, Entity::Player(P0), 3, true);
    t.settle();
    assert_eq!(t.life(P0), 16);
}

#[test]
fn barbed_wire_prevents_the_next_damage_it_would_deal() {
    cr!("615.7");
    compiles(&["Barbed Wire"]);
    let mut t = TestGame::new(2);
    let wire = t.battlefield(P0, "Barbed Wire");
    t.lands(P0, "Plains", 2);
    t.activate(P0, wire, 0, &[]).unwrap();
    t.resolve();
    t.g.deal_damage(wire, Entity::Player(P1), 1, false);
    t.settle();
    assert_eq!(t.life(P1), 20);
    t.g.deal_damage(wire, Entity::Player(P1), 1, false);
    t.settle();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn hazduhr_redirects_the_next_x_damage_to_itself() {
    cr!("614.9");
    let mut t = TestGame::new(2);
    let abbot = t.battlefield(P0, "Hazduhr the Abbot");
    let knight = t.battlefield(P0, "Benalish Knight");
    t.lands(P0, "Plains", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, abbot, 0, &[Entity::Object(knight)]).unwrap();
    t.resolve();
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(giant, Entity::Object(knight), 3, true);
    t.settle();
    assert_eq!(t.obj_now(abbot).damage, 2);
    assert_eq!(t.obj_now(knight).damage, 1);
}

#[test]
fn reverberation_deals_sorcery_damage_to_its_controller() {
    cr!("614.9");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let axe = t.hand(P0, "Lava Axe");
    let axe = t.cast(P0, axe).target(P1).go();
    t.lands(P1, "Island", 4);
    let rev = t.hand(P1, "Reverberation");
    t.cast(P1, rev).target(Entity::Object(axe)).go();
    t.resolve();
    t.resolve();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 15);
}

#[test]
fn refraction_trap_deals_the_prevented_damage_to_the_target_chosen_on_cast() {
    cr!("615.5", "615.7", "609.7a", "601.2c");
    compiles(&["Refraction Trap"]);
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 4);
    let trap = t.hand(P0, "Refraction Trap");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.cast(P0, trap).target(Entity::Object(bears)).go();
    t.resolve();
    t.g.deal_damage(giant, Entity::Player(P0), 3, true);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(!t.on_battlefield(bears), "{}", t.dump_log());
}

#[test]
fn kitsune_palliator_shields_each_creature_and_each_player_separately() {
    cr!("615.7");
    compiles(&["Kitsune Palliator"]);
    let mut t = TestGame::new(2);
    let fox = t.battlefield(P0, "Kitsune Palliator");
    let giant = t.battlefield(P1, "Hill Giant");
    t.activate(P0, fox, 0, &[]).unwrap();
    t.resolve();
    t.g.deal_damage(giant, Entity::Player(P0), 2, false);
    t.g.deal_damage(giant, Entity::Player(P1), 2, false);
    t.g.deal_damage(giant, Entity::Object(fox), 1, false);
    t.settle();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.obj_now(fox).damage, 0);
}

#[test]
fn captains_maneuver_redirects_the_next_x_damage_to_another_target() {
    cr!("614.9", "115.3");
    compiles(&["Captain's Maneuver"]);
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Mountain", 2);
    let m = t.hand(P0, "Captain's Maneuver");
    t.cast(P0, m)
        .x(2)
        .target(P0)
        .target(P1)
        .go();
    t.resolve();
    t.g.deal_damage(giant, Entity::Player(P0), 3, true);
    t.settle();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn crumbling_sanctuary_mills_to_exile_instead_of_damage() {
    cr!("614.1a");
    compiles(&["Crumbling Sanctuary", "Gloom Surgeon"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crumbling Sanctuary");
    let giant = t.battlefield(P1, "Hill Giant");
    let lib = t.library_size(P0);
    t.g.deal_damage(giant, Entity::Player(P0), 3, true);
    t.settle();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.library_size(P0), lib - 3);
}

#[test]
fn temple_altisaur_prevents_all_but_one_damage_to_other_dinosaurs() {
    cr!("615.1a");
    compiles(&["Temple Altisaur"]);
    let mut t = TestGame::new(2);
    let altisaur = t.battlefield(P0, "Temple Altisaur");
    let dino = t.battlefield(P0, "Colossal Dreadmaw");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(giant, Entity::Object(dino), 5, false);
    t.g.deal_damage(giant, Entity::Object(altisaur), 2, false);
    t.settle();
    assert_eq!(t.obj_now(dino).damage, 1);
    assert_eq!(t.obj_now(altisaur).damage, 2);
}

#[test]
fn blood_of_the_martyr_may_redirect_damage_to_creatures_to_you() {
    cr!("614.9");
    compiles(&["Blood of the Martyr"]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 3);
    let s = t.hand(P0, "Blood of the Martyr");
    t.cast(P0, s).go();
    t.resolve();
    t.answer_yes(P0, true);
    t.g.deal_damage(giant, Entity::Object(bears), 2, false);
    t.settle();
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.life(P0), 18);
}
