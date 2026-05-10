# Tentative to create a clone of Rogue

The first line of the screen is a log bar describing what just happened.
The last line of the screen indicate the level you're in, and the status of your
character.

## map

The goal is to go to the last (9th) level of the dungon, find an object
and bring it back

Map is discovered as the player (represented by `@`) progress.

* `|` `-` : wall
* `.` : part of a room currently visible
* `#` : corridor
* `+` : door
* `*`: gold
* `)`: weapon
* `]`: armor
* `?`: piece of paper
* `=`: ring with magic
* `^`: trap
* `%`: staircas to other level
* `:`: piece of food


## character

The player has the following characterisctics

* **Gold**
* **Hp**
* **Str**
* **Arm**: armor protection
* **Exp**: experience level and experience points



## encounter

When walk on an object, you put i in your inventory
When you walk on a monster, you fight it.

## inventory
