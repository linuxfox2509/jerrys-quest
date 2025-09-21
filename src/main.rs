use macroquad::prelude::*;

// Player struct
struct Player {
    pos: Vec2,
    vel: Vec2,
    on_ground: bool,
}

// Platform struct
struct Platform {
    pos: Vec2,
    size: Vec2,
    kind: PlatformKind,
}

enum PlatformKind {
    Small,
    Big,
}

// Coin struct
struct Coin {
    pos: Vec2,
    collected: bool,
}

// Game state enum
enum GameState {
    Playing,
    Win,
    GameOver,
}

#[macroquad::main("Jerry's Quest")]
async fn main() {
    // Load textures
    let player_tex = load_texture("assets/player.png").await.unwrap();
    let small_platform_tex = load_texture("assets/small_platform.png").await.unwrap();
    let big_platform_tex = load_texture("assets/big_platform.png").await.unwrap();
    let coin_tex = load_texture("assets/coin.png").await.unwrap();

    player_tex.set_filter(FilterMode::Nearest);
    small_platform_tex.set_filter(FilterMode::Nearest);
    big_platform_tex.set_filter(FilterMode::Nearest);
    coin_tex.set_filter(FilterMode::Nearest);

    // Player setup
    let mut player = Player {
        pos: vec2(50.0, 300.0),
        vel: vec2(0.0, 0.0),
        on_ground: false,
    };

    let gravity = 0.45;
    let jump_force = -16.0;

    // Platforms
    let platforms = vec![
        Platform { pos: vec2(0.0, 400.0), size: vec2(256.0, 32.0), kind: PlatformKind::Big },
        Platform { pos: vec2(300.0, 300.0), size: vec2(128.0, 32.0), kind: PlatformKind::Small },
        Platform { pos: vec2(500.0, 200.0), size: vec2(128.0, 32.0), kind: PlatformKind::Small },
    ];

    // Coins
    let mut coins = vec![
        Coin { pos: vec2(340.0, 284.0), collected: false },
        Coin { pos: vec2(540.0, 184.0), collected: false },
    ];

    let mut score = 0;
    let mut state = GameState::Playing;

    loop {
        clear_background(LIGHTGRAY);

        match state {
            GameState::Playing => {
                // --- INPUT ---
                if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
                    player.vel.x = -3.5;
                } else if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
                    player.vel.x = 3.5;
                } else {
                    player.vel.x = 0.0;
                }

                if player.on_ground && is_key_pressed(KeyCode::Space) {
                    player.vel.y = jump_force;
                    player.on_ground = false;
                }

                // --- PHYSICS ---
                player.vel.y += gravity;
                player.pos += player.vel;

                // --- COLLISIONS ---
                player.on_ground = false;
                for plat in &platforms {
                    let player_rect = Rect::new(player.pos.x, player.pos.y, 32.0, 32.0);
                    let plat_rect = Rect::new(plat.pos.x, plat.pos.y, plat.size.x, plat.size.y);

                    if player_rect.overlaps(&plat_rect) {
                        // Landing on top
                        if player.vel.y > 0.0 && player.pos.y + 32.0 - player.vel.y <= plat.pos.y {
                            player.pos.y = plat.pos.y - 32.0;
                            player.vel.y = 0.0;
                            player.on_ground = true;
                        }
                        // Hitting bottom
                        else if player.vel.y < 0.0 && player.pos.y >= plat.pos.y + plat.size.y {
                            player.pos.y = plat.pos.y + plat.size.y;
                            player.vel.y = 0.0;
                        }
                    }
                }

                // --- COINS ---
                for coin in &mut coins {
                    if !coin.collected {
                        let coin_rect = Rect::new(coin.pos.x, coin.pos.y, 16.0, 16.0);
                        let player_rect = Rect::new(player.pos.x, player.pos.y, 32.0, 32.0);
                        if player_rect.overlaps(&coin_rect) {
                            coin.collected = true;
                            score += 1;
                        }
                    }
                }

                // --- CHECK WIN / GAME OVER ---
                if coins.iter().all(|c| c.collected) {
                    state = GameState::Win;
                }

                if player.pos.y > screen_height() {
                    state = GameState::GameOver;
                }

                // --- DRAW ---
                // Platforms
                for plat in &platforms {
                    let tex = match plat.kind {
                        PlatformKind::Small => &small_platform_tex,
                        PlatformKind::Big => &big_platform_tex,
                    };
                    draw_texture_ex(
                        tex,
                        plat.pos.x,
                        plat.pos.y,
                        WHITE,
                        DrawTextureParams { dest_size: Some(plat.size), ..Default::default() },
                    );
                }

                // Coins
                for coin in &coins {
                    if !coin.collected {
                        draw_texture_ex(
                            &coin_tex,
                            coin.pos.x,
                            coin.pos.y,
                            WHITE,
                            DrawTextureParams { dest_size: Some(vec2(16.0, 16.0)), ..Default::default() },
                        );
                    }
                }

                // Player
                draw_texture_ex(
                    &player_tex,
                    player.pos.x,
                    player.pos.y,
                    WHITE,
                    DrawTextureParams { dest_size: Some(vec2(32.0, 32.0)), ..Default::default() },
                );

                // HUD
                draw_text(&format!("Score: {}", score), 20.0, 30.0, 30.0, BLACK);
            }

            GameState::GameOver => {
                clear_background(BLACK);
                draw_text("Game Over!", screen_width()/2.0 - 100.0, screen_height()/2.0, 50.0, RED);
                draw_text("Press R to restart", screen_width()/2.0 - 130.0, screen_height()/2.0 + 60.0, 30.0, WHITE);

                if is_key_pressed(KeyCode::R) {
                    player.pos = vec2(50.0, 300.0);
                    player.vel = vec2(0.0, 0.0);
                    for coin in &mut coins { coin.collected = false; }
                    score = 0;
                    state = GameState::Playing;
                }
            }

            GameState::Win => {
                clear_background(LIME);
                draw_text("You Win!", screen_width()/2.0 - 90.0, screen_height()/2.0, 50.0, GOLD);
                draw_text(&format!("Score: {}", score), screen_width()/2.0 - 50.0, screen_height()/2.0 + 60.0, 30.0, BLACK);
                draw_text("Press R to restart", screen_width()/2.0 - 130.0, screen_height()/2.0 + 100.0, 30.0, BLACK);

                if is_key_pressed(KeyCode::R) {
                    player.pos = vec2(50.0, 300.0);
                    player.vel = vec2(0.0, 0.0);
                    for coin in &mut coins { coin.collected = false; }
                    score = 0;
                    state = GameState::Playing;
                }
            }
        }

        next_frame().await;
    }
}
