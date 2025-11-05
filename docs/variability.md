# Variability

Our DSL includes several variability points that allow users to customize and extend its functionality
These variability points are designed to accommodate different use cases and preferences.

## Compile Time Variability

Some features of the DSL can be enabled or disabled at compile time which affects the game behavior only when the game is launched.
For example, users can choose :
- Game Section : define different game settings to generate the map with a seed, a size, and other parameters.
- Cities Section : define cities with different building options. You can customize the name, the position, the starting ressources, buildings and units,....
- Building Section : define buildable buildings with different properties, such as cost, production, and special abilities.
- Unit Section : define trainable units with different attributes, such as health, attack power, and movement range.
- Victory Conditions Section : define different victory conditions for the game, such as a number of turns or resources to reach.

## Runtime Variability

Some features of the DSL can be modified at runtime which affects the game behavior during the gameplay.
For example, users can choose :
- Game Section : modify the current turn of the game. This gets incremented at each end turn.
- Cities Section : the starting ressources, buildings and units of the cities are modified during the gameplay when the city produces ressources, builds buildings or trains units.

## Skin Variability

The DSL allows users to customize the visual appearance of the game through skin variability.
Users can define different color schemes for civilizations and the game window to enhance the gaming experience.
For example, users can choose :
- Civilization Colors : customize the colors representing different civilizations in the game.
- Window Colors : modify the color scheme of the game window to suit user preferences.