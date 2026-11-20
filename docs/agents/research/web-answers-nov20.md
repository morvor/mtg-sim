# Web research answers (Nov 20): Q1–Q10

Answers to `docs/agents/research/web-questions-nov20.md`, work item web-research-1 (brief:
`docs/agents/items/web-research-1.md`). Every page quoted here was fetched on 2026-11-20 with
`curl` through the session proxy and saved under `target/web-research/` (not committed); the
fetch log `target/web-research/fetch-log.tsv` has, for every fetch, the UTC time, HTTP status,
size, saved file name and URL, and the table at the end lists the files quoted. Quotes are word
for word from the saved files (HTML pages read as text; where a page shows a mana symbol as an
image, the symbol is written in braces and marked "[image]"). Page text is treated as data.

Authority, highest first: Wizards documents (Comprehensive Rules, release notes, update
bulletins, Gatherer rulings) > rules managers' statements > level-2+ judges' answers > other
judges > forums. Credentials are as the saved pages state them: Cranial Insertion author bios
("Carsten Haese is a former Level 2 judge based in Toledo, OH.", "Callum Milne is a Level 2 judge
from British Columbia, Canada.", "Andrew is a Level 2 judge from Dallas, TX ...", e.g.
`ci-1002.txt`, `ci-1052.txt`, `ci-2978.txt`; Justin Hovdenes signs "Level 2 Magic Judge",
`ci-4016.txt`); Nathan Long's Magic Judges forum profile reads "Forum Moderator / Level 2 Judge
(Judge Foundry), Judge" (`mj-post-131920.html`), and Wizards' release notes name him, Carsten
Haese and Eli Shiffrin among their compilers or contributors (e.g. Commander 2018: "Compiled by
Eli Shiffrin, with contributions from Laurie Cheers, Carsten Haese, Nathan Long, and Thijs van
Ommen.", `c18-release-notes.txt`).

Anything that is our own reasoning rather than a source is marked **Reasoning (ours)**.

**The current Comprehensive Rules.** Wizards' rules page (https://magic.wizards.com/en/rules,
saved `wizards-rules-page.html`) now links "MagicCompRules 20261106.txt" (saved
`MagicCompRules-20261106.txt`: "These rules are effective as of November 6, 2026."), newer than
the repository's `data/comprehensive-rules.txt` (effective 2026-09-25). Compared rule by rule,
every rule quoted below has the same words in both, but the newer CR inserts "713. Meld Cards",
so 713–733 become 714–734: the monarch is 726 and the initiative 727 there (the register's 725.2
and 726.2 are the repository CR's numbers). Its new 109.4b reads: "A triggered ability that has
triggered but is waiting to be placed on the stack is controlled by the player who controlled its
source at the time it triggered, unless it’s a delayed triggered ability. ..." None of the changes
bears on Q1–Q10. Rule quotes below are from `data/comprehensive-rules.txt`.

---

## Q1 (most important). "That player" of a trigger after its creatures change control

### Sources found

**Phyrexia: All Will Be One Commander (Norn's Decree).** The Commander cards are in the main
set's release notes, under "Commander Card-Specific Notes": "Phyrexia: All Will Be One Release
Notes", Jess Dunks, Wizards of the Coast, page dated "Jan 27, 2023", "Document last modified
January 23, 2023", https://magic.wizards.com/en/news/feature/phyrexia-all-will-be-one-release-notes
(saved `one-release-notes.html`). Norn's Decree's entry is its text and one note:
- "Whenever one or more creatures an opponent controls deal combat damage to you, that opponent gets a poison counter."
- "A player is poisoned if that player has one or more poison counters."

Nothing on a control change. Gatherer has the same single ruling, "(2/4/2023) A player is
poisoned if that player has one or more poison counters."
(https://gatherer.wizards.com/ONC/en-us/9/norns-decree, saved `gatherer-norns-decree-onc9.html`).

**Commander Legends: Battle for Baldur's Gate (the initiative).** "Commander Legends: Battle for
Baldur's Gate Release Notes", "Compiled by Jess Dunks", page dated "Jun 1, 2022", "Document last
modified April 28, 2022",
https://magic.wizards.com/en/articles/archive/feature/commander-legends-battle-baldurs-gate-release-notes-2022-06-01
(saved `clb-release-notes.html`), section "The Initiative":
- "The initiative carries two inherent rules. First, whenever a player takes the initiative, and at the beginning of the upkeep of the player with the initiative, that player ventures into Undercity. Second, whenever one or more creatures a player controls deal combat damage to the player who has the initiative, the first player takes the initiative."
- "In a Two-Headed Giant game, if both players on a team deal combat damage to the player that has the initiative at the same time, the player with the initiative will choose the order of the triggered abilities. Then, as those abilities resolve, one team member takes the initiative (and ventures into Undercity) and then the other team member does the same. The last player to take the initiative keeps it until the initiative changes again."

The set's mechanics article ("Commander Legends: Battle for Baldur's Gate Mechanics", Jess
Dunks, May 18, 2022,
https://magic.wizards.com/en/news/feature/commander-legends-battle-for-baldurs-gate-mechanics,
saved `clb-mechanics.html`): "The second way to take the initiative is to attack the player who
currently has the initiative. If a creature you control deals combat damage to that player, you
take the initiative."

The set's rules-change article ("Comprehensive Rules Changes", Jess Dunks, Jun 1, 2022,
https://magic.wizards.com/en/news/announcements/comprehensive-rules-changes-6-1-22, saved
`cr-changes-2022-06-01.html`) says only, of the new section: "Listed first here for obvious
reasons, this is the new section that defines the initiative designation and everything
associated with it."

The CR's own text (726.2): "... “Whenever one or more creatures a player controls deal combat
damage to the player who has the initiative, the controller of those creatures takes the
initiative,” ..." (727.2 in the CR effective 2026-11-06, same words). None of these says whether
"the controller of those creatures" / "the first player" is fixed as the ability triggers or read
as it resolves.

**Cranial Insertion 2760 and 2763 (Coveted Jewel).** "Commanding Success", published 08/13/2018,
"This Article from: Callum Milne" (Level 2 judge),
https://www.cranial-insertion.com/article/2760 (saved `ci-2760.html`); 2763 is its Spanish
translation ("Comandante exitoso", "Translated by Luis Manuel Encina Diaz",
https://www.cranial-insertion.com/article/2763, saved `ci-2763.html`), with the same content:
- "Q: If we're playing a team game with shared turns, who gets Coveted Jewel if multiple people attack me and aren't blocked at the same time?"
- "A: Well, they'll both get it for at least a moment, and both will draw cards, but you get to choose who gets to keep it. (For now—it's too shiny to stay in one place for long.)"
- "When you declare blockers and leave attackers from multiple opponents unblocked, the Jewel's ability will trigger once for each of them. Since you control the Jewel, which is the source of those triggers, you get to choose in what order you put those abilities on the stack, and therefore the order in which they resolve."
- "When each instance of the ability resolves, that opponent will draw three cards and gain control of the Jewel, and it will untap. Players will have a chance to respond, and once that's all done the next ability will resolve and the next opponent will draw their three cards and gain control of the Jewel, untapping it, and so on and so forth until all the abilities have resolved."

**Magic Judges forum topic 49911.** "Coveted Jewel and creatures not declared attacking", Rules
Q&A, https://apps.magicjudges.org/forum/topic/49911/ (saved `mj-topic-49911.html`). Question by
Eli Meyer (May 2, 2019): "Does this line of text require that the unblocked creature be one that
was declared as an attacker? ..." Answer by Callum Milne ("Forum Moderator"), May 4, 2019: "An
ability that triggers whenever something “attacks and isn't blocked” triggers if no blockers are
declared for that creature, regardless of whether or not the creature was ever actually declared
as an attacking creature. If your opponent attacks you with Parhelion II and it gets blocked but
the Angels do not, the ability will trigger and your opponent will gain control of the Jewel.
(Among its other effects.)" No control change. The forum's search finds no other topic on
Coveted Jewel (saved `mj-search-coveted-jewel.html`, `mj-search-posts-coveted-jewel.html`).

**The closest expert statement: ward's "that player" against Frost Titan's "its controller".**
Cranial Insertion, "Digging Into My Backlog", published 03/03/2025, "This Article from: Nathan
Long", https://www.cranial-insertion.com/article/4365 (saved `ci-4365.html`):
- "Q: Why doesn't Frost Titan use ward? Don't they have the same ability?"
- "A: Ok, this is getting pretty deep into the differences (so I won't blame anyone for not getting this right away), but there is a very small difference between Frost Titan's trigger and ward - and that's who will be paying when the trigger resolves."
- "Ward says "that player pays X", so the player who cast the spell targeting the creature will be the one who pays. However, Frost Titan says "its controller pays {2} [image]", which means that the spell's controller when the trigger resolves will have to pay."
- "When might this matter? If control of the spell changes after the trigger is put on the stack but before it resolves - for example, Perplexing Chimera. If A has a creature with ward and B casts a spell targeting it, but C gains control of B's spell with Perplexing Chimera's trigger before the ward trigger resolves, B will still have to pay for the ward trigger, since B controlled the spell when it was cast. But if A's creature was a Frost Titan instead, when the Frost Titan's trigger resolves, since C controls the spell when Frost Titan's trigger resolves, C would have to pay for Frost Titan's trigger, not B. It's a small difference, but enough of a difference to prevent Frost Titan from using ward."

This is the only source found anywhere that addresses "that player" of an "[object] an opponent
controls" trigger after the object changes control (ward: "Whenever this permanent becomes the
target of a spell or ability an opponent controls, counter that spell or ability unless that
player pays [cost].", CR 702.21a). It is a level-2 judge's answer (and a contributor to Wizards'
release notes), not a Wizards document.

**Object-based "its controller" read as the ability resolves (Cranial Insertion).**
- "Let's Do the Time Warp Again!", 03/13/2017, Carsten Haese, https://www.cranial-insertion.com/article/2242 (saved `ci-2242.html`): "Q: Let's say I enchant my Tumble Magnet with Relic Putrescence, and then I tap it. In response to the poison counter trigger, I donate Tumble Magnet to my opponent with Zedruu the Greathearted. Does my opponent get the poison counter?" "A: Yup! Relic Putrescence's ability checks who controls the enchanted artifact only when the ability resolves. At that time, your opponent controls it, so your opponent gets the poison counter." (Agrees with Relic Putrescence's ruling of 2011-01-01, already in the register.)
- "Make a Myth-Take", 12/07/2015, Callum Milne, https://www.cranial-insertion.com/article/1789 (saved `ci-1789.html`), on Thunderbreak Regent ("Whenever a Dragon you control becomes the target of a spell or ability an opponent controls, this creature deals 3 damage to that player."), with no control change: "Your Regent became the target of an ability controlled by your opponent, which is all the Regent requires in order to trigger. The fact that the ability's source died and the fact that the ability didn't actually accomplish anything are both completely and utterly irrelevant. They targeted it, so they take 3."

**"You" of a triggered ability fixed as it triggered (CR 109.5), for comparison.** "Carry On, My
Wayward Pun", 04/28/2025, Carsten Haese, https://www.cranial-insertion.com/article/4389 (saved
`ci-4389.html`), on Measure of Wickedness: "What your opponent is describing only happens if
Measure of Wickedness were to change controllers in response to its end-step trigger. In that
case, the player who controlled it when the ability triggered still loses 8 life, but they can't
sacrifice Measure of Wickedness because they don't control it at the time the ability resolves."

