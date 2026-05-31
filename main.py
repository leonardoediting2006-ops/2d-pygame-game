import pygame
import random
pygame.init()


FPS = 60
WIDTH, HEIGHT = 1920, 980
TILE_SIZE = 32
world_width = 5000
world_height =980
player_velocity_y = 0
gravity = 1
player_speed = 7
jump_strength = -20


#camera variables
camera_x = 0
camera_y = 0

clock = pygame.time.Clock()

screen = pygame.display.set_mode((WIDTH, HEIGHT), pygame.RESIZABLE)


pygame.display.set_caption("Terraria Clone")

player=pygame.Rect(WIDTH//2, HEIGHT//2, 50, 80) # x, y, width, height


GRASS = 1
DIRT = 2
STONE = 3



tiles= []




ground_start_y= HEIGHT-400
stone_start_y = ground_start_y + 160

for x in range(0, world_width, TILE_SIZE):
    ground_start_y += random.choice([-TILE_SIZE, 0, TILE_SIZE])

    ground_start_y = max(HEIGHT - 550, ground_start_y)
    ground_start_y = min(HEIGHT - 250, ground_start_y)

    stone_start_y += random.choice([-TILE_SIZE, 0, TILE_SIZE])

    stone_start_y = max(ground_start_y + 100, stone_start_y)
    stone_start_y = min(ground_start_y + 200, stone_start_y)

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

    keys = pygame.key.get_pressed()

    #horizzontal movement

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

    #jump logic

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
                player.top= tile_rect.bottom
                player_velocity_y = 0

            if player_velocity_y > 0:  # falling down
                player.bottom = tile_rect.top
                player_velocity_y = 0
                on_ground = True

    screen.fill((0, 0, 0))

    camera_x = int(player.x - WIDTH // 2)
    camera_y = int(player.y - HEIGHT // 2)
    camera_x = max(0,camera_x)
    camera_x = min(world_width - WIDTH, camera_x)
    camera_y = max(0,camera_y)
    camera_y = min(world_height - HEIGHT, camera_y)

    player.x = max(0, player.x)
    player.x = min(world_width - player.width, player.x)
    player.y = max(0, player.y)
    player.y = min(world_height - player.height, player.y)

    pygame.draw.rect(screen, (255, 0, 0), (player.x - camera_x, player.y - camera_y, player.width, player.height))# Draw the player relative to the camera

    for tile_rect, tile_type in tiles:
        if tile_type == GRASS:
            color = (0, 200, 0)
        elif tile_type == DIRT:
            color = (139, 69, 19)
        elif tile_type == STONE:
            color = (128, 128, 128)
            
        pygame.draw.rect(screen, color, (tile_rect.x - camera_x, tile_rect.y - camera_y, tile_rect.width, tile_rect.height))# Draw tiles relative to the camera
        
    
    pygame.display.flip()
    clock.tick(FPS)

pygame.quit()
