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

# Adventure

# Engine