**Wizards release notes already in the register, now read on magic.wizards.com itself** (they
were read through the ormos mirror before):
- Commander (2018 Edition) Release Notes, Jul 27, 2018, "Document last modified June 14, 2018.", https://magic.wizards.com/en/articles/archive/feature/commander-2018-edition-release-notes-2018-07-27 (saved `c18-release-notes.html`), Coveted Jewel: "If more than one opponent attacks you at the same time, Coveted Jewel's last ability triggers for each of them. You choose which player ends up with Coveted Jewel, but each of them draws three cards and has a chance to activate its mana ability." Gatherer has the same four rulings, dated 7/13/2018 (https://gatherer.wizards.com/C21/en-us/240/coveted-jewel, saved `gatherer-coveted-jewel-c21.html`).
- Commander (2019 Edition) Release Notes, Aug 9, 2019, https://magic.wizards.com/en/articles/archive/feature/commander-2019-edition-release-notes-2019-08-09 (saved `c19-release-notes.html`), Archfiend of Spite: "The player who loses life or permanents is the player who controls the source as Archfiend of Spite's triggered ability resolves. In some rare cases, this may not be the same player who controlled the source as it dealt damage. If the source is no longer in the zone it was in as it dealt damage, the source's last known information is used to determine who controlled it."
- Conspiracy: Take the Crown Release Notes, Aug 22, 2016, "Compiled by Matt Tabak, ...", https://magic.wizards.com/en/news/feature/conspiracy-take-crown-release-notes-2016-08-22 (saved `cn2-release-notes.html`): "If the triggered ability that causes the monarch to draw a card goes on the stack, and a different player becomes the monarch before that ability resolves, the first player will still draw the card."
- Commander Legends Release Notes, Nov 7, 2020, "Compiled by Eli Shiffrin and Matt Tabak, ...", https://magic.wizards.com/en/articles/archive/feature/commander-legends-release-notes-2020-11-06 (saved `cmr-release-notes.html`): "If another player becomes the monarch after Emberwilde Captain's last ability triggers but before it resolves, the ability will resolve as normal. The player who attacked you will be dealt damage, even if that player was the one who became the monarch." and "If combat damage dealt to the monarch causes that player to lose the game, the triggered ability that causes the controller of the attacking creature to become the monarch doesn't resolve. In most cases, the controller of the attacking creature will still become the monarch as it is likely their turn."

**Black Bolt, Inhuman King** (2026; "Whenever Black Bolt becomes the target of a spell or ability
an opponent controls, destroy target nonland permanent that player controls."): "Magic: The
Gathering® | Marvel Super Heroes Release Notes", Eric Levine, Jun 12, 2026,
https://magic.wizards.com/en/news/feature/marvel-super-heroes-release-notes (saved
`msh-release-notes.html`), has one note: "Black Bolt's second ability resolves before the spell
that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the
stack without resolving. The same is true for Black Bolt's last ability." Nothing on control.
Gideon the Oathless (Reality Fracture, 2026; "Whenever a creature an opponent controls enters,
Gideon deals 1 damage to that player.") isn't in the Reality Fracture Release Notes (Jess Dunks,
Eric Levine, Sep 18, 2026, https://magic.wizards.com/en/news/feature/reality-fracture-release-notes,
saved `fra-release-notes.html`).

### Not found (places searched)

