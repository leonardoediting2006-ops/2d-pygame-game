import pygame
pygame.init()


FPS = 60
WIDTH, HEIGHT = 1920, 980
TILE_SIZE = 32
world_width = 5000
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

tiles= []

for i in range(0, world_width, TILE_SIZE):
    tile = pygame.Rect(i, HEIGHT - 200, TILE_SIZE, TILE_SIZE)
    tiles.append(tile)



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




    for tile in tiles:
        if player.colliderect(tile):
            if horizontal_direction > 0:  # Moving right
                player.right = tile.left
            elif horizontal_direction < 0:  # Moving left
                player.left = tile.right

    #jump logic

    jump_pressed = keys[pygame.K_UP] or keys[pygame.K_w] or keys[pygame.K_SPACE]

    if jump_pressed and on_ground:
        player_velocity_y = jump_strength
        on_ground = False



    player_velocity_y += gravity
    player.y += player_velocity_y

    on_ground = False
    
    for tile in tiles:

        if player.colliderect(tile):

            if player_velocity_y < 0:  # jumping up
                player.top= tile.bottom
                player_velocity_y = 0

            if player_velocity_y > 0:  # falling down
                player.bottom = tile.top
                player_velocity_y = 0
                on_ground = True

    screen.fill((0, 0, 0))

    camera_x = player.x - WIDTH // 2
    camera_y = player.y - HEIGHT // 2




    pygame.draw.rect(screen, (255, 0, 0), (player.x - camera_x, player.y - camera_y, player.width, player.height))# Draw the player relative to the camera

    for tile in tiles:
        pygame.draw.rect(screen, (0, 255, 0), (tile.x - camera_x, tile.y - camera_y, tile.width, tile.height))# Draw tiles relative to the camera

    pygame.display.flip()
    clock.tick(FPS)

pygame.quit()