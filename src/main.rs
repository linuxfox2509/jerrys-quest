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

// Cloud struct
struct Cloud<'a> {
    pos: Vec2,
    speed: f32,
    tex: &'a Texture2D,
}

// Game state enum
enum GameState {
    TitleScreen,
    Playing,
    GameOver,
}

#[macroquad::main("Jerry's Quest")]
async fn main() {
    // Load textures
    let player_tex = load_texture("assets/player.png").await.unwrap();
    let small_platform_tex = load_texture("assets/small_platform.png").await.unwrap();
    let big_platform_tex = load_texture("assets/big_platform.png").await.unwrap();
    let coin_tex = load_texture("assets/coin.png").await.unwrap();
    let cloud_tex1 = load_texture("assets/cloud1.png").await.unwrap();
    let cloud_tex2 = load_texture("assets/cloud2.png").await.unwrap();

    player_tex.set_filter(FilterMode::Nearest);
    small_platform_tex.set_filter(FilterMode::Nearest);
    big_platform_tex.set_filter(FilterMode::Nearest);
    coin_tex.set_filter(FilterMode::Nearest);
    cloud_tex1.set_filter(FilterMode::Nearest);
    cloud_tex2.set_filter(FilterMode::Nearest);

    // Clouds
    let mut clouds = vec![
        Cloud { pos: vec2(100.0, 100.0), speed: 20.0, tex: &cloud_tex1 },
        Cloud { pos: vec2(400.0, 150.0), speed: 30.0, tex: &cloud_tex2 },
        Cloud { pos: vec2(700.0, 120.0), speed: 25.0, tex: &cloud_tex1 },
    ];

    // Player setup
    let mut player = Player {
        pos: vec2(50.0, 300.0),
        vel: vec2(0.0, 0.0),
        on_ground: false,
    };

    // --- Physics settings ---
    let gravity = 0.5;
    let jump_force = -12.0;
    let move_speed = 4.0;

    // --- Coyote time ---
    let coyote_time_max = 0.15;
    let mut coyote_timer = 0.0;

    // Start with initial platforms
    let mut platforms = vec![
        Platform { pos: vec2(0.0, 400.0), size: vec2(256.0, 32.0), kind: PlatformKind::Big },
        Platform { pos: vec2(300.0, 300.0), size: vec2(128.0, 32.0), kind: PlatformKind::Small },
        Platform { pos: vec2(500.0, 200.0), size: vec2(128.0, 32.0), kind: PlatformKind::Small },
    ];

    // Coins
    let mut coins = vec![
        Coin { pos: vec2(340.0, 284.0), collected: false },
        Coin { pos: vec2(540.0, 184.0), collected: false },
    ];

    let mut score: u32 = 0;
    let mut highscore: u32 = 0;
    let mut state = GameState::TitleScreen;

    loop {
        clear_background(LIGHTGRAY);
        let dt = get_frame_time();

        // Update clouds (move slowly to the left, wrap around screen)
        for cloud in &mut clouds {
            cloud.pos.x -= cloud.speed * dt;
            if cloud.pos.x + 128.0 < 0.0 {
                cloud.pos.x = screen_width();
                cloud.pos.y = rand::gen_range(50.0f32, 200.0f32);
            }
        }

        match state {
            GameState::TitleScreen => {
                clear_background(SKYBLUE);

                // Draw clouds
                for cloud in &clouds {
                    draw_texture_ex(
                        cloud.tex,
                        cloud.pos.x,
                        cloud.pos.y,
                        WHITE,
                        DrawTextureParams { dest_size: Some(vec2(128.0, 64.0)), ..Default::default() },
                    );
                }

                draw_text("Jerry's Quest", screen_width()/2.0 - 200.0, screen_height()/2.0 - 50.0, 60.0, WHITE);
                draw_text("Press Space to Play", screen_width()/2.0 - 170.0, screen_height()/2.0 + 20.0, 40.0, WHITE);

                if is_key_pressed(KeyCode::Space) {
                    state = GameState::Playing;
                }
            }

            GameState::Playing => {
                // --- INPUT ---
                if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
                    player.vel.x = -move_speed;
                } else if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
                    player.vel.x = move_speed;
                } else {
                    player.vel.x = 0.0;
                }

                // --- COYOTE TIME ---
                if player.on_ground {
                    coyote_timer = coyote_time_max;
                } else {
                    coyote_timer -= dt;
                }

                if is_key_pressed(KeyCode::Space) && coyote_timer > 0.0 {
                    player.vel.y = jump_force;
                    player.on_ground = false;
                    coyote_timer = 0.0;
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
                        if player.vel.y > 0.0 && player.pos.y + 32.0 - player.vel.y <= plat.pos.y {
                            player.pos.y = plat.pos.y - 32.0;
                            player.vel.y = 0.0;
                            player.on_ground = true;
                        } else if player.vel.y < 0.0 && player.pos.y >= plat.pos.y + plat.size.y {
                            player.pos.y = plat.pos.y + plat.size.y;
                            player.vel.y = 0.0;
                        }
                    }
                }

                // --- CAMERA ---
                let camera_x = player.pos.x - screen_width()/3.0;

                // --- SPAWN PLATFORMS ---
                while let Some(last) = platforms.last() {
                    if last.pos.x + last.size.x < camera_x + screen_width() {
                        let new_plat = spawn_platform(last, jump_force, gravity, move_speed);
                        platforms.push(new_plat);

                        // Spawn coin
                        let last = platforms.last().unwrap();
                        let coin_x = last.pos.x + last.size.x / 2.0 - 8.0;
                        let coin_y = last.pos.y - 16.0;
                        coins.push(Coin { pos: vec2(coin_x, coin_y), collected: false });
                    } else { break; }
                }

                // --- REMOVE OFFSCREEN ---
                platforms.retain(|p| p.pos.x + p.size.x > camera_x - 100.0);
                coins.retain(|c| c.pos.x + 16.0 > camera_x - 100.0 && !c.collected);

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

                // --- GAME OVER ---
                if player.pos.y > screen_height() {
                    state = GameState::GameOver;
                    if score > highscore { highscore = score; }
                }

                // --- DRAW CLOUDS ---
                clear_background(SKYBLUE);
                for cloud in &clouds {
                    draw_texture_ex(
                        cloud.tex,
                        cloud.pos.x - camera_x * 0.3, // parallax effect
                        cloud.pos.y,
                        WHITE,
                        DrawTextureParams { dest_size: Some(vec2(128.0, 64.0)), ..Default::default() },
                    );
                }

                // --- DRAW PLATFORMS ---
                for plat in &platforms {
                    let tex = match plat.kind {
                        PlatformKind::Small => &small_platform_tex,
                        PlatformKind::Big => &big_platform_tex,
                    };
                    draw_texture_ex(
                        tex,
                        plat.pos.x - camera_x,
                        plat.pos.y,
                        WHITE,
                        DrawTextureParams { dest_size: Some(plat.size), ..Default::default() },
                    );
                }

                // --- DRAW COINS ---
                for coin in &coins {
                    if !coin.collected {
                        draw_texture_ex(
                            &coin_tex,
                            coin.pos.x - camera_x,
                            coin.pos.y,
                            WHITE,
                            DrawTextureParams { dest_size: Some(vec2(16.0,16.0)), ..Default::default() },
                        );
                    }
                }

                // --- DRAW PLAYER ---
                draw_texture_ex(
                    &player_tex,
                    player.pos.x - camera_x,
                    player.pos.y,
                    WHITE,
                    DrawTextureParams { dest_size: Some(vec2(32.0,32.0)), ..Default::default() },
                );

                draw_text(&format!("Score: {}", score), 20.0, 30.0, 30.0, BLACK);
                draw_text(&format!("Highscore: {}", highscore), 20.0, 60.0, 30.0, BLACK);
            }

            GameState::GameOver => {
                clear_background(BLACK);
                draw_text("Game Over!", screen_width()/2.0 - 100.0, screen_height()/2.0, 50.0, RED);
                draw_text(&format!("Score: {}", score), screen_width()/2.0 - 50.0, screen_height()/2.0 + 50.0, 30.0, WHITE);
                draw_text(&format!("Highscore: {}", highscore), screen_width()/2.0 - 70.0, screen_height()/2.0 + 90.0, 30.0, WHITE);
                draw_text("Press R to restart", screen_width()/2.0 - 130.0, screen_height()/2.0 + 140.0, 30.0, WHITE);

                if is_key_pressed(KeyCode::R) {
                    // reset everything
                    player.pos = vec2(50.0, 300.0);
                    player.vel = vec2(0.0, 0.0);
                    player.on_ground = false;
                    coyote_timer = 0.0;
                    platforms.clear();
                    platforms.push(Platform { pos: vec2(0.0, 400.0), size: vec2(256.0, 32.0), kind: PlatformKind::Big });
                    platforms.push(Platform { pos: vec2(300.0, 300.0), size: vec2(128.0, 32.0), kind: PlatformKind::Small });
                    platforms.push(Platform { pos: vec2(500.0, 200.0), size: vec2(128.0, 32.0), kind: PlatformKind::Small });
                    coins.clear();
                    coins.push(Coin { pos: vec2(340.0, 284.0), collected: false });
                    coins.push(Coin { pos: vec2(540.0, 184.0), collected: false });
                    score = 0;
                    state = GameState::Playing;
                }
            }
        }

        next_frame().await;
    }
}

/// Spawn platform with physics-based spacing
fn spawn_platform(last: &Platform, jump_force: f32, gravity: f32, move_speed: f32) -> Platform {
    let t_up = -jump_force / gravity;
    let t_total = t_up * 2.0;
    let max_jump_distance = move_speed * t_total * 0.9;

    let min_gap = 120.0;
    let max_gap = max_jump_distance.min(220.0);
    let gap = rand::gen_range(min_gap, max_gap);

    let kind = if rand::gen_range(0,2) == 0 { PlatformKind::Small } else { PlatformKind::Big };
    let width = match kind { PlatformKind::Small => 128.0, PlatformKind::Big => 256.0 };

    let min_y = (last.pos.y - 80.0).max(150.0);
    let max_y = (last.pos.y + 80.0).min(400.0);
    let y = rand::gen_range(min_y, max_y);

    Platform { pos: vec2(last.pos.x + last.size.x + gap, y), size: vec2(width, 32.0), kind }
}