No source addresses Norn's Decree, Coveted Jewel, the initiative, Michiko Konda, Truth Seeker,
Blood Seeker, Suture Priest, Edric, Spymaster of Trest or Bident of Thassa after a control change
before the ability resolves. Searched:
- the release notes above, and the Adventures in the Forgotten Realms Release Notes (ward made evergreen; Jul 12, 2021, https://magic.wizards.com/en/articles/archive/feature/adventures-forgotten-realms-release-notes-2021-07-09, saved `afr-release-notes.html`: its ward notes are about several instances and gaining ward in response, e.g. Hall of Storm Giants' "Each of those instances will trigger separately and ask the opponent who controls the spell or ability to pay {3}.");
- Gatherer's rulings for Norn's Decree, Coveted Jewel (above); every ruling in `data/rulings.jsonl.gz` of Norn's Decree, Coveted Jewel, Michiko Konda, Edric, Bident of Thassa, Contested War Zone, Kazuul, Revenge of Ravens, Marchesa's Decree, Nelly Borca, Blood Seeker, Suture Priest, Archaeomancer's Map, Thunderbreak Regent, Ashenmoor Liege, Charismatic Conqueror, Nightshade Harvester, Black Bolt, Frost Titan, Perplexing Chimera, Commandeer, Bronze Guardian and Hall of Storm Giants, and every ruling with "initiative" or with "monarch" and "control" (re-read for this item; ruling-research-2 read the other cards of the family);
- Cranial Insertion's archive search (https://www.cranial-insertion.com/archive?word=..., exact phrases; results are capped at about 52 articles, so phrases with 52 hits were not searched exhaustively): "Coveted Jewel", "Norn's", "Decree", "Michiko Konda", "Edric, Spymaster of Trest", "Bident of Thassa", "Contested War Zone", "Blood Seeker", "Suture Priest", "Commandeer", "Aethersnatch", "Perplexing Chimera", "Frost Titan", "Bonecrusher Giant", "Spelljack", "Emberwilde Captain", "Archfiend of Spite", "Tainted Aether", "Tahngarth, First Mate", "Kazuul", "Revenge of Ravens", "Undercity", "initiative" (capped), "becomes the monarch", "take the initiative", "ward", "Charismatic Conqueror", "Trespasser's Curse", "Karmic Justice", "Thunderbreak Regent", "Ashenmoor Liege", "Massacre Wurm", "Scalelord", "Leyline of Combustion", "Polluted Bonds", "Tunnel Ignus", "Archaeomancer", "who loses life", "who draws the cards", "who pays", "who has to sacrifice", "new controller", "changes control", "who controlled it when", "controlled it when it triggered", "when the ability triggered", "at the time it triggered", "draws three cards", "poison counter", "aren't blocked", "deal combat damage to you", "a creature an opponent controls", "spell or ability an opponent controls" (every hit's questions read; saved `ci-search/*.html` and `ci-<id>.html`);
- the Magic Judges forum search (https://apps.magicjudges.org/forum/search/, topic titles and post texts): "Coveted Jewel", "Jewel", "Norn's Decree", "Decree", "Michiko Konda", "Konda", "Edric", "Bident of Thassa", "Contested War Zone", "Blood Seeker", "Suture Priest", "Frost Titan", "Perplexing Chimera", "Kazuul", "Archfiend of Spite", "Massacre Wurm", "Tainted Aether", "Revenge of Ravens", "Marchesa's Decree", "Emberwilde", "monarch", "initiative", "ward", "that player", "that opponent" (saved `mj-s-*.html`, `mj-search-*.html`); its Rules Q&A forum has had few topics since 2020 (its index, https://apps.magicjudges.org/forum/73/, saved `mj-forum-73.html`, lists about ten from 2020 to August 2026, none on these questions), so ward, the initiative and Norn's Decree are almost all after its active years;
- web searches restricted to apps.magicjudges.org, blogs.magicjudges.org, www.cranial-insertion.com and magic.wizards.com (summaries only, used to find URLs).

### Reasoning (ours)

- The Cranial Insertion answer draws exactly the engine's line: a player-referring "that player" whose antecedent is the trigger condition's "a spell or ability an opponent controls" is the player who controlled the spell as the ability triggered ("since B controlled the spell when it was cast"), while an object-based "its controller" is the spell's controller as the ability resolves. Norn's Decree's "one or more creatures an opponent controls deal combat damage to you, that opponent" and Coveted Jewel's "one or more creatures an opponent controls attack you and aren't blocked, that player" have the same structure (an anaphor back to the condition's "an opponent"), so the answer supports F for them by analogy; it doesn't name them.
- Cranial Insertion 2760 speaks of each Jewel instance as belonging to one opponent ("When each instance of the ability resolves, that opponent will draw three cards"), and the CLB release notes' Two-Headed Giant note has each initiative instance make "one team member" take the initiative, then "the other": each instance is tied to the player it triggered for. Both fit F and neither involves a control change.
- The initiative is still worded both ways by Wizards: the CR's object-based "the controller of those creatures", the release notes' "the first player" and the designation card's "that player". Nothing found decides it.

### Verdict against the register

Register entry `that-opponent-fixed-as-it-triggers` (provisional, call 23):
- **ward's "that player" fixed as it triggers, and the object-based "its controller" / "that spell's controller" read as it resolves (Frost Titan): confirmed** by the Cranial Insertion answer above (a level-2 judge, not a Wizards document); Relic Putrescence's "its controller" read on resolution confirmed by Cranial Insertion 2242 (and its own ruling).
- **Norn's Decree, Coveted Jewel and the other "[creatures / a permanent] an opponent controls ..., that player / that opponent" triggers: still open.** No source addresses them after a control change; the only explicit expert statement on the construction (ward) reads it as F, which strengthens the entry's reading.
- **The initiative (kept fixed, an open question in the entry): still open.**
- No source contradicts the engine.

---

## Q2. Swift Reckoning and spell mastery from the graveyard

### Sources found

**Magic Judges forum post 131920.** Topic 20232, "Swift Reckoning + Snapcaster Mage", Rules Q&A
(https://apps.magicjudges.org/forum/post/131920/ redirects to
https://apps.magicjudges.org/forum/topic/20232/?page=1#post-131920; saved `mj-post-131920.html`).
Post 131920 is the answer.
- Question, Jasper König ("Uncertified, Judge", German-speaking countries), Aug. 3, 2015: "I have exactly two instants/sorceries in my graveyard, including one Swift Reckoning. Can I use Snapcaster Mage's triggered ability to flashback Swift Reckoning? I was under the assumption that the legality of casting a spell is checked exactly before actually casting the spell, where, in this case, Spell Mastery still is “active”. But 601.2e talks about checking the legality exactly before determining the cost of the spell, at that point, Swift Reckoning would already be on the stack, making Spell Mastery “inactive”."
- Answer (post 131920), Nathan Long ("Forum Moderator", "Level 2 Judge (Judge Foundry), Judge", signed "Wizards.Com Boards NetRep"), Aug. 4, 2015: "You can only flash it back when you could cast a sorcery. The first thing we do when we announce the spell is move the spell from the zone we're casting it from to the stack (601.2a). Later on, during 601.2e, we check to see if you're able to cast the spell. If you need to cast it as though it had flash, this is the point when it checks to see if you have enough instants and sorceries in your graveyard. At this point, you just have the single instant or sorcery card in your graveyard, so you're not allowed to cast and we rewind to just before you started announcing the spell."

**The 2022 rule and its note.** "Comprehensive Rules Changes", Jess Dunks, Apr 20, 2022
(Streets of New Capenna), https://magic.wizards.com/en/news/announcements/comprehensive-rules-changes-4-20-22
(saved `cr-changes-2022-04-20.html`): "601.4a / This new rule explains what happens in the very
rare case that a spell loses flash during the spell-casting process. If that happens, the player
may finish casting the spell as though it had flash." Today's 601.6a: "Once a player has begun
casting a spell that had flash because certain conditions were met or that could be cast as
though it had flash because certain conditions were met (see 601.3d), they may continue to cast
that spell as though it had flash even if those conditions stop being met."

"Streets of New Capenna Release Notes", "Compiled by Jess Dunks", Apr 20, 2022, "Document last
modified March 31, 2022", https://magic.wizards.com/en/news/feature/streets-new-capenna-release-notes-2022-04-20
(saved `snc-release-notes.html`), Graveyard Shift: "If an effect allows you to cast Graveyard
Shift from your graveyard, you may cast it any time you could cast an instant if you have five or
more mana values among cards in your graveyard immediately before you begin to cast it, even if
moving it to the stack would reduce that number to four."

**The rulings' release notes, read on magic.wizards.com.**
- "Magic Origins Release Notes", Jul 8, 2015, https://magic.wizards.com/en/news/feature/magic-origins-release-notes-2015-07-08 (saved `ori-release-notes.html`), under "New Ability Word: Spell mastery": "Check to see if there are two or more instant and/or sorcery cards in your graveyard as the spell resolves to determine whether the spell mastery ability applies. The spell itself won't count because it's still on the stack as you make this check." and "Some spell mastery abilities cause additional effects to happen, while others replace all or part of the spell's normal effect. See the "Card-Specific Notes" section for more information on specific cards with spell mastery abilities."; under Swift Reckoning: "The number of instant and/or sorcery cards in your graveyard matters only as you begin to cast Swift Reckoning. Once it's cast, that number doesn't matter and will have no effect on Swift Reckoning resolving."
- "Ultimate Masters Release Notes", Nov 29, 2018, "Compiled by Eli Shiffrin, with contributions from Laurie Cheers, Carsten Haese, Nathan Long, and Thijs van Ommen.", "Document last modified September 18, 2018.", https://magic.wizards.com/en/news/feature/ultimate-masters-release-notes-2018-11-29 (saved `uma-release-notes.html`), Swift Reckoning: the same "matters only as you begin" note, then "If an effect allows you to cast Swift Reckoning from your graveyard, you need two other instant and/or sorcery cards in your graveyard to cast it as though it had flash."

### Not found (places searched)

No judge or Wizards statement after 2022-04-29 on a spell mastery card cast from a graveyard
counting itself. Searched: Cranial Insertion ("Swift Reckoning": no hits; "spell mastery": 8
articles, 2015–2018, read; "Graveyard Shift": none; "as though it had flash": 16 articles, read;
"601.4a", "601.5a", "601.6a": none); the Magic Judges forum (the topic above; its Rules Q&A forum
has had about ten topics since 2020, none on this); Wizards' rules-change articles of 2022 (above).

### Reasoning (ours)

Nathan Long's 2015 answer applies the check of its time after the move to the stack (then
601.2e and the second sentence of 601.3), the same reasoning as the 2018 ruling; it predates
601.4a (2022-04-29). It confirms the discrepancy entry's account of why the ruling was right when
written.

### Verdict against the register

- `flash-permission-conditions-judged-as-casting-begins` (decided) and the outdated entry "Swift Reckoning: "you need two other instant and/or sorcery cards" ..." in `docs/RULING_DISCREPANCIES.md`: **confirmed** (the forum answer is the 2015 reading; nothing after 2022 reads the rule the other way). The register's description of the post ("dated 2015 by a search index; the site was outside the review's allowed network") is now verified: Nathan Long, Aug. 4, 2015 (the entry's Why now quotes it).

---

## Q3. The Foundations Update Bulletin (2024-11-14) on CR 611.2b

### Source found

"Magic: The Gathering Foundations Update Bulletin", Announcements, "Nov 14, 2024", Jess Dunks and
Eric Levine (Wizards of the Coast),
https://magic.wizards.com/en/news/announcements/foundations-update-bulletin (saved
`fdn-update-bulletin.html`), under "Magic: The Gathering Foundations Comprehensive Rules Changes",
"Other Rules Changes". The section in full:

> 611.2b
>
> This is the rule that tells you how to handle effects that have a duration of "for as long as …" Normally, these cases are pretty straightforward—if the duration never starts, or if it ends before the effect would be applied, the effect isn't applied. For example, if you cast Sower of Temptation and it gets destroyed in response to its triggered ability, you won't gain control of the target creature as that ability resolves. The only change here is that this rule now explicitly handles the rare case where the duration for a continuous effect starts during the resolution of the spell or ability which created that effect. As an example, you put the first ability of Xolatoyac, the Smiling Flood onto the stack targeting a land that already has a flood counter on it. If another effect removes the existing flood counter before Xolatoyac's ability resolves, the previous wording of 611.2b could have been read to mean that Xolatoyac's ability doesn't make that land an Island on resolution even though the ability itself puts a flood counter on the land. The new wording makes it clearer that the land does indeed become an Island and that similar effects work as players would expect them to.

### Verdict against the register

`for-as-long-as-checked-continually-from-activation` (decided) and
`duration-the-resolution-sets-up-is-checked-as-the-effect-begins` (provisional) quoted the
bulletin from search summaries, "unverified": **confirmed**, word for word, authors and date
included. The full section adds the Sower of Temptation example. The Sources fields now say it was
read on magic.wizards.com.

---

## Q4. "For as long as" durations while the ability waits on the stack

### Sources found

None addresses (a), (b) or (c) as asked. The closest:

**(a) Phasing.** All about an effect that already exists, not an ability waiting on the stack:
- Cranial Insertion, "The Eyes of March", 03/15/2010, Eli Shiffrin, https://www.cranial-insertion.com/article/271 (saved `ci-271.html`): "Q: I cast Sower of Temptation and target my opponents' Tarmogoyf to gain control of it. Next turn, my opponent uses his Vodalian Illusionist to phase out my faerie. What happens to the Tarmogoyf? What happens when the Sower phases in?" "A: As soon as the Sower phases out, the "for as long as" duration of its gain-control ability expires since, as far as the effect's concerned, it's no longer on the battlefield. The goyf goes back to your opponent. When the Sower phases in, all excited to peek at its goyf-in-a-cage, it'll be disappointed to see it gone: the effect does not somehow resume once it's ended. Since the Sower isn't entering the battlefield, it won't trigger, either."
- Cranial Insertion, "Partly Cloudy With a Chance of Meaty Oars", 08/09/2021, Carsten Haese, https://www.cranial-insertion.com/article/3606 (saved `ci-3606.html`): "A: No. Nihiloor's control-changing effect only lasts as long as you control Nihiloor. While you technically still control Nihiloor while it's phased out, the game can't see it, so the effect ends when Nihiloor phases out. ..."
- Cranial Insertion, "Lucky Number Thirteen", 12/09/2019, Carsten Haese, https://www.cranial-insertion.com/article/3177 (saved `ci-3177.html`): "Q: If I exile something with Banishing Light and then Teferi's Protection causes Banishing Light to phase out, does the exiled thing get returned to the battlefield?" "A: No. Unlike "for as long as" effects like the effect on Sower of Temptation, which ends when Sower of Temptation phases out, the "exile until" effect of Banishing Light actually ties the return event to the event of Banishing Light leaving the battlefield. Phasing out doesn't cause Banishing Light to leave the battlefield, so the exiled thing remains exiled."
- Cranial Insertion, "Hendecaphilia", 03/28/2016, Callum Milne, https://www.cranial-insertion.com/article/1903 (saved `ci-1903.html`), on Intet phased out: "That means the "for as long as Intet remains on the battlefield" condition of her ability no longer applies, and you no longer have permission to cast the cards she exiled."
- Phyrexia: All Will Be One Release Notes (above, `one-release-notes.html`), Clever Concealment: "Any continuous effects with a "for as long as" duration, such as that of Mind Flayer, ignore phased-out objects. If ignoring those objects causes the effect's conditions to no longer be met, the duration will expire."

**(b) Copies.** Cranial Insertion, "When All You Have Is a Hammer", 11/08/2010, Eli Shiffrin,
https://www.cranial-insertion.com/article/305 (saved `ci-305.html`): "Q: If you used Rings of
Brighthearth to copy Rust Tick's ability, could you keep two artifacts tapped down?" "A: Sure,
that works! The source of the copied Rust Tick ability is the same as the source of the original
ability, so as long as that one Rust Tick stays tapped, the artifact targeted by its original
ability and the artifact targeted by the copy will both remain tapped." (No change to the source
between the original and the copy.)

**(c) Conditions that fail and hold again.** Only about the source itself, which the decided
entry covers:
- Cranial Insertion, "Jailbreak!", 01/20/2020, Andrew Villarrubia (Level 2 judge), https://www.cranial-insertion.com/article/3209 (saved `ci-3209.html`): "Q: If I activate Entrancing Lyre, and in response untap it and activate it again targeting a different creature, will both creatures remain tapped?" "A: Nope. Both creatures will get tapped right now, but the creature targeted by the first activation will untap like normal. The Lyre became untapped before the first activation resolved, which is what the ability is watching for. It doesn't matter that it became tapped again almost immediately."
- Cranial Insertion, "Let's Do the Time Warp Again!", 03/13/2017, Carsten Haese (`ci-2242.html`), on Vedalken Shackles, where the power condition is in the target, not the duration: "The ability checks the creature's power only at two points in time, namely when you activate the ability and when the ability resolves. Provided that the creature's power is less than or equal to the number of Islands you control at that time, the ability resolves and creates a control-changing effect with a duration of "as long as Vedalken Shackles remains tapped." ..."

### Not found (places searched)

Cranial Insertion: "Strionic Resonator" (31 articles), "Lithoform Engine" (9), "Rings of
Brighthearth" (21), "Illusionist's Bracers" (9), "Kurkesh" (1), "Teferi's Protection" (30),
"phases out" (45), "phased out" (52, capped), "for as long as you control" (21), "remains tapped"
(12), "remain tapped" (14), "Rubinia" (4), "Merieke" (16), "Sower of Temptation" (32), "Callous
Oppressor" (6), "Rust Tick" (7), "Preacher" (2), "Vedalken Shackles" (30), "Rootwater Matriarch"
(2), "Assimilation Aegis" (1); no hits for "Old Man of the Sea", "Old Man", "Eriette", "Seasinger",
"Willow Satyr", "Garland", "Blade of Shared Souls", "Clever Concealment", "Xolatoyac", "Braided
Net" (every hit's questions scanned for copies, phasing, untapping or control changes while an
ability waits). Magic Judges forum: "Old Man of the Sea" (none), "Seasinger", "Strionic
Resonator" (6 topics), "Sower of Temptation", "phase" (none on this). Rulings in the data of Old
Man of the Sea, Eriette, the Beguiler, Rootwater Matriarch, Garland, Strionic Resonator, Lithoform
Engine, Rings of Brighthearth, Illusionist's Bracers, Battlemage's Bracers, Kurkesh, Adric,
Teferi's Protection, Clever Concealment, Spectral Adversary, Sower of Temptation and Seasinger
(re-read): none on these cases.

