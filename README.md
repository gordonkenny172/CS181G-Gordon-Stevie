# Preliminary Notes

We began this project by first building our games and then extracting the common infrastructure used in each of our games. We first built the fighter game, then the puzzle game, and finally the adventure game. Then we built our engine using our final adventure game codebase. There will be improvements in code and
changes in the way we organized our code base from game to game which will make more sense with this task order in mind.

Our games were inspired by the adventure game lab and we began constructing our games off of the adventure game lab template which incorporated the Immediate
wrapper for frenderer.

# Fighter

The first game we made was a two-player tank shooter game. Tanks spawn on different parts of the map and can be controlled by individual players. The first
person to shoot and kill the other player wins.

The main features we implemented in this game was:

* Local Two-Player Functionality
  * We allowed for the creation and control of two players at the same time. Player1 is controlled by the arrow keys and can shoot with the space bar.            Player2 is controlled by WASD and can shoot using Q.
  * Each player has a corresponding EntityType which distinguishes the two. This allows for different input keys sprites to be used.
  * We edited our spawn mechanics by changing our level parsing to check for player1 and player2 as opposed to player.
* Projectile Bouncing
  * We implemented projectiles as an EntityType and treated them as entities. We have a separate vec of entities that represent our projectiles and help
    us keep track of our indices.
  * We implemented collision code to allow our bullets to bounce. This has to do with editing their direction depending on their displacement calculated
    in contact generation. Note that the collision can be a bit buggy at times due to everything having rectangles as their hitbox which can have some
    interesting interactions with certain portions of the map.
* Rotational Movement
  * We changed the movement from the original adventuregame template to a rotational system. Left and right inputs turn the character which can then be moved
    forward and backwards by the up and down inputs. We did this to have our tanks move like actual tanks need to in real-life.
  * We accomplished this by having a direction variable stored as a float which would represent the current angle of rotation of our entities. Then we used
    trigonometric functions to change between a float and a Vec2.

# Puzzle

Our puzzle game was built on top of our fighter game and added an "icy tile" mechanic. We edited TileData by adding a "slippery" boolean attribute. Then, in our level design we edited the levels and the parser to check for an additional parameter which would tell us whether or not that tile was slippery. Then, we added some collision detection and response code which checked if you were on a slippery tile and, if so, would force you to slide forward (you can still change direction while sliding). We didn't have enough time to create an actual puzzle and focused on implementing this slippery tile mechanic.

# Adventure

For our final game, we added to our fighter game and created a shooter game. Both players play as birds that need to maneuver around enemy projectiles and kill all three enemies on screen. The enemies will move randomly and shoot a bouncing projectile every 10 seconds. We added distinctions between player projectiles and enemy projectiles and prevented self-inflicted damage (which was a mechanic in our fighter game). However, to get collisions to work properly we needed to make large changes to our codebase.

First, we needed to reorganize the way we stored our entities. In our previous games, we kept entities (as in players and enemies) seperate from projectiles. However, to get our shooting mechanics to work properly, we needed to separate players and enemies into different vecs (or rather this was the most straightforward fix). This was because if we kept players and enemies in the same entity vec, we would run into index issues when creating contacts. When creating player rectangles and enemy rectangles from the same entity vec, the indices would change and made it hard to correctly inflict damage (or have other collision interactions work properly) on the right entity. This resulted in a lot of redundant code but also allowed the behavior to work properly.

We then had to add different collision functions for different types of entities. Since we stored players, enemies, player projectiles, and enemy projectiles in different vectors in our game state, we needed different collision functions that would correctly edit the values in each vec.

# Engine

Finally, we used our adventure game as a template to then extract our engine. We took out all of the individual game attributes and left the remaining infrastructure as our engine. This engine has support for the main features we wanted to implement, which are:

* Local Two-Player Functionality
* Bouncing Projectiles
* Rotational Movement
* Different Tile Attributes

# Conclusion and Takeaways

We are happy with how our games turned out and like our progression of added complexity from game to game. We would have liked to organize our code a lot better since there are many inconsistencies between the codebases of our game which we would like to fix if we have the time.
