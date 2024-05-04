use crossterm::event::{self, Event, KeyCode};
use invaders::frame::{self, new_frame, Drawable};
use invaders::invaders::Invaders;
use invaders::player::Player;
use invaders::render;
use rusty_audio::Audio;
use std::{io, thread, vec};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use crossterm::{terminal, ExecutableCommand};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::cursor::{Hide, Show};

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let mut audio = Audio::new();
    audio.add("explode", "explode.wav");
    audio.add("lose", "lose.wav");
    audio.add("move", "move.wav");
    audio.add("pew", "pew.wav");
    audio.add("startup", "startup.wav");
    audio.add("win", "win.wav");
    audio.play("startup");
    // Terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;
    
    //Render loop in a separate thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    // Game Loop
    let mut player: Player  = Player::new();
    let mut instant = Instant::now();
    let mut invaders = Invaders::new();
    'gameloop: loop {
        let delta: Duration = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = new_frame();

        // input
        while event::poll(Duration::default())? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    }
                    KeyCode::Up => {
                        audio.play("move");
                    }
                    KeyCode::Down => {
                        audio.play("move");
                    }
                    KeyCode::Left => {
                        player.move_left();
                        audio.play("move");
                    }
                    KeyCode::Right => {
                        player.move_right();
                        audio.play("move");
                    }
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        if player.shoot(){
                            audio.play("pew");
                        }
                    }
                    _ => {}
                }
            }
        }
        // updates
        player.update(delta);
        if invaders.update(delta){
            audio.play("move");
        }
        if player.detect_hits(&mut invaders){
            audio.play("explode");
        }

        player.draw(&mut curr_frame);
        invaders.draw(&mut curr_frame);
        let drawables: Vec<&dyn Drawable> = vec![&player, &invaders];
        for drawable in drawables{
            drawable.draw(&mut curr_frame);
        }
        // Draw & render
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));

        // Win/Lose
        if invaders.all_killed(){
            audio.play("win");
            break 'gameloop;
        }
        if invaders.reached_bottom(){
            audio.play("lose");
            break 'gameloop;
        }   
    }

    // Cleanup
    drop(render_tx);
    render_handle.join().unwrap();

    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