### Verdict against the register

- (a) `phasing-out-ends-a-waiting-for-as-long-as-duration` (provisional): **still open**; the sources on existing effects ("the effect does not somehow resume once it's ended"; phasing doesn't end an "exile until ... leaves the battlefield" effect) agree with both halves of the entry.
- (b) `copied-ability-duration-starts-as-the-copy-is-created` (provisional): **still open**.
- (c) `duration-about-another-object-ends-while-the-ability-waits` (provisional): **still open**. (The Entrancing Lyre answer agrees with the decided entry `for-as-long-as-checked-continually-from-activation`.)

---

## Q5. Tempting offers with Doubling Season / Hardened Scales / Xorn

### Sources found

**Magic Judges forum topic 14129**, "Temp With Glory + Hardened Scales", Rules Q&A,
https://apps.magicjudges.org/forum/topic/14129/ (saved `mj-topic-14129-p1.html`):
- Question, Dan Fox ("Uncertified, Judge", USA - Northeast), Nov. 18, 2014: "Tempt With Glory + Hardened Scales / I'm in a four player game and I cast Tempt With Glory. Each opponent decides to add counters to their creatures. Do I add 3 counters one time, or 1 counter three times?"
- Answer, Callum Milne ("Forum Moderator"; a Level 2 judge per his Cranial Insertion bio), Dec. 4, 2014: "Tempt with Glory is trying to add three counters to your creatures once, not one counter three times. If all your opponents decide to add counters to their creatures, your creatures will each end up with a total of six counters on them–two from the initial counter placement, before your opponents' decisions, and four from the second placement after they decide."

**Cranial Insertion**, two answers by Callum Milne:
- "Food on the Brain", 07/20/2015, https://www.cranial-insertion.com/article/1663 (saved `ci-1663.html`): "Q: My opponent casts Tempt with Discovery in a multiplayer game, and I use Savage Summoning to bring out Ob Nixilis, Unshackled in response. If I choose to search, how much life will my opponent lose? And what if some other opponents also search?" "A: Your opponent's going to be very unhappy, because if anyone else decides to fetch up a land as well, they're going to be stuck losing a whopping 20 life, since the first search, the one they perform on their own, is separate from the search they perform based on what their opponents decided to do. And neither one's optional for them—they have to search the first time, and if someone else searches they're forced to perform the second one as well."
- "4, 3, 2, 1!", 11/27/2017, https://www.cranial-insertion.com/article/2515 (saved `ci-2515.html`): "Q: How do cards with Tempting Offer, like Tempt with Discovery, work in Two-Headed Giant?" "A: Not as great as you might expect, since only you and your opponents get to benefit from the offer, not your teammate. First you perform the action, and then each of your two opponents gets to choose whether or not to take the offer. If either of them does, you repeat the action—twice if both of them did. ..."

**Wizards.** "Commander (2013 Edition) Release Notes", Oct 22, 2013, "Compiled by Matt Tabak, with
contributions from Laurie Cheers, Carsten Haese, Eli Shiffrin, Zoe Stephenson, and Thijs van
Ommen", "Document last modified September 26, 2013",
https://magic.wizards.com/en/news/feature/commander-2013-edition-release-notes-2013-10-22 (saved
`c13-release-notes.html`), "New Ability Word: Tempting Offer": "Tempting offer is an ability word
that appears in italics on a cycle of sorceries. These cards have a beneficial effect, then give
opponents the option to also enjoy that effect if they allow you to repeat it. (An ability word
has no rules meaning.)", and under Tempt with Glory: "After each opponent has decided, the effect
happens simultaneously for each one who accepted the offer. Then, the effect happens again for you
a number of times equal to the number of opponents who accepted." (The data's rulings: the same
sentence on the 2013 cycle, 2013-10-17, and on Tempt with Bunnies, 2024-07-26.)

### Not found (places searched)

No source on Xorn, Peregrin Took or Doubling Season with a tempting offer or with a "for each
opponent, you create a token" instruction. Searched: Cranial Insertion ("Tempt with": 5 articles;
"tempting offer": 1; "Xorn": 3, all about one creation, e.g. Xorn with Academy Manufactor or
Anointed Procession; "Peregrin Took": 1, with Chatterfang; "Hardened Scales": 25; "Doubling
Season": 52, capped; "Master Biomancer", "Corpsejack Menace"); the Magic Judges forum ("Tempt",
"tempting offer", "Tempt with Vengeance", "Tempt with Discovery", "Tempt with Immortality", "Tempt
with Reflections", "Xorn", "Hardened Scales", "Doubling Season", "Anointed Procession",
"Parallel Lives"); the rulings of the tempting offers, Tempting Contract, Xorn, Peregrin Took,
Academy Manufactor, Chatterfang, Jolene, Hardened Scales, Doubling Season, Anointed Procession,
Parallel Lives, Endless Ranks of HYDRA, Wurmquake and Brass's Bounty in the data.

