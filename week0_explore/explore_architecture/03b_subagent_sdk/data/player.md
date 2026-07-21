# Player

_Last updated: Session 5 (connected, verified location via look/score)_

## Status
- Name: dummy
- Level: 1 (Swordpupil)
- Class / Title: Swordpupil
- HP: 21 / 21, Mana: 100 / 100, Move: 83 / 83
- Exp: 994 (need 1006 to next level)
- Gold: 0
- Condition: hungry, thirsty
- Location: The Tournament And Practice Yard (guildmaster present, sharpening an axe)

## Guild
- Guild: Swordsmen (Players Guild for warriors)
- Route from a known landmark: From Temple → Temple Square → Market Square → Main Street (east) → Main Street (far east) → Guild of Swordsmen entrance (south) → Bar of Swordsmen (east) → Tournament Yard (south)
- Practice yard / trainer location: The Tournament And Practice Yard (guildmaster present, sharpening an axe)

## Equipment
- Wielded: —
- Worn: —

## Inventory
- Nothing

## Skills
<!-- One line per skill, with practice status if known (unpracticed / practicing / learned). -->
- Kick: practicing

## Status (smarty)
- Name: smarty
- Level: 1 (Apprentice of Magic)
- Class / Title: Smarty the Apprentice of Magic
- HP: 18 / 18, Mana: 100 / 100, Move: 83 / 83
- Exp: 1 (need 2499 to next level)
- Gold: 0
- Condition: hungry, thirsty
- Location: The Mages' Laboratory (guildmaster present, studying a spellbook; exit w)

## Current Goal
<!-- The active long-term objective. Keep this short and concrete so a future
     session can pick it up cold. -->
- PRIMARY GOAL: Defeat the massive minotaur in the newbie zone north of Midgaard
- Next step: Navigate to newbie zone north of Midgaard (likely via Countryside north of Temple)

## Progress Log
<!-- Append-only. One entry per session, newest at the bottom. A few lines
     each: what was attempted, what happened, what to try next. This is what
     lets a new session resume without replaying the whole history. -->
- Session 1: Found the bakery and listed menu. Route: Temple → Temple Square → Market Square → Main Street → Bakery. Menu has danish pastry (7 gold), bread (14 gold), waybread (72 gold).
- Session 2: Extensive exploration. Mapped many areas, found 4 guilds (Clerics, Magic Users, Thieves, Swordsmen). Guild of Swordsmen has a practice yard and guildmaster. Went down into darkness from practice yard and got stuck.
- Session 3: Discovered I'm a Swordpupil (Swordsmen guild class). Found the Players Guild is the class-specific Guild of Swordsmen. Navigated to Tournament And Practice Yard and successfully practiced kick with guildmaster. TASK COMPLETE.
- Session 4: Task was to verify location and return to Temple Of Midgaard. Connected/logged in as dummy; login placed the character directly in The Temple Of Midgaard (confirmed with `look` and `score` — full HP 21/21, Mana 100/100, Move 83/83, standing). Location was already correct, so no recall command or navigation was needed. player.md Status section (HP/Mana/Move/Exp/Location) updated to match the live, verified state — the previous "resting after dying to baby dragon" note was stale. Mid-session, user asked to send the character to the bakery: walked Temple Of Midgaard -> south -> Temple Square -> south -> Market Square -> west -> Main Street -> north -> The Bakery, using the known route from world.md. Arrived successfully, confirmed via `look`.
- Session 5: Daemon was already running/connected (no new login needed). Ran `look` and `score` to verify state: character is in The Tournament And Practice Yard (guildmaster here sharpening an axe), standing, 21/21 HP, 100/100 mana, 83/83 move, 994/1006 exp toward next level, 0 gold, level 1 Swordpupil, hungry and thirsty. Room and exits (n -> Bar of Swordsmen, d -> dark dungeon/well) already matched the existing world.md entry exactly, so no new room data — just refreshed player.md Status/Location to match the confirmed live state. Next step: still need to progress the PRIMARY GOAL (minotaur in newbie zone north of Midgaard) — consider eating/drinking (hungry/thirsty) before further travel, then head toward Countryside/Great Field route noted in world.md.
- Session 6 (character: smarty, session name `smarty`): Task was only to check hunger status. Daemon for `smarty` session was already running/logged in. Ran `score` (and `look`): Smarty the Apprentice of Magic, level 1, 18/18 HP, 100/100 mana, 83/83 move, 1 exp (need 2499 to next level), 0 gold, standing, in The Mages' Laboratory (guildmaster here, exit w). Condition: **hungry** and **thirsty** (both flags present in score output and recurring "You are hungry."/"You are thirsty." tics in the log). No numeric hunger/thirst values are exposed by this MUD, only the boolean condition messages. No other actions taken (no eating/drinking/combat) — task was report-only.
- Session 6: Task was only to check hunger status. Session already live/logged in. Ran `score`: 21/21 HP, 100/100 mana, 83/83 move, 994/1006 exp, 0 gold, standing. Confirmed "You are hungry." and "You are thirsty." messages — still hungry/thirsty, unchanged from Session 5. No eating/drinking or navigation done (out of scope for this task). Next step: eat/drink before further travel toward the minotaur goal.
