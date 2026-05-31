import pygame
import random

pygame.init()


FPS = 60
WIDTH, HEIGHT = 1920, 980
TILE_SIZE = 32
world_width = 5000
world_height = 980
player_velocity_y = 0
gravity = 1
player_speed = 7
jump_strength = -20


# camera variables
camera_x = 0
camera_y = 0

clock = pygame.time.Clock()

screen = pygame.display.set_mode((WIDTH, HEIGHT), pygame.RESIZABLE)


pygame.display.set_caption("Terraria Clone")

player = pygame.Rect(WIDTH // 2, HEIGHT // 2, 50, 80)  # x, y, width, height


GRASS = 1
DIRT = 2
STONE = 3


tiles = []
world_grid = []

for x in range(0, world_width, TILE_SIZE):
    for y in range(0, world_height, TILE_SIZE):
        world_grid_rect = pygame.Rect(x, y, TILE_SIZE, TILE_SIZE)
        world_grid.append(world_grid_rect)


ground_start_y = (HEIGHT - 400) // TILE_SIZE * TILE_SIZE
stone_start_y = ground_start_y + (160 // TILE_SIZE * TILE_SIZE)

min_ground_y = (HEIGHT - 550) // TILE_SIZE * TILE_SIZE
max_ground_y = (HEIGHT - 250) // TILE_SIZE * TILE_SIZE

for x in range(0, world_width, TILE_SIZE):
    ground_start_y += random.choice([-TILE_SIZE, 0, TILE_SIZE])

    ground_start_y = max(min_ground_y, ground_start_y)
    ground_start_y = min(max_ground_y, ground_start_y)

    stone_start_y += random.choice([-TILE_SIZE, 0, TILE_SIZE])

    stone_start_y = max(ground_start_y + 3 * TILE_SIZE, stone_start_y)
    stone_start_y = min(ground_start_y + 6 * TILE_SIZE, stone_start_y)

    for y in range(ground_start_y, HEIGHT, TILE_SIZE):
        if y == ground_start_y:
            tile_type = GRASS
        elif y < stone_start_y:
            tile_type = DIRT
        else:
            tile_type = STONE

        tile_rect = pygame.Rect(x, y, TILE_SIZE, TILE_SIZE)
        tiles.append((tile_rect, tile_type))


print("Window created")

running = True
while running:
    for event in pygame.event.get():
        if event.type == pygame.QUIT:
            running = False
        if event.type == pygame.MOUSEBUTTONDOWN:
            mouse_x, mouse_y = pygame.mouse.get_pos()

            world_mouse_x = mouse_x + camera_x
            world_mouse_y = mouse_y + camera_y

            if event.button == 1:
                # Left click: remove tile
                for tile in tiles:
                    tile_rect, tile_type = tile

                    if tile_rect.collidepoint(world_mouse_x, world_mouse_y):
                        tiles.remove(tile)
                        break

            elif event.button == 3:

                for grid_tile in world_grid:
                    if grid_tile.collidepoint(world_mouse_x, world_mouse_y):
                        
                        tile_exists = False
                        for tile_rect, tile_type in tiles:
                            if tile_rect.topleft == grid_tile.topleft:
                                tile_exists = True
                                break
                        
                        if not tile_exists:
                            new_tile_rect = pygame.Rect(
                                grid_tile.x, grid_tile.y, TILE_SIZE, TILE_SIZE
                            )
                            tiles.append((new_tile_rect, GRASS))
                        break


    keys = pygame.key.get_pressed()

    # horizzontal movement

    horizontal_direction = 0

    if keys[pygame.K_LEFT] or keys[pygame.K_a]:
        horizontal_direction -= player_speed
    if keys[pygame.K_RIGHT] or keys[pygame.K_d]:
        horizontal_direction += player_speed

    player.x += horizontal_direction

    for tile_rect, tile_type in tiles:
        if player.colliderect(tile_rect):
            if horizontal_direction > 0:  # Moving right
                player.right = tile_rect.left
            elif horizontal_direction < 0:  # Moving left
                player.left = tile_rect.right

    # jump logic

    jump_pressed = keys[pygame.K_UP] or keys[pygame.K_w] or keys[pygame.K_SPACE]

    if jump_pressed and on_ground:
        player_velocity_y = jump_strength
        on_ground = False

    player_velocity_y += gravity
    player.y += player_velocity_y

    on_ground = False

    for tile_rect, tile_type in tiles:
        if player.colliderect(tile_rect):
            if player_velocity_y < 0:  # jumping up
                player.top = tile_rect.bottom
                player_velocity_y = 0

            if player_velocity_y > 0:  # falling down
                player.bottom = tile_rect.top
                player_velocity_y = 0
                on_ground = True

    screen.fill((0, 0, 0))
    print(f"Player position: ({player.x}, {player.y})")  # Debugging player position
    camera_x = int(player.x - WIDTH // 2)
    camera_y = int(player.y - HEIGHT // 2)
    camera_x = max(0, camera_x)
    camera_x = min(world_width - WIDTH, camera_x)
    camera_y = max(0, camera_y)
    camera_y = min(world_height - HEIGHT, camera_y)

    player.x = max(0, player.x)
    player.x = min(world_width - player.width, player.x)
    player.y = max(0, player.y)
    player.y = min(world_height - player.height, player.y)

    pygame.draw.rect(
        screen,
        (255, 0, 0),
        (player.x - camera_x, player.y - camera_y, player.width, player.height),
    )  # Draw the player relative to the camera

    for tile_rect, tile_type in tiles:
        if tile_type == GRASS:
            color = (0, 200, 0)
        elif tile_type == DIRT:
            color = (139, 69, 19)
        elif tile_type == STONE:
            color = (128, 128, 128)

        pygame.draw.rect(
            screen,
            color,
            (
                tile_rect.x - camera_x,
                tile_rect.y - camera_y,
                tile_rect.width,
                tile_rect.height,
            ),
        )  # Draw tiles relative to the camera
    
    pygame.display.flip()
    clock.tick(FPS)

pygame.quit()