### Reasoning (ours)

The 2014 answer is the only source on the question as asked, and it answers it: the "for each
opponent who does" part is one placement of N counters (Hardened Scales adds one counter to it
once). The 2015 answer treats Tempt with Discovery's "for each opponent who searches" part the
same way (one search, however many opponents searched). The 2017 answer's "you repeat the
action—twice if both of them did" and the release notes' "the effect happens again for you a
number of times" are worded as repetitions but say nothing about replacement effects; the 2014
answer was written with the same release notes in force.

### Verdict against the register

`for-each-tokens-counters-life-one-event` (provisional): **confirmed by the only on-point judge
answer** (Tempt with Glory with Hardened Scales: 1 + 1, then N + 1 counters, exactly the entry's
example), with a second answer in line; no Wizards source decides it, and Wizards' wording ("a
number of times") is still the reading against.

---

## Q6. Ixidron and other permanents' "enters with" effects

### Sources found

None on a replacement effect's source losing its abilities (turned face down, or Humility) while
simultaneously entering permanents' replacement effects are applied. Related answers:
- Cranial Insertion, "Mom, Volrath Won't Stop Copying Me!", 09/09/2019, Andrew Villarrubia, https://www.cranial-insertion.com/article/3105 (saved `ci-3105.html`), Command the Dreadhorde returning Clone and Ixidron: "So, first, the easy one. Clone will not be face down, because it isn't on the battlefield yet when Ixidron's replacement effect applies. For the same reason, Clone can't enter as a copy of Ixidron."
- Cranial Insertion, "Happy Spooktober!", 10/04/2021, Carsten Haese, https://www.cranial-insertion.com/article/3643 (saved `ci-3643.html`): "No, they'll be face-up. Patriarch's Bidding returns all creatures at the same time, and Ixidron only turns creatures face down that are already on the battlefield as it's entering the battlefield. Your new Elves aren't on the battlefield yet, so they're safe from Ixidron's effect."
- Cranial Insertion, "A Tale as Old as Time", 10/21/2019, Nathan Long, https://www.cranial-insertion.com/article/3141 (saved `ci-3141.html`): "Rhythm of the Wild has to be on the battlefield when the creature enters for it to gain haste, since riot is a replacement effect that alters how the creature enters the battlefield. It won't give creatures entering the battlefield at the same time as the Rhythm riot in time for riot to work, ..."
- Cranial Insertion, "The Fix Is In", 10/02/2017, Callum Milne, https://www.cranial-insertion.com/article/2457 (saved `ci-2457.html`), on the entering creature's own abilities: "Now, when a creature is entering the battlefield, you take effects like the Spendor and Humility into account when determining which abilities to apply as they're entering the battlefield. Since Clone's copying ability, for example, will be wiped out by the Humility/Splendor, you won't be able to copy anything as it enters the battlefield. ..."
- Magic Judges forum topic 42824, "Gaining and losing abilities", answer by Callum Milne, April 23, 2018, https://apps.magicjudges.org/forum/topic/42824/ (saved `mj-topic-42824-p1.html`): "Effects that remove abilities (or make copies) that then expire (or turn things face-down that later turn face-up again) don't break linked abilities, nor do they reset activation restrictions. When the effect expires (or the creature turns face-up), the creature isn't gaining a new instance of its abilities–it's just having the same old instance of the ability revealed once more."

### Not found (places searched)

Cranial Insertion: "Ixidron" (40 articles), "Grumgully" (1: Grumgully with persist), "Humility"
(52, capped), "enters with an additional" (1), "enter the battlefield with an additional" (3),
"Rhythm of the Wild" (7), "Master Biomancer" (13), "Corpsejack Menace" (13); the Magic Judges
forum: "Ixidron" (2 topics), "Humility" (20), "face down", "enters the battlefield with",
"Corpsejack", "Master Biomancer", "Torpor Orb"; the rulings of Ixidron, Grumgully, the Generous,
Rhythm of the Wild, Thief of Blood, Master Biomancer and Humility in the data (Grumgully: "Any
other non-Human creatures that enter the battlefield at the same time as Grumgully won't get a
+1/+1 counter."; Ixidron: "If Ixidron and another creature are entering at the same time, the
other creature enters face up."; Rhythm of the Wild 2019-05-03, already in the register).

### Verdict against the register

`simultaneous-entries-ordered-by-controller` (decided, policy): the point decision-audit-1 left
open for the user (a source that stays on the battlefield but loses its abilities, Ixidron turning
Grumgully face down, and the engine's add-back of effects once a permanent then on the battlefield
has left it): **still open**; no source.

---

## Q7. Seraph's ruling

### Sources found

**Gatherer, Seraph** (https://gatherer.wizards.com/Pages/Card/Details.aspx?multiverseid=159245,
Masters Edition, saved `gatherer-seraph-me1-mid159245.html`; and
https://gatherer.wizards.com/Pages/Card/Details.aspx?printed=true&multiverseid=2711, Ice Age,
saved `gatherer-seraph-ice-mid2711-printed.html`). The pages' data carries each printing's printed
text ("instanceText"); none involves exile or removal from the game:
- Ice Age (IA): "Flying\nAt the end of a turn in which any creature is damaged by Seraph and put into the graveyard, put that creature directly into play under your control as though it were just summoned. If you lose control of Seraph or if Seraph leaves play, bury the creature."
- Fifth Edition (5E): "Flying\nWhenever any creature Seraph damaged this turn is put into any graveyard, put that creature into play under your control at end of turn. Bury the creature if you lose control of Seraph."
- Masters Edition (MED, the Magic Online set the Oracle update of September 7, 2007 was made for; see the bulletin below): "Flying\nWhenever a creature dealt damage by Seraph this turn is put into a graveyard, put that card into play under your control at end of turn. Sacrifice the creature when you lose control of Seraph."

Gatherer's rulings for Seraph: "(10/4/2004) You do not get the creature back if the creature is
not still in the graveyard at the end of the turn." and, one of four dated "(9/16/2007)": "If the
other creature is a token, it will cease to exist after being exiled. It won't be returned to the
battlefield."

**Gatherer, Tawnos's Coffin** (https://gatherer.wizards.com/Pages/Card/Details.aspx?name=Tawnos%27s+Coffin,
saved `gatherer-tawnoss-coffin-name.html`): "(9/16/2007) If the targeted creature is a token, it
will cease to exist after being exiled. Any Auras that were attached to it will remain exiled
forever." Its Masters Edition printed text: "{3}, {T}: Remove target creature and all Auras
attached to it from the game. ..."

**Masters Edition Update Bulletin**, Mark L. Gottlieb, "Posted in Feature on August 29, 2007"
(https://magic.wizards.com/en/articles/archive/feature/masters-edition-update-bulletin-2007-08-29:
404 on magic.wizards.com today, saved `me1-update-bulletin.html`; read in the ormos mirror,
https://raw.githubusercontent.com/maxmakesmagic/ormos/main/archive/en/articles/archive/feature/2007/08/masters-edition-update-bulletin-2007-08-29.md,
made from a Wayback Machine capture of 2017-03-28, saved
`ormos-masters-edition-update-bulletin-2007-08-29.md`): "As we announced on the web site, there
will be a special Oracle update in advance of the release of the *Masters Edition* set on **Magic
Online**. This update will take effect on Friday, September 7. ..." and "The *Ice Age*
[Seraph](...) was functionally different from the *Fifth Edition* [Seraph](...), and we're
sticking with the most recent printing because that's our general policy." Seraph appears nowhere
else in it; Tawnos's Coffin is in it ("phasing is leaving these cards in favor of a wording that
goes back to their original (long, clunky) wordings").

### Not found

Gatherer's Oracle-text history for Seraph between 1997 and 2007 (new Gatherer shows no history;
the Wayback Machine, which has old Gatherer captures, refuses connections from here).

### Reasoning (ours)

The ruling is dated 2007-09-16, nine days after the Masters Edition Oracle update took effect, and
the Masters Edition card (a Magic Online card, whose text we take to be that update's wording; the
bulletin promises "some minor, nonfunctional changes to make *Masters Edition* cards simpler or
bring them in line with more modern templates") puts the card onto the battlefield from the
graveyard and exiles nothing. So when the ruling was written, Seraph's text exiled
nothing, and no later text has (the current Oracle text: "Whenever a creature dealt damage by this
creature this turn dies, put that card onto the battlefield under your control at the beginning of
the next end step. ..."). Tawnos's Coffin, whose Masters Edition text removes the creature from the
game, has the same sentence on the same day, where it fits. Seraph's ruling is Tawnos's Coffin's
template with the second sentence changed: a template error. Its conclusion holds (a token that
dies ceases to exist in the graveyard, CR 704.5d, and isn't returned).

### Verdict against the register

The "Not yet investigated" note on Seraph in `docs/RULING_DISCREPANCIES.md` ("no Oracle wording of
Seraph that exiled the creature, and so no dated change, was found ... for the coordinator to
decide"): **settled**: no Oracle wording of Seraph ever exiled the creature; the ruling's "after
being exiled" is a template error (copied from Tawnos's Coffin's ruling of the same day). The note
is now a template-error entry; the engine's behavior (the token ceases to exist and isn't
returned; the ruling stays cited for that) is unchanged.

---

## Q8. Attack taxes and mana that can only pay {X} costs

### Sources found

None on X-only mana paying another tax in the same total cost to attack. Related:
- Cranial Insertion, "Legally Blocked", 02/20/2006, Thijs van Ommen, https://www.cranial-insertion.com/article/58 (saved `ci-58.html`), on Ghostly Prison: "The process for dealing with such costs is described in rules 308.2d-f for attacking and 309.2b-d for blocking. In both cases, after you have come up with an otherwise legal set of attacking or blocking assignments, you first determine the total cost you have to pay, then play mana abilities if you need mana, and finally pay the cost."
- Cranial Insertion, "Nyx picks from the sticks", 05/05/2014, James Bennett ("a Level 3 judge" per his bio), https://www.cranial-insertion.com/article/1281 (saved `ci-1281.html`): "Q: How does Oppressive Rays work with Sphere of Safety? Do those costs add up, or does he only have to pay one of them?" "A: Now that is oppressive! In order to attack with a creature, its controller needs to pay all applicable costs, and one payment only satisfies one cost. So he'll have to pay {3} [image] to satisfy Oppressive Rays, and then at least another {2} [image] (since you control at least 2 enchantments in this situation) for Sphere of Safety."
- Cranial Insertion, "CI vs Megashark vs Giant Octopus", 06/11/2012, Eli Shiffrin, https://www.cranial-insertion.com/article/485 (saved `ci-485.html`): "Q: If I cast Comet Storm for X = 1, can I use Rosheen Meanderer's mana to kick it a bunch?" "A: You sure can! Comet Storm's cost includes an {X} [image], and that's all Rosheen cares about - she doesn't care whether you spend her mana to pay {X} [image], only whether you pay a cost containing {X} [image] with it, and the cost of Comet Storm is {X}{R}{R} [images] plus {1} [image] per kick."
- Cranial Insertion, "Three Months of Halloween", 11/08/2021, Nathan Long, https://www.cranial-insertion.com/article/3664 (saved `ci-3664.html`), Elementalist's Palette and a kicked Thieving Skydiver: "While the total cost of the spell ends up with an X since you're kicking the Skydiver, there is no X in the actual mana cost of the Skydiver, so the Palette doesn't care about it and won't trigger off of the kicked Skydiver."
- Cranial Insertion, "The Stack Still Exists, Even in Combat", 05/10/2010, Eli Shiffrin, https://www.cranial-insertion.com/article/279 (saved `ci-279.html`): "Rosheen's mana can be used to cast any spell with X in the cost, not only to cover the value of X."
- Rulings in the data (Wizards): Propaganda (2004-10-04) "If there is more than one Propaganda on the battlefield, the cost is cumulative." and "Paying this cost is not an instant or any other kind of ability, it is an additional cost on the declaration of the attacker."; Nexos (2022-10-07) "A "cost that contains {X}" may be a spell's total cost, an activated ability's cost, a cost to perform a special action, or a cost you're asked to pay as part of the resolution of a spell or ability (such as Condescend). ..."; Elementalist's Palette (2021-04-16) "... That {X} could be in a spell's total cost, an ability's activation cost, or a cost you're asked to pay as a spell or ability resolves."

### Not found (places searched)

Cranial Insertion: "Sphere of Safety" (5), "Rosheen" (5), "Elementalist's Palette" (1), "Nexos"
(1), "Ghostly Prison" (37; every question read for costs, totals, X or spending restrictions);
the Magic Judges forum: "Sphere of Safety", "Rosheen", "Ghostly Prison" (4 topics); rulings of
Sphere of Safety, Ghostly Prison, Propaganda, Norn's Annex, Collective Restraint, War Tax, Rosheen
Meanderer, Nexos and Elementalist's Palette in the data.

### Reasoning (ours)

The 2006 answer and CR 508.1h describe one total cost; the 2014 answer's "one payment only
satisfies one cost" says only that each tax must be paid (the amounts add up), not that each tax
is a separate cost for a spending restriction. The Rosheen, Nexos and Palette rulings list kinds
of costs ("may be") without mentioning a cost to attack.

### Verdict against the register

`attack-total-cost-contains-x` (provisional): **still open**; no source decides it, and none
contradicts it.

---

## Q9. Siege of Towers' replicate ruling

### Sources found

"Ravnica Remastered Release Notes", "Compiled by Eric Levine", page dated "Dec 13, 2023",
"Document last modified November 20, 2023",
https://magic.wizards.com/en/news/feature/ravnica-remastered-release-notes (saved
`rvr-release-notes.html`), and its English PDF,
https://media.wizards.com/2023/downloads/RVR_Release_Notes_XrN0zlJaFn/EN_MTGRVR_ReleaseNotes_20231120.pdf
(saved `rvr-release-notes-EN.pdf`, text `rvr-release-notes-EN.pdf.txt`). Under Siege of Towers:
- "As the replicate triggered ability resolves, you'll copy Siege of Towers for each time you paid its replicate cost, even if the original spell is no longer on the stack at that time (perhaps because it was countered)."
- "The copies that replicate creates are created on the stack, so they're not "cast." Abilities that trigger when a player casts a spell won't trigger."

Under Shattering Spree the same notes name Shattering Spree.

Gatherer's Siege of Towers page (https://gatherer.wizards.com/RVR/en-us/123/siege-of-towers,
saved `gatherer-siege-of-towers-rvr.html`) has the error itself: "(1/12/2024) As the replicate
triggered ability resolves, you'll copy Shattering Spree for each time you paid its replicate cost,
even if the original spell is no longer on the stack at that time (perhaps because it was
countered)."

### Verdict against the register

The template-error entry "Siege of Towers: "copy Shattering Spree" for "copy Siege of Towers"":
**contradicted in its kind, not in the engine**: Wizards' release notes have the ruling with
Siege of Towers' name, so Gatherer's text differs from its own source: a **copying error** (the
kind decided by the user on Nov 5). The entry is moved under "Copying errors" with the release
notes quoted; the engine (Siege of Towers' replicate copies Siege of Towers) and the exemption stay.
The error is in Gatherer, so a report would go to Wizards.

---

## Q10. Hybrid mana written as two symbols in 2024 rulings

### Sources found

Every Wizards source keeps the hybrid symbol whole:
- "Magic: The Gathering Foundations Release Notes" (with the Foundations Jumpstart cards), "Compiled by Eric Levine", page dated "Nov 1, 2024", "Document last modified August 23, 2024", https://magic.wizards.com/en/news/feature/foundations-release-notes (saved `fdn-release-notes.html`): Taeko, the Patient Avalanche: "You don't choose a target for Taeko's last ability at the time it triggers. Rather, a second "reflexive" ability triggers when you pay {U/B} this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."; Slinza, the Spiked Stampede: "You don't choose a target for Slinza's last ability at the time it triggers. Rather, a second "reflexive" ability triggers when you pay {1}{R/G} this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."; Hurska Sweet-Tooth: "... a second "reflexive" ability triggers when you pay {G/W} this way. ..."
- "Murders at Karlov Manor Release Notes" (with the Ravnica: Clue Edition cards), "Compiled by Eric Levine", Jan 26, 2024, "Document last modified January 25, 2024", https://magic.wizards.com/en/news/feature/murders-at-karlov-manor-release-notes (saved `mkm-release-notes.html`), Syndicate Heavy: "You may pay {W/B} a maximum of one time for each extort triggered ability. You decide whether to pay when the ability resolves."
- "Modern Horizons 3 Release Notes", "Compiled by Eric Levine", May 31, 2024, "Document last modified May 15, 2024", https://magic.wizards.com/en/news/feature/modern-horizons-3-release-notes (saved `mh3-release-notes.html`), Sorin of House Markov: "You may pay {W/B} a maximum of one time for each extort triggered ability. You decide whether to pay when the ability resolves."
- "Outlaws of Thunder Junction Release Notes", "Compiled by Eric Levine", Apr 5, 2024, "Document last modified March 7, 2024", https://magic.wizards.com/en/news/feature/outlaws-of-thunder-junction-release-notes (saved `otj-release-notes.html`), Kellan, the Kid (the "Not yet investigated" note): "... For example, if you cast the Cease half of Cease // Desist from the Murders at Karlov Manor release, that spell's mana value is 2 because Cease's mana cost is {1}{B/G}."
- Gatherer stores the rulings with the hybrid symbol whole (the pages' data has the text; the page draws each symbol as an image): Taeko "triggers when you pay {U/B} this way" (https://gatherer.wizards.com/J25/en-us/39/taeko-the-patient-avalanche, saved `gatherer-taeko-j25.html`), Slinza "triggers when you pay {1}{R/G} this way" (https://gatherer.wizards.com/J25/en-us/55/slinza-the-spiked-stampede, `gatherer-slinza-j25.html`), Hurska "{G/W}" (`gatherer-hurska-j25.html`), Syndicate Heavy and Sorin of House Markov "You may pay {W/B} a maximum of one time for each extort triggered ability. ..." (https://gatherer.wizards.com/CLU/en-us/47/syndicate-heavy, `gatherer-syndicate-heavy-clu.html`; https://gatherer.wizards.com/MH3/en-us/245/sorin-of-house-markov, `gatherer-sorin-mh3.html`), Kellan "Cease's mana cost is {1}{B/G}" (https://gatherer.wizards.com/OTJ/en-us/213/kellan-the-kid, `gatherer-kellan-the-kid-otj.html`).

The two-symbol texts are only in Scryfall's bulk data (`data/rulings.jsonl.gz`, source "wotc"):
"when you pay {U}{B} this way", "when you pay {1}{R}{G} this way", "You may pay {W}{B} a maximum
of one time ...", "Cease’s mana cost is {1}{B}{G}".

### Reasoning (ours)

Gatherer draws a hybrid symbol as two half-symbol images with a separator (read as text, its
page shows "One blue mana{/}One black mana"). How Scryfall's copy came to drop the slash isn't
known: Hurska Sweet-Tooth's ruling of the same day, drawn the same way, kept its {G/W}. What the
sources settle is that Wizards' own text is right.

### Verdict against the register

The template-error entry "Taeko, the Patient Avalanche, Slinza, the Spiked Stampede, Syndicate
Heavy and Sorin of House Markov: a hybrid mana symbol written as two mana symbols": **contradicted
in its kind, not in the engine**: Wizards' release notes and Gatherer have the hybrid symbols
whole, so the data's text differs from its source: a **copying error**, in Scryfall's copy (a
report would go to Scryfall). The entry is moved under "Copying errors"; the engine and the
exemptions stay. Kellan, the Kid's "{1}{B}{G}" (a note under "Not yet investigated") is the same
copying error; it contradicts nothing in Kellan's own text and stays cited.

---

## For the user

### 1. Call 23 (Q1): "that player" / "that opponent" after the creatures change control (`that-opponent-fixed-as-it-triggers`, provisional)

What the web adds since ruling-research-2:
- One explicit expert answer, on ward, drawing the engine's line. Cranial Insertion, 03/03/2025, Nathan Long (Level 2 judge; a contributor to Wizards' release notes), https://www.cranial-insertion.com/article/4365: "Ward says "that player pays X", so the player who cast the spell targeting the creature will be the one who pays. However, Frost Titan says "its controller pays {2} [image]", which means that the spell's controller when the trigger resolves will have to pay." and "If A has a creature with ward and B casts a spell targeting it, but C gains control of B's spell with Perplexing Chimera's trigger before the ward trigger resolves, B will still have to pay for the ward trigger, since B controlled the spell when it was cast. But if A's creature was a Frost Titan instead, when the Frost Titan's trigger resolves, since C controls the spell when Frost Titan's trigger resolves, C would have to pay for Frost Titan's trigger, not B."
- The object-based reading again: Cranial Insertion, 03/13/2017, Carsten Haese, https://www.cranial-insertion.com/article/2242: "Relic Putrescence's ability checks who controls the enchanted artifact only when the ability resolves. At that time, your opponent controls it, so your opponent gets the poison counter."
- Norn's Decree's release notes (https://magic.wizards.com/en/news/feature/phyrexia-all-will-be-one-release-notes, Jess Dunks, Jan 27, 2023), now read: one note, "A player is poisoned if that player has one or more poison counters."
- Coveted Jewel: Cranial Insertion 2760 (08/13/2018, Callum Milne): "When each instance of the ability resolves, that opponent will draw three cards and gain control of the Jewel, and it will untap." No control change.
- The initiative: CLB release notes (Jess Dunks, Jun 1, 2022): "... whenever one or more creatures a player controls deal combat damage to the player who has the initiative, the first player takes the initiative." and "... as those abilities resolve, one team member takes the initiative (and ventures into Undercity) and then the other team member does the same."; CR 726.2: "... the controller of those creatures takes the initiative". No source on a control change.
- Nothing found contradicts the engine. Cardmarket's "#CMAskTheJudge Episode 66: A Real Jewel" couldn't be read (blocked by the network policy).

**Recommendation:** decide F for the player-referring "that player" / "that opponent" / "they"
whose antecedent is the trigger condition's "an opponent" (or "a player", "enchanted player"),
with the object-based "its controller" read on resolution, as the engine does now. The one expert
statement that addresses the construction after a control change (ward) says exactly this and
gives exactly this reason, and every ruling on the object-based wording agrees; for Norn's Decree
and Coveted Jewel it is an analogy, not a source, so record it as decided by the user on that
basis. Keep the initiative as it is (fixed, matching "the first player" and the designation's
"that player"), or leave that part provisional; no source decides it.

### 2. Q5: tempting offers and "for each" instructions as one event (`for-each-tokens-counters-life-one-event`, provisional)

- Magic Judges forum, Callum Milne (Level 2 judge), Dec. 4, 2014, https://apps.magicjudges.org/forum/topic/14129/: "Tempt with Glory is trying to add three counters to your creatures once, not one counter three times. If all your opponents decide to add counters to their creatures, your creatures will each end up with a total of six counters on them–two from the initial counter placement, before your opponents' decisions, and four from the second placement after they decide."
- Cranial Insertion, Callum Milne, 07/20/2015, https://www.cranial-insertion.com/article/1663: "... since the first search, the one they perform on their own, is separate from the search they perform based on what their opponents decided to do."
- Read the other way: Cranial Insertion, Callum Milne, 11/27/2017, https://www.cranial-insertion.com/article/2515: "If either of them does, you repeat the action—twice if both of them did."; Commander (2013 Edition) Release Notes (Matt Tabak, Oct 22, 2013): "Then, the effect happens again for you a number of times equal to the number of opponents who accepted."

**Recommendation:** decide the entry as it is (one event): the only source that addresses a
replacement effect gives the engine's answer for the engine's own example.

### 3. Q4: the three provisional "for as long as" entries (`phasing-out-ends-a-waiting-for-as-long-as-duration`, `copied-ability-duration-starts-as-the-copy-is-created`, `duration-about-another-object-ends-while-the-ability-waits`)

No source covers any of the three. The closest, on effects that already exist: Cranial Insertion,
03/15/2010, Eli Shiffrin: "When the Sower phases in, all excited to peek at its goyf-in-a-cage,
it'll be disappointed to see it gone: the effect does not somehow resume once it's ended."; Cranial
Insertion, 12/09/2019, Carsten Haese: "Unlike "for as long as" effects like the effect on Sower of
Temptation, which ends when Sower of Temptation phases out, the "exile until" effect of Banishing
Light actually ties the return event to the event of Banishing Light leaving the battlefield.";
Cranial Insertion, 11/08/2010, Eli Shiffrin: "The source of the copied Rust Tick ability is the same
as the source of the original ability, so as long as that one Rust Tick stays tapped, the artifact
targeted by its original ability and the artifact targeted by the copy will both remain tapped."

**Recommendation:** keep all three provisional, or decide them as they are; nothing found argues
for a change, and the sources on existing effects fit the first.

### 4. Q8: attack taxes and X-only mana (`attack-total-cost-contains-x`, provisional)

No source. The nearest: Cranial Insertion, 02/20/2006, Thijs van Ommen: "you first determine the
total cost you have to pay, then play mana abilities if you need mana, and finally pay the cost.";
Cranial Insertion, 05/05/2014, James Bennett: "In order to attack with a creature, its controller
needs to pay all applicable costs, and one payment only satisfies one cost."

**Recommendation:** keep provisional; the user's local agent could ask a judge (Discord/Reddit
are out of reach here).

### 5. Q6: a replacement source losing its abilities during a simultaneous entry (decision-audit-1's open point on `simultaneous-entries-ordered-by-controller`)

No source. **Recommendation:** leave it with the user as decision-audit-1 framed it.

### 6. Discrepancy log: Q7, Q9, Q10 (documentation, done in this item)

- Siege of Towers: a **copying error** in Gatherer (the release notes say "you'll copy Siege of Towers"). **Recommendation:** report to Wizards (Gatherer's ruling text).
- The hybrid symbols (Taeko, Slinza, Syndicate Heavy, Sorin of House Markov; also Kellan, the Kid's "{1}{B}{G}"): a **copying error** in Scryfall's data (the release notes and Gatherer keep {U/B}, {1}{R/G}, {W/B}, {1}{B/G}). **Recommendation:** report to Scryfall, not Wizards.
- Seraph: a **template error** (Tawnos's Coffin's same-day ruling; no Seraph text ever exiled). **Recommendation:** report to Wizards. The words "after being exiled" are not exempted: nothing in the game depends on them (the token ceases to exist in the graveyard, CR 704.5d), and the ruling stays cited for its conclusion; say if you want that part exempted under `ruling-contradicting-oracle-gives-way` instead.

### 7. The CR effective 2026-11-06

The repository's CR is effective 2026-09-25; Wizards now publishes one effective 2026-11-06
(new 713 Meld Cards, so 713–733 renumbered to 714–734; new 109.4b). The rules these questions use
are unchanged in wording. **Recommendation:** a data-update item, outside this one.

---

## Documentation changes made in this item

- `docs/RULING_DISCREPANCIES.md`: the Siege of Towers entry and the hybrid-symbol entry moved from "Template errors" to "Copying errors" (kind "copying error", the release notes and Gatherer quoted); the Seraph note under "Not yet investigated" made a template-error entry; the Kellan, the Kid note updated with the release notes and Gatherer text; the header's sentence on which hosts were outside the allowed network updated.
- `docs/rulings-exemptions/rulings-P222.tsv`, `rulings-P130.tsv`, `rulings-P214.tsv`: the reasons of the Siege of Towers and hybrid-symbol exemptions now say "copying error" and name the release notes (needles unchanged).
- Register Sources fields (no decision changed): `that-opponent-fixed-as-it-triggers` (the Cranial Insertion ward/Frost Titan and Relic Putrescence answers, the Norn's Decree release notes read; the Why's sentence on blocked hosts corrected), `for-as-long-as-checked-continually-from-activation` and `duration-the-resolution-sets-up-is-checked-as-the-effect-begins` (the Foundations Update Bulletin verified; Seasinger's Gatherer ruling verified), `flash-permission-conditions-judged-as-casting-begins` (the forum post identified and quoted), `for-each-tokens-counters-life-one-event` (the forum and Cranial Insertion answers), `phasing-out-ends-a-waiting-for-as-long-as-duration` (the Cranial Insertion phasing answers), `attack-total-cost-contains-x` (the Cranial Insertion cost answers, Propaganda's ruling), and the Affects of `ruling-contradicting-oracle-gives-way` (the two reclassified cases and Seraph); `docs/DECISIONS.md` regenerated (`docs/INTERPRETATIONS.md` was already current: no title or status changed).

## Not reachable

- **www.cardmarket.com**: refused by the network policy ("agent proxy: no rule or network setting allows this host"). Would have read "#CMAskTheJudge Episode 66: A Real Jewel" (https://www.cardmarket.com/en/Magic/Insight/Articles/CMAskTheJudge-Episode-66-A-Real-Jewel), which web-search summaries describe as Coveted Jewel with the attacker removed before blocks (unverified), to see whether it mentions a control change.
- **www.reddit.com**: the .json endpoints answer 403 "You've been blocked by network security."; the HTML pages are a JavaScript bot challenge. Would have searched r/mtgrules for Norn's Decree, Coveted Jewel, ward with Perplexing Chimera or Commandeer, the initiative and the monarch after a control change, tempting offers with Hardened Scales/Xorn/Peregrin Took, Ixidron with Grumgully, and Sphere of Safety with Rosheen Meanderer.
- **www.mtgsalvation.com**: 403 with a Cloudflare challenge ("cf-mitigated: challenge"). Same searches in its rules forum; in particular the threads a web search lists as "Frost Titan and who is controlling it" (https://www.mtgsalvation.com/forums/magic-fundamentals/magic-rulings/808984-frost-titan-and-who-is-controlling-it) and "Mark of Mutiny and Frost Titan" (https://www.mtgsalvation.com/forums/magic-fundamentals/magic-rulings/magic-rulings-archives/286994-mark-of-mutiny-and-frost-titan), titles only, contents unread.
- **web.archive.org**: connection reset during the TLS handshake. Would have read Gatherer's old pages for Seraph (multiverseid 159245, 2711, 4156) as captured 2007–2009, to see the 2007 ruling's original words ("removed from the game"?) and any Oracle text between 1997 and 2007, and the Masters Edition FAQ if one exists.
- Also refused by the network policy (not in the brief's list): scryfall.com and api.scryfall.com, boardgames.stackexchange.com, judgeacademy.com, tappedout.net, mtg.fandom.com, mtg.wiki.
- Not searched for lack of a reachable source: the Magic Judges Discord and other judge chats.

## Saved raw files quoted

All under `target/web-research/`, fetched 2026-11-20 (UTC times in `fetch-log.tsv`):

| File | URL |
|---|---|
| `one-release-notes.html` | https://magic.wizards.com/en/news/feature/phyrexia-all-will-be-one-release-notes |
| `clb-release-notes.html` | https://magic.wizards.com/en/articles/archive/feature/commander-legends-battle-baldurs-gate-release-notes-2022-06-01 |
| `clb-mechanics.html` | https://magic.wizards.com/en/news/feature/commander-legends-battle-for-baldurs-gate-mechanics |
| `cr-changes-2022-06-01.html` | https://magic.wizards.com/en/news/announcements/comprehensive-rules-changes-6-1-22 |
| `c18-release-notes.html` | https://magic.wizards.com/en/articles/archive/feature/commander-2018-edition-release-notes-2018-07-27 |
| `c19-release-notes.html` | https://magic.wizards.com/en/articles/archive/feature/commander-2019-edition-release-notes-2019-08-09 |
| `cn2-release-notes.html` | https://magic.wizards.com/en/news/feature/conspiracy-take-crown-release-notes-2016-08-22 |
| `cmr-release-notes.html` | https://magic.wizards.com/en/articles/archive/feature/commander-legends-release-notes-2020-11-06 |
| `msh-release-notes.html` | https://magic.wizards.com/en/news/feature/marvel-super-heroes-release-notes |
| `fra-release-notes.html` | https://magic.wizards.com/en/news/feature/reality-fracture-release-notes |
| `afr-release-notes.html` | https://magic.wizards.com/en/articles/archive/feature/adventures-forgotten-realms-release-notes-2021-07-09 |
| `ci-2760.html`, `ci-2763.html` | https://www.cranial-insertion.com/article/2760, /2763 |
| `ci-4365.html` | https://www.cranial-insertion.com/article/4365 |
| `ci-2242.html` | https://www.cranial-insertion.com/article/2242 |
| `ci-1789.html` | https://www.cranial-insertion.com/article/1789 |
| `ci-4389.html` | https://www.cranial-insertion.com/article/4389 |
| `mj-topic-49911.html` | https://apps.magicjudges.org/forum/topic/49911/ |
| `gatherer-coveted-jewel-c21.html` | https://gatherer.wizards.com/C21/en-us/240/coveted-jewel |
| `gatherer-norns-decree-onc9.html` | https://gatherer.wizards.com/ONC/en-us/9/norns-decree |
| `mj-post-131920.html` | https://apps.magicjudges.org/forum/post/131920/ |
| `cr-changes-2022-04-20.html` | https://magic.wizards.com/en/news/announcements/comprehensive-rules-changes-4-20-22 |
| `snc-release-notes.html` | https://magic.wizards.com/en/news/feature/streets-new-capenna-release-notes-2022-04-20 |
| `ori-release-notes.html` | https://magic.wizards.com/en/news/feature/magic-origins-release-notes-2015-07-08 |
| `uma-release-notes.html` | https://magic.wizards.com/en/news/feature/ultimate-masters-release-notes-2018-11-29 |
| `fdn-update-bulletin.html` | https://magic.wizards.com/en/news/announcements/foundations-update-bulletin |
| `ci-271.html`, `ci-3606.html`, `ci-3177.html`, `ci-1903.html`, `ci-305.html`, `ci-3209.html` | https://www.cranial-insertion.com/article/271, /3606, /3177, /1903, /305, /3209 |
| `mj-topic-14129-p1.html` | https://apps.magicjudges.org/forum/topic/14129/?page=1 |
| `ci-1663.html`, `ci-2515.html` | https://www.cranial-insertion.com/article/1663, /2515 |
| `c13-release-notes.html` | https://magic.wizards.com/en/news/feature/commander-2013-edition-release-notes-2013-10-22 |
| `ci-3105.html`, `ci-3643.html`, `ci-3141.html`, `ci-2457.html` | https://www.cranial-insertion.com/article/3105, /3643, /3141, /2457 |
| `mj-topic-42824-p1.html` | https://apps.magicjudges.org/forum/topic/42824/?page=1 |
| `gatherer-seraph-me1-mid159245.html` | https://gatherer.wizards.com/Pages/Card/Details.aspx?multiverseid=159245 |
| `gatherer-seraph-ice-mid2711-printed.html` | https://gatherer.wizards.com/Pages/Card/Details.aspx?printed=true&multiverseid=2711 |
| `gatherer-tawnoss-coffin-name.html` | https://gatherer.wizards.com/Pages/Card/Details.aspx?name=Tawnos%27s+Coffin |
| `gatherer-seasinger-med49.html` | https://gatherer.wizards.com/MED/en-us/49/seasinger |
| `me1-update-bulletin.html` (404) | https://magic.wizards.com/en/articles/archive/feature/masters-edition-update-bulletin-2007-08-29 |
| `ormos-masters-edition-update-bulletin-2007-08-29.md` | https://raw.githubusercontent.com/maxmakesmagic/ormos/main/archive/en/articles/archive/feature/2007/08/masters-edition-update-bulletin-2007-08-29.md |
| `ci-58.html`, `ci-1281.html`, `ci-485.html`, `ci-3664.html`, `ci-279.html` | https://www.cranial-insertion.com/article/58, /1281, /485, /3664, /279 |
| `rvr-release-notes.html` | https://magic.wizards.com/en/news/feature/ravnica-remastered-release-notes |
| `rvr-release-notes-EN.pdf` | https://media.wizards.com/2023/downloads/RVR_Release_Notes_XrN0zlJaFn/EN_MTGRVR_ReleaseNotes_20231120.pdf |
| `gatherer-siege-of-towers-rvr.html` | https://gatherer.wizards.com/RVR/en-us/123/siege-of-towers |
| `fdn-release-notes.html` | https://magic.wizards.com/en/news/feature/foundations-release-notes |
| `mkm-release-notes.html` | https://magic.wizards.com/en/news/feature/murders-at-karlov-manor-release-notes |
| `mh3-release-notes.html` | https://magic.wizards.com/en/news/feature/modern-horizons-3-release-notes |
| `otj-release-notes.html` | https://magic.wizards.com/en/news/feature/outlaws-of-thunder-junction-release-notes |
| `gatherer-taeko-j25.html`, `gatherer-slinza-j25.html`, `gatherer-hurska-j25.html` | https://gatherer.wizards.com/J25/en-us/39/taeko-the-patient-avalanche, /55/slinza-the-spiked-stampede, /53/hurska-sweet-tooth |
| `gatherer-syndicate-heavy-clu.html`, `gatherer-sorin-mh3.html`, `gatherer-kellan-the-kid-otj.html` | https://gatherer.wizards.com/CLU/en-us/47/syndicate-heavy, https://gatherer.wizards.com/MH3/en-us/245/sorin-of-house-markov, https://gatherer.wizards.com/OTJ/en-us/213/kellan-the-kid |
| `wizards-rules-page.html`, `MagicCompRules-20261106.txt` | https://magic.wizards.com/en/rules, https://media.wizards.com/2026/downloads/MagicCompRules%2020261106.txt |
| `mj-forum-73.html` | https://apps.magicjudges.org/forum/73/ |
| `ci-search/*.html`, `mj-s-*.html`, `mj-search-*.html` | the archive and forum searches listed under each question |
